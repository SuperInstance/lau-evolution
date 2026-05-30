use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AgeCategory
// ---------------------------------------------------------------------------

/// Evolutionary age of the world, based on total tick count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AgeCategory {
    Infant,
    Child,
    Teen,
    Mature,
    Ancient,
}

impl AgeCategory {
    /// Classify a tick count into an [`AgeCategory`].
    pub fn from_ticks(ticks: u64) -> Self {
        if ticks < 1_000 {
            AgeCategory::Infant
        } else if ticks < 5_000 {
            AgeCategory::Child
        } else if ticks < 20_000 {
            AgeCategory::Teen
        } else if ticks < 100_000 {
            AgeCategory::Mature
        } else {
            AgeCategory::Ancient
        }
    }
}

// ---------------------------------------------------------------------------
// RoomType
// ---------------------------------------------------------------------------

/// The biome / flavour of an [`EvolvingRoom`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RoomType {
    Forest,
    Desert,
    Mountain,
    Ocean,
    Crystal,
    Volcano,
    Floating,
    Nexus,
}

// ---------------------------------------------------------------------------
// Mutation
// ---------------------------------------------------------------------------

/// A single recorded mutation applied to a room.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mutation {
    pub room_id: String,
    pub attribute: String,
    pub old_value: f64,
    pub new_value: f64,
    pub beneficial: bool,
}

// ---------------------------------------------------------------------------
// EvolvingRoom
// ---------------------------------------------------------------------------

/// A room that evolves over time under selection pressure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolvingRoom {
    pub id: String,
    pub room_type: RoomType,
    /// Current vibe energy (0–1 typical, but can exceed 1 in extreme worlds).
    pub vibe: f64,
    pub age: u64,
    pub mutations: u32,
    /// The vibe value that maximises fitness for this room's type.
    pub fittest_vibe: f64,
}

impl EvolvingRoom {
    /// Create a new room with default fittest_vibe for its type.
    pub fn new(id: impl Into<String>, room_type: RoomType, vibe: f64) -> Self {
        let fittest_vibe = Self::optimal_vibe(room_type);
        Self {
            id: id.into(),
            room_type,
            vibe,
            age: 0,
            mutations: 0,
            fittest_vibe,
        }
    }

    /// The "ideal" vibe for a given room type.
    pub fn optimal_vibe(room_type: RoomType) -> f64 {
        match room_type {
            RoomType::Forest => 0.6,
            RoomType::Desert => 0.3,
            RoomType::Mountain => 0.5,
            RoomType::Ocean => 0.7,
            RoomType::Crystal => 0.8,
            RoomType::Volcano => 0.2,
            RoomType::Floating => 0.9,
            RoomType::Nexus => 0.5,
        }
    }

    /// How well-adapted is this room? Returns 1.0 when `vibe == fittest_vibe`,
    /// decaying toward 0 as vibe drifts.
    pub fn fitness(&self) -> f64 {
        let diff = (self.vibe - self.fittest_vibe).abs();
        (1.0 - diff).max(0.0)
    }

    /// Evolve the room under a given selection pressure (0–1 typical).
    /// Returns the applied mutation, if any.
    pub fn mutate(&mut self, pressure: f64) -> Option<Mutation> {
        let old_vibe = self.vibe;
        // Step toward fittest_vibe proportional to pressure, with some jitter.
        let direction = self.fittest_vibe - self.vibe;
        let jitter = (rand_simple(self.id.as_str()) - 0.5) * 0.1;
        let step = direction * pressure * 0.3 + jitter;
        self.vibe = (self.vibe + step).clamp(0.0, 1.0);
        self.age += 1;
        self.mutations += 1;
        let new_vibe = self.vibe;
        let beneficial = (new_vibe - self.fittest_vibe).abs()
            <= (old_vibe - self.fittest_vibe).abs();
        Some(Mutation {
            room_id: self.id.clone(),
            attribute: "vibe".into(),
            old_value: old_vibe,
            new_value: new_vibe,
            beneficial,
        })
    }
}

/// Deterministic-ish pseudo-random in [0, 1) based on seed string.
fn rand_simple(seed: &str) -> f64 {
    let hash = seed
        .bytes()
        .fold(0u64.wrapping_add(1), |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    (hash % 10_000) as f64 / 10_000.0
}

// ---------------------------------------------------------------------------
// SelectionPressure
// ---------------------------------------------------------------------------

/// Environmental pressures that drive evolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionPressure {
    pub temperature: f64,
    pub vibe_demand: f64,
    pub competition: f64,
}

impl Default for SelectionPressure {
    fn default() -> Self {
        Self {
            temperature: 0.5,
            vibe_demand: 0.5,
            competition: 0.5,
        }
    }
}

