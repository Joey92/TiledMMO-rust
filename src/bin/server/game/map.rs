use std::{
    collections::{HashMap, HashSet},
    fs,
};

use bevy::{
    log,
    prelude::{
        Added, BuildChildren, Changed, Children, Commands, Component, DespawnRecursiveExt, Entity,
        EventReader, EventWriter, Or, Parent, Plugin, Query, Res, ResMut, Resource, SpatialBundle,
        Transform, With,
    },
    time::{Time, Timer, TimerMode},
};
use tiled::{Loader, Map as TiledMap};
use tiled_game::{
    network::messages::server::ServerMessages,
    shared_components::{Name, Player},
};

use crate::network::{NetworkClientId, SendServerMessageEvent};

use super::scripts::handle_add_script;

#[derive(Component, Debug)]
pub struct MapName(pub String);

#[derive(Component)]
pub struct MapInstance;

#[derive(Component)]
pub struct GlobalMap;

#[derive(Component)]
pub struct TileMapData(pub TiledMap);

#[derive(Component, Debug)]
pub struct MapInstanceEntity(pub Entity);

#[derive(Component, Debug)]
pub struct NPCList(HashSet<Entity>);

#[derive(Component)]
pub struct Teleport {
    pub map: MapName,
    pub map_instance: Option<Entity>,
    pub prev_map_instance: Option<Entity>,
    pub position: Transform,
}

#[derive(Resource, Default)]
pub struct MapManager {
    pub atlas: HashMap<String, TiledMap>,
    pub instances: Vec<Entity>,
    pub global: HashMap<String, Entity>,
    pub cleanup_timer: Timer,
}

impl MapManager {
    fn new() -> Self {
        Self {
            cleanup_timer: Timer::new(std::time::Duration::from_secs(300), TimerMode::Repeating),
            ..Default::default()
        }
    }
}

pub struct MapsPlugin;

impl Plugin for MapsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(map_manager)
            // removes map instances with no players in certain intervals
            .add_system(map_instance_cleanup)
            // teleports players between map instances
            .add_system(map_change_handler)
            // sends all units to the client
            .add_system(send_map_instance_entities)
            // Spawns NPCs on new map instances
            .add_system(spawn_units)
            // Handle despawn of entities
            .add_event::<DespawnEvent>() // we use an event to despawn entities
            .add_system(send_despawn_event)
            // Updates movement of entities
            .add_system(send_movement);

        #[cfg(feature = "verbose-output")]
        {
            app.add_system(map_instance_debug);
        }
    }
}

fn map_manager(mut commands: Commands) {
    let mut map_manager = MapManager::new();

    let dir = "maps";
    let maps: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter(|f| {
            f.as_ref()
                .expect("Could not read file")
                .file_name()
                .into_string()
                .expect("Could not read filename")
                .ends_with(".tmx")
        })
        .map(|f| f.unwrap().file_name().into_string().unwrap())
        .collect();

    let mut loader = Loader::new();
    println!("Loading {} maps", maps.len());
    let mut maps_collection = HashMap::new();
    for name in maps.iter() {
        let tilemap_data = loader
            .load_tmx_map(format!("{}/{}", dir, name))
            .expect("Could not load map");
        maps_collection.insert(name.clone(), tilemap_data);
    }

    // create entities for each map instance
    let global: HashMap<String, Entity> = maps_collection
        .iter()
        .filter(|map| match map.1.properties.get("global_instance") {
            Some(val) => match val {
                tiled::PropertyValue::BoolValue(global) => global.to_owned(),
                _ => false,
            },
            _ => false,
        })
        .map(|global_maps| {
            println!("Added global map {:?}", global_maps.0);
            (
                global_maps.0.to_owned(),
                commands
                    .spawn((MapInstance, GlobalMap, MapName(global_maps.0.clone())))
                    .id(),
            )
        })
        .collect();

    map_manager.atlas = maps_collection;
    map_manager.global = global;
    // Insert maps as resource
    commands.insert_resource(map_manager);
}

