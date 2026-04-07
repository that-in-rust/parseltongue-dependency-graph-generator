# Research: Iggy Storage Pattern

Studied against the fresh upstream clone at `docs/refer-repo/iggy-sample` on `master @ 93443b17`.

## Premise Check

- The repo does not expose one single storage implementation. It currently contains two storage strata:
  - a newer VSR-oriented path in `core/metadata`, `core/journal`, and `core/partitions`
  - an older but still active server/bootstrap and streaming path in `core/server/src/bootstrap.rs` and `core/server/src/streaming/*`
- So the right question is not "what exact single format does Iggy use?" but "what storage patterns survive across both paths?"
- Direct evidence says Iggy is not winning by mmap-style zero-copy file reinterpretation. In the storage code I inspected, it uses append-at-position and read-at-position primitives via `compio`, plus vectored writes and explicit fsync/truncate/rename flows:
  - `core/journal/src/file_storage.rs:38-127`
  - `core/common/src/types/segment_storage/index_reader.rs:72-188`
  - `core/common/src/types/segment_storage/messages_reader.rs:87-148`

## Expert Lenses

- Storage-engine lens: what bytes are on disk, and how do reads find the right bytes fast?
- Durability lens: what happens on crash, restart, torn write, or partial flush?
- Runtime lens: how much state stays in memory, and what gets reconstructed on open?
- Skeptical migration lens: which parts are stable product patterns, and which parts are clearly mid-transition?

## Candidate Approaches

### 1. Monolithic database / B-tree engine

- This is not what Iggy does for hot message data.
- There is no evidence in the inspected storage path that partitions are persisted through a general-purpose DB engine.
- Why it loses for Iggy:
  - message writes want append semantics
  - reads mostly want "jump near offset/timestamp, then stream bytes"
  - DB indirection would add overhead without solving the dominant access path

### 2. Pure append-only log with no sidecar index

- Better for write throughput, but poor for offset and timestamp lookup.
- This would force scan-heavy reads.
- The codebase explicitly adds index files for exactly this reason:
  - bootstrap expects `.log` plus `.index` siblings per segment in `core/server/src/bootstrap.rs:243-315`
  - the runtime read path asks the index reader for only the relevant slice before reading messages in `core/server/src/streaming/partitions/ops.rs:494-556` and `core/server/src/streaming/partitions/ops.rs:670-730`

### 3. Actual Iggy pattern: split metadata plane + segmented append-only payload plane

- This is the design that best matches the code.
- Metadata state is treated like a replayable state machine with snapshots and WAL.
- Partition payload is treated like a segmented log of message batches with sidecar indexes.
- This is the main pattern that makes Iggy work.

### 4. Transitional hybrid inside the repo

- The repo currently carries two indexing granularities:
  - old/common path: `INDEX_SIZE = 16` with `(offset: u32, position: u32, timestamp: u64)` in `core/common/src/types/message/mod.rs:42` and `core/common/src/types/message/index.rs:19-23`
  - newer/partitions path: `IGGY_INDEX_SIZE = 24` with `(offset: u64, timestamp: u64, position: u64)` in `core/partitions/src/iggy_index.rs:18-25`
- These are not contradictions in the product thesis. They point to the same underlying pattern:
  - append bytes to segment file
  - append a compact lookup structure to a sidecar index file
  - rebuild or cache just enough lookup state in memory

## Chosen Thesis

Iggy storage works because it shapes persistence around the workload instead of around an abstract "database" idea:

- control-plane metadata is persisted as `snapshot + WAL`
- data-plane messages are persisted as segmented append-only files
- indexes are tiny seek accelerators rather than full storage engines
- the active head stays in memory first and becomes durable in larger batches
- restart reconstructs in-memory lookup state from compact files instead of deserializing a huge object graph

The deeper pattern is: **Iggy stores raw bytes in the form it wants to write, and stores just enough index to avoid scanning from zero.**

## Evidence and Verification

### Claim 1: Metadata persistence is snapshot plus WAL, not "queryable database state"

**Direct evidence**

- System base path is `local_data`, with runtime subpaths under `system.*` in `core/server/config.toml:326-425`
- Metadata recovery is explicitly documented as:
  1. load `metadata/snapshot.bin`
  2. restore state machine
  3. open `metadata/journal.wal`
  4. replay entries after the snapshot
  in `core/metadata/src/impls/recovery.rs:93-149`
- Metadata WAL entries are raw `Message<PrepareHeader>` records, and the in-memory lookup is a slot array keyed by `op % SLOT_COUNT` in `core/journal/src/metadata_journal.rs:81-103`

**Why this works**

- Snapshot keeps restart bounded.
- WAL preserves ordered intent.
- The slot array gives fast lookup for recent unsnapshotted ops without turning the WAL into a full index structure.

**Verification**

