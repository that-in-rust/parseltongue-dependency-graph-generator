# Meta-Patterns From Iggy

Studied against the fresh upstream clone at `docs/refer-repo/iggy-sample` on `master @ 93443b17`, then pressure-tested with arXiv papers and comparable open-source systems.

External research for this note is intentionally restricted to:

- arXiv papers
- GitHub repos and official repo documentation

This note is about **meta-patterns**, not about copying Apache Iggy's bytes or broker semantics literally.

## Premise Check

- Apache Iggy is a streaming/event storage system, not a graph simulator.
- So the useful question is not "should Parseltongue store graphs exactly like Iggy stores messages?"
- The useful question is: **what durable storage patterns make Iggy fast, predictable, and recoverable, and which of those patterns transfer to a tree-sitter-based Public Interface Graph simulator?**
- Direct local evidence says Iggy is **not** primarily a `mmap + cast_slice` system. In the storage paths inspected, it relies on append-at-position and read-at-position I/O, explicit fsync/truncate/rename flows, and compact sidecar indexes. That matters because the lesson is about storage architecture, not about one access primitive.

## Expert Lenses

- Storage-engine lens: how hot writes, cold reads, indexes, and recovery are physically shaped on disk.
- Dynamic-graph lens: what mutation and branching patterns matter once the stored object is a graph, not a message log.
- Product-platform lens: which patterns preserve a crisp product wedge instead of forcing an overbuilt storage platform.
- Skeptical lens: where "Iggy inspiration" risks turning into cargo-culting.

## Candidate Approaches

### 1. Copy Iggy literally

- Store everything as append-only logs plus indexes.
- Why it loses:
  - graph adjacency reads want contiguous neighborhood access, not offset-polling semantics
  - public-interface graph queries are not broker fetches
  - this would inherit broker-shaped complexity without broker-shaped workloads

### 2. Ignore Iggy and use a general graph database

- Put the graph, branches, mutations, and views behind a DB/query engine.
- Why it loses:
  - weak cold-start behavior
  - more moving parts than the current product needs
  - query generality is paid for even when workloads are narrow and repetitive

### 3. Immutable graph snapshots only

- One read-only public-interface graph artifact, rebuilt wholesale.
- Why it wins:
  - simple
  - fast reads
  - easy reasoning
- Why it loses:
  - weak scenario branching story
  - mutation history becomes awkward
  - no durable "what-if" log

### 4. Hybrid meta-pattern transfer

- Immutable graph snapshots for truth
- append-only scenario logs for mutation history
- compact indexes for lookup
- coarse compaction into new snapshots
- rebuildable derived views

This is the only candidate that actually transfers Iggy's strengths without pretending Parseltongue is a broker.

## Chosen Thesis

The right lesson from Apache Iggy is:

**shape persistence around the dominant workload, keep the mutable head small and sequential, seal old data into immutable segments, and reconstruct lightweight runtime state from compact durable artifacts.**

For Parseltongue, that implies:

- the **Public Interface Graph** should be the canonical immutable artifact
- simulation edits should be durable as **append-only scenario operations**
- indexes should be **small, direct, and workload-shaped**
- derived views and caches should be treated as **rebuildable products of truth**, not as the truth itself

In short:

- **Iggy gives us storage principles**
- **LSMGraph gives us the graph-side warning that write-friendly and read-friendly structures should be split or layered**
- **Spade gives us the algorithmic warning that evolving-graph speed comes from affected-area incrementalization, not from recomputing everything**

## Evidence and Verification

### Direct Iggy Facts From The Local Clone

#### 1. Iggy separates control-plane durability from data-plane durability

Direct local evidence:

- metadata recovery explicitly loads `metadata/snapshot.bin`, restores state, opens `metadata/journal.wal`, and replays entries after the snapshot in `core/metadata/src/impls/recovery.rs`
- metadata WAL handling and truncation logic lives in `core/journal/src/metadata_journal.rs`
- partition payload lives in segmented `.log` + `.index` pairs via `core/server/src/bootstrap.rs`, `core/partitions/src/iggy_partitions.rs`, and `core/partitions/src/iggy_index.rs`

Why this matters:

- one storage shape is used for replayable state machine metadata
- another is used for high-throughput payload bytes
- Iggy does **not** force one abstract persistence model onto both workloads

Parseltongue implication:

- separate the **graph truth artifact** from the **simulation operation history**
- do not force both into one monolithic mutable store

#### 2. Iggy keeps the hot write path append-only and sequential

Direct local evidence:

- prepares are assembled in memory in `core/partitions/src/iggy_partition.rs`
- persistence is threshold-driven in `core/partitions/src/iggy_partitions.rs`
- segment rotation creates new `{start_offset}.log` and `{start_offset}.index` pairs in `core/partitions/src/iggy_partitions.rs`

Why this matters:

- the active tail is cheap to write
- old segments become cold immutable data
- write throughput is protected from random-update costs

