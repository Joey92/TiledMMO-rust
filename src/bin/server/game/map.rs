use std::{
    collections::{HashMap, HashSet},
    fs,
};

use bevy::{
    log,
    prelude::*,
    time::{Time, Timer, TimerMode},
};
use tiled::{Loader, Map as TiledMap};
use tiled_game::{components::Interactable, network::messages::server::ServerMessages};

use crate::{
    game::{
        interactions::Portal,
        npc::{Enemy, NPCBundle},
        thinkers::get_thinker,
    },
    network::{NetworkClientId, SendServerMessageEvent},
};

use super::player::Player;

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

#[derive(Event)]
pub struct Teleport {
    pub entity: Entity,
    pub map: String,
    pub map_instance: Option<Entity>,
    pub prev_map_instance: Option<Entity>,
    pub position: Vec3,
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
        app.add_systems(Startup, setup_map_manager)
            // Handle despawn of entities
            .add_event::<DespawnEvent>()
            .add_event::<Teleport>()
            // removes map instances with no players in certain intervals
            .add_systems(
                Update,
                (
                    map_instance_cleanup,
                    handle_teleport,
                    send_map_instance_entities,
                    spawn_units,
                ),
            );

        #[cfg(feature = "verbose-output")]
        {
            app.add_system(map_instance_debug);
        }
    }
}

