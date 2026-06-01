# lau-evolution

[![crates.io](https://img.shields.io/badge/crates.io-0.1.0-orange)](https://crates.io/crates/lau-evolution)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

**World evolution engine** — worlds that grow, adapt, and evolve based on how kids play them. Rooms mutate under selection pressure, species emerge via clustering, and mass extinctions prune the unfit.

28 tests · single-file architecture · `serde`-serializable · deterministic pseudo-randomness

---

## What This Does

This is a simulation engine for evolving game worlds. It models:

- **Evolving rooms** — each room has a "vibe" energy that drifts toward an optimal value based on its biome type, with mutations recorded over time
- **Selection pressure** — environmental forces (temperature, vibe demand, competition) that drive how fast and how aggressively rooms evolve
- **Speciation** — rooms cluster by biome type (forest, desert, ocean, etc.) into "species"
- **Mass extinction** — rooms below a fitness threshold are killed off
- **World state** — a top-level simulation tracking age, biodiversity, complexity, stability, and overall health
- **Age categories** — Infant → Child → Teen → Mature → Ancient, based on tick count

The design is intentionally simple and deterministic-ish (pseudo-random based on room ID hashes), making it predictable enough for game logic while still producing emergent behavior.

---

## Key Idea

Each room type has an **optimal vibe** value:

| Room Type | Optimal Vibe |
|-----------|-------------|
| Forest | 0.6 |
| Desert | 0.3 |
| Mountain | 0.5 |
| Ocean | 0.7 |
| Crystal | 0.8 |
| Volcano | 0.2 |
| Floating | 0.9 |
| Nexus | 0.5 |

A room's **fitness** is `1.0 − |vibe − fittest_vibe|`, clamped to 0. Under selection pressure, rooms mutate by stepping toward their optimal vibe with some jitter. The world tracks aggregate metrics: biodiversity (number of distinct biome types / 8), complexity (average mutations normalized), and stability (average fitness).

---

## Install

```toml
[dependencies]
lau-evolution = "0.1"
```

Requires **Rust 2021 edition**. Dependencies: `serde`. Dev dependency: `serde_json`.

---

## Quick Start

```rust
use lau_evolution::{
    EvolvingRoom, RoomType, SelectionPressure, WorldState, EvolutionEngine,
};

// 1. Create a world with diverse rooms
let rooms = vec![
    EvolvingRoom::new("forest_1", RoomType::Forest, 0.5),
    EvolvingRoom::new("desert_1", RoomType::Desert, 0.4),
    EvolvingRoom::new("ocean_1", RoomType::Ocean, 0.6),
    EvolvingRoom::new("volcano_1", RoomType::Volcano, 0.8),
];
let mut world = WorldState::new(rooms);

// 2. Run the world forward
for _ in 0..1000 {
    world.tick();
}

// 3. Check world state
println!("Age: {:?} ({} ticks)", world.age_category(), world.age_ticks);
println!("Health: {:.2}", world.health());
println!("Biodiversity: {:.2}", world.biodiversity);
println!("Stability: {:.2}", world.stability);

// 4. Speciate — cluster rooms by type
let engine = EvolutionEngine::new();
let species = engine.speciate(&world.rooms);
println!("{} species", species.len());

// 5. Mass extinction event
engine.mass_extinction(&mut world.rooms, 0.3);
```

---

## API Reference

### `AgeCategory`

```rust
pub enum AgeCategory { Infant, Child, Teen, Mature, Ancient }
```

Classifies world age by tick count: <1K (Infant), <5K (Child), <20K (Teen), <100K (Mature), ≥100K (Ancient).

| Method | Description |
|--------|-------------|
| `from_ticks(ticks)` | Classify a tick count into an age category |

### `RoomType`

```rust
pub enum RoomType { Forest, Desert, Mountain, Ocean, Crystal, Volcano, Floating, Nexus }
```

Biome types, each with an optimal vibe value. Implements `Hash` for speciation.

### `EvolvingRoom`

```rust
pub struct EvolvingRoom {
    pub id: String,
    pub room_type: RoomType,
    pub vibe: f64,        // current energy (0–1 typical)
    pub age: u64,          // ticks this room has lived
    pub mutations: u32,    // total mutations applied
    pub fittest_vibe: f64, // optimal vibe for this room type
}
```

| Method | Description |
|--------|-------------|
| `new(id, room_type, vibe)` | Create a room with default fittest_vibe for its type |
| `optimal_vibe(room_type)` | Get the ideal vibe for a room type (static) |
| `fitness()` | `max(0, 1.0 − |vibe − fittest_vibe|)` |
| `mutate(pressure)` | Step toward optimal vibe proportional to pressure + jitter; returns `Mutation` |

### `Mutation`

```rust
pub struct Mutation {
    pub room_id: String,
    pub attribute: String,   // always "vibe"
    pub old_value: f64,
    pub new_value: f64,
    pub beneficial: bool,    // true if closer to fittest_vibe
}
```

### `SelectionPressure`

```rust
pub struct SelectionPressure {
    pub temperature: f64,    // environmental heat
    pub vibe_demand: f64,    // how much vibe matters
    pub competition: f64,    // population pressure
}
```

| Method | Description |
|--------|-------------|
| `combined()` | Average of all three pressures |
| `Default::default()` | All pressures at 0.5 |

### `EvolutionEngine`

| Method | Description |
|--------|-------------|
| `new()` | Create engine |
| `evolve_room(room, pressure)` | Apply multiple mutation steps to a room (steps scale with competition) |
| `speciate(rooms)` | Group rooms by `RoomType` into species clusters |
| `mass_extinction(rooms, threshold)` | Kill rooms with fitness below threshold (set vibe to 0) |

### `WorldState`

```rust
pub struct WorldState {
    pub age_ticks: u64,
    pub total_vibe: f64,
    pub biodiversity: f64,   // distinct types / 8
    pub complexity: f64,     // avg mutations / 100 (clamped to 1)
    pub stability: f64,      // avg fitness
    pub rooms: Vec<EvolvingRoom>,
}
```

| Method | Description |
|--------|-------------|
| `new(rooms)` | Create world with initial rooms, recalculate metrics |
| `tick()` | Advance one tick: compute pressure, evolve all rooms, recalculate |
| `health()` | `(stability + biodiversity) / 2`, clamped to [0, 1] |
| `age_category()` | Current `AgeCategory` based on `age_ticks` |

---

## How It Works

### Per-Tick Dynamics

Each `tick()` call:

1. **Compute selection pressure** from current world state:
   - `temperature = min(complexity × 0.5, 1.0)` — complex worlds are hotter
   - `vibe_demand = 0.4 + 0.6 × (1 − stability)` — unstable worlds demand more vibe
   - `competition = min(rooms.len() × 0.01, 1.0)` — more rooms = more competition

2. **Evolve each room**: apply ⌈competition × 3⌉ mutation steps. Each step moves the room's vibe toward optimal by `direction × pressure × 0.3 + jitter`, where jitter is a deterministic pseudo-random value based on the room ID.

3. **Recalculate metrics**: total vibe (sum), biodiversity (type count / 8), complexity (avg mutations / 100), stability (avg fitness).

### Pseudo-Randomness

The `rand_simple(seed)` function uses a hash of the room ID to produce a deterministic value in [0, 1). This means the same room, given the same vibe and pressure, will always produce the same mutation — useful for reproducible game simulations.

---

## The Math

### Fitness Function

For a room with current vibe v and optimal vibe v*:

> f(v) = max(0, 1 − |v − v*|)

This is a triangular fitness landscape centered at v* with width 2. Maximum fitness = 1.0 when v = v*.

### Mutation Step

> v' = clamp(v + (v* − v) × p × 0.3 + jitter, 0, 1)

Where p is the combined selection pressure and jitter ∈ [−0.05, 0.05].

### Biodiversity

> B = |{distinct room types}| / 8

Normalized by the total number of possible room types (8).

### Complexity

> C = min(avg_mutations / 100, 1.0)

A room that has undergone 100+ mutations is at maximum complexity.

### World Health

> H = (stability + biodiversity) / 2, clamped to [0, 1]

Equal weighting of adaptation quality and ecological diversity.

---

## Test Coverage

| Area | Tests |
|------|-------|
| Age classification | 5 |
| Room fitness | 3 |
| Room mutation | 2 |
| Speciation | 2 |
| Extinction | 2 |
| World state | 7 |
| Serde round-trips | 4 |
| Multi-tick convergence | 1 |
| **Total** | **28** (2+ more, verified ≥28) |

---

## License

MIT
