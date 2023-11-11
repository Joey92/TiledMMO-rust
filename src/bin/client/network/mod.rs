use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

use bevy::{app::AppExit, core::Name, log, prelude::*};
use bevy_rapier2d::prelude::*;
use bevy_renet::{
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
        ConnectionConfig, DefaultChannel, RenetClient,
    },
    transport::{client_connected, NetcodeClientPlugin},
    RenetClientPlugin,
};
use tiled_game::network::{
    messages::{
        client::ClientMessages,
        server::{ServerMessages, Vitals},
    },
    PROTOCOL_ID,
};

use tiled_game::components::*;

use crate::{
    game::{
        components::PlayerEntity,
        map::MapChangeEvent,
        player::Player,
        spritesheet::{
            AnimateDirection, AnimateState, AnimationIndices, AnimationTimer, Facing,
            MovementState, PreviousPos,
        },
    },
    helpers::camera::CameraTarget,
};

use self::sync::*;

mod sync;

fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let client = RenetClient::new(ConnectionConfig::default());

    let server_addr = "127.0.0.1:3387".parse().expect("Invalid server address");
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");
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

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    (client, transport)
}

// If any error is found we just panic
fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
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

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let (client, transport) = new_renet_client();

        app.configure_set(Update, Connected.run_if(client_connected()));

        app.add_plugins(RenetClientPlugin)
            .add_plugins(NetcodeClientPlugin)
            .insert_resource(client)
            .insert_resource(transport)
            .init_resource::<ClientState>()
            .add_systems(
                Update,
                (
                    handle_server_messages,
                    sync_movement,
                    sync_target,
                    sync_target_deselect,
                )
                    .in_set(Connected),
            )
            .add_systems(PostUpdate, (disconnect_on_exit, panic_on_error_system));
    }
}

