# FFF Front Door And Walk Runtime ELI5

## Big Idea

`FFF` should find the place, and the walk runtime should explain the place.

That means we should be careful not to judge the walk runtime by a job that belongs to `FFF`.

The clean product boundary from the `v313` PRD is:

> `FFF` finds the place; the Interface Dependency Map explains the place.

That line comes from [A-20260415152053-v313-PRD-L2.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/A-20260415152053-v313-PRD-L2.md).

## Why It Matters

We had started mixing two different kinds of speed into one conversation:

- search speed
- graph-walk speed

Those are related, but they are not the same thing.

If the product says `FFF` is the front door, then `FFF` can own:

- vague text query
- candidate finding
- choosing likely interface anchors

And the walk runtime can own:

- one-hop impact
- two-hop impact
- dependency slices
- cheap graph traversal after the anchor is known

This matters because otherwise we end up blaming the graph layer for front-door search work that the PRD already gave to `FFF`.

## Core Ideas Made Simple

### 1. `FFF` is the receptionist

If you walk into a large building and say:

- "I am looking for the payments room"

the receptionist helps you find the likely room.

That is `FFF`.

It takes a vague query and returns a short candidate list quickly.

Per the PRD, that short list should include things like:

- stable interface key
- label
- kind
- file span
- lightweight counts

That is not graph walking yet.
That is orientation.

### 2. The walk runtime is the building map

Once you know the room you care about, you do not keep talking to the receptionist.

You open the building map and ask:

- what is next to this room?
- what rooms feed into this one?
- what is one hop away?
- what is two hops away?

That is the walk runtime.

So the walk layer is best judged after the anchor is already chosen.

### 3. This makes the benchmark story cleaner

Instead of one blurry benchmark, we should think in three lanes.

Lane 1:

- `FFF -> candidates`

This measures front-door search.

Lane 2:

- `chosen candidate -> graph slice`

This measures the real walk-runtime job.

Lane 3:

- `query text -> FFF -> chosen candidate -> graph slice`

This measures the full product experience.

That is a much cleaner way to reason about the system.

### 4. Why our `by_id` benchmark suddenly makes more sense

The current benchmark’s strongest signal was hot `by_id`.

At first, that looked slightly awkward because it skipped name lookup.

But if we respect the PRD boundary, it actually makes sense.

Hot `by_id` is very close to:

- `FFF` already found the place
- now walk the graph

So the benchmark was not useless.
It was just measuring the walk layer more than the front door.

That is a valid thing to measure.

### 5. Why `by_key` still matters

`by_key` is still useful.

It tells us something about:

- anchor handoff cost
- internal lookup overhead
- whether the walk system is too dependent on another layer

But it should not be confused with the whole product.

The PRD says `FFF` is the front door.
So the user-facing system should be judged in stages, not in one undifferentiated blob.

### 6. This changes the architecture conversation

Earlier, the argument sounded like:

- should the walk runtime also find the node?

Now the cleaner version is:

- does the walk runtime need to find the node at all, if `FFF` already did?

That shifts the design pressure.

It means the walk runtime can stay more specialized:

- compact
- fast
- graph-shaped

And `FFF` can stay search-shaped.

That is healthier than forcing one layer to be both a search engine and a graph walker.

### 7. The next benchmark should match the product journey

The right next experiment is not just:

- "make `by_key` faster"

It is:

- benchmark `FFF`
- benchmark post-anchor walking
- benchmark end-to-end user flow

That way we can answer three different questions honestly:

- is search fast enough?
- is walking fast enough?
- does the combined experience feel fast enough?

## Tiny Example

Suppose the user types:

- `session save`

The clean product flow is:

1. `FFF` returns likely candidates like `save_session_record_now`
2. the user or LLM picks that candidate
3. the walk runtime returns:
   - direct dependents
   - one-hop impact
   - two-hop impact

That is better than asking one system to do all of this in one confusing step.

It is like:

- first find the room
- then read the room’s map

## What To Remember

The walk runtime does not have to be the front door if the product already gave that job to `FFF`.

So the clean way to judge the system is:

- let `FFF` find the place
- let the walk runtime explain the place
- and benchmark both separately before combining them

Sticky sentence:

**If `FFF` is the receptionist, the walk runtime should be the map, not a second receptionist.**
