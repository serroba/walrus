# Plan 10: World Map TUI

## Goal

Build a ratatui-based terminal UI that renders a stylized world map showing all
continents, with color-coded overlays for population dynamics, migrations, wars,
natural disasters, resource depletion, and civilizational emergence. The map is
the primary human interface; a parallel JSONL stream is the AI-agent interface.

## Context

The simulation has two geographic layers that need to be unified in the map view:

1. **Evolution layer** (`evolution.rs`): 4 named continents (Africa, Eurasia,
   Americas, Oceania) with `SocietyActor`s placed by continent index. Migration
   corridors, natural disasters, pandemics, resource depletion per continent.

2. **Agent layer** (`agents.rs`): Individual agents with `(x, y)` positions in a
   continuous `world_size × world_size` space. Kin groups, raids, tribute,
   cultural transmission. No geographic continent mapping yet.

The TUI must work with both layers. Phase A targets the evolution layer (continent
granularity). Phase B extends to the agent layer (sub-continent positioning).

---

## Architecture

```
SimulationEngine
    │
    │ channel (crossbeam or std::sync::mpsc)
    │
    ├── MapFrame { generation, continent_states, events, societies, corridors }
    │
    ▼
┌─────────────────────────────────────────────────────────────┐
│ TUI Thread                                                  │
│                                                             │
│  ┌─────────────────────────────────────┬──────────────────┐ │
│  │          World Map                  │  Continent Panel │ │
│  │  (80×30 cell grid, continent-      │  - name          │ │
│  │   tagged, color-filled by state)   │  - population    │ │
│  │                                     │  - complexity    │ │
│  │  Migration arrows between           │  - depletion     │ │
│  │  continent centroids                │  - governance    │ │
│  │                                     │  - shock events  │ │
│  │  Flash overlays for events          │                  │ │
│  ├─────────────────────────────────────┤  Event Log       │ │
│  │  Timeline sparklines (bottom)       │  - last N events │ │
│  │  population, complexity, SO index   │  - scrollable    │ │
│  └─────────────────────────────────────┴──────────────────┘ │
│                                                             │
│  Keybindings: [q]uit [p]ause [+/-] speed [Tab] focus       │
└─────────────────────────────────────────────────────────────┘
```

Parallel output:
```
SimulationEngine ──→ --format jsonl ──→ stdout (AI agent reads this)
```

Same data, different renderer. The simulation doesn't know or care who's consuming.

---

## The World Map Bitmap

A hardcoded `const` array mapping terminal cells to continent indices. Roughly
80 columns × 30 rows, using a simplified Mercator-ish projection recognizable
at a glance. Each cell is either ocean (`None`) or tagged with a continent index
(0=Africa, 1=Eurasia, 2=Americas, 3=Oceania).

```rust
/// Each char: ' ' = ocean, '0' = Africa, '1' = Eurasia, '2' = Americas, '3' = Oceania
const WORLD_MAP: &[&str] = &[
    //          10        20        30        40        50        60        70
    "                                                                        ",
    "                         1111111                                        ",
    "                  111111111111111111111                                  ",
    "                1111111111111111111111111                                ",
    "   222         11111111111111111111111111111                             ",
    "  22222        111111111111111111111111111111          33                ",
    "  222222        0000 1111111111111111111111           3333               ",
    " 2222222        00000  1111111111111111                333              ",
    "  2222222       000000   111111111111                   3               ",
    "  22222222      0000000    1111111                                      ",
    "   222222       00000000                                                ",
    "    22222        000000                                                 ",
    "     2222         0000                                                  ",
    "      222          00                                                   ",
    "       2                                                                ",
];
```

This is illustrative — the actual bitmap will be refined to look good at common
terminal sizes (120×40 and 80×24). The key property: each cell knows which
continent it belongs to, so we can color-fill programmatically.

Continent centroids (precomputed from the bitmap) anchor migration arrows and
event flash origins.

---

## Color Coding

All colors use ratatui `Style` with 256-color or truecolor support, falling back
to 16-color where needed.

| Signal | Base Color | Encoding |
|---|---|---|
| Population density | Green gradient | `Color::Rgb(0, intensity, 0)` scaled by pop / carrying_capacity |
| Resource depletion | Green → Grey | As depletion rises, green component drops, replaced by grey |
| Active war / raid | Red flash | Affected continent cells pulse red for N frames after event |
| Migration flow | Cyan arrows | Drawn along corridor paths between continent centroids. Arrow character density ∝ corridor strength |
| Natural disaster | Yellow/amber flash | Continent cells flash `Color::Yellow` for N frames |
| Pandemic | Magenta wash | Lighter magenta overlay for duration |
| Collapse event | Blinking red border | Continent outline blinks when a society collapses |
| Civilizational emergence | Blue tint | Blue component added proportional to mean complexity |
| Superorganism signal | White brightening | Cell brightness tracks superorganism index |

Multiple signals composite: a populous, complex continent with an active disaster
shows green+blue base with yellow flash.

