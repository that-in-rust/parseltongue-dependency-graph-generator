# Walk Runtime Product Map

## Big Idea

A **Walk Runtime** is best at products where the main job is:

- find the path
- find the dependency
- find the blast radius
- find what can reach what

In simple words:

- if the user keeps asking **"what is connected to this?"**
- and then **"what happens next?"**

then a Walk Runtime is probably the right engine.

If the real job is:

- prediction
- ranking
- recommendations
- identity stitching
- clustering

then a Walk Runtime is usually only one part of the answer, not the whole product.

That is the main lesson.

## Why It Matters

It is easy to say:

> graphs can be used for everything

That is technically true and strategically dangerous.

A hammer can hit many things.  
That does not mean every problem is really a hammer problem.

A Walk Runtime is like a very fast trail map.

It is amazing when people need to ask:

- where can I go from here?
- who can reach me?
- what breaks downstream?
- what is the path from A to B?

It is much less amazing when people really need:

- machine learning
- customer deduplication
- recommendations
- statistical scoring
- heavy community analysis

So the useful question is not:

> where can graphs be used?

The useful question is:

> where is graph walking itself the thing people will pay for?

## Core Ideas Made Simple

### 1. A Walk Runtime Is A Trail Engine

Think of a forest trail map.

You stand at one point and ask:

- what trails leave from here?
- what trails lead back here?
- how many steps until I reach the lake?
- if this bridge closes, what places become unreachable?

That is what a Walk Runtime does for graph-shaped data.

### 2. The Best Products Are "Consequence Products"

The strongest Walk Runtime products are really about **consequences**:

- change this code -> what else is touched?
- patch this dependency -> what services are affected?
- change this table -> which dashboards break?
- compromise this identity -> what crown jewels are reachable?

People care about the graph because they care about the consequence.

### 3. Some Markets Look Good, But The Runtime Is Not The Product

Fraud, recommendation, and customer 360 sound attractive.

But in those markets, traversal is usually not the hardest or most valuable part.

The hardest part is often:

- scoring
- matching
- data quality
- investigation workflow
- model accuracy

So a Walk Runtime may help, but it does not automatically create product-market fit.

### 4. Kuzu Matters Because It Proved The "Embedded Graph" Need

Kuzu was useful because it gave people:

- embedded graph storage
- fast local graph queries
- no need to run a giant graph server

That is why it matters to this discussion.

But Kuzu was not just a Walk Runtime.

It was:

- an embedded graph database
- with query language behavior
- and some graph algorithm support

That means we should not try to build "Kuzu again."

The smarter move is to ask:

> in which narrow category can a Walk Runtime be simpler and sharper than Kuzu?

### 5. Why Apple Likely Cared

This part is partly **inference**, not something Apple explained publicly.

The likely utility of Kuzu for Apple was:

- local relationship storage
- private or on-device graph querying
- personal context and cross-app relationship reasoning
- strong systems/database talent

So the interesting lesson is:

Apple probably did not need "a graph database" in the abstract.

Apple likely needed:

> a compact, embedded way to reason over connected things locally

That is much closer to a Walk Runtime story.

## The Product Categories

Below is the practical map.

The scores are **directional**, not benchmark numbers.

- `Walk Fit` = how much the product is truly about path/dependency/traversal
- `CPSR` = how high the problem likely sits in the user's problem stack
- `10x` = how likely a Walk Runtime can feel dramatically better
- `Below` = how much boring product work is still needed before users care
- `Switching` = how easy it is to get someone to try or adopt
- `PMF` = overall product-market-fit potential for a Walk Runtime shaped product

