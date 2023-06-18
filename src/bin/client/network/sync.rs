use bevy::prelude::*;
use bevy_renet::renet::{DefaultChannel, RenetClient};
use tiled_game::{components::Target, network::messages::client::ClientMessages};

use crate::game::player::Player;

use super::ServerSideEntity;

// Sync player movement to the server
pub fn sync_movement(
    movement: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut client: ResMut<RenetClient>,
) {
    if movement.is_empty() {
        return;
    }

    let transform = movement.single();

    let msg = ClientMessages::Move {
        x: transform.translation.x,
        y: transform.translation.y,
    };

    let msg = bincode::serialize(&msg).unwrap();

    client.send_message(DefaultChannel::ReliableUnordered, msg);
}

pub fn sync_target(
    target_q: Query<&Target, (Added<Target>, Changed<Target>)>,
    target_units: Query<&ServerSideEntity>,
    mut client: ResMut<RenetClient>,
) {
    if target_q.is_empty() {
        return;
    }

    let target = target_q.single();

    let target_server_side = target_units.get(target.0).unwrap();

    let msg = ClientMessages::Target {
        target: Some(target_server_side.0),
    };

    let msg = bincode::serialize(&msg).unwrap();

    client.send_message(DefaultChannel::ReliableUnordered, msg);
}

pub fn sync_target_deselect(
    mut removed_target: RemovedComponents<Target>,
    player: Query<Entity, With<Player>>,
    mut client: ResMut<RenetClient>,
) {
    if removed_target.is_empty() {
        return;
    }

    removed_target.iter().for_each(|entity| {
        if player.single() != entity {
            return;
        }

        let msg = ClientMessages::Target { target: None };

        let msg = bincode::serialize(&msg).unwrap();

        client.send_message(DefaultChannel::ReliableUnordered, msg);
    });
}
