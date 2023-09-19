use bevy::log;
use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

use self::collision::*;
use self::tiled::*;

pub mod collision;
pub mod tiled;

#[derive(Default)]
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<TiledMap>()
            .add_plugins(TilemapPlugin)
            .add_asset_loader(TiledLoader)
            .init_resource::<CurrentMap>()
            .add_event::<MapChangeEvent>()
            .add_systems(
                Update,
                (change_map, process_loaded_maps, load_collision, unload_map),
            );
        //.add_system(switch_between_maps_test);
    }
}

#[derive(Resource, Default)]
pub struct CurrentMap {
    pub name: String,
    pub player_plane_z: Option<i8>,
}

/**
 * Insert this component to every entity that should be unloaded on a map change.
 */
#[derive(Component)]
pub struct MapName(pub String);

// This system removes all entities with the MapName component that do not match the currently active map.
pub fn unload_map(
    mut commands: Commands,
    current_map_name: Res<CurrentMap>,
    map_entities: Query<(Entity, &MapName)>,
) {
    if !current_map_name.is_changed() {
        return;
    }

    for (layer_entity, map_name) in map_entities.iter() {
        if map_name.0 == current_map_name.name {
            continue;
        }

        commands.entity(layer_entity).despawn_recursive();
    }

    info!("Unloaded previous map entities");
}

/**
 * Send this event when you want to change the map.
 */
#[derive(Event)]
pub struct MapChangeEvent {
    pub map_name: String,
}

pub fn change_map(
    mut commands: Commands,
    mut map_events: EventReader<MapChangeEvent>,
    asset_server: Res<AssetServer>,
) {
    for event in map_events.iter() {
        log::info!("Changing map to {}", event.map_name);

        commands.insert_resource(CurrentMap {
            name: event.map_name.clone(),
            ..default()
        });

        let map_handle: Handle<TiledMap> = asset_server.load(format!("{}", event.map_name));

        commands
            .spawn(TiledMapBundle {
                tiled_map: map_handle,
                ..Default::default()
            })
            .insert(MapName(event.map_name.clone()));
    }
}
