# Four architectures for a persistent graph walk engine

**The mmap'd dual-CSR format delivers 1-hop queries in ~100 nanoseconds — roughly 1,000x faster than any database-mediated approach and 5,000x faster than the Iggy streaming thesis.** For Parseltongue's static, read-optimized code dependency graph (50K–500K nodes, 500K–5M edges), the architecture choice collapses to a single question: how much of the graph walk should happen through raw pointer arithmetic versus through a query engine? The hybrid approach — CSR for topology, SQLite for metadata — emerges as the clear winner, validated by production systems like Neo4j, TigerGraph, and the now-archived Kùzu. The Iggy Graph Compiler thesis, while creative, is fundamentally mismatched: a streaming platform optimized for sequential consumption cannot efficiently serve random-access graph traversals.

This analysis builds on Parseltongue's existing Storage Architecture Analysis (September 2025), which recommended a phased SQLite → in-memory WAL → distributed hybrid roadmap. The current work reframes the question around a critical insight the earlier analysis missed: **for a static graph indexed once and queried many times, the entire persistence-vs-performance tradeoff disappears.** An mmap'd binary file *is* both persistent storage and in-memory access simultaneously — the OS page cache handles the boundary transparently.

---

## Option 1: Custom mmap'd binary format delivers near-hardware-limit speed