// Despawn all map instances that have no players
fn map_instance_cleanup(
    mut commands: Commands,
    time: Res<Time>,
    mut map_manager: ResMut<MapManager>,
    current_maps: Query<&MapInstanceEntity>,
) {
    map_manager.cleanup_timer.tick(time.delta());

    if !map_manager.cleanup_timer.finished() {
        return;
    }

    let instance_in_use: Vec<Entity> = current_maps
        .iter()
        .map(|map| map.0)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let maps_to_remove = map_manager
        .instances
        .iter()
        .filter(|instance| !instance_in_use.contains(instance));

    log::info!("Despawn {} map instances", maps_to_remove.clone().count());

    for entity in maps_to_remove {
        commands.entity(*entity).despawn_recursive();
    }

    map_manager
        .instances
        .retain(|instance| instance_in_use.contains(instance));
}

fn map_change_handler(
    mut commands: Commands,
    mut map_manager: ResMut<MapManager>,
    mut server_message: EventWriter<SendServerMessageEvent>,
    mut despawn_event: EventWriter<DespawnEvent>,
    teleporting_players: Query<(Entity, &Teleport, &NetworkClientId), With<Teleport>>,
) {
    for (player_entity, map_change_request, client_id) in teleporting_players.iter() {
        println!(
            "Player {:?} wants to change map to {:?}",
            player_entity, map_change_request.map
        );

        let mut map_instance = None;

        // Check if map instance exists
        if let Some(dest_instance) = map_change_request.map_instance {
            for instance_entity in map_manager.instances.iter() {
                if *instance_entity == dest_instance {
                    // found map instance
                    // todo decrement population of previous map instance
                    // if let Some(prev_instance) = map_change_request.prev_map_instance {
                    //     map_manager.decrement_instance_population(prev_instance.to_owned());
                    // };
                    println!(
                        "Found map instance {:?} for player {:?}",
                        instance_entity, player_entity
                    );

                    map_instance = Some(*instance_entity)
                }
            }
        }

        // Check if global map instance exists
        if let Some(global_map) = map_manager.global.get(&map_change_request.map.0) {
            // Subtract player from current map instance
            println!(
                "Found global map instance {:?} for player {:?}",
                &map_change_request.map.0, player_entity
            );

            map_instance = Some(*global_map)
        }

        if map_instance.is_none() {
            println!(
                "No map instance found for {:?}, creating a new one",
                map_change_request.map.0
            );

            // If no map instance was found, create a new one
            let new_map_instance = commands
                .spawn((MapInstance, MapName(map_change_request.map.0.clone())))
                .id();

            map_manager.instances.push(new_map_instance);

            println!("Created new instance of map {:?}", map_change_request.map.0);
            map_instance = Some(new_map_instance);
        }

        // check if we need to be removed from previous map instance
        if let Some(prev_instance) = map_change_request.prev_map_instance {
            commands
                .entity(prev_instance)
                .remove_children(&[player_entity]);

            despawn_event.send(DespawnEvent {
                entity: player_entity,
                map: prev_instance,
            });
        };

        commands
            .entity(map_instance.unwrap())
            .push_children(&[player_entity]);

        // Add player to map instance
        commands
            .entity(player_entity)
            .remove::<Teleport>()
            .insert(map_change_request.position.clone());

        // send map change event to client
        // so it can load the new map
        server_message.send(SendServerMessageEvent {
            client_id: Some(client_id.0),
            message: ServerMessages::Map {
                name: map_change_request.map.0.clone(),
                position: map_change_request.position.translation,
            },
        });
    }
}

struct DespawnEvent {
    entity: Entity,
    map: Entity,
}

