# Why We Built The Tiny Moving Graph First

## Big Idea

Before building the full Parseltongue app, we needed to answer a more basic question:

**what should the graph feel like when a human actually touches it?**

That is why we built the tiny harness in [rust-test-001](../../test-harness/rust-test-001) first.
It is a small fake town for testing the map, not the final city.

The latest committed baseline was commit `ef4748393`, which added:

- a tiny 3-crate Rust workspace
- CSV graph data
- a PIXI-based graph viewer
- an Obsidian-inspired interaction direction

After that, we kept refining the idea in conversation.
This note explains those decisions in plain English.

## Why It Matters

A graph can be technically correct and still feel useless.

That is the trap here.

If the graph is too literal, it lies.
If it is too busy, it becomes soup.
If it has no zoom, no pan, and no clear focus, it feels broken even when the code is "working."

So the real job of the harness is not:

- proving we can draw circles

The real job is:

- learning what kind of graph a person can understand in one glance
- learning what kind of motion feels exciting instead of tiring
- learning what should be shown and what should stay hidden

In simple terms:

**we are prototyping the feeling, not just the plumbing.**

## Core Ideas Made Simple

### 1. Start With A Toy Town

The harness uses a deliberately tiny Rust workspace:

- 3 crates
- 3 files in each crate
- 3 functions in each file

That is small enough to reason about by eye.

It is like testing a new city map on a neighborhood before trying it on all of India.

The fixture lives in:

- [Cargo.toml](../../test-harness/rust-test-001/Cargo.toml)
- [interface_nodes.csv](../../test-harness/rust-test-001/interface_nodes.csv)
- [interface_edges.csv](../../test-harness/rust-test-001/interface_edges.csv)
- [index.html](../../test-harness/rust-test-001/index.html)

### 2. CSV Is The Truth, The Viewer Is Just A Lens

We decided not to treat JSON as the main Interface Dependency Map artifact for this phase.

Instead, the base graph stays very simple:

- one CSV for nodes
- one CSV for edges

Why?

Because CSV is easy to read, easy to diff, easy to hand-write, and easy to fake.

That makes it perfect for a test harness.

The viewer is just a lens sitting on top of those two files.
It is not the source of truth.

In everyday terms:

- the CSV files are the ingredients
- the HTML graph is the plate

### 3. Search First, Graph Second

One big realization was that the graph should not expect the user to "browse everything" visually.

That is too much.

So we moved toward a simple rule:

**FFF finds the place, then the graph explains the place.**

That is why the harness now uses a text filter as a placeholder for `FFF`.

This matters because search reduces confusion.
Instead of staring at a dark field of dots and wondering where to begin, the user can type:

- `boot`
- `auth`
- `profile`
- `cache`
- `lib.rs`

Then the graph can react around that anchor.

### 4. Honest Edges Matter More Than Fancy Edges

This was one of the most important corrections.

At first, the graph could make it seem like a folder was "connected" to another folder.
That looked tidy, but it was semantically wrong.

The truth is:

- folders do not really depend on folders
- files do not really depend on files in the same direct sense
- **functions depend on functions**

Folders and files are context containers.
They help the eye.
They are not the main dependency truth.

So the new rule became:

- crates and files can remain visible as anchors
- only function-to-function `depends_on` edges should be drawn as real dependency lines

That change makes the graph more honest, even if it looks slightly less neat.

### Why Fake Container Lines Mislead

Bad interpretation:

```text
folder A ----> folder B
```

That makes it sound like the folders themselves are calling each other.

Better interpretation:

```text
function x in folder A ----> function y in folder B
```

Then the folders can still be shown, but as containers, not as fake dependency peers.

That one change removes a lot of visual lying.

### 5. The Graph Should Show A Neighborhood, Not The Whole Planet

We also talked about scale.

What happens when the real codebase has `20,000` functions?

If the answer is "show all 20,000 moving dots," the graph will look dramatic for thirty seconds and then become unreadable.

