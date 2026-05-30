# 10 Creative Graph Visualizations for Parseltongue's ISG

**Date**: 2026-04-04
**Context**: Visualization research for the Tauri desktop app Parseltongue — a Rust code reading companion that renders an Interface Signature Graph (ISG) with compiler-verified edges, Typed Boundary Aggregation, Semantic Focus Lens, and Variant Graph Overlays.
**Constraint**: HTML + CSS + JS only (no external CDN; d3.js or similar can be bundled). Must work in a Tauri WebView (Chromium-based). Must support dark theme, keyboard navigation, color-blind safety, and graphs up to 50K entities.

---

## Visualization 1: "The Geologic Strata"

### What it looks like

The codebase is rendered as a cross-section of geological rock layers. Each horizontal stratum represents a k-core shell — the densest architectural core sits at the bottom (bedrock), and peripheral utilities sit at the top (topsoil). Within each stratum, entities are positioned as embedded "mineral deposits" — small rectangles whose width encodes out-degree (how much they call outward) and whose color intensity maps to PageRank.

Crate boundaries are rendered as geological fault lines running vertically through the strata — hard faults for CROSS-CRATE boundaries, hairline fractures for INTRA-CRATE module boundaries. Edges between entities are drawn as thin veins connecting the deposits across strata, with solid veins for compiler-verified calls and dashed veins for dynamic dispatch.

```
  ┌──────────────────────────────────────────────────────────────┐
  │  TOPSOIL (k-core shell 1-2) — periphery                     │
  │  ·  · utils::format ·  · cli::parse_args ·  ·  ·  ·         │
  │──────────────────────────────────────────────────────────────│
  │  SANDSTONE (k-core 3-4) — common utilities                  │
  │  ▓▓ Error::new ▓▓  ░░ Config::load ░░  ▓ Logger::init ▓     │
  │════════╤═══════════════════╤═════════════════════════════════│
  │  GRANITE (k-core 5-6)  │ — orchestrators    │               │
  │  ████ Shard::dispatch ████  ▓▓▓ Binary::handle ▓▓▓          │
  │        │                    │                                │
  │────────┼────────────────────┼────────────────────────────────│
  │  BEDROCK (k-core 7+) — architectural core                   │
  │  ██████████ Consumer::poll ██████████                        │
  │  ████████ StreamProcessor::run ████████                      │
  └──────────────────────────────────────────────────────────────┘
         ║                   ║
    crate boundary      crate boundary
    (HARD fault)        (HARD fault)
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Horizontal layer | k-core shell number |
| Rectangle width | out-degree (fan-out) |
| Rectangle color intensity | PageRank score |
| Vertical fault lines | Boundary type (crate=bold, module=thin, folder=dotted) |
| Veins between deposits | edges (calls, impls, type_refs) |
| Vein style | dispatch_kind (solid=static, dashed=dynamic) |
| Deposit label | entity name + signature |

### Best persona fit

**The Tech Lead (Marcus)**. When evaluating a refactor, the strata view immediately reveals which entities form the load-bearing core vs. the replaceable periphery. Moving something from bedrock to sandstone is a major architectural event; moving something within topsoil is trivial.

### Why it beats standard node-link diagrams

Standard node-link diagrams treat all nodes as equal floating objects. The Geologic Strata encodes a fundamental architectural truth: **some code is structurally load-bearing and some is peripheral**. The k-core decomposition makes this concrete — you can instantly see whether you are touching bedrock or topsoil. Developers intuitively understand "don't break the foundation" better than they understand numerical k-core values.

### HTML/CSS/JS approach

- **Rendering**: Canvas 2D for the strata layers (performance at scale), with SVG overlay for interactive labels and tooltips.
- **Layout**: Horizontal bands computed from k-core shells. Within each band, entities are positioned using a 1D force simulation (d3-force with only x-axis forces) to minimize edge crossings between adjacent strata.
- **Fault lines**: CSS `border-left` with varying styles (solid 3px for crate, solid 1px for module, dotted 1px for folder). Positioned using boundary path data.
- **Veins**: Canvas `bezierCurveTo()` with alpha blending. Solid vs. dashed via `setLineDash()`.
- **Interaction**: Click a deposit to focus. Mouseover shows tooltip with full signature.

### Semantic Focus Lens application

When a deposit is focused: its stratum band expands vertically to show detail. The focused deposit glows (box-shadow with focus color). 1-hop neighbors in adjacent strata brighten. 2-hop neighbors stay visible but desaturated. Deposits beyond 2-hop fade to 10% opacity. Fault lines intersecting the focus zone brighten; others dim. The veins connected to the focused entity animate with a flowing particle effect (Canvas dot traveling along the bezier path).

### Performance at 50K entities

- **Virtualization**: Only render strata bands visible in the viewport. Each band is a Canvas tile; off-screen bands are skipped.
- **Level-of-detail**: At workspace zoom, deposits smaller than 3px collapse to dots. Labels appear only for the top-10 PageRank entities per visible band.
- **Edge bundling**: Veins between strata are bundled by source/target boundary to reduce visual clutter. At 50K entities, raw edges would be millions — bundle by boundary crossing type and show individual edges only within the focused zone.
- **Precomputation**: k-core shells and PageRank are precomputed server-side. The client receives a flat array of `{ entity_id, shell, pagerank, x_position }` records.

---

## Visualization 2: "The Territorial Map"

### What it looks like

The codebase is rendered as a political map of territories. Each Leiden community is a colored region with smooth, organic borders — like a map of countries. Crate boundaries are drawn as thick political borders (think national frontiers); module boundaries are provincial borders within a country. Entities within each territory are represented as city dots scaled by PageRank — capital cities (highest PageRank in the community) have larger dots and bold labels.

Cross-boundary edges are drawn as trade routes — arrows between territories, with thicker routes for higher edge counts. Intra-community edges are not drawn (they are internal wiring, not interesting at the map level). The map uses a Voronoi tessellation to compute territory shapes from entity positions, clipped to community membership.

```
  ┌────────────────────────────────────────────────────────────────┐
  │                                                                │
  │    ┌─────────────────────┐    ┌──────────────────────────┐    │
  │    │  "Server Core"       │    │  "Streaming Pipeline"     │    │
  │    │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │    │  ░░░░░░░░░░░░░░░░░░░░   │    │
  │    │  ▓ ● Shard          ▓│◄═══│░ ★ Consumer::poll    ░   │    │
  │    │  ▓  · handler_a     ▓│    │░  · StreamProcessor  ░   │    │
  │    │  ▓  · handler_b     ▓│    │░  · offset_tracker   ░   │    │
  │    │  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │    │░░░░░░░░░░░░░░░░░░░░   │    │
  │    └─────────┬───────────┘    └────────────┬───────────┘    │
  │              ║                              │                 │
  │              ║ ═══ (CROSS-CRATE) ═══       │                 │
  │              ▼                              ▼                 │
  │    ┌─────────────────────────────────────────────┐            │
  │    │  "Common Library"                            │            │
  │    │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒ │            │
  │    │  ▒ ★ IggyError  · types  · commands       ▒ │            │
  │    │  ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒ │            │
  │    └─────────────────────────────────────────────┘            │
  │                                                                │
  │  Legend: ★ Capital (highest PageRank)                          │
  │          ● Major city   · Town   ═══ Trade route               │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Territory shape | Leiden community_id (Voronoi regions) |