---

## Data Flow

### Snapshot Protocol

```rust
/// Emitted by the simulation each generation, consumed by TUI and JSONL writer.
pub struct MapFrame {
    pub generation: u32,
    pub sim_time: f64,
    pub continents: Vec<ContinentFrame>,
    pub corridors: Vec<CorridorFrame>,
    pub events: Vec<MapEvent>,
    pub global: GlobalFrame,
}

pub struct ContinentFrame {
    pub index: usize,
    pub name: String,
    pub population: u64,
    pub society_count: usize,
    pub mean_complexity: f64,
    pub mean_resilience: f64,
    pub stock: f64,
    pub depletion: f64,
    pub carrying_capacity: f64,
}

pub struct CorridorFrame {
    pub from: usize,
    pub to: usize,
    pub strength: f64,
    pub active_migrations: u32,
}

pub enum MapEvent {
    War { continent: usize, casualties: u32 },
    NaturalDisaster { continent: usize, severity: f64 },
    Pandemic { continent: usize, severity: f64 },
    Collapse { continent: usize, society_id: u64 },
    ModeTransition { continent: usize, from: SubsistenceMode, to: SubsistenceMode },
    Emergence { continent: usize, superorganism_index: f64 },
}

pub struct GlobalFrame {
    pub total_population: u64,
    pub mean_complexity: f64,
    pub superorganism_index: f64,
    pub convergence_index: f64,
}
```

The simulation pushes `MapFrame` into a bounded channel. The TUI pops and renders.
If the TUI is slower than the simulation, frames are dropped (latest-wins).
The JSONL writer serializes every frame to stdout.

### Channel Design

```rust
// Simulation thread
let (tx, rx) = std::sync::mpsc::sync_channel::<MapFrame>(4);
// Bounded buffer of 4 frames. If TUI is behind, oldest dropped.

// TUI thread
loop {
    // Drain to latest frame (non-blocking)
    let mut frame = None;
    while let Ok(f) = rx.try_recv() {
        frame = Some(f);
    }
    if let Some(f) = frame {
        render(&mut terminal, &f, &mut state);
    }
    // Handle input events (crossterm)
    if crossterm::event::poll(Duration::from_millis(16))? {
        handle_input(...);
    }
}
```

This gives ~60fps rendering with zero backpressure on the simulation.

---

## Implementation Phases

### Phase A: Static Map + Evolution Layer (foundation)

**New files:**
- `src/tui.rs` — map bitmap, rendering logic, TUI state
- `examples/world_map_tui.rs` — entry point wiring `simulate_evolution` to TUI

**New dependencies:**
- `ratatui = "0.29"` (terminal UI framework)
- `crossterm = "0.28"` (terminal backend + input events)

**Steps:**

1. Add ratatui and crossterm to `Cargo.toml`
2. Define `WORLD_MAP` bitmap constant — each cell tagged with continent index
3. Implement `render_map()`: iterate bitmap, color each cell based on continent
   state from `EvolutionSnapshot` + `ContinentState`
4. Implement `render_sidebar()`: per-continent stats table
5. Implement `render_timeline()`: sparklines for population, complexity, SO index
6. Wire to `simulate_evolution()`: run simulation in a thread, push snapshots
   through a channel, TUI thread renders
7. Basic input handling: `q` to quit, `p` to pause, `+`/`-` to control speed

**Deliverable:** Running TUI showing a color-coded world map with continent
stats updating in real-time as the evolution simulation runs.

**Validates:** Map is recognizable, colors are readable, frame rate is smooth.

**Effort:** 2-3 sessions

---

### Phase B: Event Overlays + Migration Arrows

**Modifies:** `src/tui.rs`

**Steps:**

1. Parse `ActorMessage` variants (NaturalDisaster, PandemicWave, ClimateShock,
   MigrationLink) into `MapEvent` enum
2. Implement event flash system: each event sets a per-continent timer that
   decays over N frames. During decay, overlay color is blended onto cells.
3. Draw migration arrows between continent centroids using Unicode box-drawing
   or arrow characters (`→`, `↗`, `↘`, etc.). Arrow visibility proportional
   to corridor strength.
4. Draw war indicators when inter-society raids/collapses occur
5. Add event log panel (scrollable list of recent events with timestamps)

**Deliverable:** Map comes alive with flashing events, visible migration flows,
and a scrollable event history.

**Effort:** 1-2 sessions

---

### Phase C: Agent Layer Integration

**Modifies:** `src/tui.rs`, `src/agents.rs` (add MapFrame emission)

This phase bridges the agent-level simulation (`agents.rs`) to the map. The
agent sim currently uses a flat `(x, y)` coordinate space with no continent
mapping.

**Steps:**

1. Define continent regions in the agent sim's coordinate space. Partition the
   `world_size × world_size` space into regions matching the 4 continents.
   Each agent's `(x, y)` maps to a continent based on which region it falls in.
