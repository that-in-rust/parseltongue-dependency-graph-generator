# ISG Storage Format: An Iggy-Inspired Design for the Public Interface Signature Graph

**Date**: 2026-04-05
**Status**: Research document -- design exploration, not committed architecture
**Scope**: Storage layer for the ISG (entities, edges, boundaries, variants, compiler enrichment)
**Constraint**: Must work for Rust codebases up to ~50K entities, ~200K edges (Linux kernel scale is out of scope)

---

## Table of Contents

1. [Iggy's Cascading Design: What They Got Right and Why](#1-iggys-cascading-design)
2. [What Transfers to a Code Graph and What Does Not](#2-what-transfers)
3. [The Proposed ISG Storage Format](#3-the-proposed-isg-storage-format)
4. [File Layout on Disk](#4-file-layout-on-disk)
5. [Binary Formats: Index Entries and Data Records](#5-binary-formats)
6. [Query Paths: How Questions Get Answered](#6-query-paths)
7. [Hot/Cold Separation](#7-hotcold-separation)
8. [Incremental Re-Indexing via Segment Rotation](#8-incremental-re-indexing)
9. [Variant Overlays as Append-Only Logs](#9-variant-overlays)
10. [Comparison with SQLite](#10-comparison-with-sqlite)
11. [The Hybrid Proposal](#11-the-hybrid-proposal)
12. [The Cascading Design for a Code Graph](#12-the-cascading-design)
13. [Confidence and Caveats](#13-confidence-and-caveats)

---

## 1. Iggy's Cascading Design: What They Got Right and Why {#1-iggys-cascading-design}

Iggy's storage innovation is not any single technique. It is a **cascade** where every design
choice reinforces every other choice, producing a system where the dominant cost of almost every
operation is a single `memcpy` or pointer cast. The individual innovations and their interactions:

### 1.1 The Core Innovations

**Fixed-size index entries (16 bytes).**
Entry N lives at byte offset `N * 16`. No scanning, no binary search, no hash lookup. O(1) by
construction. This is the foundational trick: if every entry is the same size, position IS identity.

**Variable-size data in a separate .log file.**
Messages are packed contiguously with zero padding. The index entry stores the END offset of each
message in the .log file. To read message N, you compute `start = index[N-1].end_offset` and
`end = index[N].end_offset`. The message bytes are `log[start..end]`. One subtraction. Zero
scanning.

**On-disk format = in-memory format = network format.**
This is the deepest insight. Messages are stored on disk in exactly the byte layout they will have
in memory and on the wire. No serialization. No deserialization. You `mmap()` the file, cast a
pointer, and you are done. The cost of "parsing" a message is zero. The only operation is
`from_le_bytes()`, which is a no-op on little-endian CPUs (all modern x86 and ARM).

**64-byte message header = one CPU cache line.**
Reading a message header touches exactly one cache line. No cache-line splits. No false sharing.
The header contains: timestamp (8B), ID (16B), checksum (4B), headers length (4B), payload
length (4B), plus reserved space. Exactly 64 bytes. This alignment is deliberate -- it means that
iterating over headers is a sequential cache-line walk, which is the access pattern modern CPUs
optimize best.

**Dense indexing (every message indexed) vs. sparse (Kafka-style).**
Kafka's sparse index stores one entry per ~4 KiB of messages. Finding a specific offset requires
binary search within the index, then a linear scan within the data segment. Iggy stores one index
entry per message. This costs more disk (16 bytes per message), but the payoff is O(1) access to
any message by offset with zero scanning. For a system that does millions of reads per second,
eliminating the scan step is worth the extra index space.

**END positions in the index, not START positions.**
This is subtle but important. If you store start positions, computing the length requires reading
the NEXT entry's start position. If you store end positions, the length is `end[N] - end[N-1]`.
Same result, but end positions make range reads trivial: to read messages M through N, you read
`log[end[M-1]..end[N]]`. One `pread()`. No loop.

**Segment rotation.**
When a segment file reaches ~1 GiB, seal it (mark read-only, `mprotect(PROT_READ)`), and open a
new segment. The sealed segment can be memory-mapped once and never touched again -- the OS page
cache handles eviction. New writes always go to the hot segment. This gives you:
- Bounded file sizes (important for `mmap()` on 32-bit, though less so on 64-bit)
- Natural garbage collection boundary (drop old segments)
- Write isolation (new writes don't invalidate existing `mmap()` regions)

**20-digit zero-padded filenames.**
Segment `00000000000000000000.log`, `00000000000000000042.log`, etc. Alphabetical order =
chronological order = offset order. `ls` shows segments in order. No metadata file needed to
know the ordering.

**`writev()` for batch writes.**
Multiple messages are written in a single `writev()` system call using scatter-gather I/O. The
kernel writes all the iovec buffers in one atomic operation. This avoids the overhead of N
`write()` calls for N messages, and more importantly, it produces a single fsync boundary -- all
messages in the batch are either fully written or not written at all.

**Consumer offsets as 8-byte bookmark files.**
Instead of a table of consumer offsets (Kafka uses an internal topic), each consumer's position
is a single file containing one 8-byte little-endian u64. Seeking = reading 8 bytes. Committing
= writing 8 bytes. No locking contention because each consumer has its own file.

**Little-endian everywhere.**
`u64::from_le_bytes()` compiles to a no-op on little-endian architectures. By standardizing on
LE, there is zero conversion cost on modern hardware. This seems trivial but it eliminates an
entire class of bugs (endianness mismatches) and an entire category of runtime work.

### 1.2 The Cascade Effect

The power of Iggy's design is not any single choice -- it is that the choices form a cascade
where each reinforces the others:

```
Fixed-size index entries
    └─► O(1) position lookup (position = identity)
         └─► Dense indexing becomes affordable (one entry per message)
              └─► END offsets make range reads trivial (one subtraction)
                   └─► On-disk = in-memory = network format
                        └─► mmap() replaces deserialization
                             └─► 64-byte headers align to cache lines
                                  └─► Sequential iteration = cache-line walk
                                       └─► writev() batches are natural
                                            └─► Segment rotation bounds file size
                                                 └─► Zero-padded filenames = sorted segments
```

Remove any one choice and the cascade weakens. Make the index variable-size? You need binary
search. Make headers 70 bytes? You get cache-line splits. Use big-endian? You pay conversion on
every read. Store start offsets instead of end offsets? Range reads need a loop. Use a single
file instead of segments? mmap() regions grow unbounded.

The cascade is the design.

### 1.3 The Deeper Principle

The deepest principle underlying Iggy's cascade is: **eliminate every layer of indirection
between the disk and the CPU**. Every "format" is a layer of indirection -- a translation step
between how data is stored and how it is used. Iggy's insight is that if you can make the storage
format, the memory format, and the wire format identical, you eliminate two translation layers.
The CPU operates directly on disk bytes via `mmap()`, and the network stack sends those same bytes
via `sendfile()`.

This principle -- **format unification** -- is the lens through which we should evaluate every
design choice for the ISG storage layer.

---

## 2. What Transfers to a Code Graph and What Does Not {#2-what-transfers}

### 2.1 The Fundamental Difference

Iggy stores a **stream**: an ordered, append-only sequence of messages. The key access pattern is
"give me messages starting at offset N." This is a 1-dimensional problem: every message has a
unique, monotonically increasing offset, and the primary query is a range scan along that
dimension.

The ISG is a **graph**: a set of entities connected by typed, directed edges, grouped into
hierarchical boundaries, with precomputed metrics, variant overlays, and optional deep data
(CFG, Polonius facts, data flow). The key access patterns are:

- **Point lookups**: "Give me entity X" or "Give me boundary Y"
- **Fan-out/fan-in**: "What does X call?" or "Who calls X?"
- **Range scans within a boundary**: "All entities in module server/shard/"
- **Join-like operations**: "For each entity in boundary B, what are its outgoing edges?"
- **Aggregations**: "Sum of edges crossing from boundary A to boundary B"
- **Overlay composition**: "Apply variant V's deltas to the base graph, recompute metrics"

This is a fundamentally multi-dimensional problem. There is no single axis along which all queries
can be answered by a range scan.

### 2.2 What Transfers Directly

| Iggy Principle | ISG Application | Why It Works |
|---|---|---|
| Fixed-size index entries | Entity index: fixed 128-byte entries | Entities have a bounded set of fixed-size fields (metrics, IDs, offsets) |
| Dense indexing | Every entity gets an index entry | Entity counts (1K-50K) are small enough that dense indexing is trivial |
| On-disk = in-memory format | Entity metrics can be memory-mapped and read without deserialization | Metrics are fixed-size numeric fields: f64, u32, u16 |
| END offsets for variable data | Entity signatures, names stored in a data file with END offsets in the index | Same trick: `data[end[N-1]..end[N]]` = the variable-length field for entity N |
| Segment-like files per concern | Separate files for entities, edges, boundaries, variants | Each file has a single access pattern and can be independently `mmap()`-ed |
| Little-endian everywhere | All numeric fields stored as LE | Same zero-cost conversion benefit |
| Cache-line-aligned headers | 64-byte or 128-byte entity index entries | Entity iteration becomes a cache-line walk |

### 2.3 What Does NOT Transfer

| Iggy Property | Why It Breaks for a Graph | Proposed Workaround |
|---|---|---|
| **Append-only** | A code graph is **snapshot-based**, not append-only. Re-indexing replaces the entire graph. Editing a source file changes entities and edges in place. | Write-once snapshots + overlay deltas. Each re-index produces a new snapshot. Variants are append-only deltas on top. |
| **Single ordering axis** | Entities have no natural total order. You need to find them by ID, by file path, by boundary, by caller/callee. No single sort order satisfies all queries. | Multiple sorted indexes. A primary index sorted by entity sequence number (for mmap), plus secondary B-tree-like indexes for ID lookup, file path lookup, and boundary membership. |
| **O(1) by offset** | An entity's "offset" (its sequence number in the index) is not a meaningful user-facing key. Users query by ID (`rust:fn:server::handle`) or by file path. | A hash index or sorted string table mapping entity IDs to sequence numbers. The lookup goes: `ID -> hash -> seq_num -> index[seq_num * 128] -> entity`. Two steps instead of one, but the second step is O(1). |
| **Range reads = one contiguous slice** | Edges are queried by `src_id` OR `dst_id`. A contiguous edge file sorted by `src` enables efficient fan-out queries but makes fan-in queries require a full scan (or a separate reverse index). | Two edge files: `edges_by_src.idx` (sorted by source) and `edges_by_dst.idx` (sorted by destination). Each is a dense, fixed-size index. Redundant storage but O(1) fan-out AND fan-in. |
| **Consumer offsets (bookmarks)** | The ISG has no concept of a "consumer position." The analogous concept is the user's reading position, which is UI state, not storage. | Not applicable. Reading history lives in SQLite (already designed). |
| **writev() batch writes** | Graph writes are not batched appends. They are bulk-load-once operations (indexing produces the entire graph at once). | Replace with "write the entire file in one pass." The equivalent of writev() for a graph is: sort entities, write the index file sequentially, write the data file sequentially. One pass, sequential I/O, maximum throughput. |

### 2.4 The Key Insight: Two Orderings for Edges

The hardest part of adapting Iggy's model to a graph is **edges**. In Iggy, messages have one
natural order (offset). Edges have two natural orderings:

1. **By source**: enables "what does X call?" (fan-out / forward traversal)
2. **By destination**: enables "who calls X?" (fan-in / reverse traversal)

No single sorted file can serve both queries efficiently. Iggy's "one file, O(1) lookup" model
breaks here. The solution is to embrace the redundancy:

- **`edges_forward.idx`**: fixed-size entries sorted by `(src_seq, dst_seq)`. Each entry is the
  edge's data. Answering "what does entity #42 call?" = read entries where `src_seq == 42`, which
  is a contiguous range (because the file is sorted by src).
- **`edges_reverse.idx`**: the same edges, sorted by `(dst_seq, src_seq)`. Answering "who calls
  entity #42?" = read entries where `dst_seq == 42`, contiguous range.

Both files are written during indexing. The cost is 2x edge storage. For a 200K-edge graph with
32-byte edge entries, that is ~12 MiB total. Trivial.

To make the fan-out/fan-in lookup truly O(1) instead of binary-search, we add a **fan-out
offset table** and a **fan-in offset table**: small arrays where entry N stores the starting
position in `edges_forward.idx` (or `edges_reverse.idx`) where entity N's edges begin. This is
the exact analog of Iggy's "position = identity" trick, but applied to adjacency lists.

---

## 3. The Proposed ISG Storage Format {#3-the-proposed-isg-storage-format}

### 3.1 Design Principles

Taken directly from Iggy's cascade, adapted for a code graph:

1. **Format unification**: the on-disk format IS the in-memory format. mmap() replaces
   deserialization for all hot-path data.
2. **Fixed-size index entries**: every entity, edge, and boundary has a fixed-size index record.
   Variable-length data (names, signatures, source snippets) lives in separate data files.
3. **Dense indexing**: every entity gets an index entry. Every edge gets an index entry (in both
   forward and reverse files). Every boundary gets an index entry. No sparse indexes.
4. **Position = identity**: entity sequence number N is at byte offset `N * ENTRY_SIZE` in the
   index file. No hash table needed for the primary lookup path.
5. **Hot/cold separation**: frequently-accessed data (entity metrics, edge topology, boundary
   metrics) is in small, dense, mmap()-friendly files. Infrequently-accessed data (source
   snippets, CFG blocks, Polonius facts) is in separate files loaded on demand.
6. **Write-once snapshots**: each re-index produces a complete, immutable snapshot. No in-place
   mutation. Variant overlays are append-only deltas layered on top.
7. **Cache-line awareness**: index entries are sized to be multiples of 64 bytes.

### 3.2 The Five File Families

The ISG storage format consists of five file families, each serving a distinct access pattern:

```
                    ┌──────────────────────────────────────────────────┐
                    │              ISG STORAGE FORMAT                   │
                    │         Five File Families + Auxiliary            │
                    ├──────────────────────────────────────────────────┤
                    │                                                   │
                    │  1. ENTITY FILES  (.eidx, .edat)                 │
                    │     Fixed-size entity index + variable data       │
                    │     Primary key: entity sequence number           │
                    │                                                   │
                    │  2. EDGE FILES  (.efwd, .erev, .eoff)            │
                    │     Dual-sorted edge indexes + offset tables      │
                    │     Primary key: (src_seq, dst_seq) or reverse    │
                    │                                                   │
                    │  3. BOUNDARY FILES  (.bidx, .bdat, .bedg)        │
                    │     Boundary index + metrics + boundary edges     │
                    │     Primary key: boundary sequence number         │
                    │                                                   │
                    │  4. STRING TABLE  (.stab)                         │
                    │     Deduplicated, sorted string pool              │
                    │     All names, paths, signatures reference here   │
                    │                                                   │
                    │  5. DEEP DATA FILES  (.cfg, .pol, .dfl)          │
                    │     Per-entity cold data: CFG, Polonius, dataflow │
                    │     Loaded on demand, never mmap()-ed             │
                    │                                                   │
                    │  AUX: HASH INDEX  (.hidx)                         │
                    │     entity_id (string) -> seq_num mapping         │
                    │     For point lookups by qualified name            │
                    │                                                   │
                    │  AUX: VARIANT LOG  (.vlog, .vidx)                │
                    │     Append-only variant delta log                  │
                    │     Iggy-style: index + data separation           │
                    │                                                   │
                    └──────────────────────────────────────────────────┘
```

---

## 4. File Layout on Disk {#4-file-layout-on-disk}

### 4.1 Directory Structure

Following Iggy's naming discipline (alphabetical = chronological), with adaptation for snapshot
semantics:

```
parseltongue_workspaces/
└── my-project/
    ├── snapshots/
    │   ├── 00000000000000000001/           # Snapshot 1 (initial index)
    │   │   ├── manifest.bin                # 64-byte header: version, counts, checksums
    │   │   ├── entities.eidx               # Fixed 128-byte entries, one per entity
    │   │   ├── entities.edat               # Variable-length entity data (names, sigs)
    │   │   ├── entities.hidx               # Hash index: entity_id -> seq_num
    │   │   ├── edges_forward.efwd          # Fixed 32-byte edge entries, sorted by src
    │   │   ├── edges_reverse.erev          # Fixed 32-byte edge entries, sorted by dst
    │   │   ├── edges_fanout.eoff           # Fan-out offset table (4 bytes per entity)
    │   │   ├── edges_fanin.eoff            # Fan-in offset table (4 bytes per entity)
    │   │   ├── boundaries.bidx             # Fixed 128-byte boundary entries
    │   │   ├── boundaries.bdat             # Variable-length boundary data
    │   │   ├── boundary_edges.bedg         # Fixed 48-byte boundary edge entries
    │   │   ├── strings.stab               # Deduplicated string pool
    │   │   ├── deep/                       # Cold data (loaded on demand)
    │   │   │   ├── cfg.dat                 # Per-entity CFG blocks and edges
    │   │   │   ├── cfg.idx                 # CFG index: entity_seq -> offset in cfg.dat
    │   │   │   ├── polonius.dat            # Per-entity Polonius facts
    │   │   │   ├── polonius.idx            # Polonius index
    │   │   │   ├── dataflow.dat            # Per-entity dataflow facts
    │   │   │   └── dataflow.idx            # Dataflow index
    │   │   └── metrics/                    # Precomputed graph algorithm results
    │   │       ├── pagerank.f64            # One f64 per entity, in seq order
    │   │       ├── kcore.u32              # One u32 per entity
    │   │       ├── community.u32          # One u32 per entity (Leiden community ID)
    │   │       ├── betweenness.f64        # One f64 per entity
    │   │       ├── indegree.u32           # One u32 per entity
    │   │       └── outdegree.u32          # One u32 per entity
    │   │
    │   └── 00000000000000000002/           # Snapshot 2 (re-index)
    │       └── (same structure)
    │
    ├── variants/
    │   ├── variant_index.vidx              # Fixed-size variant headers
    │   ├── variant_deltas.vlog             # Append-only delta log
    │   └── variant_consequences/           # Cached consequence computations
    │       ├── 0001_pagerank_delta.f64
    │       ├── 0001_community_delta.u32
    │       └── ...
    │
    ├── current -> snapshots/00000000000000000002   # Symlink to active snapshot
    └── meta.bin                                    # Workspace metadata
```

### 4.2 Why This Layout

**Snapshot directories** follow Iggy's zero-padded filename convention. Alphabetical order IS
chronological order. The `current` symlink points to the active snapshot. Old snapshots can be
garbage-collected by deleting their directory.

**Separate files per concern** follows Iggy's principle of matching file granularity to access
patterns. The entity index file will be mmap()-ed for the entire lifetime of the application.
The deep CFG data file will only be loaded when a user clicks "CFG" on a specific function.
Keeping them in separate files means the OS page cache is not polluted by cold data when
iterating over hot data.

**The metrics/ directory** contains one file per metric, each file being a dense array of values
in entity sequence order. This is the purest expression of Iggy's format-unification principle:
to get entity N's PageRank, read 8 bytes at offset `N * 8` from `pagerank.f64`. No header. No
framing. No deserialization. Just bytes.

---

## 5. Binary Formats: Index Entries and Data Records {#5-binary-formats}

### 5.1 Manifest Header (64 bytes)

Every snapshot begins with a manifest that validates the snapshot and provides counts for all
structures. 64 bytes = exactly one cache line.

```
┌─────────────────────────────────────────────────────────────────┐
│  MANIFEST HEADER (64 bytes, cache-line-aligned)                  │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│   0   │  4 bytes │ magic: b"PTNG"                               │
│   4   │  2 bytes │ version: u16 (format version, starts at 1)   │
│   6   │  2 bytes │ flags: u16 (bitfield: has_cfg, has_polonius)  │
│   8   │  4 bytes │ entity_count: u32                             │
│  12   │  4 bytes │ edge_count: u32                               │
│  16   │  4 bytes │ boundary_count: u32                           │
│  20   │  4 bytes │ boundary_edge_count: u32                      │
│  24   │  4 bytes │ string_table_size: u32 (bytes)                │
│  28   │  8 bytes │ snapshot_timestamp: u64 (unix micros)         │
│  36   │ 16 bytes │ source_hash: u128 (blake3 of all source files)│
│  52   │  4 bytes │ entity_index_checksum: u32 (crc32c)           │
│  56   │  4 bytes │ edge_index_checksum: u32 (crc32c)             │
│  60   │  4 bytes │ reserved: u32                                 │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Total: 64 bytes (1 cache line)                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Entity Index Entry (128 bytes = 2 cache lines)

Each entity has a fixed 128-byte record in `entities.eidx`. Entity N is at byte offset
`N * 128`. Two cache lines per entity is a conscious tradeoff: 1 cache line (64 bytes) would
require moving too many fields to the data file, making common queries (show entity with metrics)
touch two files. 2 cache lines keeps the most-queried fields in the index.

```
┌─────────────────────────────────────────────────────────────────┐
│  ENTITY INDEX ENTRY (128 bytes, 2 cache lines)                   │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│       │          │ ── CACHE LINE 1 (identity + location) ──     │
│   0   │  4 bytes │ seq_num: u32 (this entity's sequence number) │
│   4   │  4 bytes │ name_offset: u32 (byte offset in .stab)      │
│   8   │  2 bytes │ name_length: u16 (bytes)                     │
│  10   │  1 byte  │ kind: u8 (enum: fn=0, struct=1, trait=2,     │
│       │          │   impl=3, type_alias=4, mod=5, const=6,      │
│       │          │   static=7, enum=8, union=9, macro=10)       │
│  11   │  1 byte  │ visibility: u8 (pub=0, crate=1, priv=2,      │
│       │          │   pub_super=3, pub_in=4)                     │
│  12   │  4 bytes │ file_path_offset: u32 (into .stab)           │
│  16   │  2 bytes │ file_path_length: u16                        │
│  18   │  4 bytes │ signature_offset: u32 (into .edat)           │
│  22   │  4 bytes │ signature_end: u32 (END offset in .edat)     │
│  26   │  4 bytes │ start_line: u32                               │
│  30   │  4 bytes │ end_line: u32                                 │
│  34   │  4 bytes │ boundary_seq: u32 (which boundary owns this)  │
│  38   │  4 bytes │ id_hash: u32 (lower 32 bits of FNV-1a of     │
│       │          │   the entity's qualified name -- for fast     │
│       │          │   hash-index probing)                         │
│  42   │  2 bytes │ word_count: u16 (source token estimate)       │
│  44   │  4 bytes │ deep_cfg_offset: u32 (0 = no CFG data)       │
│  48   │  4 bytes │ deep_polonius_offset: u32 (0 = no Polonius)  │
│  52   │  4 bytes │ deep_dataflow_offset: u32 (0 = no dataflow)  │
│  56   │  4 bytes │ full_id_offset: u32 (into .stab, the full    │
│       │          │   qualified entity ID string)                 │
│  60   │  2 bytes │ full_id_length: u16                           │
│  62   │  2 bytes │ _pad0: u16 (alignment padding)               │
│       │          │                                               │
│       │          │ ── CACHE LINE 2 (metrics + graph position) ── │
│  64   │  8 bytes │ pagerank: f64                                 │
│  72   │  4 bytes │ in_degree: u32                                │
│  76   │  4 bytes │ out_degree: u32                               │
│  80   │  4 bytes │ k_core: u32                                   │
│  84   │  4 bytes │ community_id: u32 (Leiden community)          │
│  88   │  8 bytes │ betweenness: f64                              │
│  96   │  4 bytes │ fan_out_start: u32 (index into edges_forward) │
│ 100   │  4 bytes │ fan_out_count: u32 (number of outgoing edges) │
│ 104   │  4 bytes │ fan_in_start: u32 (index into edges_reverse)  │
│ 108   │  4 bytes │ fan_in_count: u32 (number of incoming edges)  │
│ 112   │  4 bytes │ dispatch_static_count: u32                    │
│ 116   │  4 bytes │ dispatch_dynamic_count: u32                   │
│ 120   │  4 bytes │ cfg_block_count: u32 (0 if no CFG)            │
│ 124   │  4 bytes │ polonius_loan_count: u32 (0 if no Polonius)   │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Total: 128 bytes (2 cache lines)                                │
│  Entity N is at byte: N * 128                                    │
│  50,000 entities = 6.1 MiB index file (fits entirely in L3)     │
└─────────────────────────────────────────────────────────────────┘
```

**Why 128 bytes and not 64**: The entity index must support the three most common query patterns
without touching any other file:

1. "List entities sorted by PageRank" -- needs kind, name_offset, pagerank, community_id
2. "Show entity detail" -- needs all identity fields + all metrics
3. "What does entity N call?" -- needs fan_out_start, fan_out_count to jump into edge file

All three patterns are satisfied by reading 1-2 cache lines from the entity index. The only
fields that require touching the .stab (string table) or .edat (entity data) files are the
actual string content of names and signatures -- which are needed for display but not for
ranking, filtering, or graph traversal.

### 5.3 Edge Index Entry (32 bytes)

Each edge has a 32-byte entry. Two copies exist: one in `edges_forward.efwd` (sorted by src),
one in `edges_reverse.erev` (sorted by dst). 32 bytes = half a cache line, meaning two edges
fit per cache line. When iterating a fan-out list (contiguous edges with the same src), you
process 2 edges per cache-line fetch.

```
┌─────────────────────────────────────────────────────────────────┐
│  EDGE INDEX ENTRY (32 bytes, half cache line)                    │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│   0   │  4 bytes │ src_seq: u32 (source entity seq number)      │
│   4   │  4 bytes │ dst_seq: u32 (destination entity seq number) │
│   8   │  1 byte  │ edge_kind: u8 (calls=0, impls=1, type_ref=2,│
│       │          │   contains=3, data_flow=4, control_dep=5,    │
│       │          │   drop=6, async_await=7)                     │
│   9   │  1 byte  │ dispatch_kind: u8 (static=0, dynamic=1,     │
│       │          │   closure=2, drop_glue=3, unknown=255)       │
│  10   │  1 byte  │ confidence: u8 (exact=0, candidate=1)        │
│  11   │  1 byte  │ crossing_type: u8 (same_boundary=0,          │
│       │          │   intra_module=1, intra_crate=2,             │
│       │          │   cross_crate=3)                             │
│  12   │  4 bytes │ src_line: u32 (call site line number)        │
│  16   │  4 bytes │ dst_line: u32 (target line number)           │
│  20   │  4 bytes │ weight: f32 (edge weight for algorithms)      │
│  24   │  4 bytes │ src_boundary_seq: u32 (boundary of source)   │
│  28   │  4 bytes │ dst_boundary_seq: u32 (boundary of dest)     │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Total: 32 bytes (1/2 cache line, 2 edges per line)             │
│  200,000 edges x 2 copies = 12.2 MiB total (trivial)           │
└─────────────────────────────────────────────────────────────────┘
```

**Why the edge entry includes boundary_seq for both endpoints**: The boundary crossing type is
derivable from `(src_boundary_seq, dst_boundary_seq)`, but pre-computing it into the edge avoids
a join at query time. This is the Iggy principle of pre-materializing derived data to eliminate
computation during reads. The extra 8 bytes per edge (boundary seqs) is worth not having to look
up two entity records to determine crossing type.

### 5.4 Fan-Out / Fan-In Offset Tables

These are the O(1) adjacency-list lookup structures. Each is a dense array of `(start, count)`
pairs, one per entity.

```
┌─────────────────────────────────────────────────────────────────┐
│  FAN-OUT OFFSET TABLE (edges_fanout.eoff)                        │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│  N*8  │  4 bytes │ start: u32 (first edge index in .efwd)       │
│ N*8+4 │  4 bytes │ count: u32 (number of outgoing edges)        │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Entity N's outgoing edges: efwd[start..start+count]            │
│  50,000 entities x 8 bytes = 390 KiB (fits in L2 cache)        │
└─────────────────────────────────────────────────────────────────┘
```

The fan-in offset table (`edges_fanin.eoff`) has the identical format, pointing into
`edges_reverse.erev`.

**Query: "What does entity 42 call?"**
1. Read `fanout[42]` = `(start: 1050, count: 7)` -- 8 bytes, O(1)
2. Read `efwd[1050..1057]` = 7 edge entries -- 224 bytes, contiguous

Two reads. Zero scanning. This is the ISG equivalent of Iggy's "offset N is at byte N*16."

### 5.5 Boundary Index Entry (128 bytes = 2 cache lines)

Boundaries follow the same pattern as entities: fixed-size index with variable data elsewhere.

```
┌─────────────────────────────────────────────────────────────────┐
│  BOUNDARY INDEX ENTRY (128 bytes, 2 cache lines)                 │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│       │          │ ── CACHE LINE 1 (identity + hierarchy) ──    │
│   0   │  4 bytes │ seq_num: u32                                  │
│   4   │  4 bytes │ name_offset: u32 (into .stab)                │
│   8   │  2 bytes │ name_length: u16                              │
│  10   │  1 byte  │ boundary_type: u8 (crate=0, module=1,        │
│       │          │   folder=2, workspace=3)                     │
│  11   │  1 byte  │ depth: u8 (nesting depth from workspace root)│
│  12   │  4 bytes │ parent_seq: u32 (parent boundary seq, 0=root)│
│  16   │  4 bytes │ path_offset: u32 (into .stab)                │
│  20   │  2 bytes │ path_length: u16                              │
│  22   │  2 bytes │ child_count: u16                              │
│  24   │  4 bytes │ first_entity_seq: u32 (start of entities      │
│       │          │   belonging to this boundary, in a sorted     │
│       │          │   entity-to-boundary mapping)                │
│  28   │  4 bytes │ entity_count_in_boundary: u32                 │
│  32   │  4 bytes │ boundary_edge_start: u32 (into .bedg)        │
│  36   │  4 bytes │ boundary_edge_count: u32                      │
│  40   │ 24 bytes │ _reserved: [u8; 24]                          │
│       │          │                                               │
│       │          │ ── CACHE LINE 2 (metrics) ──                  │
│  64   │  4 bytes │ total_entity_count: u32                       │
│  68   │  4 bytes │ pub_surface: u32                              │
│  72   │  4 bytes │ internal_edges: u32                            │
│  76   │  4 bytes │ outgoing_edges: u32                           │
│  80   │  4 bytes │ incoming_edges: u32                           │
│  84   │  4 bytes │ fan_in: u32 (distinct source boundaries)      │
│  88   │  4 bytes │ fan_out: u32 (distinct target boundaries)     │
│  92   │  8 bytes │ cohesion: f64                                 │
│ 100   │  8 bytes │ coupling_in: f64                              │
│ 108   │  8 bytes │ coupling_out: f64                             │
│ 116   │  1 byte  │ is_facade: u8 (bool: >60% pub use re-exports)│
│ 117   │ 11 bytes │ _reserved_metrics: [u8; 11]                   │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Total: 128 bytes (2 cache lines)                                │
│  Boundary N is at byte: N * 128                                  │
│  ~500 boundaries x 128 bytes = 62.5 KiB (fits in L1 cache)     │
└─────────────────────────────────────────────────────────────────┘
```

### 5.6 Boundary Edge Entry (48 bytes)

```
┌─────────────────────────────────────────────────────────────────┐
│  BOUNDARY EDGE ENTRY (48 bytes)                                  │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│   0   │  4 bytes │ src_boundary_seq: u32                        │
│   4   │  4 bytes │ dst_boundary_seq: u32                        │
│   8   │  1 byte  │ crossing_type: u8 (cross_crate=0,            │
│       │          │   intra_crate=1, intra_module=2)             │
│   9   │  3 bytes │ _pad: [u8; 3]                                │
│  12   │  4 bytes │ edge_count: u32 (entity edges crossing)      │
│  16   │  4 bytes │ file_pairs: u32                               │
│  20   │  4 bytes │ distinct_items: u32                           │
│  24   │  4 bytes │ distinct_files: u32                           │
│  28   │  8 bytes │ import_breadth: f64                           │
│  36   │  8 bytes │ import_spread: f64                            │
│  44   │  4 bytes │ kinds_bitfield: u32 (bitmask of edge_kind    │
│       │          │   enum values present in this crossing)      │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Total: 48 bytes                                                 │
│  ~2000 boundary edges x 48 = 93.75 KiB                          │
└─────────────────────────────────────────────────────────────────┘
```

### 5.7 String Table (.stab)

The string table is a deduplicated, length-prefixed pool of all strings used by entities,
boundaries, and edges. Every string reference in the index files is a `(offset, length)` pair
pointing into this table.

```
┌─────────────────────────────────────────────────────────────────┐
│  STRING TABLE FORMAT                                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Header (16 bytes):                                              │
│    magic: b"STAB" (4 bytes)                                      │
│    entry_count: u32 (number of unique strings)                   │
│    total_bytes: u32 (total string content bytes)                 │
│    _reserved: u32                                                │
│                                                                  │
│  Entries (contiguous, variable-length):                           │
│    Each string is stored as raw UTF-8 bytes, packed contiguously │
│    No length prefix per string -- lengths are in the index files │
│    Strings are deduplicated: "src/main.rs" stored once even if   │
│    referenced by 100 entities                                    │
│                                                                  │
│  Access: stab[offset..offset+length] = the string bytes          │
│                                                                  │
│  For a 50K-entity codebase:                                      │
│    ~10K unique strings (paths, names, signatures)                │
│    ~500 KiB total string data                                    │
│    Entire table fits in L3 cache                                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 5.8 Hash Index (.hidx)

The hash index maps entity ID strings (e.g., `rust:fn:server::handle_message`) to entity
sequence numbers. This is the bridge between user-facing queries (by name) and the positional
index (by sequence number).

```
┌─────────────────────────────────────────────────────────────────┐
│  HASH INDEX FORMAT (open-addressing, linear probing)             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Header (16 bytes):                                              │
│    magic: b"HIDX" (4 bytes)                                      │
│    slot_count: u32 (next power of 2 above entity_count * 1.5)   │
│    entry_count: u32 (= entity_count)                             │
│    _reserved: u32                                                │
│                                                                  │
│  Slots (8 bytes each):                                           │
│    hash_fingerprint: u32 (upper 32 bits of FNV-1a)              │
│    seq_num: u32 (entity sequence number, u32::MAX = empty slot)  │
│                                                                  │
│  Lookup procedure:                                               │
│    1. Compute FNV-1a hash of entity_id string                   │
│    2. slot = hash % slot_count                                   │
│    3. Compare fingerprint at slot with hash >> 32                │
│    4. If match: read entity at entities.eidx[seq_num * 128]      │
│       Verify full ID equality via stab[full_id_offset..+len]     │
│    5. If mismatch: linear probe to next slot                     │
│                                                                  │
│  With load factor 0.67, average probes = 1.5                     │
│  50K entities, 75K slots x 8 bytes = 585 KiB                    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. Query Paths: How Questions Get Answered {#6-query-paths}

### 6.1 "Give me entity X's callers" (fan-in query)

```
User query: "Who calls server::handle_message?"

Step 1: Resolve name to seq_num                              Cost
  hash("rust:fn:server::handle_message") -> slot 4281         ~1 ns
  hidx[4281] -> (fingerprint_match, seq_num: 742)             8 bytes
  Verify: stab[entities.eidx[742].full_id_offset..+len]       ~50 bytes

Step 2: Read fan-in offset                                    Cost
  entities.eidx[742].fan_in_start -> 3850                     4 bytes
  entities.eidx[742].fan_in_count -> 5                        4 bytes

Step 3: Read caller edges                                     Cost
  erev[3850..3855] -> 5 edge entries                          160 bytes

Step 4: Resolve caller names                                  Cost
  For each edge.src_seq: entities.eidx[src_seq].name_offset   5 * 128 bytes
  stab[name_offset..+len] for each                            ~200 bytes

Total I/O: ~1.2 KiB, all mmap()-ed, no syscalls
Total cache lines touched: ~15
Latency: <1 microsecond (hot cache), <50 microseconds (cold)
```

### 6.2 "Give me boundary Y's metrics" (point lookup)

```
User query: "Show me server/shard/ metrics"

Step 1: Resolve boundary name to seq_num
  Linear scan of boundaries.bidx (small: ~500 entries)         ~62.5 KiB
  OR: secondary hash index on boundary paths (if needed)

Step 2: Read boundary entry
  boundaries.bidx[seq_num * 128] -> 128-byte entry             128 bytes
  All metrics are in cache line 2 of the entry                 64 bytes

Total I/O: 128 bytes after resolution
All metrics (cohesion, coupling, fan_in, fan_out, etc.)
are in a single contiguous read.
```

### 6.3 "Blast radius: what is affected within 2 hops of X?"

```
User query: "blast_radius(server::handle_message, hops=2)"

Step 1: Resolve X to seq_num (same as 6.1 Step 1)             ~100 bytes

Step 2: 1-hop expansion (fan-out)
  fanout[742] -> (start: 2100, count: 8)                       8 bytes
  efwd[2100..2108] -> 8 edges, extract dst_seq values          256 bytes

Step 3: 2-hop expansion (fan-out for each 1-hop target)
  For each of 8 dst_seq values:
    fanout[dst_seq] -> (start, count)                          8 * 8 = 64 bytes
    efwd[start..start+count] -> edges                          ~1 KiB est.

Step 4: Deduplicate, fetch entity details
  Unique entities in 2-hop set (est. 30-50)
  entities.eidx[seq * 128] for each                            ~5 KiB

Total I/O: ~7 KiB
BFS is a sequential scan of contiguous edge ranges.
No recursion into variable-length data structures.
```

### 6.4 "Boundary coupling between A and B"

```
User query: "coupling(server/shard/, common/)"

Step 1: Resolve both boundary names to seq_nums
  shard_seq, common_seq                                        ~256 bytes

Step 2: Scan boundary_edges.bedg for matching pair
  For shard's boundary edges:
    bedg[shard.boundary_edge_start..+count]                    ~20 entries
    Find entry where dst_boundary_seq == common_seq            ~960 bytes

Step 3: Read the matching entry
  48 bytes: edge_count, file_pairs, distinct_items,
  import_breadth, import_spread, crossing_type                 48 bytes

All coupling metrics in one 48-byte read.
```

### 6.5 "Smart context with token budget"

```
User query: "Give me the most important entities around X, 4000 tokens max"

Step 1: Resolve X, get PPR scores (query-time computation)
Step 2: Sort 1-hop and 2-hop entities by PPR * edge_weight
Step 3: Greedily add entities to budget:
  For each candidate:
    word_count from entities.eidx[seq].word_count              2 bytes per entity
    tokens = word_count * 1.3
    If fits in budget: include
  Return entity list with source snippets from .edat

The entity index contains word_count, so budget checking
requires only the index file -- no source file access until
the final entity set is selected.
```

### 6.6 "All entities in community C, ranked by PageRank"

```
This is a SCAN query, not a point lookup. The entity index
is dense and sorted by seq_num, not by community.

Option A: Linear scan of entities.eidx
  Read cache line 2 (offset 64) of each entity entry
  Filter: community_id == C
  Sort by pagerank
  50K entities = 50K * 64 bytes = 3.1 MiB scan
  With mmap() and sequential access: ~2ms

Option B: Precomputed community membership file
  community_members.u32: for each community, a list of
  entity seq_nums belonging to it.
  This is a secondary index that avoids the full scan.
  For a community with 50 entities: 200 bytes.

Option A is fast enough for ISG-scale graphs.
Option B is worth adding if profiling shows the scan is a bottleneck.
```

---

## 7. Hot/Cold Separation {#7-hotcold-separation}

### 7.1 The Temperature Model

```
┌──────────────────────────────────────────────────────────────────────┐
│  DATA TEMPERATURE MODEL                                               │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  HOT (always mmap()-ed, accessed on every interaction):               │
│  ─────────────────────────────────────────────────────                │
│  • entities.eidx     (6.1 MiB for 50K entities)                      │
│  • edges_fanout.eoff (390 KiB)                                       │
│  • edges_fanin.eoff  (390 KiB)                                       │
│  • boundaries.bidx   (62.5 KiB for 500 boundaries)                   │
│  • strings.stab      (~500 KiB)                                      │
│  • entities.hidx     (585 KiB)                                       │
│  ───────────────────                                                  │
│  Total hot: ~8 MiB (fits entirely in L3 cache)                       │
│                                                                       │
│  WARM (mmap()-ed but only page-faulted on graph traversal):           │
│  ─────────────────────────────────────────────────────                │
│  • edges_forward.efwd   (6.1 MiB for 200K edges)                     │
│  • edges_reverse.erev   (6.1 MiB for 200K edges)                     │
│  • boundary_edges.bedg  (93.75 KiB)                                  │
│  ───────────────────                                                  │
│  Total warm: ~12.3 MiB                                                │
│                                                                       │
│  COLD (loaded on demand, never mmap()-ed in full):                    │
│  ─────────────────────────────────────────────────                    │
│  • entities.edat      (signatures, doc comments: ~5-20 MiB)          │
│  • deep/cfg.dat       (CFG blocks for all functions: ~10-50 MiB)     │
│  • deep/polonius.dat  (borrow facts: ~5-30 MiB)                      │
│  • deep/dataflow.dat  (dataflow facts: ~5-20 MiB)                    │
│  • Source files (on disk, read via file_path + line range)            │
│  ───────────────────                                                  │
│  Total cold: ~25-120 MiB (only paged in per-entity on demand)        │
│                                                                       │
│  FROZEN (computed once, cached per variant):                          │
│  ─────────────────────────────────────────                            │
│  • variant_consequences/ (metric deltas per variant)                  │
│  ───────────────────                                                  │
│  Size: ~100 KiB per variant (small)                                   │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

### 7.2 Why This Split Matters

The hot data (~8 MiB) is what powers every user interaction: listing entities, ranking by
PageRank, traversing call graphs, showing boundary metrics. By keeping this data in dense,
fixed-size, mmap()-ed files, the dominant cost of these operations is cache-line reads -- no
syscalls, no allocations, no deserialization.

The cold data (CFG, Polonius, dataflow) is accessed only when a user clicks "CFG" or "Borrows"
on a specific function. Loading it on demand means it never competes with hot data for cache
space. The deep index files (cfg.idx, polonius.idx, dataflow.idx) are small offset tables (one
entry per entity) that enable O(1) lookup into the data files:

```
Deep Data Index Entry (16 bytes, Iggy-style):
  entity_seq: u32
  data_offset: u32 (start in .dat file)
  data_end: u32 (end in .dat file, Iggy's END-offset trick)
  _reserved: u32

To load CFG for entity 742:
  deep_idx[742] -> (offset: 84200, end: 85600)
  read(cfg.dat, 84200, 1400 bytes) -> CFG data
```

### 7.3 The Metrics Directory: Iggy's Purest Expression

The `metrics/` directory is where the Iggy format-unification principle reaches its purest form.
Each file is a dense array of one value type, one entry per entity, in sequence order:

```
pagerank.f64:  [f64; entity_count]   Entity N's PageRank = file[N*8..N*8+8]
kcore.u32:     [u32; entity_count]   Entity N's k-core   = file[N*4..N*4+4]
community.u32: [u32; entity_count]   Entity N's community = file[N*4..N*4+4]
indegree.u32:  [u32; entity_count]   Entity N's in-degree = file[N*4..N*4+4]
```

These files have **zero headers, zero framing, zero metadata**. The file IS the array. `mmap()`
the file, cast to `&[f64]`, and you have the PageRank vector. To update PageRank after a
re-computation: write a new file. The old file remains valid until replaced.

This design means the Python analytics sidecar can write these files directly from numpy arrays:
`pagerank_array.tofile("pagerank.f64")`. The Rust server reads them with zero parsing. Format
unification between Python and Rust, mediated by raw bytes.

For a 50K-entity graph:
- `pagerank.f64`: 390 KiB
- `kcore.u32`: 195 KiB
- All metric files combined: ~2 MiB

The entire metrics suite fits in L2 cache.

---

## 8. Incremental Re-Indexing via Segment Rotation {#8-incremental-re-indexing}

### 8.1 The Snapshot Model

Unlike Iggy, where new messages are appended to the current segment, the ISG has a fundamentally
different write pattern: a full re-index replaces the entire graph. This is because:

1. Changing one source file can add, remove, or modify many entities and edges
2. Graph metrics (PageRank, k-core, communities) are global -- changing one edge can affect every
   entity's score
3. The compiler must re-analyze the entire crate to produce correct MIR

The Iggy analog is **segment rotation at the snapshot level**: each re-index produces a new,
complete, immutable snapshot directory. The old snapshot remains valid and mmap()-ed until the
new one is ready. The switchover is atomic (update the `current` symlink).

```
Timeline:

T0: Initial index
    snapshots/00000000000000000001/ created
    current -> snapshots/00000000000000000001/

T1: User edits src/server.rs, triggers re-index
    snapshots/00000000000000000002/ written (background)
    current still -> snapshots/00000000000000000001/ (queries continue)

T2: Re-index complete
    current -> snapshots/00000000000000000002/ (atomic symlink update)
    snapshots/00000000000000000001/ can be garbage-collected
    (or kept for diffing: "what changed between indexes?")
```

### 8.2 Incremental Optimization: Content-Addressed Files

Full re-indexing is correct but expensive. For large codebases, we can optimize by detecting
unchanged files and reusing their entity/edge data from the previous snapshot.

The key: **file-level content hashing** (SHA-256 or BLAKE3).

```
Incremental re-index algorithm:

1. Hash every source file in the workspace
2. Compare with hashes from the previous snapshot's manifest
3. For unchanged files:
   - Copy entity entries from previous snapshot
   - Copy edge entries where BOTH endpoints are in unchanged files
4. For changed files:
   - Re-extract entities and edges via tree-sitter or MIR
   - Assign new seq_nums (appended after copied entities)
5. For removed files:
   - Exclude from new snapshot (their seq_nums are not reused)
6. Rewrite ALL index files (they are small and fast to write)
7. Recompute ONLY affected graph metrics:
   - PageRank: full recompute (global metric, ~100ms for 50K entities)
   - k-core: full recompute (~50ms)
   - Communities: incremental Leiden if available, else full (~200ms)
   - In-degree/out-degree: incremental (only changed entities)
   - Betweenness: full recompute (~500ms)
```

The crucial insight: even "incremental" re-indexing writes a **new, complete snapshot**. There
is no mutation of existing files. The incremental part is the analysis pipeline skipping
unchanged files, not the storage format supporting partial updates. This preserves the immutability
guarantee that makes mmap() safe.

### 8.3 Comparison with Iggy's Append-Only Model

```
┌──────────────────────────────────────────────────────────────────┐
│  IGGY vs ISG: WRITE PATTERNS                                     │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Iggy:                                                            │
│  • Messages arrive continuously                                  │
│  • Each message is appended to the current segment               │
│  • Old segments are immutable                                    │
│  • GC = delete old segments (TTL or size limit)                  │
│                                                                   │
│  ISG:                                                             │
│  • Graphs arrive as complete snapshots (re-index)                │
│  • Each snapshot is a new directory of immutable files            │
│  • Old snapshots are immutable                                   │
│  • GC = delete old snapshots (keep last N)                       │
│                                                                   │
│  The analog:                                                      │
│  • Iggy segment ≈ ISG snapshot                                   │
│  • Iggy segment rotation ≈ ISG re-index                          │
│  • Iggy GC (delete old segments) ≈ ISG GC (delete old snapshots)│
│  • Both achieve immutability after creation                      │
│  • Both allow concurrent reads on old data during writes         │
│                                                                   │
│  The difference:                                                  │
│  • Iggy: many small appends within a segment                     │
│  • ISG: one large bulk write per snapshot                        │
│  • This means ISG can afford to sort everything during write     │
│    (edges by src, edges by dst) because the write is infrequent  │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

---

## 9. Variant Overlays as Append-Only Logs {#9-variant-overlays}

This is where the Iggy model transfers most elegantly to the ISG. Variant deltas ARE an
append-only stream: each delta is a structural change operation with a timestamp, and variant
history is a log of these operations. The Iggy pattern applies directly.

### 9.1 Variant Delta Log (variant_deltas.vlog)

Each variant delta operation is a variable-size record in an append-only log file, exactly like
Iggy messages:

```
┌─────────────────────────────────────────────────────────────────┐
│  VARIANT DELTA RECORD (variable size)                            │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│   0   │  4 bytes │ variant_id: u32                               │
│   4   │  4 bytes │ delta_seq: u32 (sequence within this variant) │
│   8   │  8 bytes │ timestamp: u64 (unix micros)                  │
│  16   │  1 byte  │ op: u8 (add_edge=0, remove_edge=1,           │
│       │          │   change_edge_kind=2, add_entity=3,          │
│       │          │   remove_entity=4)                           │
│  17   │  1 byte  │ edge_kind: u8                                 │
│  18   │  1 byte  │ old_edge_kind: u8 (for change_edge_kind)     │
│  19   │  1 byte  │ confidence: u8 (always "proposed")            │
│  20   │  4 bytes │ src_name_offset: u32 (into variant string     │
│       │          │   pool or main .stab)                        │
│  24   │  2 bytes │ src_name_length: u16                          │
│  26   │  4 bytes │ dst_name_offset: u32                          │
│  30   │  2 bytes │ dst_name_length: u16                          │
│  32   │  4 bytes │ rationale_offset: u32 (into .vlog string area)│
│  36   │  2 bytes │ rationale_length: u16                         │
│  38   │  2 bytes │ _pad: u16                                     │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Fixed header: 40 bytes                                          │
│  Variable data: src_name + dst_name + rationale strings          │
│  (appended after header in the same .vlog file)                  │
└─────────────────────────────────────────────────────────────────┘
```

### 9.2 Variant Index (variant_index.vidx)

Each variant has a fixed 64-byte header entry in the variant index file, exactly like Iggy's
index entries:

```
┌─────────────────────────────────────────────────────────────────┐
│  VARIANT INDEX ENTRY (64 bytes = 1 cache line)                   │
├───────┬──────────┬──────────────────────────────────────────────┤
│ Offset│ Size     │ Field                                        │
├───────┼──────────┼──────────────────────────────────────────────┤
│   0   │  4 bytes │ variant_id: u32                               │
│   4   │  4 bytes │ name_offset: u32 (into variant string pool)  │
│   8   │  2 bytes │ name_length: u16                              │
│  10   │  2 bytes │ delta_count: u16                              │
│  12   │  8 bytes │ created_at: u64 (unix micros)                 │
│  20   │  8 bytes │ modified_at: u64                               │
│  28   │  4 bytes │ first_delta_offset: u32 (into .vlog)          │
│  32   │  4 bytes │ last_delta_end: u32 (END offset in .vlog)     │
│  36   │  4 bytes │ consequence_offset: u32 (into consequences/)  │
│  40   │  1 byte  │ status: u8 (active=0, archived=1, deleted=2)  │
│  41   │  4 bytes │ base_snapshot_id: u32 (which snapshot this    │
│       │          │   variant was created against)                │
│  45   │ 19 bytes │ _reserved: [u8; 19]                           │
├───────┴──────────┴──────────────────────────────────────────────┤
│  Total: 64 bytes (1 cache line, Iggy-style)                      │
│  Variant N is at byte: N * 64                                    │
│  5 active variants x 64 bytes = 320 bytes (trivial)             │
└─────────────────────────────────────────────────────────────────┘
```

### 9.3 Applying a Variant at Query Time

When a query includes `?variant=option-1`, the system:

1. Reads the base snapshot's edges (mmap()-ed, zero-cost)
2. Reads the variant's deltas from .vlog (small, typically <1 KiB)
3. Applies deltas to produce a modified edge set (in memory):
   - `add_edge`: insert into edge list
   - `remove_edge`: mark matching edge as removed
   - `change_edge_kind`: modify the matching edge's kind field
4. Recomputes the requested metric on the modified edge set
5. Returns the diff between base and variant metrics

For small variants (5-20 deltas, which covers most architectural what-if scenarios), step 3 is
dominated by the search for matching edges. With the sorted edge files and fan-out tables, this
is O(fan_out(src)) per delta -- typically <100 edges. Total cost: microseconds.

### 9.4 Variant Consequences Cache

Recomputing Leiden communities and PageRank on every variant query would be expensive (~200ms).
Instead, consequences are computed once when the variant is created or modified, and cached in
the `variant_consequences/` directory:

```
variant_consequences/
├── 0001_pagerank_delta.f64        # [f64; entity_count] -- delta from base
├── 0001_community_delta.u32       # [u32; entity_count] -- new community IDs
├── 0001_kcore_delta.u32           # [u32; entity_count] -- new k-core values
├── 0001_scc_changes.bin           # List of new/broken SCCs
└── 0001_summary.bin               # Fixed 64-byte summary of key changes
```

These files follow the same Iggy principle as the metrics directory: dense arrays, one value per
entity, zero headers, zero framing. To get entity N's PageRank delta for variant 1:
`0001_pagerank_delta.f64[N * 8..N * 8 + 8]`.

### 9.5 Why Variants Are the Best Iggy Analog

Variants are the one part of the ISG that IS genuinely append-only and ordered:

| Property | Iggy Messages | Variant Deltas |
|---|---|---|
| Append-only | Yes | Yes (new deltas are appended to .vlog) |
| Ordered | By offset | By (variant_id, delta_seq) |
| Variable-size | Yes | Yes (rationale strings vary) |
| Immutable after write | Yes (sealed segments) | Yes (deltas are never modified) |
| Index for O(1) lookup | 16-byte entries | 64-byte entries (richer metadata) |
| END offsets for range reads | Yes | Yes (first_delta_offset, last_delta_end) |

The Iggy model is a natural fit. Variant creation is an append to the .vlog. Variant deletion is
a status flag change in the .vidx entry (not physical deletion). Variant comparison is reading
two ranges from the .vlog and diffing the cached consequences.

---

## 10. Comparison with SQLite {#10-comparison-with-sqlite}

### 10.1 Where the ISG Format Wins

| Operation | ISG Format | SQLite | Winner |
|---|---|---|---|
| Entity lookup by seq_num | O(1), zero-copy mmap() | B-tree traverse, row deserialization | ISG |
| Entity metrics scan (list all by PageRank) | Sequential cache-line walk, ~2ms | B-tree scan + row decode, ~5-10ms | ISG |
| Fan-out query (entity's callees) | 2 reads, contiguous, zero-copy | JOIN on edges table, row decode | ISG |
| Fan-in query (entity's callers) | 2 reads, contiguous, zero-copy | JOIN on edges table (needs index), row decode | ISG |
| Boundary metrics (point lookup) | O(1), 128 bytes | B-tree traverse, row decode | ISG |
| Memory footprint for hot path | ~8 MiB (mmap, shared with page cache) | ~50 MiB (SQLite cache, private) | ISG |
| Startup time | mmap() calls only (~1ms) | Open DB, load page cache (~50-100ms) | ISG |
| Concurrent reads | Lock-free (immutable mmap) | WAL mode, still some contention | ISG |

### 10.2 Where SQLite Wins

| Operation | ISG Format | SQLite | Winner |
|---|---|---|---|
| Ad-hoc queries (GROUP BY, HAVING, subqueries) | Not supported -- queries are hardcoded in the server | Full SQL engine | SQLite |
| Schema evolution (add a column) | Requires format version bump, migration code | ALTER TABLE | SQLite |
| Partial updates (edit one entity) | Must rewrite entire snapshot | UPDATE one row | SQLite |
| Complex joins (e.g., symmetry detection) | Must be implemented in Rust code | SQL query | SQLite |
| FTS5 search | Not supported -- needs a separate search index | Built-in FTS5 | SQLite |
| Debugging/inspection | Custom tooling needed | sqlite3 CLI, DB Browser | SQLite |
| Variant application (overlay on base graph) | Manual edge-set merging in Rust | SQL VIEW or UNION ALL | SQLite |
| Aggregation queries (boundary metrics from raw edges) | Precomputed only -- cannot derive at query time | Real-time GROUP BY | SQLite |
| Developer productivity | Low-level byte manipulation | High-level SQL | SQLite |
| Correctness guarantees | Manual checksum validation | ACID transactions, WAL | SQLite |

### 10.3 The Honest Assessment

The ISG format wins on **read performance for known query patterns**. Every query that the
Parseltongue HTTP server needs to answer can be expressed as a fixed sequence of offset
calculations and contiguous reads. For these queries, the ISG format is 5-50x faster than SQLite
because it eliminates:

1. B-tree traversal (~3-4 page reads per lookup)
2. Row deserialization (SQLite stores rows as variable-length records with type headers)
3. SQL parsing and planning (even prepared statements have overhead)
4. Page cache management (SQLite maintains its own, separate from the OS page cache)

SQLite wins on **flexibility, correctness, and developer productivity**. Every new query pattern
in the ISG format requires writing Rust code to compute byte offsets and interpret raw bytes.
In SQLite, it requires writing a SQL query. For a product that is still evolving its query
patterns (new reading modes, new analysis algorithms, new variant comparison views), SQLite's
flexibility is extremely valuable.

The key question is: **are the ISG format's performance wins on the critical path?**

For a desktop app serving a single user via HTTP:
- Entity lookup: SQLite takes ~10us, ISG format takes ~1us. Both are invisible to the user.
- Fan-out query: SQLite takes ~50us, ISG format takes ~5us. Both are invisible.
- Full entity scan (50K entities by PageRank): SQLite takes ~10ms, ISG format takes ~2ms.
  SQLite is fine. ISG is better but not necessary.
- PPR computation (the actual bottleneck): ~10-50ms regardless of storage format. Storage is
  not the bottleneck.

**Verdict: For Parseltongue v3, SQLite is the pragmatic choice. The ISG format is the
performance ceiling.**

---

## 11. The Hybrid Proposal {#11-the-hybrid-proposal}

Given the analysis in Section 10, the right answer is neither pure ISG format nor pure SQLite.
It is a hybrid that uses each for what it does best.

### 11.1 The Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│  HYBRID STORAGE ARCHITECTURE                                          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │  SQLite (Turso/libSQL)                                         │  │
│  │  ─────────────────────                                          │  │
│  │                                                                 │  │
│  │  • Entity table (all fields, including metrics)                 │  │
│  │  • Edge table (all fields, with indexes on src_id and dst_id)   │  │
│  │  • Boundary table (all fields, with metrics)                    │  │
│  │  • Boundary_edges table (all fields)                            │  │
│  │  • Variant table (metadata)                                     │  │
│  │  • Variant_deltas table (all deltas)                            │  │
│  │  • FTS5 tables (for search: name, signature, doc_comment)       │  │
│  │  • Reading history, bookmarks, tours, UI state                  │  │
│  │                                                                 │  │
│  │  This is the source of truth. All writes go here.               │  │
│  │  Ad-hoc queries, GROUP BY, JOINs, schema evolution -- all here. │  │
│  │                                                                 │  │
│  └──────────────────────────────┬──────────────────────────────────┘  │
│                                 │                                     │
│                    Materialization pass (at index time)                │
│                                 │                                     │
│                                 ▼                                     │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │  Iggy-Style Materialized Files (read-only acceleration layer)  │  │
│  │  ──────────────────────────────────────────────────────────────  │  │
│  │                                                                 │  │
│  │  • metrics/pagerank.f64    (dense array, mmap()-ed)             │  │
│  │  • metrics/kcore.u32       (dense array, mmap()-ed)             │  │
│  │  • metrics/community.u32   (dense array, mmap()-ed)             │  │
│  │  • metrics/indegree.u32    (dense array, mmap()-ed)             │  │
│  │  • metrics/outdegree.u32   (dense array, mmap()-ed)             │  │
│  │  • edges_forward.efwd      (sorted edge index, mmap()-ed)      │  │
│  │  • edges_reverse.erev      (sorted edge index, mmap()-ed)      │  │
│  │  • edges_fanout.eoff       (fan-out offset table, mmap()-ed)   │  │
│  │  • edges_fanin.eoff        (fan-in offset table, mmap()-ed)    │  │
│  │                                                                 │  │
│  │  These files are DERIVED from SQLite at index time.             │  │
│  │  They accelerate the hot-path queries (graph traversal,         │  │
│  │  metric ranking, fan-out/fan-in).                               │  │
│  │  They can be regenerated from SQLite if corrupted or missing.   │  │
│  │  They are never the source of truth.                            │  │
│  │                                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                       │
│  Query routing:                                                       │
│  ─────────────                                                        │
│  • Fan-out/fan-in queries → Iggy-style files (O(1), mmap)            │
│  • Metric ranking/filtering → Iggy-style files (sequential scan)     │
│  • BFS/DFS traversal → Iggy-style files (cache-friendly)            │
│  • PPR computation → Iggy-style files (adjacency list access)        │
│  • Search queries → SQLite FTS5                                       │
│  • Boundary coupling queries → SQLite (GROUP BY)                     │
│  • Variant application → SQLite (UNION ALL / VIEW)                   │
│  • Ad-hoc/new queries → SQLite (full SQL)                            │
│  • Symmetry detection → SQLite (GROUP BY + HAVING)                   │
│  • Schema evolution → SQLite (ALTER TABLE)                            │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

### 11.2 The Materialization Pass

After every re-index (when SQLite tables are fully populated), a materialization pass generates
the Iggy-style acceleration files:

```rust
fn materialize_acceleration_files(db: &Connection, output_dir: &Path) -> Result<()> {
    // 1. Assign stable sequence numbers to entities (sorted by entity_id for determinism)
    let entities: Vec<EntityRow> = db.query("SELECT * FROM entities ORDER BY id")?;

    // 2. Write metric arrays (purest Iggy-style: raw f64/u32 arrays)
    let pageranks: Vec<f64> = entities.iter().map(|e| e.pagerank).collect();
    std::fs::write(output_dir.join("metrics/pagerank.f64"),
                   bytemuck::cast_slice(&pageranks))?;

    // 3. Write sorted edge files (forward and reverse)
    let edges: Vec<EdgeRow> = db.query(
        "SELECT * FROM edges ORDER BY src_id, dst_id")?;
    let edge_entries: Vec<EdgeEntry> = edges.iter().map(|e| {
        EdgeEntry {
            src_seq: entity_id_to_seq[&e.src_id],
            dst_seq: entity_id_to_seq[&e.dst_id],
            edge_kind: e.edge_kind as u8,
            // ... other fields
        }
    }).collect();
    std::fs::write(output_dir.join("edges_forward.efwd"),
                   bytemuck::cast_slice(&edge_entries))?;

    // 4. Build fan-out offset table
    let mut fanout: Vec<FanOutEntry> = vec![FanOutEntry { start: 0, count: 0 }; entity_count];
    // ... compute starts and counts from sorted edges

    // 5. Write reverse edges (same data, sorted by dst)
    let mut reverse_entries = edge_entries.clone();
    reverse_entries.sort_by_key(|e| (e.dst_seq, e.src_seq));
    std::fs::write(output_dir.join("edges_reverse.erev"),
                   bytemuck::cast_slice(&reverse_entries))?;

    // 6. Build fan-in offset table
    // ... same pattern
}
```

### 11.3 Why This Hybrid Works

1. **SQLite remains the source of truth.** All writes, schema changes, and data integrity go
   through SQLite. ACID guarantees. WAL mode. No data corruption from partial mmap writes.

2. **Iggy-style files accelerate the hot path.** Graph traversal, metric ranking, and BFS/DFS --
   the operations that run on every user click -- use the mmap()-ed files. Zero deserialization.
   Cache-line-friendly access patterns.

3. **Files are disposable.** The Iggy-style files can be deleted and regenerated from SQLite.
   They are a materialized cache, not a source of truth. This means format migrations are free:
   change the binary format, regenerate from SQLite.

4. **Progressive adoption.** Start with SQLite-only (correct, fast enough). Add materialized
   files for the queries that profiling shows are bottlenecks. This avoids premature optimization
   while preserving the option for Iggy-level performance where it matters.

5. **The Python sidecar writes the metric files.** The analytics sidecar (python-igraph,
   NetworkKit) computes PageRank, k-core, Leiden communities, etc. It writes the results both to
   SQLite (for queryability) and to dense metric files (for mmap performance). The file write is
   a single `numpy.tofile()` call -- trivial.

### 11.4 What to Materialize First

Based on the query patterns in the v3 PRD:

| Priority | File | Query Pattern | Why |
|---|---|---|---|
| P0 | `edges_forward.efwd` + `edges_fanout.eoff` | Fan-out (callees) | Every entity click triggers a 1-hop expansion |
| P0 | `edges_reverse.erev` + `edges_fanin.eoff` | Fan-in (callers) | Second most common graph query |
| P0 | `metrics/pagerank.f64` | Ranking entities by importance | Used in every entity list, every focus lens computation |
| P1 | `metrics/community.u32` | Community membership filtering | Used in architecture map, Leiden overlay |
| P1 | `metrics/kcore.u32` | Core/periphery filtering | Used in hotspot heatmap, architecture depth |
| P2 | `metrics/indegree.u32`, `outdegree.u32` | Degree-based filtering | Used in hub detection, god-object warnings |
| P2 | `metrics/betweenness.f64` | Bridge detection | Used in chokepoint analysis |

Start with P0. Add P1 when the architecture map ships. Add P2 when post-dive analysis ships.

---

## 12. The Cascading Design for a Code Graph {#12-the-cascading-design}

### 12.1 Can We Find an Iggy-Like Cascade?

Iggy's cascade works because messages have one dimension (offset) and one access pattern
(sequential read from a position). A code graph has multiple dimensions and multiple access
patterns. No single cascade can optimize all of them.

But we CAN find a cascade for the **dominant access pattern**: "focus on entity X, show its
neighborhood, rank by importance." This is the Semantic Focus Lens -- the navigation model that
powers every user interaction.

### 12.2 The Focus Lens Cascade

```
Entity sequence numbers are dense integers (0..N)
    └─► Entity metrics are dense arrays (pagerank.f64, kcore.u32)
         └─► Metric access is O(1): file[seq * sizeof(T)]
              └─► On-disk = in-memory (mmap, no deserialization)
                   └─► Sorted edge files enable O(1) fan-out/fan-in
                        └─► Fan-out offset tables: entity N's edges start at eoff[N]
                             └─► BFS traversal = sequential reads of contiguous edge ranges
                                  └─► PPR = sparse vector dot products on metric arrays
                                       └─► Focus lens = PPR scores + BFS distance + metrics
                                            └─► All data for one focus transition: ~2-5 KiB
```

This cascade is weaker than Iggy's (it has more steps, and PPR is the bottleneck, not I/O), but
it shares the key property: **each design choice enables the next**.

- Dense seq_nums enable dense metric arrays
- Dense metric arrays enable zero-deserialization mmap
- Sorted edges with offset tables enable O(1) adjacency access
- O(1) adjacency access enables fast BFS and PPR
- Fast BFS and PPR enable sub-50ms focus lens transitions

Remove any one choice and the cascade weakens. Use string IDs instead of seq_nums? Metric
arrays need hash lookups. Use unsorted edges? Fan-out queries need full scans. Use row-based
storage? Metric scans touch 10x more cache lines.

### 12.3 The Second Cascade: Boundary Metrics

```
Boundary entries are dense, fixed-size (128 bytes)
    └─► All metrics in cache line 2 (cohesion, coupling, fan_in, fan_out)
         └─► Boundary N's metrics: bidx[N * 128 + 64..N * 128 + 128]
              └─► Boundary edges are sorted by (src_boundary, dst_boundary)
                   └─► Coupling query = contiguous range read
                        └─► Variant boundary metrics = base metrics + delta recomputation
```

This cascade enables the typed boundary aggregation model from the thesis: "server/ [crate,
228 files, pub_surface=40] depends on common/ [crate, leaf, pub_surface=80] -- 330 edges,
CROSS-CRATE" -- all from contiguous reads of fixed-size entries.

### 12.4 The Third Cascade: Variant Consequences

```
Variant deltas are append-only records in .vlog
    └─► Variant index has fixed 64-byte entries (Iggy-style)
         └─► Delta range: vlog[first_offset..last_end]
              └─► Consequence cache: dense metric arrays per variant
                   └─► Consequence diff = base_metric[N] - variant_metric[N]
                        └─► On-disk = in-memory (mmap, no deserialization)
```

This cascade enables "Variant A reduces coupling by 26%" as a single subtraction on mmap()-ed
arrays.

### 12.5 What the Three Cascades Share

All three cascades share the same foundation:

1. **Dense integer keys** (seq_nums for entities, boundaries, variants)
2. **Fixed-size entries** in index files (128, 128, 64 bytes respectively)
3. **Dense metric arrays** (one value per entity/boundary, indexed by seq_num)
4. **Sorted edge files** with offset tables for O(1) range access
5. **mmap() as the read API** (on-disk = in-memory, zero deserialization)
6. **Immutable snapshots** (write-once, read-many, no locking)

This is the ISG's version of Iggy's cascade. It is not as tight as Iggy's (because a graph is
inherently more complex than a stream), but it shares the same fundamental principle: **eliminate
every layer of indirection between the disk and the CPU for the dominant access patterns.**

---

## 13. Confidence and Caveats {#13-confidence-and-caveats}

### Confidence Levels

**High confidence:**
- The hybrid architecture (SQLite + materialized Iggy-style files) is the right approach for
  Parseltongue v3. It combines SQLite's correctness and flexibility with Iggy's read performance
  for the hot path.
- Dense metric arrays (one file per metric, mmap()-ed) are unconditionally correct for the ISG.
  They are trivial to generate, trivial to consume, and maximally cache-friendly.
- Dual sorted edge files (forward + reverse) with fan-out/fan-in offset tables are the right
  structure for graph traversal queries. The 2x storage cost is negligible (~12 MiB).
- Variant deltas as append-only logs are a natural Iggy analog and the right storage model for
  architectural what-if analysis.

**Medium confidence:**
- The 128-byte entity index entry size. This may need adjustment as the ISG schema evolves. The
  principle (fixed-size, cache-line-aligned) is solid; the exact field layout will need iteration.
- The hash index (.hidx) for entity ID lookup. FNV-1a with open addressing is simple and fast,
  but if the key distribution is pathological (e.g., many entities differing only in the last
  character), probe chains could degrade. A minimal perfect hash (built at index time) would be
  better but more complex.
- The snapshot-based write model. For very large codebases (100K+ entities), writing a complete
  new snapshot on every re-index may take too long. The incremental optimization (content-addressed
  file hashing) helps but has not been validated at scale.

**Lower confidence:**
- Whether the full Iggy-style entity index (.eidx) is worth building for v3. The hybrid
  proposal's metric files and edge files capture 80% of the performance benefit with 20% of the
  complexity. The full entity index may be premature optimization for a desktop app serving one
  user.
- Whether the materialization pass will be fast enough to run after every re-index without
  noticeable delay. For 50K entities + 200K edges, it should take ~100-500ms. For larger graphs,
  it may need to run in the background.

### Key Assumptions

1. **ISG scale**: up to ~50K entities and ~200K edges. The format is designed for this scale.
   At 500K entities, the entity index alone would be 61 MiB, which is still manageable but
   pushes against L3 cache limits.

2. **Read-dominated workload**: the graph is written once (at index time) and read many times
   (during user interaction). This is why immutable, mmap()-able files work well. If the workload
   were write-heavy, the snapshot model would be a bottleneck.

3. **Single-machine deployment**: Parseltongue runs as a desktop app on one machine. The format
   does not need to support distributed access, replication, or network-transparent storage.

4. **Little-endian architecture**: all modern macOS machines (Apple Silicon) are little-endian.
   The `from_le_bytes()` no-op assumption holds.

### Areas for Independent Verification

- **mmap() behavior on macOS/Apple Silicon**: verify that mmap()-ing many small files (metrics
  directory) does not cause excessive TLB misses. If it does, consider merging metric files into
  one large file.

- **SQLite FTS5 performance for the search step**: FTS5 with RRF fusion (exact + fuzzy + trigram)
  should be benchmarked against the target latency (<10ms). If FTS5 is too slow, a standalone
  search index (tantivy) may be needed.

- **Python sidecar latency for metric recomputation**: PageRank on 50K nodes via python-igraph
  should be ~100ms, but this should be profiled on real codebases.

- **Variant consequence caching invalidation**: when the base snapshot changes (re-index),
  cached variant consequences become stale. The system must detect this (base_snapshot_id in the
  variant header) and recompute. Verify that this does not create a surprising UX where variant
  metrics change after a re-index even though the variant itself was not modified.

---

## Appendix A: Size Estimates for Real Codebases

```
┌──────────────────────────────────────────────────────────────────────┐
│  SIZE ESTIMATES BY CODEBASE SCALE                                     │
├────────────────────┬──────────────────────────────────────────────────┤
│                    │  Small         Medium        Large               │
│                    │  (1K entities) (10K entities)(50K entities)      │
├────────────────────┼──────────────────────────────────────────────────┤
│ entities.eidx      │  125 KiB       1.2 MiB       6.1 MiB            │
│ edges_forward.efwd │  62 KiB        625 KiB       6.1 MiB            │
│ edges_reverse.erev │  62 KiB        625 KiB       6.1 MiB            │
│ edges_fanout.eoff  │  7.8 KiB       78 KiB        390 KiB            │
│ edges_fanin.eoff   │  7.8 KiB       78 KiB        390 KiB            │
│ boundaries.bidx    │  6.25 KiB      25 KiB        62.5 KiB           │
│ strings.stab       │  50 KiB        200 KiB       500 KiB            │
│ entities.hidx      │  11.7 KiB      117 KiB       585 KiB            │
│ metrics/ (all)     │  39 KiB        390 KiB       1.95 MiB           │
├────────────────────┼──────────────────────────────────────────────────┤
│ Total HOT          │  ~370 KiB      ~3.3 MiB      ~22 MiB            │
│ Total WARM (edges) │  ~130 KiB      ~1.3 MiB      ~12.5 MiB          │
│ Total COLD (deep)  │  ~1-5 MiB      ~10-30 MiB    ~50-100 MiB        │
├────────────────────┼──────────────────────────────────────────────────┤
│ Fits in L2 cache?  │  HOT: Yes      HOT: No       HOT: No            │
│ Fits in L3 cache?  │  ALL: Yes      HOT+WARM: Yes HOT: Borderline    │
└────────────────────┴──────────────────────────────────────────────────┘
```

## Appendix B: The Iggy Lesson in One Sentence

The lesson from Iggy is not "use fixed-size index entries" or "use mmap()" or "align to cache
lines." Those are techniques. The lesson is:

**Design the storage format and the query patterns as one inseparable system, so that the format
makes the queries trivially cheap and the queries justify every byte of the format.**

For a message stream, that produces Iggy's cascade. For a code graph, it produces the hybrid of
SQLite (for flexibility) and Iggy-style materialized files (for the hot path). Different
problems, same principle: **the format serves the queries, the queries justify the format.**

---

*This document is a research exploration. Decisions about which parts to implement, and in what
order, should be driven by profiling real query patterns on real codebases, not by the
theoretical appeal of zero-copy mmap().*
