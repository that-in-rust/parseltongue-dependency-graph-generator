# v313 Walk Runtime Test Harness ELI5

## Big Idea

The walk runtime is just a **stored map of neighbors**.

It is not trying to understand everything about the code every time you ask a question.
It is trying to make one thing cheap:

- find a node
- jump to its neighbors
- keep walking

For `v313`, this map is intentionally small and readable.

The PRD in [A-20260415152053-v313-PRD-L2.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/A-20260415152053-v313-PRD-L2.md) says the surfaced walk layer should default to:

- node types: `folder`, `file`, `public_interface`
- edge types: `contains`, `depends_on`

So the runtime should feel like:

- `FFF` finds the place
- the Interface Dependency Map explains the place
- one-hop and two-hop impact just walk that stored map

## Why It Matters

If we store the graph in the wrong shape, every question becomes too much work.

Suppose someone asks:

- what does `login_user_flow_now` depend on?
- who is affected one hop away from `save_session_record_now`?

If the system only stores a giant edge table, it keeps having to search for matching rows.

That is like asking for directions and getting a spreadsheet of all roads in the city.

The better idea is:

- store each node's neighbors together
- store where each node's neighbor list starts
- jump straight to the right slice

That is why the walk runtime feels like a map instead of a database.

## Core Ideas Made Simple

### 1. There Are Two Layers In `v313`

The PRD is explicit that `v313` should use two layers.

Layer 01 is the richer metadata layer.
It can hold:

- workspaces
- files
- folders
- Rust entities
- imports
- comments
- tests
- counts
- spans

Layer 02 is the surfaced walk layer.
It stays smaller and easier to explain.

That surfaced layer is what humans and LLMs should walk by default.

### 2. The Walk Layer Uses Simple Node Kinds

For `v313`, the surfaced nodes are:

- `folder`
- `file`
- `public_interface`

And the surfaced edges are:

- `contains`
- `depends_on`

This is a deliberate simplification.

The product is saying:

- internally, we may know more
- externally, we only surface the part that is easy to read and trust

### 3. The Runtime Uses Dense IDs

Names are good for humans, but arrays work better with small integer IDs.

So the build step turns names like:

```text
login_user_flow_now
logout_user_flow_now
issue_login_token_now
save_session_record_now
```

into IDs like:

```text
0 = login_user_flow_now
1 = logout_user_flow_now
2 = issue_login_token_now
3 = revoke_login_token_now
4 = check_password_match_now
5 = check_session_guard_now
6 = save_session_record_now
```

That is not the user-facing format.
That is just the compact machine format.

### 4. The Runtime Stores Node Facts Separately

The runtime still needs a node table.

That table answers:

- what is node `6`?
- what label should I show?
- what file is it in?
- what span should I hyperlink to?

In plain language:

- the node table tells you **who** a node is
- the adjacency storage tells you **where it connects**

### 5. The Main Trick: Group Neighbors By Source

This is the part that feels strange at first.

Suppose we start with these dependency edges:

```text
login_user_flow_now   -> check_password_match_now
login_user_flow_now   -> issue_login_token_now
login_user_flow_now   -> save_session_record_now

logout_user_flow_now  -> check_session_guard_now
logout_user_flow_now  -> revoke_login_token_now
logout_user_flow_now  -> save_session_record_now

issue_login_token_now -> save_session_record_now
revoke_login_token_now -> save_session_record_now
```

The first improvement is to regroup them like this:

```text
0: [4, 2, 6]
1: [5, 3, 6]
2: [6]
3: [6]
4: []
5: []
6: []
```

That means:

- node `0` points to `4`, `2`, and `6`
- node `1` points to `5`, `3`, and `6`
- and so on

This was invented because the hot question is not:

- "show me every edge"

It is:

- "show me this node's neighbors"

So the storage should match that question.

### 6. Why Flatten The Neighbor Lists

You could store little lists separately.
But that is awkward and less compact.

So the runtime flattens them into one long strip:

```text
forward_peers = [4, 2, 6, 5, 3, 6, 6, 6]
```

Now we have a new problem:

- where does node `0`'s list begin?
- where does node `1`'s list begin?

That is why we store offsets.

### 7. Why Offsets Exist

Offsets are just a table of contents.

For the example above:

```text
node 0 starts at 0
node 1 starts at 3
node 2 starts at 6
node 3 starts at 7
node 4 starts at 8
node 5 starts at 8
node 6 starts at 8
end     is       8
```