The Compressed Sparse Row (CSR) format stores a directed graph as two arrays: an **offsets array** of size V+1 (where `offsets[v]` marks where node v's neighbors begin) and an **edges array** of size E (contiguous destination node IDs). A "dual-CSR" adds a reverse copy, enabling both "who does X call?" and "who calls X?" in O(1) index lookups plus O(degree) sequential scans.

For 50K nodes and 500K edges using `u32` indices, the forward CSR consumes just **2.2 MB** — offsets at 200 KB plus edges at 2.0 MB. The full dual-CSR with node metadata and a string table totals approximately **11–15 MB**, small enough to fit entirely in L3 cache on modern CPUs. At the 500K-node / 5M-edge scale, the file grows to roughly **110–120 MB**, still trivially fitting in RAM.

The Rust implementation pattern is elegant. The `memmap2` crate (216+ million downloads, actively maintained) maps the file into the process address space with a single `unsafe` call. The `bytemuck` crate's `Pod` trait enables zero-copy reinterpretation of the mapped bytes as typed slices — `cast_slice::<u8, u32>(&mmap[offset..])` yields a `&[u32]` with no deserialization. Structs must be `#[repr(C)]` with no padding bytes, using fixed-size types (`u32`, `u64`, never `usize`). The alignment constraint is satisfied automatically since mmap returns page-aligned memory.

**Cold start is effectively instant.** The `mmap()` syscall itself takes ~10–100µs, creating a virtual memory area without reading any data. Pages fault in on demand at ~1–2µs per minor fault (page already in OS cache) or ~100–200µs per major fault (read from SSD). With `MAP_POPULATE` to prefault the entire 15 MB file from SSD, cold start takes **1–5ms**. Compare this to the 50–200ms required for deserializing and building an in-memory graph with `bincode` — a **10–100x advantage**.

The performance characteristics are extraordinary for small graphs. A **1-hop query** requires two array reads (offsets[x] and offsets[x+1]) plus a sequential scan of ~10 `u32` values. When cached in L1/L2, this completes in **~50–200 nanoseconds**. **BFS to depth 3** touching ~200 nodes runs in **~10–50µs**, with the entire offsets array (200 KB for 50K nodes) fitting in L2 cache. **Full PageRank** (20 iterations over 50K nodes / 500K edges) completes in approximately **100–120ms**, with each iteration scanning all edges at memory bandwidth. These numbers are consistent with academic benchmarks: the PCSR paper found CSR **2–10x faster** than adjacency lists for BFS and PageRank, while Ligra's CSR-based framework processes billion-edge graphs in seconds.

Production precedents exist for this approach. Cloudflare's `mmap-sync` uses mmap'd files with `rkyv` zero-copy deserialization for ML model serving at scale. The `epserde-rs` framework provides mmap-backed zero-copy data structures. GraphChi achieved PageRank on Twitter's 1.5B-edge graph in ~13 minutes using sharded CSR on SSD, and Ligra's mmap variant maps CSR graph data directly from files with the OS page cache handling eviction.

The risks are manageable. File corruption causes `SIGBUS` on access — mitigated by validating the header checksum on open. Endianness is native-only, which is fine for developer tools running on the same machine. Schema evolution requires version bumps but can be handled with reserved padding bytes in structs. The implementation requires roughly **3–4 person-weeks**: one week for the format specification and writer, one for the mmap reader and query API, one for BFS/PageRank algorithms and tests, and one for CLI tooling and edge cases.

| Metric | 50K nodes / 500K edges | 500K nodes / 5M edges |
|---|---|---|
| Cold start (SSD) | 1–5ms | 10–50ms |
| 1-hop latency | 50–200ns | 50–200ns |
| BFS depth-3 (~200 nodes) | 10–50µs | 10–50µs |
| PageRank (20 iterations) | 100–120ms | 1–1.5s |
| Disk footprint | ~11–15 MB | ~110–120 MB |
| Implementation effort | 3–4 person-weeks | — |

---

## Option 2: Embedded databases trade traversal speed for query convenience

Five embedded database options were evaluated: SQLite/libSQL, DuckDB with the DuckPGQ extension, Kùzu, redb, and CozoDB (Parseltongue's current backend). Each makes a fundamentally different tradeoff between query ergonomics and raw traversal performance.

**SQLite/libSQL** is the baseline. With an edge-list schema (`CREATE TABLE edges (src INTEGER, dst INTEGER)`) and composite B-tree indexes on `(src, kind)` and `(dst, kind)`, a 1-hop query completes in **<0.5ms** via a single indexed lookup. The `rusqlite` crate is battle-tested with millions of downloads. However, recursive CTEs for multi-hop traversals degrade rapidly: 2-hop queries take **1–10ms**, while BFS to depth 3 ranges from **10–100ms** depending on fan-out. The row-at-a-time execution model and string-based cycle detection (`WHERE NOT path LIKE '%' || node_id || '%'`) add substantial overhead per traversal step. PageRank is not natively supported and would require application-level iteration, estimated at **5–30 seconds**. Cold start is excellent at **<5ms**.

**DuckDB + DuckPGQ** is the most promising database option. DuckDB's vectorized execution engine processes data in batches of 2,048 values with morsel-driven parallelism. The **DuckPGQ extension** implements SQL/PGQ (the SQL:2023 property graph standard), constructing **in-memory CSR representations** on the fly and using Multi-Source BFS with AVX-512 vectorization. DuckDB's 2025 `USING KEY` modifier for recursive CTEs dramatically improves graph algorithm performance — on LDBC benchmark graphs, vanilla recursive CTEs took 78.4 seconds while `USING KEY` completed orders of magnitude faster. The `duckdb-rs` crate (1.3M+ downloads, officially maintained by the DuckDB team) provides ergonomic Rust bindings. The main drawback is **cold start latency of 50–200ms** — substantially heavier than SQLite — and the columnar storage that optimizes for scans rather than point lookups makes 1-hop queries **1–5ms** rather than sub-millisecond.

**Kùzu was the best graph database option but is now dead.** Apple acquired Kùzu Inc. in October 2025, and the GitHub repository was archived on October 10, 2025. Before its demise, Kùzu achieved remarkable performance: its embedded C++ engine with columnar CSR storage delivered **374x speedups** over Neo4j on 2nd-degree path queries (0.009s vs 3.22s per the Vela Partners benchmark). Community forks (Bighorn by Kineviz, LadybugDB) exist but are too immature to depend on.

**redb** offers the fastest pure-Rust embedded storage but requires building all graph logic manually. As a B-tree key-value store, it delivers **~1µs per random point lookup** (975ms for 1M reads in official benchmarks). Pre-serialized adjacency lists make 1-hop queries a single B-tree read at **<5µs**, and application-level BFS over ~200 nodes completes in **200µs–1ms**. The tradeoffs are severe: no query language, no built-in graph algorithms, no secondary indexes, no full-text search. You would be building a graph engine from scratch.

**CozoDB**, Parseltongue's current backend, delivers solid performance with built-in graph algorithms (PageRank at ~50ms for 10K vertices, ~1 second for 100K vertices) and native Rust integration. Its Datalog query language handles recursive graph queries naturally. The main limitations are the steep Datalog learning curve, slow RocksDB compilation (5–10 minutes), and single-developer maintenance risk (Ziyang Hu). For 50K nodes, CozoDB estimates 1-hop at **<1ms**, BFS depth-3 at **5–20ms**, and PageRank at **50–100ms**.

The fundamental insight across all database options: **every database interposes a query parser, planner, and execution engine between your code and the data.** For the narrow workload of graph traversal on static data, this overhead ranges from 10x (redb) to 1,000x (SQLite recursive CTEs) compared to direct array indexing on an mmap'd CSR.

---

## Option 3: The hybrid pattern is validated by every major graph database

The hybrid architecture — mmap'd CSR for graph topology, embedded database for metadata — is not a novel idea. **It is the dominant pattern in production graph systems**, though implementations vary.

**Neo4j** has always separated storage layers: `neostore.nodestore.db` (15 bytes/record), `neostore.relationshipstore.db` (34 bytes/record), and `neostore.propertystore.db` (41 bytes/record) are distinct fixed-size record files. Neo4j's newest architecture, **Infinigraph**, makes this explicit with "graph shards" (topology, labels, identifiers) separated from "property shards" (attributes distributed by hash). **TigerGraph** stores graph topology in CSR format (confirmed in the GraphLake paper) through its Graph Storage Engine (GSE), with property data handled separately. **Kùzu** stored edges in **columnar CSR-based adjacency indices** with node properties in vanilla columnar files — the closest analog to Parseltongue's proposed design. **ArcadeDB** recently demonstrated the pattern's power: building a CSR from its OLTP row store for analytics yielded a **462x speedup for PageRank** (from ~1 minute to 117ms).

For Parseltongue's static workload, the hybrid approach eliminates the consistency problem that normally makes dual-store architectures complex. The graph is built once during indexing: a single pipeline parses the codebase, assigns integer node IDs (0..N-1), writes the CSR binary file, and populates the SQLite metadata database. Both stores share the same node ID space — the CSR's implicit node ordering maps 1:1 to the database's primary key. Re-indexing builds both stores into a temporary directory and atomically swaps them. The "consistency problem" reduces to "did the build complete successfully?" — verified by matching build UUIDs stored in both files.

The query flow is clean and predictable. A query like "what does `parse_expression` call?" executes in three steps: (1) SQLite lookup by name to get the node ID (~50–200µs), (2) CSR forward-adjacency slice to get neighbor IDs (~100–500ns), (3) SQLite batch lookup for neighbor metadata (`SELECT name, entity_type, file_path FROM nodes WHERE id IN (...)`, ~50–200µs for 10 results). **Total: ~100–400µs.** Pure structural queries — BFS, PageRank, cycle detection, connected components — run entirely on the CSR without touching the database at all. The metadata store is accessed only at the boundary: name resolution at query start, and result enrichment at query end.

The recommended metadata store is **SQLite via `rusqlite`**, not DuckDB or redb. SQLite's FTS5 extension provides full-text search over node names and documentation. Its `WHERE id IN (...)` syntax handles batch lookups naturally. Its format stability, battle-tested reliability, and sub-5ms cold start make it the safest choice. DuckDB's columnar analytics strengths are unnecessary for point lookups and batch retrieval. redb lacks SQL, secondary indexes, and FTS — you would have to build all query logic manually.

**Implementation complexity is approximately 6–8 person-weeks**: one week for the CSR builder, one for the binary format and mmap loader, half a week for the SQLite schema and writer, 1.5 weeks for the query layer joining CSR traversals with metadata lookups, half a week for the atomic-swap re-index pipeline, one week for graph algorithms (BFS, PageRank, cycle detection), one week for testing and error handling, and half a week for CLI/API integration.

| Metric | 50K / 500K | 500K / 5M |
|---|---|---|
| Cold start | ~10–100ms (mmap + SQLite open) | ~50–200ms |
| 1-hop + metadata | ~100–400µs | ~100–400µs |
| BFS depth-3 (topology only) | ~10–50µs | ~10–50µs |
| BFS depth-3 + metadata enrichment | ~1–5ms | ~1–5ms |
| PageRank (20 iterations) | ~100–120ms | ~1–2s |
| Disk footprint | ~15–20 MB (6 MB CSR + 10 MB SQLite) | ~130 MB |
| Implementation effort | 6–8 person-weeks | — |

The UltraGraph Rust project provides an instructive precedent: it reported **1,300x performance improvements** over petgraph after switching to a frozen dual-CSR representation with struct-of-arrays layout for cache efficiency. The BACH system (VLDB 2024) formally validated the pattern of storing edge topology in CSR-tables with separate property arrays, achieving purely sequential edge scans for analytics while maintaining random-access property lookups.

---

## Option 4: The Iggy Graph Compiler is a 1,000x performance regression

The Iggy Graph Compiler thesis proposes treating graph traversal as stream consumption — pre-compiling adjacency lists into Iggy topics and reading them via `poll_messages`. This is creative but **fundamentally mismatched** with the access patterns of graph queries.

Apache Iggy is a legitimate, well-engineered message streaming platform. Built in Rust with a thread-per-core, shared-nothing architecture (inspired by ScyllaDB/Seastar), it uses `io_uring` via the `compio` async runtime, achieves **~10 million messages/second** throughput on TCP at 402 MB/s, and stores data in append-only segment files with offset-based indexes. Its 64-byte message header carries an xxHash3 checksum, 128-bit UUID, sequential offset, timestamps, and payload length — well-designed for streaming integrity but excessive for graph edges.

Three critical findings disqualify this architecture:

**The topic count hard limit is 4,096.** Iggy's `IggyNamespace` bit-packing scheme allocates 12 bits for `topic_id`, creating a hard ceiling of 4,096 topics per stream. The "one topic per node" approach is therefore **physically impossible** for graphs with more than 4,096 nodes. Parseltongue targets 50K–500K nodes. The workaround — using partitions instead — creates a different problem: 500K partitions means 500K filesystem directories, each containing `.log` and `.index` files, totaling over 1 million files. OS file descriptor limits, directory traversal overhead, and partition metadata loading at server startup make this operationally nightmarish.

**Iggy has no embedded mode.** Every graph query requires a TCP round-trip to a separate `iggy-server` process. On localhost, `poll_messages` over TCP takes **~100–500µs minimum** — accounting for TCP framing, authentication checks (mandatory), shard routing via `DashMap`, index lookup, segment file read, response serialization, and TCP send. Compare this to the **~100–200 nanoseconds** for an mmap'd CSR neighbor lookup. BFS to depth 3 touching 200 nodes would require hundreds of `poll_messages` calls at **50–500ms total** versus **10–50µs** for direct array traversal. This represents a **1,000–5,000x performance penalty**.

**The 64-byte header overhead is catastrophic for small payloads.** A single graph edge (a 4-byte target node ID) wrapped in a 64-byte Iggy message header produces **94% overhead**. Even a batched adjacency list of 20 edges (80 bytes payload) carries 64 bytes of header — 44% overhead that provides checksum, UUID, and timestamp fields irrelevant to a static graph.

Server cold start compounds the problem. Iggy's startup sequence — loading metadata, initializing the memory pool (minimum 512 MiB, default 4 GiB), constructing shard executors, loading partition states — takes **5–30 seconds** for a graph mapped to hundreds of thousands of partitions. The mmap'd CSR file opens in **<5ms**.

PageRank computation is effectively **impractical** through Iggy. Each of the 20 iterations would need to read every edge in the graph, requiring millions of `poll_messages` calls at ~200µs each. The CSR approach completes the same computation in ~100ms by scanning contiguous arrays at memory bandwidth.

Iggy's genuine strengths — consumer groups, partition-based horizontal scaling, message routing, retention policies, multi-protocol access — solve problems Parseltongue does not have. A single-process, static, read-only graph query engine has no use for multi-consumer coordination or automatic message deletion. The operational complexity of running a separate server with Docker capabilities (`SYS_NICE`, `seccomp=unconfined`, `memlock=-1`) is entirely unnecessary overhead.

The thesis does contain one valid kernel: **Iggy as a publication layer for graph change events** (new codebase indexed, delta updates) feeding a downstream CSR builder makes architectural sense. Iggy as the *read layer* for graph queries does not.

| Metric | Iggy Graph Compiler | mmap'd CSR | Ratio |
|---|---|---|---|
| Cold start | 5–30s | <5ms | 1,000–6,000x worse |
| 1-hop latency | 200–500µs | 50–200ns | 1,000–2,500x worse |
| BFS depth-3 | 50–500ms | 10–50µs | 1,000–5,000x worse |
| PageRank | Impractical | 100–120ms | ∞ |
| Disk footprint | 10–20x overhead (64B headers) | ~11 MB | 10–20x worse |
| Implementation | 8–12 person-weeks | 3–4 person-weeks | 3x more work |

---

## The decision matrix favors Option 3, but Option 1 is the core engine

Evaluating all four options against Parseltongue's requirements — sub-millisecond graph traversal, HTTP API for LLMs, static codebase indexed once, persistence to disk, Rust ecosystem — yields a clear hierarchy.

| Criterion (weight) | Option 1: mmap'd CSR | Option 2: Embedded DB | Option 3: Hybrid | Option 4: Iggy |
|---|---|---|---|---|
| **Walk speed (40%)** | ★★★★★ | ★★☆☆☆ | ★★★★★ | ★☆☆☆☆ |
| **Query richness (20%)** | ★★☆☆☆ | ★★★★★ | ★★★★☆ | ★★☆☆☆ |
| **Impl. complexity (20%)** | ★★★★★ | ★★★★☆ | ★★★☆☆ | ★★☆☆☆ |
| **Rust ecosystem (10%)** | ★★★★☆ | ★★★★★ | ★★★★☆ | ★★★★☆ |
| **Operational simplicity (10%)** | ★★★★★ | ★★★★★ | ★★★★☆ | ★★☆☆☆ |

**Option 3 (Hybrid) is the recommended architecture** because it delivers Option 1's walk speed for the hot path while adding the query flexibility, full-text search, and metadata richness that an LLM-facing HTTP API needs. The implementation cost premium over pure CSR (6–8 weeks vs 3–4 weeks) buys substantial capability: name-based node resolution, type-filtered queries, source snippet retrieval, and FTS5 search — all essential for Parseltongue's "who calls X" and blast-radius workflows.

**Option 1 (pure mmap'd CSR) is the correct choice if metadata can live in the CSR file itself** — for example, encoding node names as a string table with an index array, as described in the file layout specification above. This trades SQL query flexibility for maximum simplicity and the fastest possible cold start. The **11 MB footprint** and **<5ms cold start** are compelling for a developer tool that might be launched frequently.

**Option 2 (embedded database) is appropriate only as a stepping stone.** CozoDB is already integrated and delivers adequate performance at the 50K-node scale. DuckDB + DuckPGQ is the strongest future-proof database option, with SQL/PGQ standardization and built-in graph algorithms via CSR construction. But any database-only approach hits a performance ceiling at 10–100ms for multi-hop traversals — orders of magnitude slower than CSR.

**Option 4 (Iggy) should be abandoned for graph reads.** Its value, if any, lies in event-driven indexing pipelines, not query serving.

## Conclusion

The key insight this analysis surfaces is that **Parseltongue's static workload eliminates the hardest problems in graph storage.** No concurrent writes, no consistency protocols, no WAL management, no cache invalidation — just build once and read forever. This makes the mmap'd CSR format not just viable but optimal: it provides persistence (it's a file on disk), in-memory speed (via the OS page cache), zero cold start (via demand paging), and zero memory management overhead (the kernel handles everything). The hybrid approach adds SQLite's query flexibility at the boundaries — name resolution and result enrichment — while keeping the critical traversal path at hardware-limit speed. For a tool serving graph queries to LLMs over HTTP, the ~100–400µs end-to-end latency of the hybrid approach is effectively invisible compared to the ~100ms+ HTTP round-trip and ~1–10 second LLM inference time that bookend each query.