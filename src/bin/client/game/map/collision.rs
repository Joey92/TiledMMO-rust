use bevy::{
    log,
    prelude::{
        Added, AssetEvent, Assets, Commands, EventReader, Handle, Name, Query, Res, Transform,
    },
    transform::TransformBundle,
};
use bevy_rapier2d::prelude::Collider;
use tiled::{LayerType, ObjectShape};

use crate::game::map::MapName;

use super::{tiled::TiledMap, CurrentMap};

/**
 * Calculate collisions when a new tilemap gets loaded
 */
pub fn load_collision(
    maps: Res<Assets<TiledMap>>,
    map_name: Res<CurrentMap>,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    loaded_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
    mut commands: Commands,
) {
    let mut changed_maps = Vec::<Handle<TiledMap>>::default();
    for event in map_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                changed_maps.push(handle.clone());
            }
            AssetEvent::Modified { handle } => {
                changed_maps.push(handle.clone());
            }
            AssetEvent::Removed { handle } => {
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == handle);
            }
        }

        // If we have new map entities add them to the changed_maps list.
        for new_map_handle in loaded_maps.iter() {
            changed_maps.push(new_map_handle.clone_weak());
        }

        for changed_map in changed_maps.iter() {
            // load tilemap from maps
            let tilemap = maps.get(changed_map);
            if tilemap.is_none() {
                log::info!("No tilemap found");
                return;
            }

            tilemap.map(|tilemap_container| {
                let map = &tilemap_container.map;

                map.layers()
                    .find(|layer| layer.name == "collision")
                    .and_then(|layer| {
                        // match as ObjectLayer and give back Result
                        match layer.layer_type() {
                            LayerType::Objects(layer) => Some(layer),
                            _ => None,
                        }
                    })
                    .map(|layer| layer.objects())
                    .expect("No collision layer found")
                    .for_each(|object| match object.shape {
                        ObjectShape::Rect { width, height } => {
                            log::info!("Loading collision object");
                            // spawn fixed collision box using width and hight
                            commands
                                .spawn(Collider::cuboid(width / 2., height / 2.))
                                .insert(TransformBundle::from(Transform::from_xyz(
                                    object.x + width / 2.,
                                    (-object.y - height / 2.)
                                        + (map.height * map.tile_height) as f32,
                                    0.0,
                                )))
                                .insert((MapName(map_name.name.clone()), Name::new("Collider")));
                        }
                        _ => {}
                    })
            });
        }
    }
}