| Territory color | Community hue (deterministic from community_id) |
| Thick borders | Crate boundary (type="crate") |
| Thin borders | Module boundary (type="module") |
| City dot size | PageRank |
| Capital star | max(PageRank) within community |
| Trade route thickness | COUNT(edges) between communities |
| Trade route arrow | Edge direction (outgoing_edges from boundary metrics) |
| Trade route style | Crossing type (double-line=CROSS-CRATE, single=INTRA-CRATE) |

### Best persona fit

**The New Hire (Sarah)**. In the first 30 seconds, she sees "this codebase has 8 territories — Server Core, Streaming Pipeline, Common Library..." instead of "this codebase has 400 .rs files." The capitals tell her where to start reading in each territory. The trade routes show her which territories talk to each other.

### Why it beats standard node-link diagrams

Standard node-link diagrams at the workspace level are a ball of spaghetti. The Territorial Map leverages spatial metaphor — something every human understands intuitively. "Countries that trade heavily should be close together" is immediately legible. "This territory has one massive capital and many tiny towns" = hub-and-spoke architecture. "This territory has many medium cities" = distributed architecture. These patterns are invisible in node-link diagrams but jump out from a map.

### HTML/CSS/JS approach

- **Layout**: d3-force simulation to position entities, constrained by community membership. Community centroids computed as mean position of member entities.
- **Territory shapes**: d3-delaunay Voronoi tessellation from entity positions, clipped by community membership. Smooth borders via Catmull-Rom curve fitting on the Voronoi cell boundaries.
- **Borders**: SVG `<path>` elements with `stroke-width` varying by boundary type.
- **Trade routes**: SVG `<path>` with arrowhead markers. Width proportional to `log(edge_count)`.
- **Colors**: HSL palette with fixed saturation/lightness, hue distributed evenly across communities. Color-blind safe: use both hue AND pattern fills (hatching, dots, crosshatch) for accessibility.

### Semantic Focus Lens application

Clicking a city (entity) dims all territories except the focused entity's territory and any territory connected by a trade route. Within the focused territory, 1-hop neighbors brighten. The focused territory gently expands (CSS scale transform with transition), showing more detail — individual entity labels, internal edges as thin lines. Adjacent territories stay at map zoom but show the boundary entities that connect to the focused one (exit portals rendered as gate icons on the border).

### Performance at 50K entities

- **LOD switching**: At workspace zoom, show only territory shapes, capitals, and trade routes (~100 elements). At subsystem zoom (click a territory), load that community's entities (~100-500) on demand. At entity zoom, load the ego network (~20-50 nodes).
- **Voronoi caching**: Territory shapes are computed once per snapshot and cached as SVG path strings. Only recompute when focus changes and a territory needs expansion.
- **Trade route aggregation**: At 50K entities, thousands of inter-community edges. Aggregate to 1 trade route per (community_A, community_B) pair. Show individual edges only when zoomed into a border region.

---

## Visualization 3: "The Circuit Board"

### What it looks like

The codebase is rendered as a printed circuit board (PCB). Modules and crates are IC chips — rectangular components with labeled pin headers on their edges. Each pin represents a pub function or pub type (the public surface of the boundary). Traces (copper pathways) run between pins, representing dependency edges. Trace width encodes the number of edges bundled together. Trace color encodes edge type: copper for calls, silver for impls, gold for type_refs.

The background is the characteristic dark green of a PCB. IC chips are black with white labels. Pins on the left edge are inputs (fan-in); pins on the right edge are outputs (fan-out). The chip body shows internal metrics: entity_count in small text, cohesion as a fill bar.

```
  ┌──── PCB: iggy workspace ──────────────────────────────────────┐
  │  ┏━━━━━━━━━━━━━━━━━━━━━━━┓                                   │
  │  ┃  server/shard/         ┃                                   │
  │  ┃  entities: 90          ┃                                   │
  │  ┃  cohesion: ████░░ 0.45 ┃                                   │
  │  ┃                        ┃   ┏━━━━━━━━━━━━━━━━━━━━┓          │
  │──┨ dispatch()     ────────╂───┨  common/            ┃          │
  │──┨ get_consumer() ────────╂───┨  pub_surface: 80    ┃          │
  │──┨ handle_cmd()   ─ ─ ─ ─╂─ ─┨  cohesion: ███████  ┃          │
  │  ┃                        ┃   ┃                     ┃──── out  │
  │  ┗━━━━━━━━━━━━━━━━━━━━━━━┛   ┃  IggyError  ────────┃──────   │
  │                               ┃  CommandCode ───────┃──────   │
  │  ┏━━━━━━━━━━━━━━━━━━━━━━━┓   ┃  MessageType ───────┃──────   │
  │  ┃  server/streaming/     ┃   ┗━━━━━━━━━━━━━━━━━━━━┛          │
  │──┨ Consumer::poll ────────╂─── (2 edges to shard)              │
  │  ┃  cohesion: ██████ 0.63 ┃                                   │
  │  ┗━━━━━━━━━━━━━━━━━━━━━━━┛                                   │
  │                                                                │
  │  ─── solid trace = static dispatch                             │
  │  ─ ─ dashed trace = dynamic dispatch                           │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| IC chip | Boundary (crate or module) |
| Chip label | Boundary name + boundary type |
| Left pins | Entities with incoming_edges from outside |
| Right pins | Entities with outgoing_edges to outside |
| Pin label | Entity name (pub functions/types) |
| Trace between pins | Edge (calls, impls, type_refs) |
| Trace width | COUNT(edges) between those two entities |
| Trace color | Edge kind (copper=calls, silver=impls, gold=type_refs) |
| Trace style | dispatch_kind (solid=static, dashed=dynamic) |
| Cohesion bar inside chip | boundary.cohesion metric |
| Entity count inside chip | boundary.entity_count |

### Best persona fit

**The Auditor (Diana)**. The PCB metaphor makes trust boundaries immediately visible — every connection between chips is explicit, traceable, and typed. She can ask: "How many traces cross from server/ to common/?" and get an exact count from the pin layout. The fact that each pin is a pub item maps directly to the public API surface she needs to audit.

### Why it beats standard node-link diagrams

The PCB metaphor solves the "boundary problem" that node-link diagrams fail at. In a standard graph, boundaries are invisible — you see nodes and edges but not containers. The PCB makes the container (the IC chip) the primary visual object, with edges constrained to pass through defined pins (public interfaces). This naturally enforces the architectural truth that cross-boundary communication must go through the public surface. Internal wiring is hidden inside the chip, reducing clutter by 70-90% at the workspace level.

### HTML/CSS/JS approach

- **Chips**: HTML `<div>` elements with CSS grid layout for pin placement. `border: 2px solid` with rounded corners. Dark background with white text.
- **Pins**: Small circles positioned along chip edges using CSS `position: absolute` with calculated offsets.
- **Traces**: SVG `<path>` elements using orthogonal routing (Manhattan routing) — traces travel horizontally and vertically with 90-degree bends, like real PCB traces. d3 provides the path interpolation.
- **Trace routing**: Use a simple grid-based A* pathfinding to avoid chip-to-chip overlaps. Each trace finds a non-overlapping path on the PCB grid.
- **Chip layout**: d3-force with collision detection to prevent chip overlap. Chips connected by many traces are pulled closer.

### Semantic Focus Lens application

Clicking a pin (entity) highlights the trace path from that pin through the PCB, illuminating connected pins on other chips with a "powered on" glow effect (CSS `box-shadow` in copper/gold). The focused chip expands to show its internal entities as a mini-schematic inside the chip body. Other chips shrink slightly. Unconnected chips fade to 30% opacity.

### Performance at 50K entities

- **Boundary-level default**: At workspace zoom, show only chips (boundaries, ~50-200) with aggregated pin counts, not individual pins. Click a chip to expand to pin-level.
- **Trace bundling**: Multiple edges between the same two chips are bundled into one trace with a width label (e.g., "42 calls"). Individual traces shown only when zoomed into a chip pair.
- **Manhattan routing cache**: Route computations are cached per layout. Only recompute when the user drags a chip to a new position.

---

## Visualization 4: "The Borrow Weather Map"

### What it looks like

A single function's ownership and borrowing behavior is rendered as a weather map. The function body is the terrain — source lines run horizontally as latitude bands. Borrow scopes are weather systems — high-pressure zones (blue, shared `&` borrows) and low-pressure zones (red, mutable `&mut` borrows) with isobars showing their extent across line ranges. Where borrows overlap, a storm front appears (a bold clashing boundary with lightning bolt icons marking borrow conflicts). Moves are cold fronts sweeping across the terrain. Drops are the weather clearing.

The timeline runs top to bottom (matching source code line order). Active borrows are translucent colored regions spanning their live ranges. The "atmospheric pressure" at each line is the count of simultaneously active borrows — more borrows = more visual density.

```
  ┌──── Weather Map: process_batch() ─────────────────────────────┐
  │  Line  Source                    Weather                       │
  │  ────  ────────────────────────  ──────────────────────────    │
  │   10   let items = &self.queue   ┃ L1: & HIGH PRESSURE (blue) │
  │   11   let count = items.len()   ┃         ↓                  │
  │   12   for item in items.iter()  ┃ L2: & high pressure        │
  │   13     self.process(item)      ┃ ┃ ⛈ STORM FRONT ⚡         │
  │        ════════════════════════  ┃ ┃ L1(shared) vs            │
  │        &self needs &mut self     ┃ ┃ process(&mut self)       │
  │   14   }                         ┃ ┗━ L2 clears               │
  │   15   // calm between systems   ┃                             │
  │   16   drop(items)               ┗━━━ L1 clears ☀             │
  │   17   self.process_remaining()  ☀ CLEAR SKIES ☀              │
  │                                                                │
  │  Pressure: ░ 0 borrows  ▒ 1 borrow  ▓ 2+  ⚡ conflict        │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Terrain (line bands) | Source lines from entity file_path + line range |
