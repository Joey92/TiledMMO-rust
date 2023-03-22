use bevy::prelude::*;

pub mod components;
pub mod map;
pub mod player;
pub mod systems;

use self::{components::MousePointerTarget, systems::*};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(map::MapPlugin)
            .init_resource::<MousePointerTarget>()
            .add_plugin(player::PlayerPlugin)
            .add_system(set_y_to_z_transform)
            .add_system(cursor_system)
            .add_system(highlight_entities)
            .add_system(handle_mouse_rightclick)
            .add_system(handle_mouse_leftclick);
    }
}
