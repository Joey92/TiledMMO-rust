use bevy::prelude::{Entity, Vec3};

use serde::{Deserialize, Serialize};

use crate::components::ThreatMap;

#[derive(Serialize, Deserialize, Debug)]
pub enum Vitals {
    Health(i32),
    Mana(i32),
    Dead(bool),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DisconnectionReason {
    ServerShutdown,
    ServerRestart,
    Kicked,
    Banned,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessages {
    Disconnect {
        reason: DisconnectionReason,
    },

    Map {
        name: String,
        position: Vec3,
    },
    // The server has despawned an entity
    // Mostly players leaving the map
    Despawn {
        entity: Entity,
    },

    // The server has spawned an entity
    // Mostly players joining the map
    Spawn {
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
        friendly: bool,
        health: i32,
        max_health: i32,
        mana: i32,
        max_mana: i32,
        threat: Option<ThreatMap>,
    },
    Move {
        entity: Entity,
        pos: Vec3,
    },

    Vitals {
        entity: Entity,
        vital: Vitals,
    },

    // send the client the entity ID of the server side player entity
    PlayerInfo {
        entity: Entity,
        translation: Vec3,
    },

    Threat {
        entity: Entity,
        threat: ThreatMap,
    },

    CombatState {
        entity: Entity,
        in_combat: bool,
    },
}
