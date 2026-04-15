# Build Time Walk Time Visualization ELI5

## Big Idea

If we want fast graph walking with persistence, we should separate the system into two very different jobs:

- **build time**: do the hard thinking once
- **walk time**: use the finished graph map quickly

And the nice part is:

- if we build good walk-time graph data
- we can also use that same data to draw the graph in HTML, CSS, and JavaScript

Think of it like this:

- build time is **making the map**
- walk time is **using the map**
- visualization is **drawing the map on screen**

## Why It Matters

If we mix everything together, the system becomes slow and confused.

That would look like:

- parse source files during every query
- resolve names during every query
- scan edges again and again
- figure out structure live

That is like redrawing the whole city every time someone asks for directions.

The better way is:

1. build the city map once
2. store it
3. answer travel questions from the stored map

That is the main design principle.

## Core Ideas Made Simple

### 1. Build Time Is Allowed To Be Heavy

Build time is where we are allowed to use:

- large hash maps
- temporary vectors
- sorting
- deduplication
- multiple passes
- temporary indexes

Its job is to turn source code into a good graph image.

So build time can do things like:

```text
source code
  -> tree-sitter parse
  -> collect entities
  -> assign dense IDs
  -> collect edges
  -> build CSR / CSC
  -> persist graph snapshot
```

This is the place where we think hard.

### 2. Walk Time Should Be Small And Boring

Walk time should not redo hard work.

Its job is just:

- look up the node
- jump to its persisted neighbors
- walk 1, 2, or 3 hops
- return the answer

That is the place where we cash in the build-time work.

So walk time should feel like:

```text
query
  -> lookup entity id
  -> jump to graph page
  -> read neighbors
  -> continue walking
```

not like:

```text
query
  -> reparse files
  -> rebuild symbol tables
  -> search everything
```

### 3. Large Hash Maps Are Fine At Build Time

For a compiler-like stage, big hash maps are normal.

For example, build time may keep:

- `entity_key -> dense_id`
- `dense_id -> metadata`
- `file_path -> file_id`
- raw edge lists
- unresolved symbols until resolution finishes

That is okay.

The important rule is:

- use big maps to **build the map**
- do not make the final walk path depend on them for every hop

### 4. The Runtime Should Mostly Use Arrays And Pages

Once the graph is built, the runtime should mostly use:

- dense integer IDs
- CSR / CSC offsets
- packed adjacency pages
- a small manifest
- maybe a tiny sidecar lookup index

That way the hot path stays simple:

```text
entity key
  -> dense id
  -> page address
  -> neighbors
```

### 5. Visualization Mostly Uses The Same Walk Data

If we want an HTML/CSS/JS dependency graph, the frontend mainly needs:

- nodes
- edges
- labels
- maybe groups and colors

That means the same graph data used for walking is already most of what the UI needs.

So the graph runtime can power both:

- graph questions
- graph pictures

### 6. Visualization Needs A Few Extra Hints

To draw a graph nicely, the UI may also want:

- file or module group
- node kind
- edge kind
- public/test/comment flags
- cluster or SCC ID
- hop depth
- importance score

Those are not the main walk data.

They are more like **display hints**.

So the right idea is:

- keep the walk runtime focused on graph truth
- add a few persisted view hints for the UI

## Tiny Example

Here is the full mental model in one small picture.

```text
SOURCE CODE
   |
   v
TREE-SITTER PARSE
   |
   v
BUILD-TIME STRUCTURES
  - hash maps
  - temp edge lists
  - sorting
  - dense IDs
   |
   v
GRAPH IMAGE
  - node metadata
  - CSR forward pages
  - CSC reverse pages
  - contains pages
  - public projection pages
  - view hints
   |
   v
PERSISTED SNAPSHOT
   |
   +----------------------------+
   |                            |
   v                            v
WALK TIME                    VISUALIZATION
lookup node                  fetch subgraph
read neighbors               fetch nodes + edges + hints
run BFS                      draw in HTML/CSS/JS
return answer                show graph
```

That means the same stored graph can answer:

- `who calls this?`
- `what does this call?`
- `what is the blast radius?`
- `show me the local dependency graph`

## What To Remember

If you only remember five things, remember these:

1. Build time and walk time should be treated as different systems.
2. Build time is allowed to be heavy and smart.
3. Walk time should be light, direct, and predictable.
4. Large hash maps are okay during build time, not ideal during hot traversal.
5. Good walk-time graph data is already most of what a graph UI needs.

The sticky sentence is:

**Build time makes the map, walk time uses the map, and visualization simply paints the same map on screen.**