| High pressure zone (blue) | Polonius loan_issued_at where kind = shared (&) |
| Low pressure zone (red) | Polonius loan_issued_at where kind = mutable (&mut) |
| Isobars (extent) | Polonius origin_live_at range (start_line to end_line) |
| Storm front | Polonius conflicts (loan_a, loan_b, point, reason) |
| Cold front (move) | path_moved_at_base location |
| Clear skies | Lines with 0 active borrows |
| Pressure density | COUNT(active borrows at this line) |

### Best persona fit

**The Rust Learner (Priya)**. The weather metaphor transforms the abstract, terrifying borrow checker into something visceral and intuitive. "Storm fronts happen when two pressure systems collide" translates directly to "borrow conflicts happen when a shared and mutable borrow overlap." She can look at the weather map and SEE why her code fails — the storm front is literally drawn at the conflict point, with lightning bolts showing exactly which two borrows clash.

### Why it beats standard node-link diagrams

Standard node-link diagrams cannot represent borrow lifetimes at all — they are about inter-entity relationships, not intra-entity temporal phenomena. The Weather Map is a purpose-built metaphor for temporal ranges that overlap. It leverages something everyone understands (weather forecasts) to represent something almost nobody understands intuitively (Rust borrow scoping). The visual vocabulary is self-explanatory: blue = calm shared access, red = exclusive mutable access, lightning = conflict, clear skies = safe to mutate.

### HTML/CSS/JS approach

- **Terrain**: HTML table or CSS grid with one row per source line. Line numbers on the left, source code in the center, weather visualization on the right.
- **Pressure zones**: SVG rectangles spanning the line range, positioned to the right of the source. Blue fill with low opacity for `&`, red fill for `&mut`. Multiple overlapping borrows stack with additive opacity.
- **Storm fronts**: SVG path with a zigzag (storm front) line pattern at the conflict point. Animated CSS `@keyframes` for a subtle pulsing glow on the conflict zone.
- **Cold fronts**: SVG triangles (traditional cold front symbols) at move locations.
- **Pressure gauge**: A thin bar on the far right showing borrow count per line — like a heatmap column.

### Semantic Focus Lens application

At the Flow zoom level, the Weather Map IS the focus lens — the entire view is the internal structure of one function. The "boundaries" are the exit portals: callers and callees shown as small chips above and below the weather map. Clicking a borrow zone highlights the entity that the borrow relates to (e.g., clicking `L1: &self.queue` highlights `self.queue` everywhere in the source). The conflict zone, when clicked, shows the LLM's explanation of why the conflict occurs and how to fix it.

### Performance at 50K entities

This visualization operates at the Flow zoom level — it shows ONE function at a time. Performance is bounded by function size, not codebase size. Even a 500-line function produces at most 500 rows and ~20 active borrow zones. No scaling concern.

---

## Visualization 5: "The Solar System"

### What it looks like

A single entity is the Sun at the center. Its 1-hop neighbors orbit as planets, with orbital radius proportional to BFS distance and planet size proportional to PPR score. Each planet's color encodes edge kind — blue for callers, orange for callees, green for trait impls, purple for type refs. Planets in the same module orbit on the same ring (like Saturn's rings grouping). 2-hop neighbors are asteroids in the outer belt — small, faint, but clickable.

Boundary exits are rendered as deep-space portals at the edge of the system — glowing gateway icons that, when clicked, re-center the solar system on a new sun.

Edges are not lines — they are orbital paths. The planet orbits the sun on a visible elliptical track. The animation is slow and smooth, giving the view a living, breathing quality.