Parseltongue implication:

- branch/simulation operations should be append-only
- do not mutate the baseline public-interface graph in place

#### 3. Iggy uses tiny seek accelerators instead of a heavyweight storage engine

Direct local evidence:

- old/common index entries are 16 bytes in `core/common/src/types/message/index.rs`
- newer partition index entries are 24 bytes in `core/partitions/src/iggy_index.rs`
- read paths consult cached or loaded indexes before touching message bytes in `core/server/src/streaming/partitions/ops.rs`

Why this matters:

- the index is just enough to jump near the right bytes
- it is not a general-purpose query layer

Parseltongue implication:

- prefer exact-name hash indexes, public-surface filters, and adjacency offsets
- avoid a generalized query planner until the product proves the need

#### 4. Iggy treats restart as reconstruction, not blind restoration

Direct local evidence:

- WAL open validates and truncates corrupt tails in `core/journal/src/metadata_journal.rs`
- bootstrap scans `.log` files, pairs indexes, and reconstructs segment state in `core/server/src/bootstrap.rs`

Why this matters:

- correctness is maintained by scanning, validating, and rebuilding lightweight runtime state
- this is simpler and more durable than serializing a giant mutable in-memory object graph

Parseltongue implication:

- startup should validate headers, scans, checksums, and indexes
- derived views should be rebuildable from snapshots plus scenario logs

#### 5. Iggy persists protocol-native batches, not row-shaped database records

Direct local evidence:

- `SendMessages2Header` in `core/common/src/types/send_messages2.rs`
- `persist_frozen_batches_to_disk()` strips the outer prepare wrapper and persists batch bodies in `core/partitions/src/iggy_partitions.rs`

Why this matters:

- the disk format is close to the runtime/wire format
- persistence avoids unnecessary re-serialization layers

Parseltongue implication:

- the on-disk public-interface graph should be stored in the form the read path actually wants
- if we use CSR-like adjacency, the bytes on disk should already be adjacency-oriented

### arXiv Reinforcement

#### A. LSM meta-patterns are still the canonical answer for update-heavy persistence

Sourced facts:

