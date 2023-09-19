use bevy::prelude::*;

use crate::{game::unit::UnitBundle, network::NetworkClientId};

use super::map::{DespawnEvent, Teleport};

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
        app.add_systems(Update, (player_logout, player_join));
    }
}

pub fn player_join(
    mut commands: Commands,
    mut teleport_event: EventWriter<Teleport>,
    new_connected_players: Query<Entity, Added<NetworkClientId>>,
) {
    for entity in new_connected_players.iter() {
        println!("Adding new player: {:?}", entity);
        // fetch player info from database
        commands.entity(entity).insert((
            UnitBundle::new(
                "John Doe".into(),
                String::from("Fantasy Dreamland/Characters/Character_001"),
                Transform::from_xyz(30., 30., 0.),
            ),
            Player,
        ));

        teleport_event.send(Teleport {
            entity,
            map: "start.tmx".to_string(),                // todo: db
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