```
                        · · · · · · · · · · 2-hop asteroid belt · · · · ·
                    ·                                                       ·
                ·       ◇ Portal: "Storage"                                    ·
              ·                                                                  ·
            ·       ┌─────────┐                                                   ·
          ·         │ batch_  │ ← caller                                           ·
         ·          │ flush() │    PPR: 0.09                                         ·
        ·           └─────────┘                                                      ·
       ·     ╭────────────────────────────────────╮                                   ·
      ·      │  ┌──────────────┐                  │                                    ·
     ·       │  │ consumer_    │ ← caller          │                                    ·
     ·       │  │ loop::run()  │    PPR: 0.18       │                                    ·
    ·        │  └──────────────┘                    │                                    ·
    ·        │          ████████████                │                                    ·
    ·        │          █ FOCUS:   █                │                                    ·
    ·        │          █ Consumer █                │                                    ·
    ·        │          █ ::poll() █                │                                    ·
    ·        │          ████████████                │                                    ·
    ·        │                    ┌────────────────┐│                                    ·
     ·       │                    │ msg_queue::    ││                                    ·
     ·       │                    │ dequeue()      ││ → callee                           ·
      ·      │                    │ PPR: 0.15      ││                                   ·
       ·     ╰────────────────────┴────────────────╯│                                  ·
        ·                     ┌─────────────┐       │                                 ·
         ·                    │ offset_     │ → callee                                ·
          ·                   │ advance()   │   PPR: 0.12                            ·
            ·                 └─────────────┘                                       ·
              ·                                            ◇ Portal: "Server"     ·
                ·                                                               ·
                    · · · · · · · · · · · · · · · · · · · · · · · · · · · · ·
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Sun | Focus entity (full detail: name, signature, file_path, line range) |
| Planet size | PPR score from focus entity |
| Planet color | Edge kind (blue=caller, orange=callee, green=impl, purple=type_ref) |
| Orbital radius | BFS distance (1-hop = inner orbit, 2-hop = outer orbit) |
| Orbital ring grouping | Module boundary (same module = same ring) |
| Planet label | Entity name + key metric |
| Deep-space portal | Boundary exit (community or crate border entity) |
| Asteroid | 2-hop entity (small circle, faint) |
| Edge animation direction | Call direction (caller orbits counterclockwise, callee clockwise) |

### Best persona fit

**The New Hire (Sarah)** and **The OSS Contributor (Alex)**. When navigating an unfamiliar function, the Solar System instantly answers: "What is this thing's universe? Who depends on it? What does it depend on?" The spatial layout is more memorable than a flat list — after seeing the solar system for `Consumer::poll()`, the user remembers that `consumer_loop::run()` is the biggest planet orbiting it.

### Why it beats standard node-link diagrams

Standard ego-network diagrams (the closest equivalent) render all neighbors at the same distance, creating a flat donut. The Solar System uses two additional visual channels — orbital ring (BFS distance) and planet size (PPR score) — to encode importance hierarchy within the neighborhood. The slow orbital animation also gives the visualization a sense of aliveness that static diagrams lack, making the user feel they are observing a living system, not inspecting a dead snapshot.

### HTML/CSS/JS approach

- **Rendering**: Canvas 2D with requestAnimationFrame for smooth orbital animation. The animation is purely cosmetic (planets slowly orbit) and uses minimal CPU — just rotating the angle of each planet's position on its ellipse.
- **Layout**: Polar coordinates. Inner ring at radius R1 for 1-hop, outer ring at R2 for 2-hop. Within each ring, planets are evenly distributed by angle, grouped by module membership.
- **Planets**: Canvas circles with gradient fills. Labels rendered as Canvas text, oriented horizontally (not rotated with the orbit).
- **Portals**: SVG overlay for the gateway icons at the system edge — these are interactive and benefit from SVG's built-in event handling.
- **Animation**: requestAnimationFrame loop. Each planet has an angular velocity inversely proportional to its orbital radius (inner planets orbit faster, like real physics). Animation rate is slow: ~0.5 RPM to avoid distraction.

### Semantic Focus Lens application

The Solar System IS the entity-level focus lens. The sun is the focus. Ring 1 planets are the visible PPR-ranked neighbors. Ring 2 asteroids are the faint 2-hop. Portals are the boundary exits. Clicking a planet makes it the new sun — the entire system smoothly recenters with a CSS transition (old sun shrinks to planet size, new sun grows, orbits rearrange).

### Performance at 50K entities

The Solar System only renders the ego network of the focused entity — typically 5-30 planets and 10-50 asteroids. The 50K entities exist in the data layer but are not rendered. Rendering cost is O(ego_network_size), which is always small. The only computational cost is the PPR query (cached for top-20 entities per community, ~50ms for cold entities).

---

## Visualization 6: "The Subway Map"

### What it looks like

Call chains are rendered as subway/metro lines. Each function in a call chain is a station (a circle on the line). Lines are colored by the crate they originate from. Transfer stations (entities called from multiple chains) have the characteristic interchange symbol. Branch points (match/if in CFG) are rendered as line splits. Error paths are a distinct "express line" in red.

The map follows the classic London Underground design principles: 45-degree angles only, stations evenly spaced regardless of code distance, and a clean sans-serif typeface for station names.

```
  ┌──── Subway Map: Call chain from main() ────────────────────────┐
  │                                                                 │
  │  ═══ Blue Line (server crate) ══════════════════════════       │
  │                                                                 │
  │  ○ main()                                                      │
  │  │                                                              │
  │  ○ Server::start()                                             │
  │  ├──────────────────────┐                                      │
  │  │                      │                                      │
  │  ○ listen()             ○ setup_routes()                       │
  │  │                      │                                      │
  │  ○ accept_conn() ◎──────○ Router::dispatch() ── ═══            │
  │  │              transfer│                    Green Line         │
  │  │                      ○ Consumer::poll()  (streaming crate)  │
  │  │                      │                                      │
  │  │                      ├────────────────┐                     │
  │  │                      │                │                     │
  │  │                      ○ dequeue()      ○ handle_error()      │
  │  │                      │                │  ─── Red Express    │
  │  │                      ○ advance()      ○ cleanup()           │
  │  │                      │                │                     │
  │  ○ ◎──────────────────── ○ return Ok(())  ○ return Err          │
  │  join                                                          │
  │                                                                 │
  │  Legend: ○ Station  ◎ Transfer  ─── Normal  ─── Error path     │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Station (circle) | Entity (function) in the call chain |
| Line color | Crate boundary of the entity's source |
| Line segments | edges with kind=calls, following MIR TerminatorKind::Call |
| Transfer station | Entity with in-degree > 1 (called from multiple chains) |
| Line split | CFG branch (SwitchInt terminator in MIR) |
| Red express line | Error path (edges following ? operator / Err path) |
| Station label | Entity name (function::method format) |
| Station size | PageRank of the entity |

### Best persona fit

**The OSS Contributor (Alex)**. When trying to understand "what does TcpStream::connect() do under the hood?", the subway map shows the exact execution path as a clean linear diagram. Branch points show where the code splits into success/error/timeout paths. Transfer stations reveal where shared infrastructure is used. The map is exactly the mental model a contributor needs to trace a feature's implementation.

### Why it beats standard node-link diagrams

Standard call graphs show everything connected to everything — no sense of direction, no sense of path. The Subway Map imposes a narrative on the graph: there is a START (the entry point) and branching PATHS (the execution routes). This matches how developers actually reason about call chains — "first A calls B, then B calls C or D depending on..." The schematic simplification (45-degree angles, even spacing) removes visual noise and focuses attention on the topology, not the geometry.

### HTML/CSS/JS approach

- **Layout**: Custom layout algorithm: start from the root entity, perform DFS. Position each station at grid coordinates (column = call depth, row = branch index). Apply the classic metro map constraint: only 0/45/90-degree line segments.
- **Lines**: SVG `<polyline>` with rounded `stroke-linejoin`. Each line segment uses `stroke` color from the crate palette.
- **Stations**: SVG `<circle>` at grid intersections. Transfer stations use a double-circle (SVG `<circle>` with white fill and a second smaller circle inside).
- **Branch rendering**: At a SwitchInt, the line forks into two paths at 45-degree angles. The main path (largest PPR) continues straight; alternate paths branch off.
- **Labels**: SVG `<text>` positioned at 45 degrees from the station, following London Underground typography conventions.

### Semantic Focus Lens application

Clicking a station makes it the "you are here" marker (filled red circle, pulsing). The 1-hop stations (next stations in both directions along the line) brighten. Stations beyond 2-hop fade. Lines not connected to the focused station gray out. Clicking a transfer station shows a tooltip listing all lines (call chains) that pass through it.

### Performance at 50K entities

