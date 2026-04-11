
Here's the whole thing end-to-end with a real example.

---

**THE CODE**

```rust
// main.rs — 5 functions

fn main() {
    let tokens = tokenize("2 + 3");
    let tree = parse(tokens);
    let result = eval(tree);
    println!("{}", result);
}

fn tokenize(input: &str) -> Vec<Token> {
    let chars = validate(input);
    // split into tokens...
}

fn validate(input: &str) -> &str {
    // check for bad characters
}

fn parse(tokens: Vec<Token>) -> Expr {
    // build syntax tree
}

fn eval(tree: Expr) -> i64 {
    // walk tree, compute result
}
```

---

**WHO CALLS WHO?**

```
  main ──calls──► tokenize
  main ──calls──► parse
  main ──calls──► eval
  tokenize ──calls──► validate

  That's 5 nodes and 4 edges.
```

---

**STEP 1: PICK A TAB COUNT**

```
  We only have 5 functions, so let's use 3 tabs.
  (In real life with 50K functions you'd use 5,000 tabs.
   Same idea, just bigger.)

  formula(name) = hash(name) % 3

  formula("main")     = 0
  formula("tokenize") = 1
  formula("validate") = 2
  formula("parse")    = 1    ← same tab as tokenize!
  formula("eval")     = 0    ← same tab as main!

  That's fine. Two things per tab. Scan 2 instead of 1.
```

---

**STEP 2: WRITE THE NODES**

```
  "nodes" notebook — 3 tabs

  Tab #0                Tab #1                Tab #2
  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
  │ page 0: main │     │ page 0: tokenize│   │ page 0: validate│
  │ page 1: eval │     │ page 1: parse   │   │              │
  └──────────────┘     └──────────────┘     └──────────────┘

  Each page holds the full function source code as payload.

  Now every function has a permanent ADDRESS:
    main     = Tab 0, Page 0
    eval     = Tab 0, Page 1
    tokenize = Tab 1, Page 0
    parse    = Tab 1, Page 1
    validate = Tab 2, Page 0
```

---

**STEP 3: WRITE THE EDGES**

```
  "edges_by_source" notebook — same 3 tabs
  (tab number = hash of the SOURCE function)

  Tab #0 (edges FROM main and eval)
  ┌──────────────────────────────────────────────┐
  │ page 0: main CALLS → Tab 1 Page 0 (tokenize)│
  │ page 1: main CALLS → Tab 1 Page 1 (parse)   │
  │ page 2: main CALLS → Tab 0 Page 1 (eval)    │
  └──────────────────────────────────────────────┘

  Tab #1 (edges FROM tokenize and parse)
  ┌──────────────────────────────────────────────┐
  │ page 0: tokenize CALLS → Tab 2 Page 0 (validate)│
  └──────────────────────────────────────────────┘

  Tab #2 (edges FROM validate)
  ┌──────────────────────────────────────────────┐
  │ (empty — validate doesn't call anything)     │
  └──────────────────────────────────────────────┘


  Each edge is just: "source calls → Tab X Page Y"
  Those two numbers (Tab + Page) are the address of the target.
```

---

**STEP 4: WRITE REVERSE EDGES (for "who calls ME?")**

```
  "edges_by_target" notebook — same 3 tabs
  (tab number = hash of the TARGET function)

  Tab #0 (edges pointing TO main and eval)
  ┌──────────────────────────────────────────────┐
  │ page 0: eval CALLED BY ← Tab 0 Page 0 (main)│
  └──────────────────────────────────────────────┘

  Tab #1 (edges pointing TO tokenize and parse)
  ┌──────────────────────────────────────────────┐
  │ page 0: tokenize CALLED BY ← Tab 0 Page 0 (main)│
  │ page 1: parse    CALLED BY ← Tab 0 Page 0 (main)│
  └──────────────────────────────────────────────┘

  Tab #2 (edges pointing TO validate)
  ┌──────────────────────────────────────────────┐
  │ page 0: validate CALLED BY ← Tab 1 Page 0 (tokenize)│
  └──────────────────────────────────────────────┘
```

---

**NOW: QUERY 1 — "What does main call?"**

