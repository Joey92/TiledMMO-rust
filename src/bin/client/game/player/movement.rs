use bevy::prelude::{Input, KeyCode, Query, Res, Transform, Vec3, With, Without};
use tiled_game::components::Dead;

use crate::game::spritesheet::{AnimateDirection, AnimateState, Facing, MovementState};

use super::Player;

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<
        (&mut Transform, &mut AnimateDirection, &mut AnimateState),
        (With<Player>, Without<Dead>),
    >,
) {
    if player.is_empty() {
        return;
    }

    let (mut transform, mut animate, mut state) = player.single_mut();

    if !keyboard_input.any_pressed(vec![KeyCode::A, KeyCode::D, KeyCode::W, KeyCode::S]) {
        state.0 = MovementState::Idle;
        return;
    }

    let mut speed: f32 = 1.;

    if keyboard_input.pressed(KeyCode::LShift) {
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

    // todo: use the first movement direction that user pressed
    // so if you start running to the right and then up/down it will still animate to the right
    if direction.y < 0. {
        animate.0 = Facing::Down;
    } else if direction.y > 0. {
        animate.0 = Facing::Up;
    } else if direction.x < 0. {
        animate.0 = Facing::Left;
    } else if direction.x > 0. {
        animate.0 = Facing::Right;
    }

    if state.0 != MovementState::Moving {
        state.0 = MovementState::Moving;
    }

    transform.translation += direction * speed;
}
