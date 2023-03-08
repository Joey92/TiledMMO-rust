use bevy::prelude::*;

use crate::{game::unit::UnitBundle, network::NetworkClientId};

use super::map::{DespawnEvent, MapName, Teleport};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct LoggingOut;

// Set this on a player
// when the server should dictate the position of the player
#[derive(Component)]
pub struct Charmed;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_join).add_system(player_logout);
    }
}

pub fn player_join(
    mut commands: Commands,
    new_connected_players: Query<Entity, Added<NetworkClientId>>,
) {
    for player_entity in new_connected_players.iter() {
        println!("Adding new player: {:?}", player_entity);
        // fetch player info from database
        commands
            .entity(player_entity)
            .insert((
                UnitBundle::new("John Doe".into(), Transform::from_xyz(30., 30., 0.)),
                Player,
            ))
            // add teleport component so the map change system can handle it
            .insert(Teleport {
                map: MapName("start.tmx".to_string()),       // todo: db
                position: Transform::from_xyz(30., 30., 0.), // todo: db
                map_instance: None,                          // todo: db
                prev_map_instance: None,
            });
    }
}

pub fn player_logout(
    mut commands: Commands,
    players_logging_out: Query<(Entity, &Parent), With<LoggingOut>>,
    mut despawn_events: EventWriter<DespawnEvent>,
) {
    for (player_entity, map_instance_entity) in players_logging_out.iter() {
        // remove child from map instance
        commands
            .entity(map_instance_entity.get())
            .remove_children(&[player_entity]);

        // remove player from map
        commands.entity(player_entity).despawn_recursive();

        despawn_events.send(DespawnEvent {
            entity: player_entity,
            map: map_instance_entity.get(),
        });
    }
}
