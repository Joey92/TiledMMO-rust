mod sync_systems;

use std::{net::UdpSocket, time::SystemTime};

use bevy::{app::AppExit, prelude::*};
use bevy_renet::{
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ConnectionConfig, DefaultChannel, RenetServer, ServerEvent,
    },
    transport::NetcodeServerPlugin,
    RenetServerPlugin,
};
use tiled_game::{
    components::Target,
    network::{
        messages::{client::ClientMessages, server::ServerMessages},
        PROTOCOL_ID,
    },
};

use bevy_spatial::SpatialAccess;

use sync_systems::*;

use crate::game::{player::LoggingOut, ClientsNearby};

#[derive(Component, Debug)]
pub struct NetworkClientId(pub u64);

#[derive(Default, Resource)]
pub struct NetworkResource {
    player_entity_map: std::collections::HashMap<u64, Entity>,
}

pub enum BroadcastType {
    Direct {
        client_id: u64,
    },
    Nearby {
        map: Entity,
        origin: Vec2,
        range: f32,
    },
    Map {
        map: Entity,
    },
    Global,
}

#[derive(Event)]
pub struct SendServerMessageEvent {
    pub broadcast: BroadcastType,
    pub message: ServerMessages,
}

impl SendServerMessageEvent {
    pub fn directly_to(client_id: u64, message: ServerMessages) -> Self {
        Self {
            broadcast: BroadcastType::Direct { client_id },
            message,
        }
    }
    pub fn to_map(map: Entity, message: ServerMessages) -> Self {
        Self {
            broadcast: BroadcastType::Map { map },
            message,
        }
    }

    pub fn to_players_nearby(
        map: Entity,
        origin: Vec2,
        range: f32,
        message: ServerMessages,
    ) -> Self {
        Self {
            broadcast: BroadcastType::Nearby { map, origin, range },
            message,
        }
    }

    pub fn to_everyone(message: ServerMessages) -> Self {
        Self {
            broadcast: BroadcastType::Global,
            message,
        }
    }
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        let (server, transport) = new_renet_server();
        app
            // Initialize Network
            .add_event::<SendServerMessageEvent>()
            .add_event::<SendEntityInfoEvent>()
            .insert_resource(server)
            .insert_resource(transport)
            .insert_resource(NetworkResource::default())
            .add_plugins(RenetServerPlugin)
            .add_plugins(NetcodeServerPlugin)
            // Receive Server Events
            .add_systems(
                PreUpdate,
                (
                    handle_connection_events,
                    handle_client_messages.after(handle_connection_events),
                ),
            )
            .add_systems(Update, send_message_system)
            .add_systems(
                PostUpdate,
                (
                    send_movement,
                    send_despawn,
                    send_vitals_changed,
                    send_death_events,
                    send_threat,
                    send_entered_combat,
                    send_spawn,
                    send_exit_combat,
                    send_entity_info,
                    send_chat,
                ),
            )
            .add_systems(PostUpdate, disconnect_clients_on_exit);
    }
}

fn send_message_system(
    mut server: ResMut<RenetServer>,

    clients: Query<(&NetworkClientId, &Parent)>,
    clients_nearby: Res<ClientsNearby>,
    mut send_server_message_event: EventReader<SendServerMessageEvent>,
) {
    for event in send_server_message_event.iter() {
        let message = bincode::serialize(&event.message).expect("Unserializable packet");

        match event.broadcast {
            BroadcastType::Direct { client_id } => {
                server.send_message(client_id, DefaultChannel::ReliableUnordered, message)
            }
            BroadcastType::Nearby { map, origin, range } => {
                // check which clients are nearby on the same map
                clients_nearby
                    .within_distance(origin, range)
                    .iter()
                    .flat_map(|client| client.1)
                    .flat_map(|client| clients.get(client))
                    .filter(|(_, client_map)| map == client_map.get())
                    .map(|(network_id, _)| network_id.0)
                    .for_each(|client_id| {
                        server.send_message(
                            client_id,
                            DefaultChannel::ReliableUnordered,
                            message.clone(),
                        );
                    });
            }
            BroadcastType::Map { map } => {
                clients
                    .iter()
                    .filter(|(_, client_map)| map == client_map.get())
                    .map(|(network_id, _)| network_id.0)
                    .for_each(|client_id| {
                        server.send_message(
                            client_id,
                            DefaultChannel::ReliableUnordered,
                            message.clone(),
                        );
                    });
            }
            BroadcastType::Global => {
                server.broadcast_message(DefaultChannel::ReliableUnordered, message)
            }
        }
    }
}