Call chains are typically 5-20 stations deep with 2-5 branches. The subway map renders one chain at a time — never the entire 50K graph. The computation is BFS/DFS from the selected root, bounded by a depth limit (default: 10 hops). Even with aggressive branching, the rendered station count is rarely above 100.

---

## Visualization 7: "The Tectonic Plates"

### What it looks like

A visualization designed specifically for the Variant Graph Overlays — architecture simulation. The current architecture is rendered as stable tectonic plates (crate/module boundaries as continental shapes). When the user creates a variant, the proposed changes are animated as tectonic shifts — plates slide apart (decoupling), collide (new dependency), or subduct (one module absorbed into another).

The before/after comparison is not a static side-by-side — it is a smooth morphing animation. The user sees the architecture physically move from current state to proposed state, watching plates separate, collide, and reform. Metric changes appear as seismograph readings along the bottom — "coupling magnitude: 3.2 (before) to 1.8 (after)."

```
  ┌──── Tectonic Plates: Variant A — "Decouple Server/Consumer" ──┐
  │                                                                 │
  │  BEFORE:                         AFTER (animated transition):   │
  │  ┌─────────────────────┐         ┌────────────┐ ┌────────────┐ │
  │  │  Server + Consumer  │   ═══►  │  Server    │ │ Consumer   │ │
  │  │  (tightly coupled)  │         │            │ │            │ │
  │  │  8 edges between    │         │     1 edge │►│            │ │
  │  │  same plate         │         │    (trait)  │ │            │ │
  │  └─────────────────────┘         └────────────┘ └────────────┘ │
  │                                        ↑  rift zone                │
  │                                                                 │
  │  ═══════════════ Seismograph ═══════════════════════════════   │
  │  Coupling:     ████████ 8   →   ██ 1        ▼ 87%              │
  │  PageRank(C):  ████ 0.042  →   ███ 0.031   ▼ 26%              │
  │  Communities:  ███ 3       →   ████ 4       ▲ new boundary     │
  │  Cycles:       ░░ 0       →   ░░ 0         = no change        │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Tectonic plate | Boundary (crate or module) |
| Plate size | entity_count within boundary |
| Plate proximity | coupling between boundaries (closer = higher coupling) |
| Rift zone (splitting) | Variant delta: remove_edge operations |
| Collision zone | Variant delta: add_edge operations |
| Subduction | Variant delta: collapse_node (one boundary absorbed) |
| Seismograph readings | Consequence engine output: pagerank_delta, coupling change, community_changes, scc_changes |
| Magnitude numbers | Absolute metric values (before/after with % change) |
| Plate border thickness | fan_in + fan_out (how connected this plate is) |

### Best persona fit

**The Tech Lead (Marcus)**. When presenting a refactoring option to the architecture review, the Tectonic Plates visualization shows the architectural change as a physical transformation, not a diff table. The seismograph gives him the exact metrics he needs: "This split reduces coupling 87%, drops Consumer's centrality 26%, creates one new community boundary, and introduces zero cycles." That is a 15-minute architecture decision instead of a 90-minute debate.

### Why it beats standard node-link diagrams

No standard visualization even attempts to show the *transition* between two graph states. Side-by-side diffs show two snapshots but force the user to mentally compute the differences. The Tectonic Plates show the change as motion — plates physically separating, colliding, reforming. This leverages human spatial reasoning: we are extremely good at tracking objects that move, but poor at comparing two static, complex images. The seismograph provides the quantitative complement to the qualitative animation.

### HTML/CSS/JS approach

- **Plates**: HTML `<div>` elements with `clip-path: polygon()` for organic continental shapes (derived from Voronoi of entity positions within each boundary, simplified to ~8 vertices).
- **Animation**: CSS `transition` on `transform: translate()` for plate movement. d3-interpolate for smooth shape morphing (interpolate between before/after polygon vertices). Total animation duration: 1.5 seconds.
- **Rift zones**: SVG `<path>` with a jagged zigzag pattern, animated via `stroke-dashoffset` to create a "cracking" effect.
- **Seismograph**: Canvas 2D rendering a seismograph waveform animation. The wave amplitude corresponds to the magnitude of change for each metric. This is purely decorative but sells the metaphor.
- **Metric bars**: CSS bar chart with `transition: width 1.5s` matching the plate animation timing.

### Semantic Focus Lens application

The Tectonic Plates operate at the workspace/subsystem zoom level. The focus lens applies at the plate level — clicking a plate highlights it and its connected plates, dimming the rest. The rift zones and collision zones within the focused cluster brighten. The seismograph zooms into the metrics relevant to the focused plates.

### Performance at 50K entities

Tectonic Plates render at the boundary level, not the entity level. Even a 50K-entity codebase typically has ~50-200 boundaries. The animation interpolates between two states of ~50-200 polygons — trivially fast. The consequence engine computation (recomputing PageRank/Leiden/SCC on a 50K-entity graph with a few edge modifications) is the bottleneck, but this is server-side and cached per variant.

---

## Visualization 8: "The Library Shelves"

### What it looks like

The codebase is rendered as a bookshelf wall. Each shelf is a module or crate. Each book on the shelf is an entity (function, struct, trait). Book height is proportional to the entity's line count (LOC). Book spine thickness encodes out-degree. Book spine color encodes entity type (blue for fn, green for struct, gold for trait, red for impl). Books are arranged left-to-right by PageRank within their shelf — the most important entities are at eye level (the middle shelves), less important ones on top/bottom shelves.

Shelves are grouped into bookcases (crate boundaries). Each bookcase has a label plaque at the top. The user can pull a book off the shelf (click) to see its contents — the book opens to reveal its signature, callers, callees, and source preview.

Reading progress is shown as bookmark ribbons sticking out of previously-read books.

```
  ┌──── Library: iggy codebase ────────────────────────────────────┐
  │                                                                 │
  │  ╔══ server/ ═══════════════════════════════════════════════╗   │
  │  ║  ┌──┬────┬──┬───┬──┬──┬──┐  shard/ (eye level)         ║   │
  │  ║  │  │    │  │   │  │  │  │                              ║   │
  │  ║  │D │ G  │H │ D │C │R │S │  D=dispatch  G=get_consumer ║   │
  │  ║  │i │ e  │a │ i │o │e │t │  H=handle_cmd               ║   │
  │  ║  │s │ t  │n │ s │n │s │a │  (sorted by PageRank)       ║   │
  │  ║  │p │ C  │d │ p │s │p │r │                              ║   │
  │  ║  │a │ o  │l │ a │u │o │t │  🔖 = already read           ║   │
  │  ║  │t │ n  │e │ t │m │n │  │                              ║   │
  │  ║  │c │ 🔖 │C │ c │e │d │  │                              ║   │
  │  ║  │h │    │m │ h │r │  │  │                              ║   │
  │  ║  └──┴────┴──┴───┴──┴──┴──┘                              ║   │
  │  ║  ┌─┬─┬─┬─┬─┬─┐  streaming/ (upper shelf)               ║   │
  │  ║  │ │ │ │ │ │ │ │  smaller books, less critical          ║   │
  │  ║  └─┴─┴─┴─┴─┴─┘                                         ║   │
  │  ╚══════════════════════════════════════════════════════════╝   │
  │                                                                 │
  │  ╔══ common/ ══════════════════════════════════════════════╗   │
  │  ║  ┌────┬───┬────┬──┐  error/ types/ commands/            ║   │
  │  ║  │Iggy│Cmd│Type│  │  ALL books have 🔖 (heavily read)   ║   │
  │  ║  │Err │Cod│s   │  │                                     ║   │
  │  ║  │🔖  │🔖  │🔖  │  │                                     ║   │
  │  ║  └────┴───┴────┴──┘                                     ║   │
  │  ╚══════════════════════════════════════════════════════════╝   │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Bookcase | Crate boundary |
