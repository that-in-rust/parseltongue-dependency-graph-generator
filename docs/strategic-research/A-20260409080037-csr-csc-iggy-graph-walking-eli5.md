# CSR CSC Iggy Graph Walking ELI5

## Big Idea

If we want graph walking to be fast **and** persisted, the trick is to stop thinking about storing one edge at a time and start thinking about storing each node's neighbors in a compact, jumpable layout.

In plain English:

- `CSR` helps answer: **where can I go from here?**
- `CSC` helps answer: **who can reach me?**
- Apache Iggy does **not** do this natively
- but we can store a CSR/CSC-like graph image **inside** Iggy

Think of it like this:

- Iggy is the **warehouse**
- CSR and CSC are the **drawer system inside the warehouse**

## Why It Matters

Suppose the graph is:

```text
A -> B
A -> C
B -> D
C -> D
```

If someone asks:

- what does `A` point to?
- who points to `D`?

we do **not** want to scan the whole graph every time.

We want to jump straight to the right answer.

That is what CSR and CSC are for.

## Core Ideas Made Simple

### 1. CSR Means Outgoing Drawers

Imagine every node has an outgoing drawer:

```text
A: [B, C]
B: [D]
C: [D]
D: []
```

That is the core idea of `CSR`.

It is just a compact way of storing:

```text
from this node, here are the neighbors I can go to
```

### 2. CSC Means Incoming Drawers

Now imagine every node has an incoming drawer:

```text
A: []
B: [A]
C: [A]
D: [B, C]
```

That is the core idea of `CSC`.

It is just a compact way of storing:

```text
for this node, here are the nodes that point to me
```

### 3. The Big Trick: One Long Strip Plus Starts

Instead of storing lots of separate little lists, we flatten everything into:

- one big strip of neighbor IDs
- one guide that says where each node's list starts

For the outgoing example:

```text
A: [B, C]
B: [D]
C: [D]
D: []
```

the big strip becomes:

```text
[B, C, D, D]
```

and the guide becomes:

```text
A starts at 0
B starts at 2
C starts at 3
D starts at 4
END = 4
```

So the arrays are:

```text
offsets   = [0, 2, 3, 4, 4]
neighbors = [B, C, D, D]
```

That means:

- A uses `neighbors[0..2]` -> `[B, C]`
- B uses `neighbors[2..3]` -> `[D]`
- C uses `neighbors[3..4]` -> `[D]`
- D uses `neighbors[4..4]` -> `[]`

### 4. Why The Offset Array Has `n + 1`

If there are `n` entities, the offset guide always has `n + 1` numbers.

Why?

Because each node needs:

- where its neighbor list starts
- and the next node's start tells us where its list ends

So:

```text
start = offsets[i]
end   = offsets[i + 1]
```

That is why the offset guide is always:

```text
number_of_entities + 1
```

### 5. Why The Neighbor Array Does Not Mention The Source

The neighbor array stores only the **other side** of the edge.

In CSR:

- the source is implied by the offset slice
- the stored value is the destination

So yes, this is the right idea:

```text
the neighbor strip stores edge endpoints
and the offset guide tells us whose endpoints those are
```

### 6. Why We Need Dense Integer IDs

In real systems, CSR and CSC use dense integer IDs, not names.

So we first map:

```text
main     -> 0
tokenize -> 1
validate -> 2
parse    -> 3
eval     -> 4
```

Then the graph uses IDs:

```text
0 -> 1
0 -> 3
0 -> 4
1 -> 2
```

So in practice we need:

```text
real entity key -> dense integer id
dense integer id -> metadata
```

That mapping might live in:

- a manifest
- a sidecar index
- a database
- a small in-memory map after open

## Tiny Example

Here is the whole outgoing picture one more time.

### The graph

```text
A -> B
A -> C
B -> D
C -> D
```

### The outgoing lists

```text
A: [B, C]
B: [D]
C: [D]
D: []
```

### The flattened strip

```text
neighbors
+---+---+---+---+
| B | C | D | D |
+---+---+---+---+
  0   1   2   3
```

### The guide

```text
offsets
+---+---+---+---+---+
| 0 | 2 | 3 | 4 | 4 |
+---+---+---+---+---+
  A   B   C   D  END
```

### How we read it

```text
A uses 0..2 -> [B, C]
B uses 2..3 -> [D]
C uses 3..4 -> [D]
D uses 4..4 -> []
```

That is all CSR really is.

CSC is the same shape, but for incoming neighbors.

## How Apache Iggy Fits In

Apache Iggy does **not** store graphs natively as CSR or CSC.

Its own intelligence is different.

It is built around:

- streams
- topics
- partitions
- append-only logs
- offset and timestamp indexes

So Iggy's native question is:

```text
where in the log does this message live?
```

CSR and CSC ask a different question:

```text
where are this node's neighbors?
```

So the move is not:

```text
use Iggy exactly as-is and hope it becomes graph-native
```

The move is:

```text
compile a graph image into payloads that Iggy stores
```

## What A Good Iggy Design Would Look Like

```text
+--------------------------------------+
|                IGGY                  |
|                                      |
|  stream: graph_snapshot_001          |
|                                      |
|  [manifest]                          |
|  [node_meta]                         |
|  [csr_forward_pages]                 |
|  [csc_reverse_pages]                 |
|  [contains_pages]                    |
|  [public_projection_pages]           |
+--------------------------------------+
```

Then the read path becomes:

```text
query("main")
    |
    v
lookup entity id
    |
    v
find page address
    |
    v
jump to CSR page in Iggy
    |
    v
read outgoing neighbors
```

For reverse lookup:

```text
query("validate")
    |
    v
lookup entity id
    |
    v
jump to CSC page in Iggy
    |
    v
read incoming neighbors
```

## Why This Matters For Speed

This is the whole speed story:

- bad design = one edge per message, lots of tiny lookups
- good design = one jump to the right packed adjacency page

That is why CSR and CSC matter so much.

They make graph walking feel like:

```text
jump to the drawer
open it
take the neighbors
done
```

instead of:

```text
search the whole warehouse every time
```

## What To Remember

If you only remember five things, remember these:

1. `CSR` means outgoing neighbors.
2. `CSC` means incoming neighbors.
3. Both are just one big neighbor strip plus an offset guide.
4. The offset guide always has `n + 1` numbers for `n` entities.
5. Apache Iggy is the durable container, but we must compile the graph layout ourselves if we want fast walking.

The sticky sentence is:

**Iggy can store the warehouse, but CSR and CSC are the drawer labels that let us grab neighbors fast.**