pub fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let server = RenetServer::new(ConnectionConfig::default());
    let public_addr = "127.0.0.1:3387".parse().expect("Invalid server address");
    let socket = UdpSocket::bind(public_addr).expect("Failed to bind socket");
    let server_config = ServerConfig {
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addr,
        authentication: ServerAuthentication::Unsecure,
    };
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let transport = NetcodeServerTransport::new(current_time, server_config, socket).unwrap();

    (server, transport)
}

fn handle_connection_events(
    mut commands: Commands,
    mut client_entities: ResMut<NetworkResource>,
    mut connection_events: EventReader<ServerEvent>,
    mut server_message_events: EventWriter<SendServerMessageEvent>,
) {
    for event in connection_events.iter() {
        match event {
            ServerEvent::ClientConnected {
                client_id, /* auth message */
            } => {
                println!("Client connected: {}", client_id);
                // create an empty entity with the client id
                // other systems should add the rest of the components
                let entity = commands
                    .spawn_empty()
                    .insert(NetworkClientId(*client_id))
                    .id();
                client_entities.player_entity_map.insert(*client_id, entity);

                server_message_events.send(SendServerMessageEvent::directly_to(
                    *client_id,
                    ServerMessages::PlayerInfo {
                        entity,
                        pos: Vec3::new(0., 0., 0.),
                        img: "dreamland/Characters/Character_001".to_string(),
                    },
                ));
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client disconnected: {}", client_id);
                let client = client_entities.player_entity_map.remove(client_id);

                if let Some(entity) = client {
                    commands.entity(entity).insert(LoggingOut);
                }
            }
        }
    }
}

fn handle_client_messages(
    mut server: ResMut<RenetServer>,
    client_entities: ResMut<NetworkResource>,
    mut entity_info_request: EventWriter<SendEntityInfoEvent>,
    mut commands: Commands,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) =
            server.receive_message(client_id, DefaultChannel::ReliableUnordered)
        {
            let message: ClientMessages = bincode::deserialize(&message).unwrap();
            // println!("Received message from client {}: {:?}", client_id, message);
            if let Some(entity) = client_entities.player_entity_map.get(&client_id) {
                match message {
                    ClientMessages::Move { x, y } => {
                        commands
                            .entity(*entity)
                            .insert(Transform::from_xyz(x, y, 0.));
                    }
                    ClientMessages::Ready => todo!("Handle ready message"),
                    ClientMessages::RequestEntityInfo { entity } => {
                        entity_info_request.send(SendEntityInfoEvent { client_id, entity });
                    }
                    ClientMessages::Disconnect => {
                        // todo: handle remove children on map instance
                        commands.entity(*entity).insert(LoggingOut);
                    }

                    // user selected or unselected a target
                    ClientMessages::Target { target } => {
                        println!("Target: {:?}", target);
                        if let Some(target_entity) = target {
                            commands.entity(*entity).insert(Target(target_entity));
                            continue;
                        }

                        commands.entity(*entity).remove::<Target>();
                    }
                    ClientMessages::Interact { entity } => {
                        println!("Interact with {:?}", entity);
                    }
                }
            }
        }
    }
}

fn disconnect_clients_on_exit(exit: EventReader<AppExit>, mut server: ResMut<RenetServer>) {
    if !exit.is_empty() {
        let msg = ServerMessages::Disconnect {
            reason: tiled_game::network::messages::server::DisconnectionReason::ServerShutdown,
        };
        let disconnect_msg = bincode::serialize(&msg).unwrap();
        server.broadcast_message(DefaultChannel::ReliableUnordered, disconnect_msg);
        server.disconnect_all();
    }
}