```
  Step 1: formula("main") = 0

  Step 2: Open "edges_by_source" Tab #0
          Scan all pages (just 3):
            page 0: main CALLS → Tab 1 Page 0
            page 1: main CALLS → Tab 1 Page 1
            page 2: main CALLS → Tab 0 Page 1

  Step 3: Follow the pointers:
            "nodes" Tab 1 Page 0 → tokenize  ✓
            "nodes" Tab 1 Page 1 → parse     ✓
            "nodes" Tab 0 Page 1 → eval      ✓

  Answer: main calls [tokenize, parse, eval]

  Work done: 1 hash + 1 scan of 3 pages + 3 page lookups
  RAM used: 0
```

---

**QUERY 2 — "Who calls validate?"**

```
  Step 1: formula("validate") = 2

  Step 2: Open "edges_by_target" Tab #2
          Scan all pages (just 1):
            page 0: validate CALLED BY ← Tab 1 Page 0

  Step 3: Follow the pointer:
            "nodes" Tab 1 Page 0 → tokenize  ✓

  Answer: validate is called by [tokenize]

  Work done: 1 hash + 1 scan of 1 page + 1 page lookup
  RAM used: 0
```

---

**QUERY 3 — "Show me the full call chain from main"**

```
  Depth 0: main
            │
            ├─ calls → tokenize (Tab 1 Page 0)
            │           │
            │           └─ calls → validate (Tab 2 Page 0)
            │                       │
            │                       └─ calls → (nothing)
            │
            ├─ calls → parse (Tab 1 Page 1)
            │           │
            │           └─ calls → (nothing)
            │
            └─ calls → eval (Tab 0 Page 1)
                        │
                        └─ calls → (nothing)

  How it works:
    - Start with main, look up its edges → 3 targets
    - For each target, look up THEIR edges → 1 more target
    - For each of those, look up THEIR edges → 0 more
    - Done!

  Total lookups: 5 edge scans + 4 node reads = 9 Iggy reads
  RAM used: still 0
```

---

**EVERYTHING ON DISK — THE COMPLETE PICTURE**

```
  IGGY SERVER (on your Mac, saved to disk)
  ┌─────────────────────────────────────────────────────┐
  │                                                     │
  │  "nodes" notebook         3 tabs, 5 pages total     │
  │  ┌────────┐┌────────┐┌─────────┐                   │
  │  │Tab 0   ││Tab 1   ││Tab 2    │                   │
  │  │main    ││tokenize││validate │                   │
  │  │eval    ││parse   ││         │                   │
  │  └────────┘└────────┘└─────────┘                   │
  │                                                     │
  │  "edges_by_source"        3 tabs, 4 pages total     │
  │  ┌────────┐┌────────┐┌─────────┐                   │
  │  │Tab 0   ││Tab 1   ││Tab 2    │                   │
  │  │main→..³││tok→val¹││(empty)  │                   │
  │  └────────┘└────────┘└─────────┘                   │
  │                                                     │
  │  "edges_by_target"        3 tabs, 4 pages total     │
  │  ┌────────┐┌────────┐┌─────────┐                   │
  │  │Tab 0   ││Tab 1   ││Tab 2    │                   │
  │  │→eval¹  ││→tok¹   ││→val¹    │                   │
  │  │        ││→parse¹ ││         │                   │
  │  └────────┘└────────┘└─────────┘                   │
  │                                                     │
  │  ALL saved to disk. Survives reboot. No database.   │
  └─────────────────────────────────────────────────────┘

  To find ANYTHING:
    1. hash(name) → tab number
    2. open that tab → scan a few pages
    3. follow the "Tab X Page Y" pointers

  That's the whole architecture.
```

You're right. Here goes.

---

**THE PROBLEM**

```
Your Rust code has 50,000 things (functions, structs, etc.)
and 500,000 connections between them ("who calls who").

You need to store all of this somewhere so an LLM can ask:
"What does parse_expr call?"

We're going to store it in Iggy.
```

---

**WHAT IS IGGY?**