| Product / use case | Representative products | What people really ask | Walk Fit | CPSR | 10x | Below | Switching | PMF | Simple read |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Code graph context / agent runtime | Sourcegraph Cody, Joern, CodeQL | "What code is connected to this symbol?" | 95 | 79 | 88 | 70 | 74 | **84** | One of the best fits. Traversal is the product. |
| Dependency reachability / SBOM impact | GitHub Dependency Graph, Snyk, Endor Labs, Socket, deps.dev | "If this package is bad, what really breaks?" | 93 | 86 | 83 | 58 | 63 | **82** | Very strong fit. People already think in paths and blast radius. |
| Data lineage / impact analysis | DataHub, Collibra, OpenLineage | "If I change this table, who gets hurt?" | 94 | 82 | 79 | 62 | 56 | **80** | Another very strong fit. Upstream/downstream is a walk problem. |
| Cloud / identity attack path analysis | BloodHound, Wiz, CrowdStrike, Defender for Cloud | "How can an attacker move from here to there?" | 96 | 92 | 76 | 34 | 39 | **79** | Huge pain, but much harder product to ship. |
| Embedded graph query engine | Kuzu, DuckPGQ, PuppyGraph, FalkorDB | "Can I query connected data locally without running a big server?" | 98 | 63 | 82 | 78 | 69 | **73** | Best pure infrastructure wedge. |
| Cloud asset inventory / relationship explorer | Cartography, Wiz graph views, cloud inventory tools | "What assets, identities, and permissions connect here?" | 92 | 69 | 70 | 64 | 58 | **70** | Good fit if kept read-only and simple. |
| GraphRAG local entity search | Microsoft GraphRAG, Neo4j GraphRAG, IBM examples | "What nearby entities and facts should I pull in?" | 83 | 66 | 64 | 57 | 61 | **68** | Real category, but noisy and hype-heavy. |
| Supply chain / BOM / manufacturing tracing | Stardog, Neo4j, ArangoDB deployments | "What parts and suppliers are downstream of this change?" | 88 | 74 | 71 | 43 | 42 | **65** | Good walk fit, but enterprise data plumbing is heavy. |
| Investigative link analysis | Linkurious, Cambridge Intelligence, Neo4j stacks | "Show me the links between people, events, and accounts." | 94 | 75 | 70 | 36 | 37 | **64** | Traversal matters a lot, but workflow and trust matter even more. |
| Service topology / observability maps | Datadog, Grafana, New Relic | "What services are upstream and downstream?" | 89 | 73 | 58 | 42 | 45 | **59** | Useful feature, weaker standalone wedge. |
| Developer portal dependency maps | Backstage, Compass, OpsLevel, Port | "Who owns this and what depends on it?" | 85 | 55 | 47 | 46 | 58 | **51** | Helpful, but the portal workflow matters more than the graph walk. |
| Digital twin / asset topology | Azure Digital Twins, enterprise twins | "How do these systems and parts connect?" | 81 | 58 | 57 | 39 | 34 | **50** | Real need, but too integration-heavy for a sharp first repo. |
| Fraud / AML operations | Linkurious, Neo4j, TigerGraph | "What suspicious relationships exist?" | 73 | 91 | 48 | 28 | 29 | **47** | Big market, but walking alone is not enough. |
| Customer 360 / identity resolution | Neo4j, Stardog, entity resolution stacks | "What records belong to the same customer?" | 69 | 71 | 43 | 30 | 34 | **41** | The hard part is matching dirty data, not graph walking. |
| Recommendation / personalization | Neo4j demos, recsys stacks | "What item is close to this user or item?" | 60 | 72 | 32 | 24 | 31 | **31** | Mostly a ranking and ML problem, not a walk-runtime product. |
| Logistics / routing / pathfinding | route engines, map systems | "What is the best route?" | 79 | 69 | 30 | 21 | 26 | **29** | Purpose-built route engines are usually better. |

## What The Internet Evidence Says

Here are the important facts this note is built on.

### Traversal-heavy products clearly exist