2. Add `MapFrame` emission to `simulate_agents()` — aggregate per-continent
   stats from the Population struct every N ticks
3. Map agent-level events to `MapEvent`: kin-group raids → War, migration
   (resource-stress relocation) → migration arrows, institutional emergence →
   Emergence
4. Wire `examples/world_map_tui.rs` to support both `--evolution` and `--agents`
   modes, or auto-detect based on config

**Design decision:** The agent layer has richer data (individual positions, kin
groups, cultural traits). The map view aggregates to continent level for
readability. A future zoom mode could show sub-continent detail.

**Deliverable:** Same TUI works with both evolution-layer and agent-layer
simulations.

**Effort:** 2-3 sessions

---

### Phase D: JSONL Output + AI Interface

**New files:** `src/jsonl.rs` or inline in `src/tui.rs`

**Steps:**

1. Add `serde` and `serde_json` as dependencies (feature-gated behind `serde`)
2. Derive `Serialize` on `MapFrame` and all sub-structs
3. Add `--format jsonl` flag to the example binary. When set, skip TUI setup
   and write each `MapFrame` as a JSON line to stdout
4. Add `--format summary` for a compact text summary per generation (for quick
   AI iteration without full JSON)
5. Document the schema so I (AI agent) know what fields to expect

**Deliverable:** `cargo run --example world_map_tui -- --format jsonl` streams
structured data I can pipe, grep, and analyze.

**Effort:** 1 session

---

### Phase E: Scenario Comparison Mode

**Modifies:** `src/tui.rs`

**Steps:**

1. Accept multiple scenario configs on the command line
2. Run simulations in parallel threads, each emitting `MapFrame` on its own channel
3. Split-screen layout: two maps side by side, synchronized by generation
4. Shared timeline at the bottom with overlaid sparklines (different colors per
   scenario)
5. Delta panel: show divergence metrics (population diff, complexity diff, etc.)
   between scenarios at current generation

**Layout:**
```
┌──────────── Scenario A ────────────┬──────────── Scenario B ────────────┐
│         [world map]                │         [world map]                │
│                                    │                                    │
├────────────────────────────────────┴────────────────────────────────────┤
│  ▁▃▅▇█▇▅▃ Pop A (green)  ▁▂▃▅▇█▇▅ Pop B (cyan)    Δpop: +1,240      │
│  ▁▂▃▅▇█▇▅ CX  A (blue)   ▁▁▂▃▅▇▇▅ CX  B (yellow)  ΔCX: -0.12       │
└─────────────────────────────────────────────────────────────────────────┘
```

**Deliverable:** Side-by-side "what-if" comparison in the terminal.

**Effort:** 2 sessions

---

### Phase F: Interactivity + Playback

**Modifies:** `src/tui.rs`

**Steps:**

1. Store all received `MapFrame`s in a ring buffer (capped at N generations)
2. Arrow keys to scrub forward/backward through history
3. Tab to cycle focus between continents; focused continent shows expanded stats
4. `s` to save current frame as JSON (for later analysis)
5. `r` to restart simulation with a new seed
6. Mouse support: click a continent to focus it

**Deliverable:** Full interactive exploration of simulation history.

**Effort:** 1-2 sessions

---

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `ratatui` | 0.29 | Terminal UI framework (widgets, layout, styling) |
| `crossterm` | 0.28 | Terminal backend (raw mode, input events, colors) |
| `serde` | 1 | Serialization (feature-gated, for JSONL output) |
| `serde_json` | 1 | JSON serialization (feature-gated) |

All are well-maintained, widely used crates. `serde` is feature-gated so the
core engine stays dependency-light.

---

## Relationship to Plan 09 (Event-Driven Migration)

Plan 09 replaces the tick-based loop with a discrete-event simulation. The TUI
design is compatible with both models:

- **Tick-based (current):** Emit `MapFrame` every N ticks
- **Event-driven (future):** Emit `MapFrame` at `WorldEvent::MeasureState` intervals

The `MapFrame` protocol abstracts over the time model. The TUI doesn't care
whether the simulation advances by tick or by event — it just renders the latest
frame. This means Plan 10 can be built now and will survive the Plan 09 migration
unchanged.

---

## Open Questions

1. **Map resolution:** 80×30 is safe for most terminals. Should we detect terminal
   size and scale the map bitmap? Or ship two sizes (compact and wide)?

2. **Sub-continent detail:** The agent layer has `(x, y)` positions. Should we
   eventually support zooming into a continent to see individual agent clusters?
   This would require a second bitmap per continent at higher resolution.

3. **Color accessibility:** Some users are colorblind. Should we support a
   high-contrast mode using shapes/patterns instead of color alone?

4. **Recording:** Should we support recording a full session to a file for
   replay? This is trivially a sequence of `MapFrame` in JSONL — the same
   format the AI agent reads.

5. **Multiple map projections:** The hardcoded bitmap is one projection. Worth
   supporting alternatives (e.g., a more equal-area layout) or is one enough?
