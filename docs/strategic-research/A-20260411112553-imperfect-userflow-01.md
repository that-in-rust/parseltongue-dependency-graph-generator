# Imperfect Userflow 01

## Big Idea

The current best explanation for `v302` is this:

**Parseltongue should feel like a local graph query pipeline for LLMs, not like a bag of endpoints and not like a fancy graph browser.**

This note is intentionally imperfect.
It captures the current understanding before the final PRD is rewritten properly.

## Why It Matters

The earlier framing leaned too much on:

- a neat fixed workflow
- a runtime name
- a desktop app as the main surface

But that is not how LLMs usually ask questions while solving code problems.

An LLM behaves more like this:

1. search for something vague
2. find a few likely anchors
3. enrich those anchors with graph facts
4. filter noise
5. rank what matters
6. shrink the answer to a budget
7. ask the next question

That means the product is closer to a **pipeline** than a rigid wizard.

Think of it like cooking with a few reusable kitchen steps:

- fetch ingredients
- chop
- sort
- filter
- plate

The meal changes each time, but the small steps stay useful.

## Core Ideas Made Simple

### 1. Tauri Is The Local Control Room

For `v302`, the Tauri app should mostly be the place where the user:

- adds a codebase
- starts indexing
- reindexes
- sees freshness and status
- starts and stops the local server
- copies the local URL for the LLM

At most, it should show a tiny graph preview.

It is not the main search surface.

So the Tauri app is like the control room at a train station.
It starts the trains and shows status.
It is not the train journey itself.

### 2. The Main Product Primitive Is A Graph Query Pipeline

The LM-facing system should be explained as a small pipeline:

- `RETRIEVE`
- `ANCHOR`
- `ENRICH`
- `FILTER`
- `RANK`
- `BUDGET`
- `OUTPUT`

This is the real center of gravity.

Not:

- “26 endpoints”
- “all graph algorithms”
- “a graph runtime brand”

The runtime is still important, but it sits underneath.

### 3. `FFF` Is The Front Door

`FFF` should be the fast search step.

Its job is simple:

- take a vague query like `auth`
- return a few good candidates quickly

Then Parseltongue can do graph work on top of those candidates.

So:

- `FFF` finds the place
- graph walk explains the place
- the LM decides the next move

### 4. The Walk Graph Runtime Is The Quiet Engine Underneath

The underlying graph subsystem should still be called:

- **`Walk Graph Runtime`**

Its job is narrow:

- forward walk
- backward walk
- callers
- callees
- bounded BFS
- blast radius
- local subgraph extraction

It is the engine under the car hood.
The user does not need that to be the main product slogan.

### 5. The Product Should Return Small Trustworthy Packets

The output should not be:

- huge graph dumps
- giant code files
- too many endpoints for the LM to memorize

The output should be:

- a small anchor
- a few important neighbors
- a few important edges
- confidence / freshness
- maybe one mini graph payload

That is enough for the LM to keep going.

## Tiny Example

Here is a one-page codebase:

```rust
pub fn login_handler(email: &str, password: &str) -> Result<String, String> {
    let user = load_user_record(email)?;
    verify_password(&user.password_hash, password)?;
    issue_session(&user.id)
}

fn load_user_record(email: &str) -> Result<User, String> {
    todo!()
}

fn verify_password(hash: &str, password: &str) -> Result<(), String> {
    if hash == password { Ok(()) } else { Err("invalid password".into()) }
}

fn issue_session(user_id: &str) -> Result<String, String> {
    Ok(format!("session-{user_id}"))
}

pub fn logout_handler(session_id: &str) -> Result<(), String> {
    revoke_session(session_id)
}

fn revoke_session(_session_id: &str) -> Result<(), String> {
    Ok(())
}

struct User {
    id: String,
    password_hash: String,
}
```

Now imagine the user asks an LM:

> “How does auth work here, and what breaks if password verification changes?”

Parseltongue should help like this:

### Step 1: Tauri admin console

The user has already:

- added the folder
- indexed it
- started the local server
- copied the local URL

### Step 2: `RETRIEVE`

The LM asks for `auth`.

`FFF` returns likely candidates:

- `login_handler`
- `logout_handler`
- `verify_password`
- `load_user_record`

### Step 3: `ANCHOR`

The LM picks `login_handler` as the best starting point because it is public and central.

### Step 4: `ENRICH`

Parseltongue walks the graph and returns:

- `login_handler -> load_user_record`
- `login_handler -> verify_password`
- `login_handler -> issue_session`

It also computes the reverse relationship:

- `verify_password <- login_handler`

And a simple blast radius:

- changing `verify_password` affects `login_handler`

### Step 5: `FILTER`

The LM asks:

- ignore tests
- ignore comments
- stay within 2 hops

### Step 6: `RANK`

Parseltongue ranks:

1. `login_handler`
2. `verify_password`
3. `issue_session`

because those are the most useful things for this question.

### Step 7: `BUDGET`

The LM says:

> “Give me the top 2 in under 1000 tokens.”

### Step 8: `OUTPUT`

Parseltongue returns a small packet like:

```json
{
  "query": "auth",
  "anchor": "login_handler",
  "top_nodes": [
    "login_handler",
    "verify_password"
  ],
  "edges": [
    ["login_handler", "load_user_record"],
    ["login_handler", "verify_password"],
    ["login_handler", "issue_session"]
  ],
  "freshness": "fresh",
  "confidence": "tree_sitter_structural"
}
```

That is enough for the LM to continue reasoning.

## What This Note Is Really Saying

This note is trying to correct one thing:

`v302` should probably not be sold as:

- a runtime brand
- a graph explorer
- a giant HTTP catalog

It should be sold more honestly as:

- a local indexing app
- a local graph walk engine
- and an LM-facing graph query pipeline

That is the current best explanation.
It may still be wrong or incomplete.

## What To Remember

**The Tauri app runs the station, the walk runtime powers the tracks, and the real product is the small graph query pipeline that helps the LLM ask better next questions.**