```
Iggy = a bookshelf of notebooks.

  BOOKSHELF (called "stream")
  ┌──────────────────────────────────────────────┐
  │                                              │
  │  NOTEBOOK 1      NOTEBOOK 2      NOTEBOOK 3  │
  │  "nodes"         "edges"         "metrics"   │
  │  ┌──────────┐   ┌──────────┐   ┌──────────┐ │
  │  │ Tab 1    │   │ Tab 1    │   │ Tab 1    │ │
  │  │ Tab 2    │   │ Tab 2    │   │ Tab 2    │ │
  │  │ Tab 3    │   │ Tab 3    │   │ Tab 3    │ │
  │  │ ...      │   │ ...      │   │ ...      │ │
  │  │ Tab 5000 │   │ Tab 5000 │   │ Tab 5000 │ │
  │  └──────────┘   └──────────┘   └──────────┘ │
  └──────────────────────────────────────────────┘

  Each tab has numbered pages: page 0, page 1, page 2...

  RULES:
  - You can only write at the END. No erasing.
  - You CAN jump to any page number instantly.
  - Everything is saved to disk. Survives restarts.
```

---

**THE BIG PROBLEM: HOW DO YOU FIND STUFF?**

```
  BAD: 1 notebook, 50,000 pages
  ┌─────────────────────────────────────────┐
  │ page 0: some_fn                         │
  │ page 1: another_fn                      │
  │ page 2: yet_another                     │
  │ ...                                     │
  │ page 49,999: last_fn                    │
  └─────────────────────────────────────────┘
  Finding "parse_expr" = read ALL 50,000 pages. Terrible.


  GOOD: 5,000 notebooks, ~10 pages each
  ┌──────────┐ ┌──────────┐ ┌──────────┐
  │ Tab #1   │ │ Tab #2   │ │ Tab #3   │ ...5,000 tabs
  │ 10 pages │ │ 10 pages │ │ 10 pages │
  └──────────┘ └──────────┘ └──────────┘
  Finding "parse_expr" = read ~10 pages. Instant!

  But HOW do you know which tab to open?
```

---

**THE TRICK: A FORMULA THAT NEVER CHANGES**

```
  "parse_expr"  ──► FORMULA ──► 2371
  "emit_mir"    ──► FORMULA ──► 4099
  "Token"       ──► FORMULA ──► 814

  The formula (called a "hash") turns any name into a number.
  SAME name = SAME number. Always. Forever.

  WRITING:
    formula("parse_expr") = 2371
    → write it into Tab #2371

  READING:
    formula("parse_expr") = 2371
    → look in Tab #2371, scan ~10 pages, found it!

  NO MEMORIZING NEEDED.
  The formula IS the index.
```

---

**HOW CONNECTIONS WORK**

```
  Tab #2371 in "nodes"          Tab #4099 in "nodes"
  ┌─────────────────────┐      ┌─────────────────────┐
  │ page 0: some_util   │      │ page 0: fmt_output   │
  │ page 3: another_fn  │      │ page 1: emit_mir  ◄──── THIS ONE
  │ page 7: parse_expr ◄── THIS│ page 2: gen_code     │
  │ page 9: helper_fn   │      │ page 4: link_obj     │
  └─────────────────────┘      └─────────────────────┘

  A SEPARATE notebook called "edges" stores the link:

  Tab #2371 in "edges"
  ┌─────────────────────────────────────────────┐
  │ page 0: parse_expr CALLS → Tab #4099 page 1│
  │ page 1: parse_expr CALLS → Tab #814  page 3│
  │ page 2: helper_fn  CALLS → Tab #2371 page 3│
  └─────────────────────────────────────────────┘

  The connection is just two numbers:
     WHICH TAB  +  WHICH PAGE
  That's it. A pointer written on paper.
```

---

**THE FULL QUERY: "WHAT DOES parse_expr CALL?"**

```
  Step 1:  formula("parse_expr") = 2371
                │
                ▼
  Step 2:  Open "edges" Tab #2371
           Scan ~10 pages
           Found: "calls → Tab #4099 page 1"
           Found: "calls → Tab #814  page 3"
                │
                ▼
  Step 3:  Open "nodes" Tab #4099, go to page 1 → emit_mir
           Open "nodes" Tab #814,  go to page 1 → Token
                │
                ▼
  Step 4:  Done! Answer: parse_expr calls emit_mir and Token.

  TOTAL WORK:
  - 1 formula computation (instant)
  - 1 scan of ~10 pages (instant)
  - 2 direct page lookups (instant)
  - Everything on disk. Nothing in RAM.
```

---

**WHAT COULD GO WRONG**

