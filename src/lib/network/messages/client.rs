use bevy::prelude::Entity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessages {
    // Event the client should send when the assets of the maps are loaded
    Ready,
    Disconnect,
    Target { target: Option<Entity> },
    Move { x: f32, y: f32 },
    RequestEntityInfo { entity: Entity },
    Interact { entity: Entity },
}