| Shelf | Module boundary within crate |
| Book | Entity (fn, struct, trait, impl) |
| Book height | Line count (end_line - start_line) |
| Book spine thickness | out-degree |
| Book spine color | entity type (fn=blue, struct=green, trait=gold, impl=red) |
| Book left-to-right position | PageRank rank within module (highest = leftmost) |
| Shelf vertical position | Module importance (highest cohesion = eye level) |
| Bookmark ribbon | Reading history (entity marked as read) |
| Book label | Entity short name |

### Best persona fit

**The New Hire (Sarah)** and **The Rust Learner (Priya)**. The library metaphor is deeply intuitive — "start with the thickest book on the middle shelf" translates to "read the most important function in the most important module." The bookmark ribbons turn reading progress into something visible and satisfying — "I've read 6 of the 12 books on this shelf." It transforms codebase exploration from an invisible, anxiety-inducing activity into a visible, completable one.

### Why it beats standard node-link diagrams

Standard graphs have no concept of "progress" or "what I've read." They are spatial but not experiential. The Library Shelves visualization is designed for the *reading* use case — it shows the codebase as something to be consumed sequentially, with visible progress, rather than something to be analyzed as a structure. This maps directly to Parseltongue's core thesis: "a reading environment, not a coding tool." The bookshelf metaphor also solves the orientation problem — you know immediately which sections exist, how large they are, and where you are in your reading journey.

### HTML/CSS/JS approach

- **Bookcases**: HTML `<div>` with CSS `border` and a wood-texture background gradient (CSS `linear-gradient` in brown tones for the shelf planks).
- **Shelves**: CSS flexbox rows within each bookcase. Shelf height is fixed; book heights are scaled within the shelf.
- **Books**: HTML `<div>` elements with CSS `writing-mode: vertical-lr` for spine text. Width proportional to entity size, height fills the shelf. CSS `border-left` creates the spine appearance. Background gradient creates the book cover illusion.
- **Book interaction**: On click, CSS `transform: rotate3d(0,1,0,-30deg) translateZ(20px)` creates a "pull off shelf" 3D rotation effect. The book face reveals entity details.
- **Bookmarks**: CSS `::after` pseudo-element with a red ribbon clip-path, positioned at the top of books the user has visited.

### Semantic Focus Lens application

Clicking a book (entity) pulls it off the shelf with a 3D rotation. The pulled book expands to show full details (signature, callers, callees, source preview). Other books on the same shelf dim but remain visible (1-hop = same module). Books on other shelves fade (2-hop). Books on other bookcases become ghosted outlines. The bookcase containing the focused book slides to the center of the viewport.

### Performance at 50K entities

- **Virtualization**: Only render bookcases visible in the viewport. Each bookcase is a DOM element; off-screen bookcases are not rendered (IntersectionObserver).
- **Book LOD**: At full-library view, books below a minimum pixel width (< 4px) collapse to colored lines (just the spine). Labels appear only for the top-5 PageRank books per shelf.
- **Shelf pagination**: If a module has > 50 entities, show the top-20 by PageRank and add a "... and 30 more" indicator. Expand on click.

---

## Visualization 9: "The Pulse Monitor"

### What it looks like

The architecture is rendered as a medical vital signs monitor. Each "vital sign" is a continuously-updating metric trace that represents an architectural health indicator. The display is divided into channels:

- **Heart Rate** (coupling frequency): how many cross-boundary edges per boundary. A high, erratic heart rate = unhealthy coupling.
- **Blood Pressure** (fan-in/fan-out ratio): balanced = healthy. Very high fan-in with zero fan-out = pure library (healthy). Very high fan-out = god-module (concerning).
- **Oxygen Saturation** (cohesion): how well-connected the internal edges are. High SpO2 = well-structured module.
- **Temperature** (PageRank concentration): if too few entities hold too much PageRank, the architecture has a "fever" (single-point-of-failure risk).

When the user applies a variant overlay, the vital signs smoothly transition from their current readings to the projected readings under the variant — the user literally watches the architecture's health change in response to a proposed restructuring.

