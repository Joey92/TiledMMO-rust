use bevy::prelude::{AddAsset, Plugin};
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
            .add_plugin(TilemapPlugin)
            .add_asset_loader(TiledLoader)
            .init_resource::<CurrentMap>()
            .add_event::<MapChangeEvent>()
            .add_system(change_map)
            .add_system(process_loaded_maps)
            .add_system(load_collision)
            .add_system(unload_map);
        //.add_system(switch_between_maps_test);
    }
}
