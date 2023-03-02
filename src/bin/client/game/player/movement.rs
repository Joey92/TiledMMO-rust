use bevy::prelude::{Input, KeyCode, Query, Res, Transform, Vec3, With};

use super::Player;

pub fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut speed: f32 = 1.;

    if keyboard_input.pressed(KeyCode::LShift) {
        speed = 2.;
    }

    for mut transform in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation += Vec3::new(-1.0 * speed, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation += Vec3::new(1.0 * speed, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation += Vec3::new(0.0, 1.0 * speed, 0.0);
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation += Vec3::new(0.0, -1.0 * speed, 0.0);
        }
    }
}
