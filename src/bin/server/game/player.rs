use bevy::prelude::*;
use tiled_game::shared_components::Player;

use crate::network::NetworkClientId;

use super::map::{MapName, Teleport};

pub fn add_new_player(
    mut commands: Commands,
    new_connected_players: Query<Entity, Added<NetworkClientId>>,
) {
    for player_entity in new_connected_players.iter() {
        println!("Adding new player: {:?}", player_entity);
        // fetch player info from database
        commands
            .entity(player_entity)
            .insert(Player)
            // add name
            .insert(Name::new("John Doe")) // todo: db
            .insert(SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            })
            // add teleport component so the map change system can handle it
            .insert(Teleport {
                map: MapName("start.tmx".to_string()),       // todo: db
                position: Transform::from_xyz(30., 30., 0.), // todo: db
                map_instance: None,                          // todo: db
                prev_map_instance: None,
            });
    }
}
