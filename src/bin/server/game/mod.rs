use bevy::prelude::{App, Plugin};
pub mod components;
pub mod map;
pub mod player;
pub mod scripts;

use self::map::*;
use self::player::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MapsPlugin).add_system(add_new_player);
    }
}