So:

```text
forward_offsets = [0, 3, 6, 7, 8, 8, 8, 8]
```

This means:

- node `0` owns `forward_peers[0..3]`
- node `1` owns `forward_peers[3..6]`
- node `2` owns `forward_peers[6..7]`
- node `3` owns `forward_peers[7..8]`

and nodes `4`, `5`, and `6` own empty slices.

That is the whole trick.

### 8. Why This Reduces Work

Without offsets, the system tends to do this:

```text
question:
what does login_user_flow_now depend on?

work:
scan every edge
check whether the source matches login_user_flow_now
collect matches
```

With offsets, the system does this:

```text
question:
what does login_user_flow_now depend on?

work:
look up node_id = 0
read start = forward_offsets[0]
read end   = forward_offsets[1]
take forward_peers[start..end]
done
```

So the runtime is not recomputing the neighbor list.
It is jumping straight to the already-packed answer.

That is why people use CSR-style storage:

- not because it sounds academic
- because it turns repeated searching into direct slicing

### 9. Reverse Walking Uses The Same Idea

`v313` wants:

- one-hop impact
- two-hop impact

So the runtime also stores the reverse direction.

For the same tiny example:

```text
0: []
1: []
2: [0]
3: [1]
4: [0]
5: [1]
6: [0, 1, 2, 3]
```

Flattened:

```text
backward_peers = [0, 1, 0, 1, 0, 1, 2, 3]
```

Offsets:

```text
backward_offsets = [0, 0, 0, 1, 2, 3, 4, 8]
```

Now if we ask:

```text
who is affected one hop away from save_session_record_now?
```

and `save_session_record_now = 6`, then:

```text
start = backward_offsets[6] = 4
end   = backward_offsets[7] = 8
backward_peers[4..8] = [0, 1, 2, 3]
```

So the direct callers are:

- `login_user_flow_now`
- `logout_user_flow_now`
- `issue_login_token_now`
- `revoke_login_token_now`

That is exactly the kind of answer `v313` wants for impact walking.

### 10. How This Fits The Current Test Harness

The harness in [test-harness/rust-test-001](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001) already contains a readable export:

- [interface_nodes.csv](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001/interface_nodes.csv)
- [interface_edges.csv](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001/interface_edges.csv)

That export is already very close to the `v313` surfaced map idea.

The main difference is naming.

Right now it uses labels like:

- `crate`
- `file`
- `function`

For `v313`, the default surfaced story should instead be framed as:

- `folder`
- `file`
- `public_interface`

So the existing harness is a good demo artifact.
It just needs to be explained through the `v313` surface contract.

## Tiny Example

Here is the smallest concrete slice from the harness.

Source functions:

- [login_user_flow_now](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001/auth-token-demo-core/src/lib.rs)
- [issue_login_token_now](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001/auth-token-demo-core/src/tokens.rs)
- [check_password_match_now](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001/auth-token-demo-core/src/guards.rs)
- [save_session_record_now](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/test-harness/rust-test-001/store-profile-demo-core/src/lib.rs)

Readable dependency view:

```text
login_user_flow_now
  -> check_password_match_now
  -> issue_login_token_now
  -> save_session_record_now

issue_login_token_now
  -> save_session_record_now
```

Machine-friendly walk-runtime view:

```text
0 = login_user_flow_now
2 = issue_login_token_now
4 = check_password_match_now
6 = save_session_record_now

forward_offsets = [0, 3, 6, 7, 8, 8, 8, 8]
forward_peers   = [4, 2, 6, 5, 3, 6, 6, 6]
```

Question:

```text
what does login_user_flow_now depend on?
```

Answer path:

```text
node_id = 0
start = forward_offsets[0] = 0
end   = forward_offsets[1] = 3
forward_peers[0..3] = [4, 2, 6]
```

So the runtime instantly knows:

- `check_password_match_now`
- `issue_login_token_now`
- `save_session_record_now`

No full edge scan needed.

## What To Remember

The walk runtime is just a stored answer shape for graph walking.

It works because:

- names become dense IDs
- neighbors are grouped by node
- grouped lists are flattened into one strip
- offsets tell us where each node's slice begins and ends
- reverse slices power one-hop and two-hop impact

The test harness already shows the right idea in CSV form.
The runtime just stores the same map in a more jumpable internal shape.

**The simplest way to think about it is: build time writes the neighbor lists once, so walk time never has to search for them again.**
