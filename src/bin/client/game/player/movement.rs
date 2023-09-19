use bevy::prelude::{Input, KeyCode, Query, Res, Transform, Vec3, With, Without};
use tiled_game::components::Dead;

use crate::game::spritesheet::{AnimateState, MovementState};

use super::Player;

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<(&mut Transform, &mut AnimateState), (With<Player>, Without<Dead>)>,
) {
    if player.is_empty() {
        return;
    }

    let (mut transform, mut state) = player.single_mut();

    if !keyboard_input.any_pressed(vec![KeyCode::A, KeyCode::D, KeyCode::W, KeyCode::S]) {
        state.0 = MovementState::Idle;
        return;
    }

    let mut speed: f32 = 1.;

    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        speed = 2.;
    }
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::A) {
        direction += Vec3::NEG_X;
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction += Vec3::X;
    }
    if keyboard_input.pressed(KeyCode::W) {
        direction += Vec3::Y;
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction += Vec3::NEG_Y;
    }

    if state.0 != MovementState::Moving {
        state.0 = MovementState::Moving;
    }

    transform.translation += direction * speed;
}
