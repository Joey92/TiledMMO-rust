use bevy::prelude::{Entity, Quat, Vec3};

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
pub enum PlayerErrorMessage {
    TooFarAway,
    ManaTooLow,
    Unusable,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessages {
    Disconnect {
        reason: DisconnectionReason,
    },

    // Makes the client load a particular map
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
    EntityInfo {
        entity: Entity,
        pos: Vec3,
        name: String,
        is_player: bool,
        friendly: bool,
        health: i32,
        max_health: i32,
        mana: i32,
        max_mana: i32,
        unit: String,
        threat: Option<ThreatMap>,
        interactable: bool,
        rotation: Quat,
    },

    // entity has moved
    Move {
        entity: Entity,
        pos: Vec3,
        rotation: Quat,
    },

    // Update the client with the current vital values
    // such as health, mana, etc
    Vitals {
        entity: Entity,
        vital: Vitals,
    },

    // send the client the entity ID of the server side player entity
    PlayerInfo {
        entity: Entity,
        pos: Vec3,
        img: String,
    },

    Threat {
        entity: Entity,
        threat: ThreatMap,
    },

    CombatState {
        entity: Entity,
        in_combat: bool,
    },

    PlayerError {
        error: PlayerErrorMessage,
    },

    // Entity is saying something
    // display a chat bubble or something?
    Saying {
        entity: Entity,
        msg: String,
    },
}