```
  PROBLEM 1: Picking the number of tabs

    Too few tabs (100):   500 pages each → slow scan
    Sweet spot (5,000):    10 pages each → instant scan  ✓
    Too many tabs (50,000): 1 page each  → computer chokes
                                           on open files

  PROBLEM 2: Never change the tab count

    formula("parse_expr") % 5000 = 2371
    formula("parse_expr") % 10000 = 7841  ← DIFFERENT!

    Every sticky note says "go to Tab #2371" but now
    parse_expr lives in Tab #7841. ALL links break.
    You have to rebuild everything from scratch.

  PROBLEM 3: Each lookup = a network call (~2ms)

    Following 1 link   = 2ms   (fine)
    Following 200 links = 400ms (noticeable but OK)
```

---

**THE WHOLE THING IN ONE PICTURE**

```
  LLM: "What does parse_expr call?"
        │
        ▼
  formula("parse_expr") = 2371
        │
        ├──► "edges" Tab #2371 ──► scan 10 pages
        │    found: [Tab #4099 p1, Tab #814 p3]
        │
        ├──► "nodes" Tab #4099 page 1 ──► emit_mir
        │
        └──► "nodes" Tab #814  page 3 ──► Token
        
  Answer: [emit_mir, Token]
  
  Storage: 100% on Iggy's disk
  RAM used: 0
  Magic: the formula + tiny tabs
```

You're right. Here goes.

---

**THE PROBLEM**

```
Your Rust code has 50,000 things (functions, structs, etc.)
and 500,000 connections between them ("who calls who").

You need to store all of this somewhere so an LLM can ask:
"What does parse_expr call?"

We're going to store it in Iggy.
```

---

**WHAT IS IGGY?**

```
Iggy = a bookshelf of notebooks.

  BOOKSHELF (called "stream")
  ┌──────────────────────────────────────────────┐
  │                                              │
  │  NOTEBOOK 1      NOTEBOOK 2      NOTEBOOK 3  │
  │  "nodes"         "edges"         "metrics"   │
  │  ┌──────────┐   ┌──────────┐   ┌──────────┐ │
  │  │ Tab 1    │   │ Tab 1    │   │ Tab 1    │ │
  │  │ Tab 2    │   │ Tab 2    │   │ Tab 2    │ │
  │  │ Tab 3    │   │ Tab 3    │   │ Tab 3    │ │
  │  │ ...      │   │ ...      │   │ ...      │ │
  │  │ Tab 5000 │   │ Tab 5000 │   │ Tab 5000 │ │
  │  └──────────┘   └──────────┘   └──────────┘ │
  └──────────────────────────────────────────────┘

  Each tab has numbered pages: page 0, page 1, page 2...

  RULES:
  - You can only write at the END. No erasing.
  - You CAN jump to any page number instantly.
  - Everything is saved to disk. Survives restarts.
```

---

**THE BIG PROBLEM: HOW DO YOU FIND STUFF?**

```
  BAD: 1 notebook, 50,000 pages
  ┌─────────────────────────────────────────┐
  │ page 0: some_fn                         │
  │ page 1: another_fn                      │
  │ page 2: yet_another                     │
  │ ...                                     │
  │ page 49,999: last_fn                    │
  └─────────────────────────────────────────┘
  Finding "parse_expr" = read ALL 50,000 pages. Terrible.


  GOOD: 5,000 notebooks, ~10 pages each
  ┌──────────┐ ┌──────────┐ ┌──────────┐
  │ Tab #1   │ │ Tab #2   │ │ Tab #3   │ ...5,000 tabs
  │ 10 pages │ │ 10 pages │ │ 10 pages │
  └──────────┘ └──────────┘ └──────────┘
  Finding "parse_expr" = read ~10 pages. Instant!

  But HOW do you know which tab to open?
```

---

**THE TRICK: A FORMULA THAT NEVER CHANGES**

```
  "parse_expr"  ──► FORMULA ──► 2371
  "emit_mir"    ──► FORMULA ──► 4099
  "Token"       ──► FORMULA ──► 814

  The formula (called a "hash") turns any name into a number.
  SAME name = SAME number. Always. Forever.

  WRITING:
    formula("parse_expr") = 2371
    → write it into Tab #2371

  READING:
    formula("parse_expr") = 2371
    → look in Tab #2371, scan ~10 pages, found it!

  NO MEMORIZING NEEDED.
  The formula IS the index.
```

---

**HOW CONNECTIONS WORK**

