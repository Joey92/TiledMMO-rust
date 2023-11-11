use bevy::prelude::*;

pub mod components;
pub mod map;
pub mod player;
pub mod resources;
pub mod spritesheet;
pub mod systems;
pub mod unit;

use self::{components::MousePointerTarget, resources::GameState, systems::*};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(map::MapPlugin)
            .add_state::<GameState>()
            .init_resource::<MousePointerTarget>()
            .add_plugins(player::PlayerPlugin)
            .add_systems(Update, (cursor_system, set_y_to_z_transform))
            .add_plugins(unit::UnitPlugin)
            .add_plugins(spritesheet::SpriteSheetPlugin);
    }
}