- `LSM-based Storage Techniques: A Survey` says LSM-trees are widely adopted for modern NoSQL storage layers and surveys their strengths and trade-offs ([arXiv:1812.07527](https://arxiv.org/abs/1812.07527))
- `A survey of LSM-Tree based Indexes, Data Systems and KV-stores` frames LSM structures as the standard answer for update-intensive workloads and surveys modern variants ([arXiv:2402.10460](https://arxiv.org/abs/2402.10460))

Reasoned inference:

- Iggy's append/compact flavor is part of a broader family: sequential writes now, compaction later, index enough for reads

What changes the conclusion:

- if Parseltongue mutation volume is tiny and read-only snapshots dominate, we may not need an LSM-like branch log at all in `v1`

#### B. Dynamic graph systems independently converge on "write-friendly layer + read-friendly CSR"

Sourced facts:

- `LSMGraph` explicitly combines write-friendly LSM structure with read-friendly CSR, uses a multi-level index, and adds version control for correctness under concurrent read/write ([arXiv:2411.06392](https://arxiv.org/abs/2411.06392))

Reasoned inference:

- this is the strongest external support for a hybrid Parseltongue design
- if we want simulation over a graph, the immutable/read-optimized adjacency structure and the mutable/update-optimized mutation structure should not be the same thing

What changes the conclusion:

- if we deliberately postpone durable mutation and branch history, then a pure immutable snapshot is still the cleaner starting point

#### C. Evolving-graph speed comes from affected-area maintenance, not full recomputation

Sourced facts:

- `Spade` argues that recomputing from scratch on evolving graphs misses real-time requirements and reports large gains by incrementally maintaining only the affected area ([arXiv:2211.06977](https://arxiv.org/abs/2211.06977))

Reasoned inference:

- if Parseltongue ever supports low-latency mutation simulation, it should update local blast radius, SCC neighborhoods, and dependency deltas from a touch set first
- full global recomputation should be the fallback, not the default

What changes the conclusion:

- if our simulation outputs stay simple enough, we may be able to recompute whole derived packets cheaply for a long time before needing true incremental maintenance

### GitHub And Adjacent System Reinforcement

#### 1. Kafka reinforces segmented logs, offsets, and recovery-by-validation

Sourced facts:

- Kafka documents segment-named log files such as `00000000000000000000.log`, serial appends to the last file, binary search over segments for reads, and startup recovery by validating/truncating the newest segment ([Apache Kafka log docs](https://kafka.apache.org/37/implementation/log/))
- Kafka also explains why it prefers a simple offset-based lookup structure over a heavyweight persistent random-access mapping ([Apache Kafka log docs](https://kafka.apache.org/37/implementation/log/))
- the repo remains the canonical reference implementation ([apache/kafka](https://github.com/apache/kafka))

Reasoned inference:

- Kafka and Iggy converge on the same storage instinct: monotonic append path, compact lookup structure, startup scan/repair

#### 2. Redpanda reinforces sealed local segments plus indexed remote tiering

Sourced facts:

- Redpanda documents tiered storage as offloading log segments to object storage and explicitly says it indexes where data is offloaded so it can read it back later ([Tiered Storage docs](https://docs.redpanda.com/24.1/manage/tiered-storage/))
- Redpanda positions itself as a Kafka-compatible streaming platform with configurable tiered storage ([redpanda-data/redpanda](https://github.com/redpanda-data/redpanda))

Reasoned inference:

- the meta-pattern here is not "use object storage"
- the meta-pattern is: once segments are sealed, they can move to colder tiers while a lightweight index keeps them discoverable

Parseltongue implication:

- old scenario packs or archived graph snapshots can be pushed to colder storage later without changing query semantics

#### 3. KurrentDB reinforces event streams plus metadata-as-first-class-state

Sourced facts:

- KurrentDB describes itself as event-native and organizes state changes into streams ([kurrent-io/KurrentDB](https://github.com/kurrent-io/KurrentDB))
- its docs state that it stores each state alteration as an independent event and attaches metadata to streams and events ([Kurrent event streams docs](https://docs.kurrent.io/server/v25.0/features/streams))

Reasoned inference:

- if we want simulation provenance, branch authorship, causation, and explanation packets, metadata must be durable and queryable alongside operations

Parseltongue implication:

- scenario ops should carry metadata such as author, timestamp, rationale, tool, and parent snapshot

#### 4. Lance reinforces versioned immutable artifacts plus later compaction

Sourced facts:

- Lance documents versioned data, metadata-tracked versions, zero-copy data evolution, and compaction that writes new compacted files while old versions remain queryable ([Lance format docs](https://docs.lancedb.com/lance))
- the repo frames Lance as high-random-access, versioned storage with no extra infrastructure for versioning ([lance-format/lance](https://github.com/lance-format/lance))

Reasoned inference:

- this is the clearest non-streaming analogue to what Parseltongue may want for immutable Public Interface Graph snapshots

Parseltongue implication:

- snapshot publication should be versioned and manifest-driven
- old versions should remain readable until compaction/retention policy removes them

#### 5. Differential Dataflow reinforces the "delta first" computation worldview

Sourced facts:

- Differential Dataflow describes itself as a framework that can quickly respond to arbitrary changes in input collections ([TimelyDataflow/differential-dataflow](https://github.com/TimelyDataflow/differential-dataflow))

Reasoned inference:

- this is not a storage pattern, but it is a useful mental model for Parseltongue:
  - truth artifact
  - change set
  - derived output deltas

Parseltongue implication:

- simulation results should probably be stored and explained as delta packets, not just recomputed rendered graphs

## Final Synthesis

The transferable meta-patterns from Iggy are:

1. **Workload-shaped persistence beats abstract database purity.**
2. **Separate truth storage from mutation history.**
3. **Keep the hot path append-only and sequential.**
4. **Seal old data into immutable segments or versions.**
5. **Use tiny direct indexes, not generalized query engines, for the common path.**
6. **Treat restart as validate + reconstruct, not deserialize + hope.**
7. **Persist data close to the runtime access shape.**
8. **Version first, compact later.**
9. **Incrementalize only the affected area when mutation pressure becomes real.**
10. **Make provenance and metadata durable, not incidental.**

For Parseltongue, the resulting design direction is:

- canonical immutable **Public Interface Graph snapshots**
- append-only **scenario operation logs**
- compact embedded indexes for:
  - exact symbol lookup
  - public/export filtering
  - adjacency traversal
- optional rebuildable derived artifacts:
  - delta packets
  - LOD views
  - cached metrics

That is the right "Iggy-inspired" architecture.

It is **not**:

- "turn Parseltongue into a broker"
- "copy `.log + .index` literally for graph truth"
- "replace graph adjacency with generic event polling"
- "pretend tree-sitter graph simulation has compiler truth"

## What To Steal

- snapshot + operation-log split
- immutable sealed history
- compact seek indexes
- startup scan/validation
- atomic publish of new heads
- versioned manifests
- metadata/provenance attached to operations

## What Not To Cargo-Cult

- broker fetch semantics
- timestamp/offset as the primary graph query model
- literal Iggy file framing for graph adjacency
- message-oriented runtime assumptions
- building full mutation infrastructure before the Public Interface Graph schema is stable

## Open Questions

- What is the smallest durable mutation vocabulary for the Public Interface Graph?
- Should scenario logs store only operations, or operations plus computed delta packets?
- When do we cross the threshold where incremental maintenance is worth more than full recompute?
- Which indexes belong inside the snapshot file versus sidecar files?
- Do we want branch merges in `v1`, or only linear scenario histories with explicit rebase/replay?
