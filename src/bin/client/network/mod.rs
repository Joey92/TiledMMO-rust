use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use bevy::{
    app::AppExit,
    core::Name,
    log,
    prelude::*,
    sprite::{Sprite, SpriteBundle},
};
use bevy_rapier2d::prelude::*;
use bevy_renet::{
    renet::{ClientAuthentication, RenetClient, RenetError},
    run_if_client_connected, RenetClientPlugin,
};
use tiled_game::network::{
    client_connection_config,
    messages::{
        client::ClientMessages,
        server::{ServerMessages, Vitals},
    },
    ClientChannel, ServerChannel, PROTOCOL_ID,
};

use tiled_game::components::*;

use crate::{
    game::{
        components::PlayerEntity,
        map::tiled::{MapChangeEvent, MapName},
        player::Player,
    },
    helpers::camera::CameraTarget,
};

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

// Every entity with this component
// can be controlled by the server
#[derive(Debug, Component)]
pub struct ServerSideEntity(pub Entity);

#[derive(Resource, Default)]
struct ClientState {
    player_entity: Option<Entity>,

    server_client_entity_mapping:
        HashMap<Entity /* Server side */, Entity /* client side */>,
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
                    .with_system(handle_server_messages)
                    .with_system(sync_movement)
                    .label("network"),
            )
            .add_system_to_stage(CoreStage::Last, disconnect_on_exit)
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

fn handle_server_messages(
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
        log::debug!("Received message from server: {:?}", server_message);
        match server_message {
            ServerMessages::Despawn { entity } => {
                let client_side_entity = client_state.server_client_entity_mapping.remove(&entity);
                if let Some(client_side_entity) = client_side_entity {
                    log::info!("Despawn entity: {:?}", entity);
                    commands.entity(client_side_entity).despawn_recursive();
                }
            }

            ServerMessages::Spawn {
                entity: server_entity,
            } => {
                log::debug!("Spawn entity: {:?}", server_entity);
                let client_side_entity = commands.spawn(ServerSideEntity(server_entity)).id();
                client_state
                    .server_client_entity_mapping
                    .insert(server_entity, client_side_entity);

                // request info about the entity
                let msg = ClientMessages::RequestEntityInfo {
                    entity: server_entity,
                };
                let msg = bincode::serialize(&msg).unwrap();
                client.send_message(ClientChannel::Input, msg);
            }
            ServerMessages::EntityInfo {
                entity: server_entity,
                pos,
                name,
                is_player,
                friendly,
                health,
                max_health,
                mana,
                max_mana,
                threat,
            } => {
                let client_entity = client_state
                    .server_client_entity_mapping
                    .get(&server_entity);

                if client_entity.is_none() {
                    // maybe spawn it anyway?
                    continue;
                }

                let mut cmd = commands.entity(*client_entity.unwrap());

                let color = if friendly {
                    Color::rgb(0.25, 0.75, 0.25)
                } else {
                    Color::rgb(0.75, 0.25, 0.25)
                };

                cmd.insert((
                    MapName(name.clone()),
                    Name::new(name),
                    SpriteBundle {
                        transform: Transform::from_translation(pos),
                        sprite: Sprite {
                            color,
                            custom_size: Some(Vec2::new(50.0, 100.0)),
                            ..default()
                        },
                        ..default()
                    },
                    Health(health),
                    MaxHealth(max_health),
                    Mana(mana),
                    MaxMana(max_mana),
                    Threat(threat.unwrap_or(ThreatMap::default())),
                ));

                if is_player {
                    cmd.insert(PlayerEntity);
                }
            }
            ServerMessages::Move {
                entity: server_entity,
                pos,
            } => {
                let client_side_entity = client_state
                    .server_client_entity_mapping
                    .get(&server_entity);

                if let Some(client_side_entity) = client_side_entity {
                    log::debug!("Move entity: {:?}", server_entity);
                    commands
                        .entity(*client_side_entity)
                        .insert(Transform::from_translation(pos));
                }
            }
            ServerMessages::PlayerInfo {
                entity: server_entity,
                pos,
            } => {
                client_state.player_entity = Some(server_entity);
                let client_entity = commands
                    .spawn((
                        Player,
                        ServerSideEntity(server_entity),
                        RigidBody::Dynamic,
                        GravityScale(0.0),
                        Restitution {
                            coefficient: 0.,
                            combine_rule: CoefficientCombineRule::Min,
                        },
                        LockedAxes::ROTATION_LOCKED,
                        SpriteBundle {
                            transform: Transform::from_translation(pos),
                            sprite: Sprite {
                                color: Color::rgb(0., 0.25, 0.75),
                                custom_size: Some(Vec2::new(40.0, 40.0)),
                                ..default()
                            },
                            ..default()
                        },
                        Collider::cuboid(20., 20.),
                        Name::new("player"),
                        CameraTarget,
                    ))
                    .id();

                client_state
                    .server_client_entity_mapping
                    .insert(server_entity, client_entity);
            }
            ServerMessages::EntityList {
                entities: server_entities,
            } => {
                for server_entity in server_entities {
                    // use get_or_spawn to mirror entity IDs on the server
                    let client_entity = commands.spawn(ServerSideEntity(server_entity)).id();
                    client_state
                        .server_client_entity_mapping
                        .insert(server_entity, client_entity);

                    // request info about the entity
                    let msg = ClientMessages::RequestEntityInfo {
                        entity: server_entity,
                    };
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
            ServerMessages::Vitals {
                entity: server_entity,
                vital,
            } => {
                let client_entity = client_state
                    .server_client_entity_mapping
                    .get(&server_entity);

                if let Some(client_entity) = client_entity {
                    match vital {
                        Vitals::Health(health) => {
                            commands.entity(*client_entity).insert(Health(health));
                        }
                        Vitals::Mana(mana) => {
                            commands.entity(*client_entity).insert(Mana(mana));
                        }
                        Vitals::Dead(is_dead) => {
                            if is_dead {
                                commands.entity(*client_entity).insert(Dead);
                                continue;
                            }
                            commands.entity(*client_entity).remove::<Dead>();
                        }
                    }
                }
            }
            ServerMessages::Threat {
                entity: server_entity,
                threat,
            } => {
                let client_entity = client_state
                    .server_client_entity_mapping
                    .get(&server_entity);

                if let Some(client_entity) = client_entity {
                    commands.entity(*client_entity).insert(Threat(threat));
                }
            }
            ServerMessages::CombatState {
                entity: server_entity,
                in_combat,
            } => {
                let client_entity = client_state
                    .server_client_entity_mapping
                    .get(&server_entity);

                if let Some(client_entity) = client_entity {
                    if in_combat {
                        commands.entity(*client_entity).insert(InCombat);
                        continue;
                    }

                    commands.entity(*client_entity).remove::<InCombat>();
                }
            }
            ServerMessages::Disconnect { reason } => {
                panic!("Disconnected from server: {:?}", reason)
            }
        }
    }
}

fn disconnect_on_exit(exit: EventReader<AppExit>, mut client: ResMut<RenetClient>) {
    if !exit.is_empty() {
        let msg = ClientMessages::Disconnect;
        let msg = bincode::serialize(&msg).unwrap();
        client.send_message(ClientChannel::Input, msg);
        client.disconnect();
    }
}