```
  ┌──── Vital Signs: server/ crate ────────────────────────────────┐
  │                                                                 │
  │  HR (coupling): 8.3 edges/boundary  ⚠ elevated                │
  │  ╭─╮  ╭─╮  ╭──╮  ╭─╮  ╭──╮                                   │
  │  │ │  │ │  │  │  │ │  │  │                                    │
  │  ╯ ╰──╯ ╰──╯  ╰──╯ ╰──╯  ╰──────                             │
  │                                                                 │
  │  BP (fan ratio): 445 out / 20 in = 22.3:1  ⚠ hypertensive     │
  │  ╭───────╮     ╭───────╮     ╭───────╮                         │
  │  │       │     │       │     │       │                         │
  │  ╯       ╰─────╯       ╰─────╯       ╰─────                   │
  │                                                                 │
  │  SpO2 (cohesion): 0.50  ◯ borderline                           │
  │  ────────────────────────────────── 50%                         │
  │                                                                 │
  │  Temp (PageRank concentration): 0.07  ● healthy                │
  │  ──────────────────────── 36.8C equivalent                     │
  │                                                                 │
  │  ═══ Applying Variant A... ═══                                 │
  │  HR: 8.3 → 3.1 ✓ normalizing                                  │
  │  BP: 22.3:1 → 8.2:1 ✓ improving                               │
  │  SpO2: 0.50 → 0.58 ✓ rising                                   │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Heart rate trace | (outgoing_edges + incoming_edges) / entity_count per boundary |
| Blood pressure trace | outgoing_edges / incoming_edges ratio (fan-out vs fan-in) |
| SpO2 line | cohesion metric (internal_edges / entity_count) |
| Temperature | max(PageRank) in boundary / mean(PageRank) — concentration |
| Alert icons | Thresholds: HR > 5 = elevated, BP > 15:1 = hypertensive, SpO2 < 0.4 = low |
| Variant transition | Consequence engine delta: before metrics vs after metrics |

### Best persona fit

**The Tech Lead (Marcus)** and **The Auditor (Diana)**. The vital signs metaphor answers the question "is this part of the codebase healthy?" without requiring graph literacy. A tech lead can look at the monitor and say "shard/ has elevated coupling and low cohesion — it needs attention" without understanding what PageRank is. An auditor can scan all boundaries' vitals and immediately flag the unhealthy ones for deeper inspection.

### Why it beats standard node-link diagrams

Standard graphs encode structure, not health. You can see that node A connects to node B, but you cannot see whether that pattern is healthy or concerning without mental computation. The Pulse Monitor pre-digests architectural metrics into an intuitive health vocabulary that every human understands from medical contexts. The animated traces also create a sense of urgency and aliveness — an erratic heart rate on a code module feels intuitively worrying in a way that "coupling_out = 0.56" does not.

### HTML/CSS/JS approach

- **Traces**: Canvas 2D polyline rendering with antialiased strokes. The "heartbeat" shape is a composite bezier curve mimicking an ECG waveform, with amplitude proportional to the metric value.
- **Animation**: `requestAnimationFrame` loop scrolling the trace left, adding new data points on the right edge. For static analysis (not real-time), the "heartbeat" is generated from the metric value — higher coupling = more erratic waveform, lower = calmer.
- **Grid**: CSS grid background with thin horizontal lines (like ECG paper). Dark background (#1a1a2e), green traces (#00ff41 — classic monitor green).
- **Variant transition**: When a variant is applied, the traces smoothly morph from old metric values to new ones using d3-interpolate over 2 seconds. The color shifts from green to yellow to red based on whether the metric improves or degrades.

### Semantic Focus Lens application

Each boundary gets its own vital signs panel. The focus lens determines which boundary's vitals are shown at full size (focused boundary = large panel), with 1-hop connected boundaries shown as thumbnail vital signs strips below. The user can click between boundary panels to "check vitals" on different modules.

### Performance at 50K entities

Vital signs are computed at the boundary level (~50-200 boundaries), not the entity level. Each boundary has 4 metric values. The Canvas animation is lightweight — rendering 4 polylines with ~200 data points each is trivially fast. The only scaling concern is the number of boundary panels visible simultaneously, which is capped at ~10 by the focus lens.

---

## Visualization 10: "The Constellation Atlas"

### What it looks like

The entire codebase is rendered as a night sky. Entities are stars. Star brightness (luminosity) encodes PageRank — the most important entities are the brightest stars. Leiden communities are constellations — stars within the same community are connected by faint constellation lines. The user sees the familiar night-sky view where major constellations have names ("Server Core," "Streaming Pipeline," "Common Library") and the brightest stars within each constellation have individual names.

Crate boundaries are rendered as celestial coordinate grid lines (like right ascension / declination lines on a star chart). The Milky Way band represents the densest region of inter-community edges — a bright, diffuse band of light running through the sky where many entities connect across constellation boundaries.

Double stars (binary systems) represent entities that are mutually recursive or in the same SCC. Variable stars (pulsing brightness) represent entities with high betweenness centrality — they are bottlenecks whose removal would affect many paths.

```
  ┌──── Constellation Atlas: iggy workspace ───────────────────────┐
  │                                                                 │
  │          .  ·  *  ·  .  ·  .  ·  .  ·  ★  ·  .  ·  .          │
  │       ·        ·           ·           ·                        │
  │    .     ★ Shard::dispatch    .    .      .                     │
  │       · / \  .     .    .    ·     .      ·                     │
  │    .  · ·   ★ get_consumer  .    .    .     .                   │
  │      ╱   \  .    ·     .  "Server Core"   .    .                │
  │    ·╱  .  ╲    ·   .         ·     .      ·     .               │
  │    ╱       ★ handle_cmd      .    .   .     .                   │
  │ ──╱──────────╲──────────────────────────────────────── grid     │
  │  ╱    .    .  ╲   .    ★ Consumer::poll   .     .               │
  │ ·    .   .  .  ·    · ╱ \ ·    .     .     .    .               │
  │    .    .  ·   .   · ★   ★ ·    .     .    .                    │
  │   .   .     .   .   dequeue advance   "Streaming"               │
  │      .     .   .  · ╲ ╱ ·    .    .    .    .                   │
  │  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ Milky Way ░░░░             │
  │    .  ★ IggyError     .    .    .    .    .                     │
  │   · / | \ ·   .    .    .    .   .     .                        │
  │  ★  ★  ★  "Common Library"   .     .     .                     │
  │    .   .   .    .    .    .     .      .     .                   │
  │                                                                 │
  │  ★ Bright star = high PageRank   · Faint star = low PageRank   │
  │  ╱╲ Constellation line = same community                         │
  │  ░░ Milky Way = dense cross-community edge zone                 │
  └────────────────────────────────────────────────────────────────┘