```
  Tab #2371 in "nodes"          Tab #4099 in "nodes"
  ┌─────────────────────┐      ┌─────────────────────┐
  │ page 0: some_util   │      │ page 0: fmt_output   │
  │ page 3: another_fn  │      │ page 1: emit_mir  ◄──── THIS ONE
  │ page 7: parse_expr ◄── THIS│ page 2: gen_code     │
  │ page 9: helper_fn   │      │ page 4: link_obj     │
  └─────────────────────┘      └─────────────────────┘

  A SEPARATE notebook called "edges" stores the link:

  Tab #2371 in "edges"
  ┌─────────────────────────────────────────────┐
  │ page 0: parse_expr CALLS → Tab #4099 page 1│
  │ page 1: parse_expr CALLS → Tab #814  page 3│
  │ page 2: helper_fn  CALLS → Tab #2371 page 3│
  └─────────────────────────────────────────────┘

  The connection is just two numbers:
     WHICH TAB  +  WHICH PAGE
  That's it. A pointer written on paper.
```

---

**THE FULL QUERY: "WHAT DOES parse_expr CALL?"**

```
  Step 1:  formula("parse_expr") = 2371
                │
                ▼
  Step 2:  Open "edges" Tab #2371
           Scan ~10 pages
           Found: "calls → Tab #4099 page 1"
           Found: "calls → Tab #814  page 3"
                │
                ▼
  Step 3:  Open "nodes" Tab #4099, go to page 1 → emit_mir
           Open "nodes" Tab #814,  go to page 1 → Token
                │
                ▼
  Step 4:  Done! Answer: parse_expr calls emit_mir and Token.

  TOTAL WORK:
  - 1 formula computation (instant)
  - 1 scan of ~10 pages (instant)
  - 2 direct page lookups (instant)
  - Everything on disk. Nothing in RAM.
```

---

**WHAT COULD GO WRONG**

```
  PROBLEM 1: Picking the number of tabs

    Too few tabs (100):   500 pages each → slow scan
    Sweet spot (5,000):    10 pages each → instant scan  ✓
    Too many tabs (50,000): 1 page each  → computer chokes
                                           on open files

  PROBLEM 2: Never change the tab count

    formula("parse_expr") % 5000 = 2371
    formula("parse_expr") % 10000 = 7841  ← DIFFERENT!

    Every sticky note says "go to Tab #2371" but now
    parse_expr lives in Tab #7841. ALL links break.
    You have to rebuild everything from scratch.

  PROBLEM 3: Each lookup = a network call (~2ms)

    Following 1 link   = 2ms   (fine)
    Following 200 links = 400ms (noticeable but OK)
```

---

**THE WHOLE THING IN ONE PICTURE**

```
  LLM: "What does parse_expr call?"
        │
        ▼
  formula("parse_expr") = 2371
        │
        ├──► "edges" Tab #2371 ──► scan 10 pages
        │    found: [Tab #4099 p1, Tab #814 p3]
        │
        ├──► "nodes" Tab #4099 page 1 ──► emit_mir
        │
        └──► "nodes" Tab #814  page 3 ──► Token
        
  Answer: [emit_mir, Token]
  
  Storage: 100% on Iggy's disk
  RAM used: 0
  Magic: the formula + tiny tabs
```

Fair enough! Let me start from absolute zero. No jargon, just the idea.Now the core trick — how do you FIND stuff in a notebook you can only write at the end of?Now — how do you store "parse_expr CALLS emit_mir" so you can follow the link?And finally — the one thing you need to get right, and the one thing that still hurts:So the whole thing in plain English:

**Iggy is a bookshelf of notebooks.** Each notebook has numbered tabs (partitions) and numbered pages (offsets). You can only write at the end, never erase — but you can jump to any page instantly.

**The trick:** A math formula (hash) turns any function name into a tab number. Same name = same number, every time, forever. So you don't need to memorize where anything is — just re-run the formula.

**Connections** between functions are sticky notes that say "go to tab #4099, page 1." Following a link = opening the right tab and flipping to the right page.

**The catch:** Pick the number of tabs (5,000) once and never change it. If you do, every sticky note's directions become wrong and you start over.

# Building a graph database on Iggy's append-only log

