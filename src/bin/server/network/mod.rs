use std::{net::UdpSocket, time::SystemTime};

use bevy::{
    app::AppExit,
    prelude::{
        App, Commands, Component, CoreStage, DespawnRecursiveExt, Entity, EventReader, EventWriter,
        Events, Plugin, Query, ResMut, Resource, Transform, World,
    },
};
use bevy_renet::{
    renet::{RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use tiled_game::{
    network::{
        messages::{client::ClientMessages, server::ServerMessages},
        server_connection_config, ClientChannel, ServerChannel, PROTOCOL_ID,
    },
    shared_components::{Name, Player},
};

#[derive(Component, Debug)]
pub struct NetworkClientId(pub u64);

#[derive(Default, Resource)]
pub struct NetworkResource {
    player_entity_map: std::collections::HashMap<u64, Entity>,
}

pub struct SendServerMessageEvent {
    pub client_id: Option<u64>,
    pub message: ServerMessages,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app
            // Initialize Network
            .add_plugin(RenetServerPlugin::default())
            .insert_resource(new_renet_server())
            .insert_resource(NetworkResource::default())
            // Receive Server Events
            .add_system_to_stage(CoreStage::PreUpdate, handle_server_events)
            .add_system(handle_client_messages)
            .add_event::<SendServerMessageEvent>()
            .add_system_to_stage(CoreStage::PostUpdate, send_message_system)
            .add_event::<SendEntityInfoEvent>()
            .add_system(handle_entity_info_system)
            .add_system(disconnect_clients_on_exit);
    }
}

fn send_message_system(
    mut server: ResMut<RenetServer>,
    mut send_server_message_event: EventReader<SendServerMessageEvent>,
) {
    for event in send_server_message_event.iter() {
        let message = bincode::serialize(&event.message).unwrap();

        match event.client_id {
            Some(client) => server.send_message(client, ServerChannel::ServerMessages, message),
            None => server.broadcast_message(ServerChannel::ServerMessages, message),
        }
    }
}

pub fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:3387".parse().expect("Invalid server address");
    let socket = UdpSocket::bind(server_addr).expect("Failed to bind socket");
    let connection_config = server_connection_config();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket)
        .expect("Failed to create server")
}

fn handle_server_events(
    mut commands: Commands,
    mut client_entities: ResMut<NetworkResource>,
    mut connection_events: EventReader<ServerEvent>,
    mut server_message_events: EventWriter<SendServerMessageEvent>,
) {
    for event in connection_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _ /* auth message */) => {
                println!("Client connected: {}", id);
                // create an empty entity with the client id
                // other systems should add the rest of the components
                let entity = commands.spawn_empty().insert(NetworkClientId(*id)).id();
                client_entities.player_entity_map.insert(*id, entity);

                server_message_events.send(SendServerMessageEvent {
                    client_id: Some(*id),
                    message: ServerMessages::EntityAssignment { entity },
                });
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Client disconnected: {}", id);
                let client = client_entities.player_entity_map.remove(id);

                client.map(|entity| {
                    commands.entity(entity).despawn_recursive();
                });
            }
        }
    }
}

struct SendEntityInfoEvent {
    client_id: u64,
    entity: Entity,
}

fn handle_entity_info_system(
    mut events: EventReader<SendEntityInfoEvent>,
    mut server_message: EventWriter<SendServerMessageEvent>,
    entities: Query<(&Name, &Transform, Option<&Player>)>,
) {
    for event in events.iter() {
        let entity = event.entity;
        let entity_ref = entities.get(entity).ok();

        let event = match entity_ref {
            Some((name, transform, player)) => SendServerMessageEvent {
                client_id: Some(event.client_id),
                message: ServerMessages::EntityInfo {
                    entity,
                    x: transform.translation.x,
                    y: transform.translation.y,
                    name: name.0.clone(),
                    is_player: player.is_some(),
                },
            },
            _ => {
                // entity doesn't exist.. send a despawn message
                SendServerMessageEvent {
                    client_id: Some(event.client_id),
                    message: ServerMessages::Despawn { entity },
                }
            }
        };

        server_message.send(event);
    }
}

fn handle_client_messages(
    mut server: ResMut<RenetServer>,
    client_entities: ResMut<NetworkResource>,
    mut entity_info_request: EventWriter<SendEntityInfoEvent>,
    mut commands: Commands,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let message: ClientMessages = bincode::deserialize(&message).unwrap();
            println!("Received message from client {}: {:?}", client_id, message);
            if let Some(entity) = client_entities.player_entity_map.get(&client_id) {
                match message {
                    ClientMessages::Move { x, y } => {
                        commands
                            .entity(*entity)
                            .insert(Transform::from_xyz(x, y, 0.0));
                    }
                    ClientMessages::LoadReady => todo!(),
                    ClientMessages::RequestEntityInfo { entity } => {
                        entity_info_request.send(SendEntityInfoEvent { client_id, entity });
                    }
                }
            }
        }
    }
}

fn disconnect_clients_on_exit(exit: EventReader<AppExit>, mut server: ResMut<RenetServer>) {
    if !exit.is_empty() {
        server.disconnect_clients();
    }
}
