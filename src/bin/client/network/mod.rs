use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    app::AppExit,
    log,
    prelude::*,
    sprite::{Sprite, SpriteBundle},
    transform::TransformBundle,
};
use bevy_rapier2d::prelude::*;
use bevy_renet::{
    renet::{ClientAuthentication, RenetClient, RenetError},
    run_if_client_connected, RenetClientPlugin,
};
use tiled_game::network::{
    client_connection_config,
    messages::{client::ClientMessages, server::ServerMessages},
    ClientChannel, ServerChannel, PROTOCOL_ID,
};

use crate::{
    game::{
        components::PlayerEntity,
        map::tiled::{MapChangeEvent, MapName},
        player::Player,
    },
    helpers::camera::CameraTarget,
};

use crate::game::components::Name;

fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:3387".parse().expect("Invalid server address");
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
    let connection_config = client_connection_config();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

#[derive(Resource, Default)]
struct ClientState {
    player_entity: Option<Entity>,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetClientPlugin::default())
            .insert_resource(new_renet_client())
            .init_resource::<ClientState>()
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(run_if_client_connected)
                    .with_system(handle_client_messages)
                    .with_system(sync_movement),
            )
            .add_system(disconnect_on_exit)
            .add_system(panic_on_error_system);
    }
}

// Sync player movement to the server
fn sync_movement(
    movement: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut client: ResMut<RenetClient>,
) {
    for transform in movement.iter() {
        let msg = ClientMessages::Move {
            x: transform.translation.x,
            y: transform.translation.y,
        };

        let msg = bincode::serialize(&msg).unwrap();

        client.send_message(ClientChannel::Input, msg);
    }
}

fn handle_client_messages(
    mut client: ResMut<RenetClient>,
    mut client_state: ResMut<ClientState>,
    mut commands: Commands,
    mut send_map_change: EventWriter<MapChangeEvent>,
    mut player: Query<(&mut Transform, With<Player>)>,
) {
    // let client_id = client.client_id();
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message: Option<ServerMessages> = bincode::deserialize(&message).ok();
        if server_message.is_none() {
            log::error!("Failed to deserialize server message");
            continue;
        }

        let server_message = server_message.unwrap();
        log::info!("Received message from server: {:?}", server_message);
        match server_message {
            ServerMessages::Despawn { entity } => {
                log::info!("Despawn entity: {:?}", entity);
                commands.get_entity(entity).map(|mut e| e.despawn());
            }
            ServerMessages::EntityInfo {
                entity,
                x,
                y,
                name,
                is_player,
            } => {
                log::info!("Spawning entity: {:?}", entity);

                // use get_or_spawn to mirror entity IDs on the server
                let mut cmd = commands.get_or_spawn(entity);

                cmd.insert(MapName(name.clone()))
                    .insert(TransformBundle::from(Transform::from_xyz(x, y, 6.0)))
                    .insert(Name(name))
                    .insert(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.25, 0.25, 0.75),
                            custom_size: Some(Vec2::new(50.0, 100.0)),
                            ..default()
                        },
                        ..default()
                    });

                if is_player {
                    cmd.insert(PlayerEntity);
                }
            }
            ServerMessages::Move { entity, x, y } => {
                log::info!("Move entity: {:?}", entity);
                let cmd = commands.get_entity(entity);

                if let Some(mut entity) = cmd {
                    entity.insert(Transform::from_xyz(x, y, 0.0));
                }
            }
            ServerMessages::EntityAssignment { entity } => {
                client_state.player_entity = Some(entity);
                commands
                    .spawn(RigidBody::KinematicPositionBased)
                    .insert(GravityScale(0.0))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(TransformBundle::from(Transform::from_xyz(0., 0., 1.0)))
                    .insert(Collider::cuboid(20., 20.))
                    .insert(Player)
                    .insert(CameraTarget);
            }
            ServerMessages::EntityList { entities } => {
                for entity in entities {
                    // use get_or_spawn to mirror entity IDs on the server
                    commands.get_or_spawn(entity);

                    // request info about the entity
                    let msg = ClientMessages::RequestEntityInfo { entity };
                    let msg = bincode::serialize(&msg).unwrap();
                    client.send_message(ClientChannel::Input, msg);
                }
            }
            ServerMessages::Map { name, position } => {
                send_map_change.send(MapChangeEvent { map_name: name });

                if !player.is_empty() {
                    // change player location so where the server wants us to be
                    player.single_mut().0.translation = position;
                }
            }
        }
    }
}

fn disconnect_on_exit(exit: EventReader<AppExit>, mut server: ResMut<RenetClient>) {
    if !exit.is_empty() {
        server.disconnect();
    }
}