Apache Iggy's storage primitives — **deterministic Murmur3 partition routing, dense per-message indexes, and up to 1 million partitions per topic** — make the proposed on-disk graph database architecturally feasible but with significant caveats around resource overhead, the absence of server-side filtering, and ingestion complexity. The partition-as-hash-bucket insight is sound: if you hash a qualified path to a partition ID, that partition becomes a narrow scan bucket, eliminating the need for a persistent in-memory index at query time. However, the design requires careful calibration of partition counts to balance lookup speed against file-descriptor and memory pressure, and the ingestion phase cannot fully avoid temporary state without accepting O(k) scans per edge.

This report answers each of the six technical questions from the source code and documentation, then evaluates the proposed five-topic architecture end to end.

---

## Murmur3 key routing is deterministic and client-reproducible

Iggy routes `MessagesKey`-partitioned messages using **Murmur3 32-bit hashing on the server side**. The formula is `partition_id = (murmur3_32(key_bytes) % partitions_count) + 1`, producing 1-based partition IDs (confirmed by the architecture documentation: "The partition ID starts from 1 and is incremented by 1 for each partition"). The key can be arbitrary bytes up to **255 bytes** in length.

The hash is computed by the server, not the client. The `Murmur3Hasher::default()` constructor almost certainly uses **seed 0** (standard Rust default trait convention), and the hasher exposes `.write_u64()` and `.finish32()` methods internally. However, this algorithm is **not formally documented as a stable API contract** — the Getting Started guide merely says "murmur3 hash of the received value" without specifying the variant or seed.

**The critical design recommendation**: bypass `MessagesKey` entirely and use `Partitioning::partition_id(id)` with a **client-side hash**. This way the client computes `partition_id = your_hash(qualified_path) % N + 1` using whatever hash function you choose (Murmur3, xxHash, FNV — anything deterministic), then routes explicitly. This eliminates any dependency on Iggy's internal hash implementation and guarantees reproducibility across versions. At read time, the same client-side hash tells you exactly which partition to poll.

The `Partitioner` trait in `iggy_common` supports exactly this pattern. It allows custom client-side partition calculation based on stream ID, topic ID, and arbitrary parameters — designed for cases where "the partition ID is not constant and might be calculated based on the stream ID, topic ID and other parameters."

---

## Offset is the only random-access primitive — no message-ID lookup exists

Iggy stores a **u128 message ID** (typically UUIDv4) in bytes 8–24 of the 64-byte message header, but **there is no message ID index**. You cannot fetch a message by its UUID. The only indexed access paths are:

- **By offset** (primary): the `.index` file maps message offsets to byte positions in the `.log` file
- **By timestamp** (secondary): the same `.index` file maps timestamps to offsets
- **First/Last/Next**: convenience wrappers around offset-based polling

The five polling strategies exposed by `PollingKind` are `Offset`, `Timestamp`, `First`, `Last`, and `Next`. There is no `ById` variant. The message ID serves exactly two purposes: **client-side correlation** and **optional server-side deduplication** (when `deduplicate_messages` is enabled per-partition, duplicate IDs are silently dropped). Deduplication state is maintained in-memory per-partition and is not a persistent index.

This means the proposed architecture's cross-references **must use (partition_id, offset) tuples**, not message IDs. Offsets are stable, monotonically increasing integers within a partition — once written, a message's offset never changes. The only risk is message deletion via retention policies, which must be **disabled** (`IggyExpiry::NeverExpire`, `MaxTopicSize::ServerDefault`) for graph data topics.

---

## The .index file is a dense combined offset-and-timestamp index

Each segment on disk consists of exactly two files: a `.log` file (append-only message data) and a single `.index` file that stores **both positional and temporal indexes** — unlike Kafka, which uses separate `.index` and `.timeindex` files. The architecture documentation states the index keeps "track of the offsets and timestamps of the records."

The index uses a **dense layout** (every message is indexed) with fixed-size entries accessed through `IggyIndexView`, a zero-copy struct that provides typed field access into a binary buffer via `bytemuck::Pod` casting. Each entry almost certainly contains three fields:

- **Relative offset** (u32) — the message's offset relative to the segment's start offset
- **Position** (u32) — byte offset into the `.log` file where the message begins
- **Timestamp** (u64) — server-assigned timestamp in microseconds

This gives an estimated **16 bytes per index entry** (or 20 bytes with alignment padding). A dense, fixed-size layout enables **O(log n) binary search** by offset or timestamp — jump to `entry[n] = buffer[n * ENTRY_SIZE]` for direct positional access, or binary search for timestamp-based queries.

