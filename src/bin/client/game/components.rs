use bevy::prelude::{Component};

#[derive(Debug, Component)]
pub struct Name(pub String);

// Entities with this component are controlled by another player
#[derive(Debug, Component)]
pub struct PlayerEntity;