- [GitHub Dependency Graph](https://docs.github.com/en/code-security/concepts/supply-chain-security/about-the-dependency-graph) is built around dependency paths and transitive relationships.
- [Snyk Reachability Analysis](https://docs.snyk.io/manage-risk/prioritize-issues-for-fixing/reachability-analysis) explicitly uses call graph style reasoning to prioritize vulnerability impact.
- [DataHub Lineage](https://datahub.com/products/data-lineage/) is sold around upstream/downstream lineage and downstream impact.
- [Microsoft Defender for Cloud attack path analysis](https://learn.microsoft.com/en-us/azure/defender-for-cloud/concept-attack-path) says it uses a cloud security graph to identify exploitable paths.
- [CrowdStrike Attack Path Analysis](https://www.crowdstrike.com/en-us/platform/exposure-management/attack-path-analysis/) sells path visualization and path-based prioritization directly.
- [Joern Code Property Graph docs](https://docs.joern.io/code-property-graph/) explain that earlier versions used general-purpose graph databases and later moved to their own graph database because of limitations.
- [Sourcegraph Cody code graph docs](https://sourcegraph.com/docs/cody/core-concepts/code-graph) describe code graph relationships as context for retrieval.
- [Microsoft GraphRAG docs](https://microsoft.github.io/graphrag/) describe local search as entity-centered reasoning over graph neighborhoods.

### Kuzu proved the embedded graph need

- [Kuzu docs](https://docs.kuzudb.com/) positioned it as an embedded graph database.
- [Kuzu graph algorithms docs](https://docs.kuzudb.com/get-started/graph-algorithms) and the [algo extension docs](https://docs.kuzudb.com/extensions/algo/) show support for algorithms like PageRank, Louvain, K-Core, SCC, and WCC.
- The [Kuzu GitHub repository](https://github.com/kuzudb/kuzu) is archived.

### Apple and Kuzu

What is directly visible:

- Kuzu's repo is archived.
- The University of Waterloo published a note saying Apple agreed to acquire the company and hire select team members: [Waterloo note](https://cs.uwaterloo.ca/news/waterloo-based-graph-database-start-up-kuzu-acquired-apple).

What is **inference**:

- Apple likely cared about embedded relationship querying, local reasoning, and systems talent.
- That fits Apple's public direction around personal context and on-device intelligence, but Apple did not publicly spell out the Kuzu product rationale in detail.

### OSS gravity also matters

As of April 9, 2026:

- [microsoft/graphrag](https://github.com/microsoft/graphrag) has 32,070 stars
- [backstage/backstage](https://github.com/backstage/backstage) has 33,049 stars
- [datahub-project/datahub](https://github.com/datahub-project/datahub) has 11,773 stars
- [JanusGraph/janusgraph](https://github.com/JanusGraph/janusgraph) has 5,761 stars
- [petgraph/petgraph](https://github.com/petgraph/petgraph) has 3,837 stars
- [cartography-cncf/cartography](https://github.com/cartography-cncf/cartography) has 3,820 stars
- [kuzudb/kuzu](https://github.com/kuzudb/kuzu) has 3,831 stars and is archived
- [SpecterOps/BloodHound](https://github.com/SpecterOps/BloodHound) has 2,945 stars

These numbers do **not** prove PMF by themselves.

But they do show:

- graph-adjacent OSS can get real adoption
- the strongest adoption often happens when the graph serves a very clear user job

## Tiny Example

Imagine four different people asking four different questions.

### Person 1: code engineer

"If I change this function, what files and callers are affected?"

That is a Walk Runtime question.

### Person 2: security engineer

"If this cloud identity is compromised, what systems can it reach?"

That is a Walk Runtime question.

### Person 3: data engineer

"If I rename this table or break this column, which dashboards and jobs explode?"

That is a Walk Runtime question.

### Person 4: marketing analyst

"Which product should we recommend to this customer next?"

That is usually **not** a pure Walk Runtime question.
That is more likely a ranking, similarity, or ML problem.

## What To Remember

The strongest Walk Runtime products are not "graph products."

They are **consequence products**:

- dependency impact
- upstream/downstream impact
- attack path analysis
- code context navigation

So the most useful summary is:

> Build where people pay to understand connected consequences, not where graphs merely sound fashionable.