Index caching is configurable via `segment.cache_indexes` with three strategies: `"all"` (all segment indexes in memory), `"open_segment"` (only the active segment's index cached — **the default**), or `"none"` (on-demand disk reads). For the graph database use case where data is written once and then queried, **`"all"` is the optimal setting** since the dataset is static and read-heavy.

---

## One million partitions per topic is the hard ceiling, but practical limits are lower

The `IggyNamespace` type packs identifiers into a single u64 using bit fields: **12 bits for stream ID** (max 4,096), **12 bits for topic ID** (max 4,096 per stream), and **20 bits for partition ID** (max **1,048,576** per topic). This is a hard architectural limit enforced by the bit-packing scheme used for shard routing.

Each partition carries non-trivial per-instance overhead:

- **On disk**: at least 1 `.log` file + 1 `.index` file per active segment (minimum **2 file descriptors**)
- **In memory**: a `SegmentedLog` with sealed/active segments, a `MemoryMessageJournal` buffer (flushed at 1,024 messages or 1 MiB), consumer offsets, an atomic offset counter, and optionally a `MessageDeduplicator`
- **Shard distribution**: partitions are assigned to CPU-pinned shards via Murmur3 hashing of the `IggyNamespace`

For a codebase with **100,000 symbols**, using 100,000 partitions means **200,000+ file descriptors** just for the nodes topic, plus another 200,000+ each for `edges_by_source` and `edges_by_target`. This exceeds typical Linux defaults (`ulimit -n` is often 1,024 or 65,536) and requires aggressive tuning. Memory overhead from 100,000 `MemoryMessageJournal` buffers is also significant.

**The practical sweet spot is 1,000–10,000 partitions per topic.** With 10,000 partitions and 100,000 symbols, each partition averages **10 nodes** — small enough that a full partition scan completes in microseconds. This keeps file descriptors under 100,000 across all topics while maintaining fast lookups. The partition count should be chosen as a power of 2 or prime to improve hash distribution uniformity.

---

## No server-side header filtering makes partition co-location essential

Iggy's message headers are **typed key-value pairs** supporting `HeaderKind` variants: `Raw`, `String`, `Bool`, `Int8` through `Int128`, `Uint8` through `Uint128`, `Float32`, and `Float64`. Total header size is capped at **100 KB** per message. Headers are serialized after the 64-byte message header and before the payload.

However, **the server performs zero inspection, indexing, or filtering on header content**. Headers are opaque metadata returned alongside messages during polling. All filtering is client-side. The documentation notes that "in the future, we might introduce some reserved headers used by the streaming server for specific purposes" but this does not exist today.

This has a critical implication for the graph architecture: **every lookup requires polling an entire partition and filtering client-side**. This is why partition sizing matters enormously. With 10 messages per partition, client-side filtering is trivial. With 1,000 messages per partition, it becomes a meaningful cost — each poll returns all messages, and the client must deserialize and inspect each one.

The typed header system is nevertheless valuable for storing cross-references. Edge messages can encode `target_partition_id` as a `Uint32` header and `target_offset` as a `Uint64` header, giving strongly typed, compact cross-references without payload parsing.

---

## The five-topic architecture with concrete trade-offs

The proposed architecture is fundamentally workable. Here is a refined version with concrete implementation details:

**Topic "nodes"** (N partitions, e.g., 5,000): Each message represents one graph node. The qualified path (e.g., `my_crate::parser::parse_expr`) is hashed client-side to determine the partition: `partition = hash(qualified_path) % N + 1`. The payload contains serialized node data (kind, span, signature). The message's server-assigned offset becomes the node's permanent address within that partition.

**Topic "edges_by_source"** (N partitions, same count as nodes): Partitioned by hashing the SOURCE node's qualified path. Each message represents one edge. Headers carry `target_partition: Uint32`, `target_offset: Uint64`, `edge_kind: String` (e.g., "calls", "imports", "implements"). The payload can carry additional edge metadata. To find all outgoing edges from node X: hash X's path → poll that partition → filter for edges whose source matches X.

**Topic "edges_by_target"** (N partitions): Partitioned by hashing the TARGET node's qualified path. Same structure but reversed: headers carry `source_partition: Uint32`, `source_offset: Uint64`. This enables efficient "who calls this function?" queries.

**Topic "index_by_prefix"** (M partitions, e.g., 1,000): Partitioned by hashing a configurable prefix (e.g., the module path). Each message contains a full qualified path in the payload and `node_partition: Uint32`, `node_offset: Uint64` in headers. This enables fuzzy/prefix searches: hash the known prefix → scan that partition for matching names.

**Topic "metrics"** (N partitions): Offset-aligned with nodes — the metric for node at offset K in partition P is stored at offset K in the same partition P of the metrics topic. This requires writing nodes and metrics in lockstep to maintain offset alignment. Since Iggy assigns offsets sequentially per partition, writing one node then one metric alternately to the same partition index guarantees alignment.

**Ingestion protocol without a persistent HashMap:**

The user's core requirement — no in-memory HashMap — can be achieved for the **query phase** but faces a bootstrapping challenge during **ingestion**. Two approaches exist:

The **scan-based approach** (zero in-memory state) works as follows. First, write all node messages across partitions. Second, for each edge, compute both source and target partition IDs via the hash function. Third, poll the target's partition to scan for the target node's offset (scanning ~10 messages on average). Fourth, poll the source's partition for the source node's offset. Fifth, write the edge message to `edges_by_source` and `edges_by_target` with resolved (partition, offset) cross-references. This is O(E × k) where E is edge count and k is average partition size — for 500,000 edges and k=10, that is 5 million small polls, which at Iggy's throughput of millions of messages per second completes in seconds.

The **two-pass approach** (temporary state, faster) uses a temporary in-memory map only during the build phase: write all nodes, record `qualified_path → (partition, offset)` in a transient map, then write all edges using resolved references. After ingestion completes, discard the map. All subsequent queries navigate purely on-disk using (partition, offset) tuples. This is the pragmatic choice — the temporary map exists only during the one-time build and is not part of the runtime query architecture.

---

## Feasibility verdict and what could break this design

The architecture works for its stated goal: a **static, write-once, read-many** dependency graph stored entirely in Iggy topics with no persistent in-memory index. The key strengths are deterministic hash-based partition routing (giving O(1) partition lookup), dense per-message indexes (giving O(log n) offset-based seeks within segments), and typed headers for compact cross-references.

Three risks deserve attention. First, **partition count changes invalidate all cross-references**. If you create the nodes topic with 5,000 partitions and later need 10,000, every `hash(path) % 5000` changes. The entire graph must be rebuilt. Choose the partition count carefully and treat it as immutable. Second, **Iggy's write pipeline is optimized for throughput, not point reads**. The `MemoryMessageJournal` buffers messages before flushing (default threshold: 1,024 messages or 1 MiB). For a graph with very few messages per partition, many partitions will have unflushed data until an explicit flush. Use `FlushUnsavedBuffer` or configure `messages_required_to_save = 1` for immediate persistence during ingestion. Third, **the lack of server-side filtering means every cross-reference resolution requires the client to receive and inspect messages**. For single-hop lookups this is negligible (poll 10 messages, find the one you want). For multi-hop graph traversals (e.g., transitive closure of all dependencies), the cumulative I/O of many small polls could become the bottleneck — each hop requires a network round-trip to the Iggy server. Consider batching: resolve all targets at the same depth level in parallel, then proceed to the next depth.

The design is ultimately a creative repurposing of a streaming log as a hash-indexed key-value store with cross-references. It trades the flexibility of a traditional graph database for the durability, append-only integrity, and zero-GC performance characteristics of Iggy's Rust-native storage engine. For a static Rust codebase dependency graph that is built once and queried many times, this trade-off is defensible — especially if the alternative is pulling in a full database dependency just to store a dependency tree.

## Conclusion

Iggy's internals support this design more than they obstruct it. The **20-bit partition ID ceiling** (1M partitions) gives ample headroom for even very large codebases. The **dense .index files** with binary-searchable offset and timestamp entries make point reads fast within any partition. The **64-byte fixed message header** with typed user headers provides a natural container for cross-reference tuples without payload parsing overhead. The main architectural discipline required is choosing a stable partition count upfront, using `Partitioning::partition_id()` with client-side hashing instead of relying on server-side `MessagesKey` routing, disabling retention policies, and accepting that all filtering is client-side — which is tolerable when each partition contains only a handful of messages. The design transforms Iggy from a streaming platform into something closer to a persistent, partitioned hash map with append-only semantics — an unconventional but structurally sound use of its primitives.

