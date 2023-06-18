use bevy::prelude::*;

use self::movement::player_movement;

mod movement;

#[derive(Component)]
pub struct Player;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct PlayerTarget;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(player_movement);
    }
}
