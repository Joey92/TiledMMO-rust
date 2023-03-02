use bevy::prelude::{Entity, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessages {
    Map {
        name: String,
        position: Vec3,
    },
    Despawn {
        entity: Entity,
    },
    // Send the client a list of all entities
    // on the map instance
    // client needs to request entity info for any missing components
    EntityList {
        entities: Vec<Entity>,
    },
    // Information about any spawnable game object
    // This should be re-sent to update missing info on the client as well
    // consider adding a EntityInfo ClientMessage to request this info
    EntityInfo {
        entity: Entity,
        x: f32,
        y: f32,
        name: String,
        is_player: bool,
    },
    Move {
        entity: Entity,
        x: f32,
        y: f32,
    },

    // send the client the entity ID of the server side player entity
    EntityAssignment {
        entity: Entity,
    },
}
