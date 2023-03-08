use bevy::prelude::{App, Component, Plugin};

use self::movement::player_movement;

mod movement;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_movement);
    }
}
