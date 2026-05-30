# Walk Runtime Isolation Step ELI5

## Big Idea

We did not take the split `libsql + walk snapshot` path because it was the final dream architecture.

We took it because it was the cleanest way to answer one narrower question first:

- is the walk snapshot itself a good way to store graph traversal data?

That made it a good **isolation step**, but not necessarily the best final design.

## Why It Matters

There were really two different questions hiding inside one architecture discussion.

Question 1:

- can the persisted graph walk fast if we store it as `offsets + peers`?

Question 2:

- can one persisted snapshot both find the node and walk the graph without asking SQL first?

The benchmark we built answered Question 1 very well.
It only partly answered Question 2.

That is why the result was useful and still a little frustrating.

It proved the walking part.
It did not fully benchmark the all-in-one snapshot path we may actually want.

## Core Ideas Made Simple

### 1. We were testing the road, not the whole city

Imagine you want to know whether a new road surface is good.

One way to test it is:

- keep the same street signs
- keep the same map desk
- replace only the road surface

Then if travel gets faster, you know the road helped.

That is what the benchmark did.

- `libsql` kept doing the sign desk job: `node_key -> dense_id`
- the walk snapshot did the road job: `dense_id -> neighbors`

This made the experiment easier to interpret.

### 2. Easy does not always mean wrong

It is fair to say we took the easier path.

But it was not "easy" in the sense of avoiding the hard problem forever.
It was "easy" in the sense of reducing the number of moving parts in the experiment.

If we had added a snapshot-native key index at the same time, then a faster result would have been harder to explain:

- was the win from better adjacency storage?
- was the win from better key lookup?
- or both?

So the benchmark traded completeness for cleaner evidence.

### 3. The mismatch came from two different goals

The benchmark goal was:

- isolate the walk-time storage claim

Your product goal is bigger:

- build one persisted runtime that can both find the node and walk the graph well

Those are close, but they are not identical.

That is why the experiment can be both:

- technically useful
- and still unsatisfying as a final architecture answer

### 4. What we proved

We proved that the walk snapshot is a strong structure for graph walking.

The current harness benchmark showed that, for hot `by_id` traversal:

- walk beat raw CSV clearly
- walk also beat `libsql` clearly

That means the core walk shape looks good:

- dense IDs
- offsets
- peer arrays
- `mmap`

In plain words:

- once you know where you are
- the runtime can move quickly

### 5. What we did not prove

We did **not** yet prove that the current split architecture is the best final one.

We also did **not** yet prove whether this would be better:

- snapshot-native key lookup
- plus snapshot-native graph walking

That is the missing experiment.

So the honest position is:

- we validated the walk layer
- we did not fully validate the final end-to-end lookup layer

### 6. The likely better next architecture

Right now, the strongest likely next design is:

- snapshot stores adjacency
- snapshot also stores a fast `node_key -> dense_id` index
- SQL becomes optional for richer metadata, search, and catalog work

That would make the hot path simpler:

- key lookup inside snapshot
- then graph walk inside snapshot

This is like moving the reception desk into the same building as the map.

You no longer walk to a second building just to ask where the room number is.

### 7. Why the current work still matters

The current benchmark is not wasted.

It already gave us the hardest part of the answer:

- the adjacency format seems good

That means the next experiment can be very focused.

We do **not** need to redesign everything.
We mostly need to add and measure one more piece:

- persisted snapshot-native key lookup

That is a much smaller and safer next step than starting over.

## Tiny Example

Suppose you ask:

- what depends on `save_session_record_now`?

In the current split experiment, the runtime does this:

1. ask SQL for the dense ID of `save_session_record_now`
2. open the walk snapshot
3. read the reverse neighbor slice
4. keep walking if needed

In the likely next experiment, the runtime would do this:

1. ask the snapshot itself for the dense ID
2. read the reverse neighbor slice
3. keep walking if needed

The second path is probably better for the final product.

But the first path was a good laboratory setup because it let us test the walking step by itself.

## What To Remember

The split benchmark was not the final architecture.
It was the cleanest way to test the walking half of the architecture first.

So the honest answer is:

- yes, we took the easier experimental path
- no, that does not mean the work was wrong
- and yes, the next benchmark should test the fuller snapshot-native design we actually want

Sticky sentence:

**We tested the road first so we could trust the pavement, but the real product still needs one map, one desk, and one building.**