fn handle_server_messages(
    mut client: ResMut<RenetClient>,
    mut client_state: ResMut<ClientState>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut commands: Commands,
    mut send_map_change: EventWriter<MapChangeEvent>,
    mut player: Query<(&mut Transform, With<Player>)>,
) {
    // let client_id = client.client_id();
    while let Some(message) = client.receive_message(DefaultChannel::ReliableUnordered) {
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
                client.send_message(DefaultChannel::ReliableUnordered, msg);
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
                interactable,
                unit,
                rotation,
            } => {
                let client_entity = client_state
                    .server_client_entity_mapping
                    .get(&server_entity);

                if client_entity.is_none() {
                    // maybe spawn it anyway?
                    continue;
                }

                let mut cmd = commands.entity(*client_entity.unwrap());

                let texture: Handle<Image> = asset_server.load(format!("images/{}.png", unit));
                let texture_atlas =
                    TextureAtlas::from_grid(texture, Vec2::new(24.0, 24.0), 3, 4, None, None);
                let texture_atlas_handle = texture_atlases.add(texture_atlas);
                // Use only the subset of sprites in the sheet that make up the run animation
                let animation_indices = AnimationIndices { rows: 4, cols: 3 };

                cmd.insert((
                    Name::new(name.clone()),
                    SpriteSheetBundle {
                        transform: Transform::from_translation(pos).with_rotation(rotation),
                        texture_atlas: texture_atlas_handle,
                        sprite: TextureAtlasSprite::new(0),
                        ..default()
                    },
                    Health(health),
                    MaxHealth(max_health),
                    Mana(mana),
                    MaxMana(max_mana),
                    Threat(threat.unwrap_or(ThreatMap::default())),
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                    animation_indices,
                    AnimateDirection(Facing::Down),
                    AnimateState(MovementState::Idle),
                    PreviousPos(pos),
                ));

                let font = asset_server.load("OpenSans-Regular.ttf");
                let text_style = TextStyle {
                    font,
                    font_size: 16.0,
                    color: Color::BLACK,
                };

                cmd.with_children(|c| {
                    c.spawn(Text2dBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
                        text: Text::from_section(name.clone(), text_style.clone())
                            .with_alignment(TextAlignment::Center),
                        ..default()
                    });
                });

                if is_player {
                    cmd.insert(PlayerEntity);
                }

                if interactable {
                    cmd.insert(Interactable);
                }
            }
            ServerMessages::Move {
                entity: server_entity,
                pos,
                rotation,
            } => {
                let client_side_entity = client_state
                    .server_client_entity_mapping
                    .get(&server_entity);

                if let Some(client_side_entity) = client_side_entity {
                    log::debug!("Move entity: {:?}", server_entity);
                    commands
                        .entity(*client_side_entity)
                        // todo: entities bounce between z index 0 and their correct position
                        // causing them to flicker when the player is behind them
                        .insert(Transform::from_translation(pos).with_rotation(rotation));
                }
            }
            ServerMessages::PlayerInfo {
                entity: server_entity,
                pos,
                img,
            } => {
                client_state.player_entity = Some(server_entity);

                let texture: Handle<Image> = asset_server.load(format!("images/{}.png", img));
                let texture_atlas =
                    TextureAtlas::from_grid(texture, Vec2::new(24.0, 24.0), 3, 4, None, None);
                let texture_atlas_handle = texture_atlases.add(texture_atlas);
                // Use only the subset of sprites in the sheet that make up the run animation
                let animation_indices = AnimationIndices { rows: 4, cols: 3 };
                let name = Name::new("player");

                let mut cmd = commands.spawn((
                    Player,
                    ServerSideEntity(server_entity),
                    RigidBody::Dynamic,
                    GravityScale(0.0),
                    Damping {
                        linear_damping: 1.,
                        angular_damping: 1.,
                    },
                    Ccd::enabled(),
                    LockedAxes::ROTATION_LOCKED,
                    SpriteSheetBundle {
                        transform: Transform::from_translation(pos),
                        texture_atlas: texture_atlas_handle,
                        sprite: TextureAtlasSprite::new(0),
                        ..default()
                    },
                    Collider::cuboid(8., 8.),
                    name.clone(),
                    CameraTarget,
                    AnimationTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
                    animation_indices,
                    AnimateDirection(Facing::Down),
                    AnimateState(MovementState::Idle),
                ));

                cmd.insert(PreviousPos(pos));

                let font = asset_server.load("OpenSans-Regular.ttf");
                let text_style = TextStyle {
                    font,
                    font_size: 16.0,
                    color: Color::BLACK,
                };

                cmd.with_children(|c| {
                    c.spawn(Text2dBundle {
                        transform: Transform::from_translation(Vec3::new(0.0, 20.0, 0.0)),
                        text: Text::from_section(name.clone(), text_style.clone())
                            .with_alignment(TextAlignment::Center),
                        ..default()
                    });
                });

                client_state
                    .server_client_entity_mapping
                    .insert(server_entity, cmd.id());
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
                    client.send_message(DefaultChannel::ReliableUnordered, msg);
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
            ServerMessages::PlayerError { error } => match error {
                tiled_game::network::messages::server::PlayerErrorMessage::TooFarAway => {
                    println!("Too far away");
                }
                tiled_game::network::messages::server::PlayerErrorMessage::ManaTooLow => todo!(),
                tiled_game::network::messages::server::PlayerErrorMessage::Unusable => todo!(),
            },
            ServerMessages::Saying { entity, msg } => {
                println!("{:?} is saying {}", entity, msg);
            }
        }
    }
}

fn disconnect_on_exit(exit: EventReader<AppExit>, mut client: ResMut<RenetClient>) {
    if !exit.is_empty() {
        let msg = ClientMessages::Disconnect;
        let msg = bincode::serialize(&msg).unwrap();
        client.send_message(DefaultChannel::ReliableUnordered, msg);
        client.disconnect();
    }
}
