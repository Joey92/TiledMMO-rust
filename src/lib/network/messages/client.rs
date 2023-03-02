use bevy::prelude::Entity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessages {
    LoadReady,
    Move { x: f32, y: f32 },
    RequestEntityInfo { entity: Entity },
}