```

### Data mapping

| Visual element | ISG field |
|---|---|
| Star brightness | PageRank |
| Star position | d3-force layout (force-directed with community attraction) |
| Constellation (connected group) | Leiden community_id |
| Constellation lines | Edges between entities in the same community (internal_edges) |
| Constellation name | Community label (derived from representative entity names) |
| Celestial grid lines | Crate boundaries |
| Milky Way band | Dense zone of cross-community edges |
| Double star (binary) | SCC membership (mutual recursion) |
| Variable star (pulsing) | High betweenness centrality (> threshold) |
| Star color | Entity type (blue=fn, yellow=struct, white=trait) |

### Best persona fit

**The New Hire (Sarah)**. The night sky is the most emotionally resonant first-impression visualization. Where the Territorial Map is pragmatic (political boundaries, trade routes), the Constellation Atlas is evocative — it makes the codebase feel like a universe to explore, not a problem to solve. This maps directly to the emotional journey: anxiety to curiosity. "Oh, there are constellations. I can see the shape of this codebase. It has a bright center and a quiet periphery. I want to explore the bright stars."

### Why it beats standard node-link diagrams

Standard force-directed layouts produce the same visual output but without semantic encoding. All nodes are the same size and brightness — you cannot tell what matters. The Constellation Atlas uses brightness (the most intuitive visual channel for importance — we are evolved to notice bright objects) and constellation grouping (the most intuitive spatial channel for belonging — humans have been grouping stars for millennia). The Milky Way band is a novel way to show inter-community edge density without drawing individual edges — a diffuse glow instead of a spaghetti mess.

### HTML/CSS/JS approach

- **Rendering**: WebGL (via a minimal custom shader, no library dependency) for the star field. Stars are point sprites with a Gaussian blur shader for the glow effect. Brightness is controlled by the alpha channel of each point.
- **Milky Way**: A large-radius Gaussian blur applied to a density map of cross-community edge midpoints. Rendered as a textured quad in WebGL with additive blending.
- **Constellation lines**: Canvas 2D overlay for the thin lines connecting community members. Drawn with low alpha (0.15) to be visible but not dominant.
- **Labels**: HTML overlay (`position: absolute`) for star/constellation names. Only shown for the top-N brightest stars (N determined by zoom level).
- **Background**: CSS `background: radial-gradient(ellipse at center, #0a0a2e 0%, #000 100%)` for the deep space feel.

### Semantic Focus Lens application

Clicking a star creates a "telescope zoom" effect — the view smoothly pans and zooms to center the focused star, which grows to reveal its full identity (name, signature, metrics). Its constellation brightens. Adjacent constellations (connected by cross-community edges) stay visible at reduced brightness. Distant constellations fade to near-invisible. The focused star's 1-hop neighbors glow as labeled stars. 2-hop neighbors are visible as faint, unnamed stars. The Milky Way band dims except in regions connected to the focused constellation.

### Performance at 50K entities

- **WebGL point sprites**: 50K point sprites is trivially fast for WebGL (well under a million points, which is the typical GPU comfort zone). No LOD switching needed — all 50K stars are always rendered as points.
- **Label culling**: Only show labels for the top-50 brightest stars at full zoom-out. Increase label density as the user zooms in. This is the primary optimization — DOM labels are expensive, star points are cheap.
- **Constellation lines**: Only draw constellation lines for the top-5 most visible constellations. Draw all lines only within the focused constellation.
- **Force layout**: Run the d3-force simulation server-side during indexing, store (x, y) positions per entity. The client loads pre-computed positions and renders immediately — no simulation on the client.

---

## Top 3 Recommendations

### Selection framework: Shreyas Doshi's "Magic Moment" Sequence

From Section 14.2 of the thesis, the magic moment sequence is:

| Stage | Magic Moment | Visualization needed |
|---|---|---|
| **Acquisition** (30 seconds) | Architecture map appears, 8 labeled clusters | Workspace-level overview |
| **Activation** (2 minutes) | Click a call, see exact implementation | Entity-level navigation |
| **Retention** (session 2) | Reading-aware suggestions, progress visible | Reading progress tracking |
| **Expansion** (week 2) | Variant overlays with computed consequences | Architecture simulation |
| **Loyalty** (ongoing) | Borrow checker explained visually | Flow-level deep dive |

The top 3 visualizations must cover the first three stages — acquisition, activation, and retention — because these determine whether the user ever reaches the later stages.

---

### Recommendation 1: Build "The Constellation Atlas" first

**Magic moment served**: Acquisition (30 seconds).

**Why this one, not the Territorial Map**: Both serve the acquisition moment, but the Constellation Atlas has two advantages:

1. **Emotional resonance**. The thesis repeatedly emphasizes the emotional journey: anxiety to relief to curiosity. A night sky evokes wonder and curiosity more powerfully than a political map. The user's first reaction should be "wow, I want to explore" — not "okay, I can see the structure." The Constellation Atlas achieves the former; the Territorial Map achieves the latter.

2. **Performance at scale**. WebGL point sprites handle 50K entities with zero degradation. The Territorial Map requires Voronoi computation, polygon rendering, and label management that becomes expensive at scale. The Constellation Atlas is the only proposed visualization where rendering ALL 50K entities simultaneously is not just feasible but visually coherent — a dense star field with bright stars standing out.

3. **Natural Focus Lens integration**. The "telescope zoom" metaphor (click a star to zoom in) is an intuitive physical metaphor for the Semantic Focus Lens. Users already understand that a telescope narrows your view to show more detail. The focus/dim/ghost behavior maps perfectly to star brightness adjustment.

**LNO classification**: LEVERAGE. This is the 30-second first impression. If the Constellation Atlas does not make the user want to keep exploring, nothing else matters.

**Build cost**: Medium. WebGL setup requires a custom point-sprite shader (~200 lines of GLSL), but the layout is pre-computed server-side. The main engineering work is the focus transition animation and label management.

---

### Recommendation 2: Build "The Solar System" second

**Magic moment served**: Activation (2 minutes).

**Why this one**: The activation moment is "click a trait method call, see the EXACT implementation." That happens at the entity zoom level. The Solar System is designed precisely for this — it renders the ego network of a single focused entity with PPR-ranked importance. When the user clicks a function from the Constellation Atlas, the view transitions from the night sky to the Solar System: the clicked star becomes the sun, its neighbors orbit as planets.

This creates a cinematic zoom transition: **constellation view (macro) to solar system (micro)** — a physically consistent metaphor. The user is "telescoping" from seeing a star in the sky to visiting the star's planetary system.

The Solar System also serves the "dispatch resolution" magic moment directly. When the user clicks a trait method call (a planet in the system), the resolved implementation is shown as a specific planet — "This calls Consumer::poll via the Stream trait, static dispatch." The planet is labeled with the concrete type, not the abstract trait.

**LNO classification**: LEVERAGE. The entity-level ego network is the #1 navigation tool for actual codebase reading.

**Build cost**: Low-Medium. Canvas 2D with simple polar layout. The orbital animation is optional decoration — the core value is the radial layout with PPR-sized planets.

---

### Recommendation 3: Build "The Library Shelves" third

**Magic moment served**: Retention (session 2).

**Why this one**: Retention is about the user coming back on Day 2. The thesis identifies two retention drivers: (a) "it remembers where I was" (reading history), and (b) "I can see my progress" (coverage visualization). The Library Shelves is the only proposed visualization that makes reading progress a first-class visual element — bookmark ribbons on read entities, visible gaps on unread shelves.

The Library Shelves also serve a critical UX role: they provide a **stable spatial reference**. The Constellation Atlas and Solar System are focus-driven — they change every time you click. The Library Shelves are static — the books are always in the same position on the shelf, like a real library. This gives the user a "home base" to return to, a place where they can see the whole landscape and their progress within it.

This maps to the thesis's "Kindle for codebases" metaphor. A Kindle shows your book collection, reading progress per book, and bookmarks. The Library Shelves do the same for a codebase.

**LNO classification**: LEVERAGE for retention. The "coverage visibility" feature is called out in multiple persona journeys (Sarah's "40% explored," Priya's session continuity).

**Build cost**: Low. Pure HTML/CSS with simple flexbox layout. No Canvas, no WebGL, no animation framework. The 3D book-pull effect is pure CSS transforms. This is the cheapest of the three to build.

---

### Build sequence rationale

| Order | Visualization | Zoom level | Magic moment | Emotional stage |
|---|---|---|---|---|
| 1 | Constellation Atlas | Workspace | Acquisition (30s) | Anxiety to relief |
| 2 | Solar System | Entity | Activation (2min) | Relief to curiosity |
| 3 | Library Shelves | Subsystem | Retention (Day 2) | Curiosity to competence |

This sequence creates a coherent visual journey:
1. Open the app. See the constellation. "Oh, this codebase has structure."
2. Click a bright star. Zoom into the solar system. "I can see what this function connects to, and I know exactly which implementation it calls."
3. Switch to library view. See your shelves, your bookmarks, your progress. "I know what I have read and what I need to read next." Come back tomorrow.

The three visualizations cover three different zoom levels, three different magic moments, and three different emotional states. Together, they constitute a complete visual navigation system for the reading workflow that is Parseltongue's core thesis.

---

## Confidence and Caveats

**High confidence**:
- The data mappings are well-grounded in the ISG schema and the thesis's data model. Every visual element maps to a specific, queryable ISG field.
- The performance assessments are realistic for Chromium-based WebViews. WebGL point sprites, Canvas 2D, and CSS-only approaches are all well-understood rendering paths with known scaling characteristics.
- The top-3 selection is directly tied to the Shreyas Doshi magic moment framework from Section 14 of the thesis. The reasoning chain (acquisition to activation to retention) is explicit and verifiable.

**Areas for independent verification**:
- WebGL shader performance on Tauri's specific WebView version should be benchmarked with a 50K-point test. Tauri's WebView may have GPU acceleration limitations on certain macOS hardware.
- The "Milky Way" density rendering in the Constellation Atlas may require experimentation to find the right Gaussian blur parameters that look good without being computationally expensive.
- The d3-force layout for entity positioning in the Constellation Atlas should be benchmarked for convergence time on 50K entities. If server-side pre-computation exceeds 30 seconds, consider using a faster layout algorithm (e.g., ForceAtlas2).

**Assumptions that could change the analysis**:
- These recommendations assume the primary user entry point is the visual map, not a search bar. If testing reveals that most users start with search (not map browsing), the Library Shelves or Subway Map might be better first-build candidates than the Constellation Atlas.
- The "retention via visible progress" hypothesis (Library Shelves for Day 2) is intuitive but unvalidated. If user testing shows that Day 2 retention is driven by LLM suggestions rather than visual progress, the Library Shelves could be deferred in favor of the Pulse Monitor (which surfaces actionable architectural insights).
- The k-core shells required by the Geologic Strata may not yet be precomputed in the current pipeline. If k-core is a P1 algorithm (per the PRD), the Strata visualization would need to wait for that computation to be implemented.