So the better idea is:

- store the large codebase
- show only a small neighborhood at a time
- when something is selected, focus on its first hop

This turns the graph into a **local neighborhood explorer**.

That is a much better mental model than a giant global swarm.

It is like Google Maps:

- you do not need the entire planet in full detail all the time
- you need the block you are standing on, plus the roads right around it

### 6. Make It Feel Alive, But Keep The Meaning Simple

You wanted the graph to feel more like a game.
That was a good instinct.

But the danger is making the graph exciting in a way that destroys clarity.

So the compromise we settled on was:

- keep node types easy to read
- keep edges quiet and mostly one neutral color
- let the motion create the delight

That means:

- soft spring motion
- dragging nodes should tug nearby nodes
- dragging an edge should pull its two endpoints a little
- hovering should reveal, not overload

In other words:

**use physics for feel, not for extra meaning.**

### 7. Obsidian Was A Technology Clue, Not A Copy Target

We checked the local Obsidian extraction and confirmed that its graph stack uses PIXI in the renderer.

That helped in two ways:

1. it told us what kind of rendering family gives that "alive canvas" feel
2. it reminded us that what feels like a draggable edge is usually a spring system, not a hand-authored line editor

So we copied the technology family idea:

- PIXI
- spring-like motion
- live interaction

But we did **not** decide to copy Obsidian's exact graph product.

Parseltongue has different semantics.
It cares more about code entities, first-hop meaning, and honest dependency edges.

### 8. The Missing Camera Controls Were A Real Product Bug

Another important realization was that no zoom and no pan is not a small issue.

It is a core interaction failure.

Even a good graph feels wrong if the user cannot:

- zoom in
- zoom out
- drag the world around
- reset the view

So the current working direction after the last commit is:

- wheel zoom
- empty-space panning
- reset view
- circular clustering so the scene feels intentional instead of randomly scattered

That is not just polish.
That is basic map literacy.

### 9. Circular Clusters Fit The Tiny Harness Better

For the test harness, circular clustering made more sense than arbitrary scatter.

Why?

Because the data is small, symmetrical, and deliberately synthetic.

So instead of letting the graph feel like loose confetti, we moved toward:

- crates sitting in a larger ring
- files orbiting their crate
- functions orbiting their file

That gives the scene a clearer "small solar system" feeling.

It is easier for the eye to parse, especially when combined with zoom and pan.

## What Changed After The Last Commit

The commit `ef4748393` gave us a working baseline harness.

After that baseline, the discussion led to these clearer rules:

1. The graph should be a **viewer-first test harness**, not a full Tauri feature yet.
2. The source-of-truth export for this phase should stay **CSV**, not JSON.
3. The graph should be driven by a **text search entry point** (`FFF` placeholder), not only by type filters.
4. Files and crates are **context anchors**, not the main dependency lines.
5. Only **function-to-function** dependency edges should be drawn as real dependency edges.
6. The graph should focus on the **first hop** around a selected item.
7. The motion should feel **Obsidian-like and alive**, but visually simpler.
8. The graph needs **zoom, pan, reset, and better clustering** to be legible.

Some of those choices are already reflected in the current working tree.
Some are still design guidance that we are actively testing.

That is normal for a harness.
The harness is supposed to teach us what the product should become.

## Tiny Example

Imagine you search for `login`.

The graph should not answer by showing you the entire codebase.

It should answer more like this:

1. find the login function
2. brighten that node
3. show its direct neighbors
4. keep files and crates faintly visible as context
5. let you drag, zoom, and inspect that little neighborhood

That is the useful version.

Not:

1. show every function in the repo
2. color everything differently
3. draw every edge with a special meaning
4. make the user decode a fireworks show

## What To Remember

The small graph harness is teaching us one simple lesson:

**a good code graph is not the one that shows the most things, it is the one that shows the right nearby things honestly and lets you touch them without getting lost.**
