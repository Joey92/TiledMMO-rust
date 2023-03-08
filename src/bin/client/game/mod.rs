use bevy::prelude::*;

pub mod components;
pub mod map;
pub mod player;
pub mod systems;

use self::systems::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(map::MapPlugin)
            .add_plugin(player::PlayerPlugin)
            .add_system(set_y_to_z_transform.after("network"));
    }
}