- Question: does the code repair torn WAL tails instead of blindly trusting the file?
- Answer: yes. `MetadataJournal::open()` scans forward, validates header size/command, and truncates corrupt or incomplete tail entries in `core/journal/src/metadata_journal.rs:122-189`.

### Claim 2: The metadata WAL is optimized for replay safety, not ad hoc querying

**Direct evidence**

- Each WAL entry is `[PrepareHeader][body]` in `core/journal/src/metadata_journal.rs:81-87`
- `MAX_ENTRY_SIZE` guards against pathological header corruption in `core/journal/src/metadata_journal.rs:32-37`
- Append goes to end-of-file and immediately fsyncs in `core/journal/src/metadata_journal.rs:411-442`
- Drain compacts the WAL by rewriting live entries to a temp file and atomically renaming it in `core/journal/src/metadata_journal.rs:314-409`

**Why this works**

- The WAL stays append-friendly on the hot path.
- Replay remains linear and simple.
- Compaction is explicit and coarse-grained rather than hidden in a background storage engine.

**Inference**

- This is closer to a replicated log implementation than a classic embedded database. The code is optimized for ordered consensus operations and deterministic replay, not secondary query flexibility.

### Claim 3: Partition payload storage is segmented and append-only

**Direct evidence**

- Server config sets the storage tree under `local_data/streams/.../topics/.../partitions/...` in `core/server/config.toml:421-445`
- Bootstrap loads partition segments by scanning `.log` files, deriving the start offset from the file stem, and pairing each with a same-stem `.index` file in `core/server/src/bootstrap.rs:232-315`
- In the newer partitions crate, `init_partition()` creates a `Segment` at offset `0`, plus a message file and an index file in `core/partitions/src/iggy_partitions.rs:308-389`
- Segment rotation seals the old segment and creates a new `{start_offset}.log` plus `{start_offset}.index` pair in `core/partitions/src/iggy_partitions.rs:1222-1308`

**Important caveat**

- In the new `core/partitions` crate, the path builders are still stubbed to `/tmp/iggy_stub/...` rather than the real configured system path:
  - `core/partitions/src/types.rs:221-256`
- So the structural pattern is clear, but that crate is not yet the final authoritative runtime path provider by itself.

**Why this works**

- Segmenting by start offset gives natural rotation, retention, and bounded file sizes.
- Old sealed segments become cold immutable data.
- The active segment remains the only mutable tail.

### Claim 4: The write path is deliberately two-stage: prepare in memory, persist in batches

**Direct evidence**

- `IggyPartition::append_messages()`:
  - computes the next dirty offset
  - stamps the prepare with `base_offset` and `base_timestamp`
  - updates `dirty_offset`
  - advances the active segment's logical `current_position`
  - appends the prepared batch into the in-memory journal
  in `core/partitions/src/iggy_partition.rs:75-137`
- `stamp_prepare_for_persistence()` updates the `SendMessages2Header` inside the prepare message before it is journaled in `core/common/src/types/send_messages2.rs:624-643`
- `commit_messages()` only persists once thresholds are crossed:
  - segment full
  - enough unsaved messages
  - enough unsaved bytes
  in `core/partitions/src/iggy_partitions.rs:759-790`

**Why this works**

- Offset assignment and protocol shaping happen immediately.
- Disk I/O is amortized across batches.
- The system can pipeline prepares before commit without losing ordering.

**Verification**

- Question: is the durable offset advanced before bytes hit disk?
- Answer: no. `commit_messages()` persists first, then updates segment metadata and only then stores the durable partition offset in `core/partitions/src/iggy_partitions.rs:842-879`.

### Claim 5: The payload bytes on disk are protocol-native message batches, not row-structured records

**Direct evidence**

- `SendMessages2Header` is a fixed 256-byte command header with:
  - `partition_id`
  - `base_offset`
  - `base_timestamp`
  - `origin_timestamp`
  - `batch_length`
  - `batch_checksum`
  - `message_count`
  in `core/common/src/types/send_messages2.rs:25-120`
- `decode_prepare_slice()` validates a prepared message by decoding that command header and verifying the batch checksum in `core/common/src/types/send_messages2.rs:586-622`
- `persist_frozen_batches_to_disk()` strips the outer `PrepareHeader` and persists only the batch body to the segment `.log` file in `core/partitions/src/iggy_partitions.rs:1128-1171`

**Why this works**

- The disk format is already close to the wire/runtime batch format.
- Iggy avoids per-message reserialization during flush.
- Reads can pull a contiguous byte range and reconstruct a message batch from the indexes plus bytes.

**Skeptical note**

- This is not "on-disk format = mmap-cast in-memory structs" in the Parseltongue CSR sense.
- It is "on-disk format = protocol batch bytes + compact index records", which is a different but still very strong pattern.

### Claim 6: Reads are journal-first, then index-guided disk reads

**Direct evidence**

- In the newer partitions crate, `poll_messages()` reads from the journal only:
  - offset lookups via `MessageLookup::Offset`
  - timestamp lookups via `MessageLookup::Timestamp`
  in `core/partitions/src/iggy_partition.rs:140-189`