fn setup_map_manager(mut commands: Commands) {
    let mut map_manager = MapManager::new();

    let dir = "assets";
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

// Despawns an entity on the client side
#[derive(Event)]
pub struct DespawnEvent {
    pub entity: Entity,
    pub map: Entity,
}

fn handle_teleport(
    mut commands: Commands,
    mut map_manager: ResMut<MapManager>,
    mut server_message: EventWriter<SendServerMessageEvent>,
    mut despawn_event: EventWriter<DespawnEvent>,
    mut teleport_events: EventReader<Teleport>,
    players: Query<(Entity, &NetworkClientId)>,
) {
    for teleport in teleport_events.iter() {
        println!(
            "Unit {:?} wants to change map to {:?}",
            teleport.entity, teleport.map
        );

        let entity = players.get(teleport.entity).ok();

        if entity.is_none() {
            println!("Entity {:?} not found", teleport.entity);
            continue;
        }

        let teleporting_entity = entity.unwrap();

        let mut map_instance = None;

        // Check if map instance exists
        if let Some(dest_instance) = teleport.map_instance {
            for instance_entity in map_manager.instances.iter() {
                if *instance_entity == dest_instance {
                    // found map instance
                    // todo decrement population of previous map instance
                    // if let Some(prev_instance) = map_change_request.prev_map_instance {
                    //     map_manager.decrement_instance_population(prev_instance.to_owned());
                    // };
                    println!(
                        "Found map instance {:?} for player {:?}",
                        instance_entity, teleporting_entity
                    );

                    map_instance = Some(*instance_entity)
                }
            }
        }

        // Check if global map instance exists
        if let Some(global_map) = map_manager.global.get(&teleport.map) {
            // Subtract player from current map instance
            println!(
                "Found global map instance {:?} for player {:?}",
                &teleport.map, teleporting_entity
            );

            map_instance = Some(*global_map)
        }

        if map_instance.is_none() {
            println!(
                "No map instance found for {:?}, creating a new one",
                teleport.map
            );

            // If no map instance was found, create a new one
            let new_map_instance = commands
                .spawn((MapInstance, MapName(teleport.map.clone())))
                .id();

            map_manager.instances.push(new_map_instance);

            println!("Created new instance of map {:?}", teleport.map);
            map_instance = Some(new_map_instance);
        }

        // check if we need to be removed from previous map instance
        if let Some(prev_instance) = teleport.prev_map_instance {
            commands
                .entity(prev_instance)
                .remove_children(&[teleporting_entity.0]);

            despawn_event.send(DespawnEvent {
                entity: teleporting_entity.0,
                map: prev_instance,
            });
        };

        commands
            .entity(map_instance.unwrap())
            .push_children(&[teleporting_entity.0]);

        // Add player to map instance
        commands
            .entity(teleporting_entity.0)
            .insert(Transform::from_xyz(
                teleport.position.x,
                teleport.position.y,
                teleport.position.z,
            ));

        // send map change event to client
        // so it can load the new map
        server_message.send(SendServerMessageEvent::directly_to(
            teleporting_entity.1 .0,
            ServerMessages::Map {
                name: teleport.map.clone(),
                position: teleport.position,
            },
        ));
    }
}

// Spawn units for each newly created map instance
fn spawn_units(
    mut commands: Commands,
    query: Query<(Entity, &MapName), Added<MapInstance>>,
    map_manager: Res<MapManager>,
) {
    for (map_instance_entity, map_name) in query.iter() {
        if let Some(map) = map_manager.atlas.get(&map_name.0) {
            map.layers()
                .flat_map(|layer| {
                    // match as ObjectLayer and give back Result
                    match layer.layer_type() {
                        tiled::LayerType::Objects(layer) => Some(layer),
                        _ => None,
                    }
                })
                .flat_map(|layer| layer.objects())
                .flat_map(|object| match object.shape {
                    tiled::ObjectShape::Point(_, _) => Some(object),
                    _ => None,
                })
                .for_each(|obj| {
                    let mut cmd = commands.spawn_empty();

                    let spawn_point = Transform::from_xyz(
                        obj.x,
                        // flipping the y coordinate to match bevy's coordinate system
                        -obj.y + (map.height * map.tile_height) as f32,
                        1.,
                    );

                    cmd.insert((
                        NPCBundle::new(obj.name.to_owned(), obj.user_type.clone(), spawn_point),
                        MapInstanceEntity(map_instance_entity),
                    ));

                    obj.properties.get("thinker").map(|script| match script {
                        tiled::PropertyValue::StringValue(val) => {
                            if let Ok(thinker) = get_thinker(val).map_err(|err| println!("{}", err))
                            {
                                cmd.insert(thinker);
                            }
                        }
                        _ => {}
                    });

                    obj.properties.get("enemy").map(|script| match script {
                        tiled::PropertyValue::BoolValue(val) => {
                            if *val {
                                cmd.insert(Enemy);
                            }
                        }
                        _ => {}
                    });

                    obj.properties
                        .get("interactable")
                        .map(|script| match script {
                            tiled::PropertyValue::BoolValue(val) => {
                                if *val {
                                    cmd.insert(Interactable);
                                }
                            }
                            _ => {}
                        });

                    if obj.user_type == "Portal" {
                        let map_name = obj
                            .properties
                            .get("map")
                            .map(|script| match script {
                                tiled::PropertyValue::StringValue(map_name) => {
                                    Some(map_name.to_owned())
                                }
                                _ => None,
                            })
                            .flatten();

                        let x = obj
                            .properties
                            .get("x")
                            .map(|script| match script {
                                tiled::PropertyValue::FloatValue(map_name) => {
                                    Some(map_name.to_owned())
                                }
                                _ => None,
                            })
                            .flatten();

                        let y = obj
                            .properties
                            .get("y")
                            .map(|script| match script {
                                tiled::PropertyValue::FloatValue(map_name) => {
                                    Some(map_name.to_owned())
                                }
                                _ => None,
                            })
                            .flatten();

                        match (map_name, x, y) {
                            (Some(map_name), Some(x), Some(y)) => {
                                cmd.insert(Portal {
                                    map: map_name,
                                    position: Transform::from_xyz(x, y, 0.),
                                });
                            }
                            _ => {}
                        }
                    }

                    let id = cmd.id();

                    println!(
                        "Spawning unit {:?} ({:?}) Server ID: {:?}",
                        obj.name, obj.user_type, id
                    );

                    commands.entity(map_instance_entity).push_children(&[id]);
                });
        }
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
            println!("Sending entity list to client {:?}", client_id.0);
            server_messages.send(SendServerMessageEvent::directly_to(
                client_id.0,
                ServerMessages::EntityList {
                    entities: npc_list
                        .iter()
                        .filter(|e| &player_entity != *e)
                        .copied()
                        .collect(),
                },
            ));
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