// Sends a despawn message to each player on the map
// When a player or entity leaves the map
fn send_despawn_event(
    mut server_message: EventWriter<SendServerMessageEvent>,
    mut despawn_events: EventReader<DespawnEvent>,
    all_players: Query<
        (
            Entity,
            &NetworkClientId,
            &Parent, /* Map Instances Entity */
        ),
        With<Player>,
    >,
) {
    for despawn in despawn_events.iter() {
        // get players on the map
        all_players
            .iter()
            .filter(|(_, _, map_instance_entity)| map_instance_entity.get() == despawn.map)
            .for_each(|(_, client_id, _)| {
                // send despawn event to all players in map
                server_message.send(SendServerMessageEvent {
                    client_id: Some(client_id.0),
                    message: ServerMessages::Despawn {
                        entity: despawn.entity,
                    },
                });
            });
    }
}

// Spawn units for each newly created map instance
fn spawn_units(
    mut commands: Commands,
    query: Query<(Entity, &MapName), Added<MapInstance>>,
    map_manager: Res<MapManager>,
) {
    for (map_instance_entity, map_name) in query.iter() {
        map_manager.atlas.get(&map_name.0).map(|map| {
            map.layers()
                .flat_map(|layer| {
                    // match as ObjectLayer and give back Result
                    match layer.layer_type() {
                        tiled::LayerType::ObjectLayer(layer) => Some(layer),
                        _ => None,
                    }
                })
                .flat_map(|layer| layer.objects())
                .flat_map(|object| match object.shape {
                    tiled::ObjectShape::Point(_, _) => Some(object),
                    _ => None,
                })
                .for_each(|obj| {
                    println!("Spawning unit {:?}", obj.name);
                    let mut cmd = commands.spawn_empty();

                    cmd.insert(Name(obj.name.clone()))
                        .insert(MapInstanceEntity(map_instance_entity))
                        .insert(SpatialBundle {
                            transform: Transform::from_xyz(obj.x, obj.y, 0.),
                            ..Default::default()
                        });

                    obj.properties.get("script").map(|script| match script {
                        tiled::PropertyValue::StringValue(script_name) => {
                            handle_add_script(script_name.to_owned(), &mut cmd);
                        }
                        _ => {}
                    });

                    let id = cmd.id();

                    commands.entity(map_instance_entity).push_children(&[id]);
                });
        });
    }
}

// Sends a list of entities to the client
// when they spawn in a new map instance
fn send_map_instance_entities(
    mut server_messages: EventWriter<SendServerMessageEvent>,
    players_spawning_in: Query<
        (Entity, &NetworkClientId, &Parent),
        (With<Player>, Or<(Changed<Parent>, Added<Parent>)>),
    >,
    map_instances: Query<&Children, With<MapInstance>>,
) {
    for (player_entity, client_id, current_map_instance) in players_spawning_in.iter() {
        let npc = map_instances.get(current_map_instance.get()).ok();

        if let Some(npc_list) = npc {
            server_messages.send(SendServerMessageEvent {
                client_id: Some(client_id.0),
                message: ServerMessages::EntityList {
                    entities: npc_list
                        .iter()
                        .filter(|e| &player_entity != *e) // remove the player entity from the list
                        .map(|e| *e)
                        .collect(),
                },
            });
        }
    }
}

fn send_movement(
    mut server_messages: EventWriter<SendServerMessageEvent>,
    moved_entities: Query<(Entity, &Transform, &Parent), Changed<Transform>>,
    players: Query<(Entity, &Parent, &NetworkClientId)>,
) {
    for (moved_entity, transform, map_instance) in moved_entities.iter() {
        for (player_entity, player_map_instance, client_id) in players.iter() {
            if player_map_instance.get() == map_instance.get() && player_entity != moved_entity {
                server_messages.send(SendServerMessageEvent {
                    client_id: Some(client_id.0),
                    message: ServerMessages::Move {
                        entity: player_entity,
                        x: transform.translation.x,
                        y: transform.translation.y,
                    },
                });
            }
        }
    }
}

#[cfg(feature = "verbose-output")]
fn map_instance_debug(
    map_instances: Query<(Entity, &PlayerCount, &MapName), Changed<PlayerCount>>,
) {
    for (entity, player_count, map_name) in map_instances.iter() {
        println!(
            "Map instance {:?} (Name: {:?}) has {} players",
            entity.index(),
            map_name.0,
            player_count.0
        );
    }
}
