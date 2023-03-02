use bevy::prelude::*;

pub mod components;
pub mod map;
pub mod player;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(map::MapPlugin)
            .add_plugin(player::PlayerPlugin);
    }
}
