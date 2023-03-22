use bevy::{
    prelude::{Component, Entity},
    reflect::Reflect,
    utils::HashMap,
};

#[derive(Component, Reflect, Default)]
pub struct Health(pub i32);

#[derive(Component, Reflect, Default)]
pub struct Mana(pub i32);

#[derive(Component, Reflect, Default)]
pub struct MaxMana(pub i32);

#[derive(Component, Reflect, Default)]
pub struct MaxHealth(pub i32);

#[derive(Component)]
pub struct Target(pub Entity);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct InCombat;

// This is a marker component that is added to entities that can right-clicked on.
#[derive(Component)]
pub struct Interactable;

// Use bevy's HashMap type because Reflect is implemented for it
pub type ThreatMap = HashMap<Entity, i32>;

// Threat is a map of entities to their threat value
// The entity with the highest threat value should be treated as the target
// The threat value is increased by damage done to the creature
// An empty Threat components also means that the creature is not in combat
#[derive(Component, Default, Reflect)]
pub struct Threat(pub ThreatMap);

impl Threat {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add(&mut self, entity: Entity, threat: i32) {
        let current_threat = self.0.entry(entity).or_insert(0);
        *current_threat += threat;
    }

    pub fn get(&self, entity: Entity) -> Option<&i32> {
        self.0.get(&entity)
    }
}
