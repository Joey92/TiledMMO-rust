use bevy::{prelude::Component, utils::Instant};
use tiled_game::shared_components::Name;

#[derive(Component)]
pub struct Player;

pub struct NPC_bundle {
    pub name: Name,
    pub spawn_time: Instant,
}