- In the older streaming server path, reads first check whether the requested range is fully covered by the journal, and return from memory if so:
  - by offset in `core/server/src/streaming/partitions/ops.rs:471-492`
  - by timestamp in `core/server/src/streaming/partitions/ops.rs:654-668`
- If the journal misses, the server:
  - gets cached indexes if available
  - otherwise asks `IndexReader` for just the relevant offset/timestamp slice
  - then asks `MessagesReader` to load only the referenced byte range
  in `core/server/src/streaming/partitions/ops.rs:494-556` and `core/server/src/streaming/partitions/ops.rs:670-730`

**Why this works**

- The hot tail is memory-first.
- Disk reads stay bounded because the index narrows the byte range.
- Full segment scans are avoided for normal offset/timestamp polling.

### Claim 7: Restart is reconstruction, not blind trust

**Direct evidence**

- Bootstrap scans existing `.log` files, optionally rebuilds missing index files, loads indexes, and reconstructs segment metadata in `core/server/src/bootstrap.rs:232-459`
- Missing index files can be rebuilt from the message file by `IndexRebuilder` in `core/server/src/bootstrap.rs:266-294`
- Optional checksum validation re-reads segment data in chunks during startup in `core/server/src/bootstrap.rs:354-401`

**Why this works**

- The on-disk files are authoritative enough to rebuild the in-memory state.
- The index file is an accelerator, not the sole source of truth.
- Operationally, this is a resilient log-structured design: sealed data is recoverable and reindexable.

### Claim 8: The repo is in a real migration, and that matters

**Direct evidence**

- Old/common path uses per-message 16-byte indexes:
  - `core/common/src/types/message/mod.rs:42`
  - `core/common/src/types/message/index.rs:19-23`
- New partitions path uses 24-byte sparse index records:
  - `core/partitions/src/iggy_index.rs:18-25`
- In `commit_messages()`, the new partitions path only serializes a single `flush_index` for the first batch encountered in that flush:
  - `core/partitions/src/iggy_partitions.rs:792-829`
- The path builders in that same crate are still explicit stubs:
  - `core/partitions/src/types.rs:221-256`

**Inference**

- The stable storage idea is already decided.
- The exact final segment index granularity and plumbing are still converging.
- So the pattern to copy is the architecture, not every byte choice in the transitional code.

## Final Synthesis

The patterns that make Iggy storage work are:

1. **Separate control-plane state from data-plane bytes.**
   Metadata gets snapshot + WAL. Message payload gets segmented logs + sidecar indexes.

2. **Append raw bytes in the shape the producer already generated.**
   Iggy persists message batches, not a per-row normalized format.

3. **Use tiny indexes as seek accelerators, not as the storage of record.**
   The message file is the truth. The index file helps jump into it.

4. **Keep the active head in memory first.**
   The journal absorbs writes and answers hot reads. Disk is for the durable tail.

5. **Treat restart as reconstruction.**
   Scan, validate, rebuild caches, rebuild missing indexes, then continue.

6. **Rotate by segment, not by rewriting the world.**
   The active segment mutates; sealed segments become stable, cold files.

7. **Prefer simple file primitives over a general storage engine.**
   `write_all_at`, `write_vectored_all_at`, `read_exact_at`, `truncate`, `rename`, `fsync` are the core moves.

My strongest conclusion is this:

> Iggy works because it is a workload-shaped log-structured storage system, not because it discovered a magical file format.

The storage is successful where it is boring and disciplined:

- append-only for throughput
- sidecar index for seek
- snapshot + WAL for control-plane replay
- rebuild-on-open for operational resilience

That is the pattern worth stealing.

## Open Questions

- Which index format is the long-term canonical one for Apache Iggy:
  - the older 16-byte per-message index
  - or the newer 24-byte sparse checkpoint index?
- When `core/partitions` becomes fully authoritative, will it keep the current sparse-per-flush indexing strategy or adopt a denser read-side index?
- How much of the old `core/server/src/streaming/*` read path is still the production path today versus compatibility/bootstrap scaffolding?
- If we wanted to copy Iggy's best idea into Parseltongue, should we copy:
  - segmented append-only + sidecar index
  - or the even deeper rule: file layout should directly reflect the dominant query path?

## What I Would Copy For Parseltongue

- Copy:
  - flat files instead of a database when the access path is narrow and stable
  - separate durable bytes from lightweight lookup structures
  - explicit restart reconstruction
  - batch-oriented writes
  - atomic replace for compaction-style operations

- Do not copy literally:
  - the exact `.log/.index` message format
  - the VSR metadata journal
  - the transitional stub path code
  - the split-brain index formats

- Strong inference:
  - For Parseltongue, the Iggy lesson is not "use a sparse log index because Iggy does."
  - The lesson is "choose the simplest file layout whose cold-open and read path directly match the product's dominant question."
