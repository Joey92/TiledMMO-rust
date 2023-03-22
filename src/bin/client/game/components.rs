use bevy::prelude::*;

// Entities with this component are controlled by another player
#[derive(Debug, Component)]
pub struct PlayerEntity;

#[derive(Resource, Default)]
pub struct MousePointerTarget(pub Vec2);

#[derive(Debug, Component)]
pub struct Highlighted;