impl SelectionPressure {
    /// Combined pressure magnitude (0–1).
    pub fn combined(&self) -> f64 {
        (self.temperature + self.vibe_demand + self.competition) / 3.0
    }
}

// ---------------------------------------------------------------------------
// EvolutionEngine
// ---------------------------------------------------------------------------

/// Drives evolution across rooms.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvolutionEngine;

impl EvolutionEngine {
    pub fn new() -> Self {
        Self
    }

    /// Evolve a single room under the given pressure.
    pub fn evolve_room(
        &mut self,
        room: &mut EvolvingRoom,
        pressure: &SelectionPressure,
    ) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        // Number of evolution steps scales with competition.
        let steps = (pressure.competition * 3.0).ceil() as u32;
        for _ in 0..steps.max(1) {
            if let Some(m) = room.mutate(pressure.combined()) {
                mutations.push(m);
            }
        }
        mutations
    }

    /// Cluster rooms by type similarity — rooms of the same [`RoomType`] form a species.
    pub fn speciate<'a>(
        &self,
        rooms: &'a [EvolvingRoom],
    ) -> Vec<Vec<&'a EvolvingRoom>> {
        let mut species: Vec<Vec<&'a EvolvingRoom>> = Vec::new();
        for room in rooms {
            if let Some(slot) = species
                .iter_mut()
                .find(|s| s.first().is_some_and(|r| r.room_type == room.room_type))
            {
                slot.push(room);
            } else {
                species.push(vec![room]);
            }
        }
        species
    }

    /// Kill off rooms whose fitness is below `threshold`.
    /// Rooms that "die" have their vibe set to 0 (marked dead).
    pub fn mass_extinction(&self, rooms: &mut [EvolvingRoom], threshold: f64) {
        for room in rooms.iter_mut() {
            if room.fitness() < threshold {
                room.vibe = 0.0;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// WorldState
// ---------------------------------------------------------------------------

/// The top-level world that houses evolving rooms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub age_ticks: u64,
    pub total_vibe: f64,
    pub biodiversity: f64,
    pub complexity: f64,
    pub stability: f64,
    pub rooms: Vec<EvolvingRoom>,
}

impl WorldState {
    pub fn new(rooms: Vec<EvolvingRoom>) -> Self {
        let mut ws = Self {
            age_ticks: 0,
            total_vibe: 0.0,
            biodiversity: 0.0,
            complexity: 0.0,
            stability: 1.0,
            rooms,
        };
        ws.recalculate();
        ws
    }

    /// Advance the world by one tick.
    pub fn tick(&mut self) {
        let pressure = SelectionPressure {
            temperature: (self.complexity * 0.5).min(1.0),
            vibe_demand: 0.4 + 0.6 * (1.0 - self.stability),
            competition: (self.rooms.len() as f64 * 0.01).min(1.0),
        };
        let mut engine = EvolutionEngine::new();
        for room in &mut self.rooms {
            engine.evolve_room(room, &pressure);
        }
        self.age_ticks += 1;
        self.recalculate();
    }

    /// Overall world health (0–1).
    pub fn health(&self) -> f64 {
        ((self.stability + self.biodiversity) / 2.0).clamp(0.0, 1.0)
    }

    /// Evolutionary age category.
    pub fn age_category(&self) -> AgeCategory {
        AgeCategory::from_ticks(self.age_ticks)
    }

    fn recalculate(&mut self) {
        if self.rooms.is_empty() {
            self.total_vibe = 0.0;
            self.biodiversity = 0.0;
            self.complexity = 0.0;
            self.stability = 1.0;
            return;
        }
        let n = self.rooms.len() as f64;
        self.total_vibe = self.rooms.iter().map(|r| r.vibe).sum();

        // Biodiversity: number of distinct room types / total possible types.
        let type_count = {
            let mut types = std::collections::HashSet::new();
            for r in &self.rooms {
                types.insert(r.room_type);
            }
            types.len()
        };
        self.biodiversity = type_count as f64 / 8.0;

        // Complexity: average mutations.
        let avg_mutations: f64 = self.rooms.iter().map(|r| r.mutations as f64).sum::<f64>() / n;
        self.complexity = (avg_mutations / 100.0).min(1.0);

        // Stability: average fitness.
        let avg_fitness: f64 = self.rooms.iter().map(|r| r.fitness()).sum::<f64>() / n;
        self.stability = avg_fitness;
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- AgeCategory ---------------------------------------------------------

    #[test]
    fn age_category_infant() {
        assert_eq!(AgeCategory::from_ticks(0), AgeCategory::Infant);
        assert_eq!(AgeCategory::from_ticks(999), AgeCategory::Infant);
    }

    #[test]
    fn age_category_child() {
        assert_eq!(AgeCategory::from_ticks(1_000), AgeCategory::Child);
        assert_eq!(AgeCategory::from_ticks(4_999), AgeCategory::Child);
    }

    #[test]
    fn age_category_teen() {
        assert_eq!(AgeCategory::from_ticks(5_000), AgeCategory::Teen);
        assert_eq!(AgeCategory::from_ticks(19_999), AgeCategory::Teen);
    }

    #[test]
    fn age_category_mature() {
        assert_eq!(AgeCategory::from_ticks(20_000), AgeCategory::Mature);
        assert_eq!(AgeCategory::from_ticks(99_999), AgeCategory::Mature);
    }

    #[test]
    fn age_category_ancient() {
        assert_eq!(AgeCategory::from_ticks(100_000), AgeCategory::Ancient);
        assert_eq!(AgeCategory::from_ticks(999_999), AgeCategory::Ancient);
    }

    // -- EvolvingRoom --------------------------------------------------------

    #[test]
    fn room_fitness_perfect() {
        let room = EvolvingRoom::new("r1", RoomType::Forest, 0.6);
        assert!((room.fitness() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn room_fitness_zero() {
        let room = EvolvingRoom::new("r1", RoomType::Forest, 0.0);
        // optimal is 0.6, diff = 0.6, fitness = 1.0 - 0.6 = 0.4
        assert!((room.fitness() - 0.4).abs() < 1e-9);
    }

    #[test]
    fn room_fitness_negative_clamped() {
        let room = EvolvingRoom::new("r1", RoomType::Volcano, 1.0);
        // optimal is 0.2, diff = 0.8, fitness = 1.0 - 0.8 = 0.2
        assert!((room.fitness() - 0.2).abs() < 1e-9);
    }

    #[test]
    fn room_mutate_changes_vibe() {
        let mut room = EvolvingRoom::new("r1", RoomType::Desert, 0.0);
        let m = room.mutate(0.8).unwrap();
        assert_ne!(m.old_value, m.new_value);
        assert_eq!(room.mutations, 1);
    }

    #[test]
    fn optimal_vibe_varies() {
        assert_ne!(
            EvolvingRoom::optimal_vibe(RoomType::Forest),
            EvolvingRoom::optimal_vibe(RoomType::Desert)
        );
    }

    // -- SelectionPressure ---------------------------------------------------

    #[test]
    fn pressure_combined() {
        let p = SelectionPressure {
            temperature: 0.6,
            vibe_demand: 0.3,
            competition: 0.9,
        };
        assert!((p.combined() - 0.6).abs() < 1e-9);
    }

    #[test]
    fn pressure_default() {
        let p = SelectionPressure::default();
        assert!((p.combined() - 0.5).abs() < 1e-9);
    }

    // -- EvolutionEngine -----------------------------------------------------

    #[test]
    fn evolve_room_returns_mutations() {
        let mut engine = EvolutionEngine::new();
        let mut room = EvolvingRoom::new("r1", RoomType::Crystal, 0.1);
        let pressure = SelectionPressure {
            temperature: 0.5,
            vibe_demand: 0.5,
            competition: 0.8,
        };
        let ms = engine.evolve_room(&mut room, &pressure);
        assert!(!ms.is_empty());
        for m in &ms {
            assert_eq!(m.room_id, "r1");
        }
    }

    #[test]
    fn speciate_groups_by_type() {
        let engine = EvolutionEngine::new();
        let rooms = vec![
            EvolvingRoom::new("1", RoomType::Forest, 0.5),
            EvolvingRoom::new("2", RoomType::Desert, 0.5),
            EvolvingRoom::new("3", RoomType::Forest, 0.5),
            EvolvingRoom::new("4", RoomType::Desert, 0.5),
        ];
        let species = engine.speciate(&rooms);
        assert_eq!(species.len(), 2);
    }

    #[test]
    fn speciate_empty() {
        let engine = EvolutionEngine::new();
        let species = engine.speciate(&[]);
        assert!(species.is_empty());
    }

    #[test]
    fn mass_extinction_kills_weak() {
        let engine = EvolutionEngine::new();
        let mut rooms = vec![
            EvolvingRoom::new("strong", RoomType::Ocean, 0.7), // fitness 1.0
            EvolvingRoom::new("weak", RoomType::Volcano, 1.0), // fitness 0.2
        ];
        engine.mass_extinction(&mut rooms, 0.5);
        assert!(rooms[0].vibe > 0.0); // survives
        assert_eq!(rooms[1].vibe, 0.0); // killed
    }

    #[test]
    fn mass_extinction_spares_all() {
        let engine = EvolutionEngine::new();
        let mut rooms = vec![EvolvingRoom::new("r", RoomType::Ocean, 0.7)];
        engine.mass_extinction(&mut rooms, 0.0); // threshold 0, everyone survives
        assert!(rooms[0].vibe > 0.0);
    }

    // -- WorldState ----------------------------------------------------------

    #[test]
    fn world_health_range() {
        let ws = WorldState::new(vec![EvolvingRoom::new("r", RoomType::Forest, 0.6)]);
        let h = ws.health();
        assert!((0.0..=1.0).contains(&h));
    }

    #[test]
    fn world_health_perfect() {
        // Multiple types for biodiversity, perfect fitness for stability.
        let ws = WorldState::new(vec![
            EvolvingRoom::new("1", RoomType::Forest, 0.6),
            EvolvingRoom::new("2", RoomType::Desert, 0.3),
            EvolvingRoom::new("3", RoomType::Ocean, 0.7),
            EvolvingRoom::new("4", RoomType::Crystal, 0.8),
            EvolvingRoom::new("5", RoomType::Volcano, 0.2),
            EvolvingRoom::new("6", RoomType::Mountain, 0.5),
            EvolvingRoom::new("7", RoomType::Floating, 0.9),
            EvolvingRoom::new("8", RoomType::Nexus, 0.5),
        ]);
        assert!((ws.health() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn world_tick_advances_age() {
        let mut ws = WorldState::new(vec![EvolvingRoom::new("r", RoomType::Forest, 0.5)]);
        assert_eq!(ws.age_ticks, 0);
        ws.tick();
        assert_eq!(ws.age_ticks, 1);
    }

    #[test]
    fn world_age_category() {
        let mut ws = WorldState::new(vec![EvolvingRoom::new("r", RoomType::Forest, 0.5)]);
        assert_eq!(ws.age_category(), AgeCategory::Infant);
        ws.age_ticks = 5_000;
        assert_eq!(ws.age_category(), AgeCategory::Teen);
    }

    #[test]
    fn world_biodiversity_single_type() {
        let ws = WorldState::new(vec![
            EvolvingRoom::new("1", RoomType::Forest, 0.5),
            EvolvingRoom::new("2", RoomType::Forest, 0.5),
        ]);
        assert!((ws.biodiversity - 0.125).abs() < 1e-9); // 1/8
    }

    #[test]
    fn world_empty() {
        let ws = WorldState::new(vec![]);
        assert_eq!(ws.health(), 0.5); // stability=1.0, biodiversity=0.0 -> (1+0)/2 = 0.5
        assert_eq!(ws.total_vibe, 0.0);
    }

    // -- Serde round-trip ----------------------------------------------------

    #[test]
    fn serde_world_state() {
        let ws = WorldState::new(vec![
            EvolvingRoom::new("a", RoomType::Forest, 0.5),
            EvolvingRoom::new("b", RoomType::Ocean, 0.8),
        ]);
        let json = serde_json::to_string(&ws).unwrap();
        let ws2: WorldState = serde_json::from_str(&json).unwrap();
        assert_eq!(ws.age_ticks, ws2.age_ticks);
        assert_eq!(ws.rooms.len(), ws2.rooms.len());
    }

    #[test]
    fn serde_mutation() {
        let m = Mutation {
            room_id: "x".into(),
            attribute: "vibe".into(),
            old_value: 0.1,
            new_value: 0.2,
            beneficial: true,
        };
        let json = serde_json::to_string(&m).unwrap();
        let m2: Mutation = serde_json::from_str(&json).unwrap();
        assert_eq!(m.room_id, m2.room_id);
        assert_eq!(m.beneficial, m2.beneficial);
    }

    #[test]
    fn serde_age_category() {
        let cat = AgeCategory::Mature;
        let json = serde_json::to_string(&cat).unwrap();
        assert!(json.contains("Mature"));
        let cat2: AgeCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(cat, cat2);
    }

    #[test]
    fn serde_room_type() {
        let rt = RoomType::Floating;
        let json = serde_json::to_string(&rt).unwrap();
        assert!(json.contains("Floating"));
        let rt2: RoomType = serde_json::from_str(&json).unwrap();
        assert_eq!(rt, rt2);
    }

    #[test]
    fn multiple_ticks_stability_changes() {
        let mut ws = WorldState::new(vec![EvolvingRoom::new("r", RoomType::Forest, 0.1)]);
        let initial_stability = ws.stability;
        for _ in 0..50 {
            ws.tick();
        }
        // After 50 ticks the room should have evolved toward fittest_vibe (0.6).
        assert!(ws.stability > initial_stability || ws.rooms[0].mutations > 0);
    }
}
