# Walk Rank Storage Explainer

## Big Idea

We probably should not force one graph storage design to do every job.

For small, local code graphs, storage is cheap. That means we can keep **different saved versions of the same graph** for different kinds of work:

- one version for **walking**
- one version for **ranking**

That is often simpler and better than trying to build one magical format for everything.

## Why It Matters

Think of it like tools in a kitchen.

- A **knife** is great for cutting.
- A **blender** is great for mixing.

You could try to invent one tool that does both, but it would probably be worse at both jobs.

Graph storage is similar.

- Some graph questions are mostly about **walking roads**
- Some graph questions are mostly about **repeating math over the whole map**

Those two jobs like different storage shapes.

## Core Ideas Made Simple

### 1. Walk Graph Runtime

This is the graph engine for questions like:

- what does this node connect to?
- who calls this function?
- what breaks if I change this?
- what can I reach from here?

The best storage shape for this is usually:

- **forward adjacency**
- **backward adjacency**
- stored in a compact snapshot

In plain English:

Pensieve keeps a very neat street map where every place already knows:

- which roads go out
- which roads come in

That makes graph walking very fast.

### 2. Rank Graph Runtime

This is the graph engine for questions like:

- what matters most?
- what is most central?
- which node should I look at first?

These algorithms act less like “walk one road” and more like:

- sweep across the whole graph
- update scores
- sweep again
- repeat until the numbers settle down

That means a **matrix-like** storage shape is often better.

In plain English:

Walk storage is like a road atlas.

Rank storage is more like a giant score sheet where you keep recalculating numbers for every place.

### 3. Why Two Stored Views Are Fine

For code graphs, the data is usually small enough that duplication is cheap.

If a code graph artifact is:

- 10 MB
- 20 MB
- even 100 MB

that is still tiny on a normal developer machine.

So instead of trying to save disk space at all costs, we can optimize for:

- simpler code
- clearer architecture
- better runtime fit

### 4. What This Means For Pensieve

Pensieve does not need to be “the one true graph database.”

It can be a family of small, honest runtimes:

- **Walk runtime** for traversal-style work
- **Rank runtime** for score-style work

Later, there may be other runtimes too:

- shape runtime
- batch runtime
- semantic delta runtime

But we do not have to build all of them at once.

## Tiny Example

Imagine the same codebase stored in two ways.

### Walk view

Best for:

- `who calls this?`
- `what does this call?`
- `what is the blast radius?`

Storage idea:

- per-dimension forward lists
- per-dimension backward lists

### Rank view

Best for:

- `what is central?`
- `what is likely most important?`
- `what should I inspect first?`

Storage idea:

- sparse matrix style layout
- score vectors
- degree vectors

Same codebase.

Two saved views.

Each one shaped for its job.

## What To Remember

The smartest move may not be building one universal graph store.

The smarter move may be:

- store the same small graph in more than one useful shape
- let each shape serve one workload family well
- keep the product honest about what each runtime is for

**When storage is cheap, the best design is often the one that makes the job easiest, not the one that looks the most elegant on paper.**
