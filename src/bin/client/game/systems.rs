use bevy::{prelude::*, window::PrimaryWindow};

use tiled_game::calc_z_pos;

use crate::network::ServerSideEntity;

use super::components::MousePointerTarget;

pub fn set_y_to_z_transform(
    mut query: Query<
        &mut Transform,
        (
            With<ServerSideEntity>,
            Or<(Changed<Transform>, Added<Transform>)>,
        ),
    >,
) {
    for mut transform in query.iter_mut() {
        transform.translation.z = calc_z_pos(transform.translation.y);
    }
}

// todo update player target when camera moves as well
pub fn cursor_system(
    mut cursor_evr: EventReader<CursorMoved>,

    // need to get window dimensions
    windows: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    camera_q: Query<(&Camera, &GlobalTransform)>,

    mut player_target: ResMut<MousePointerTarget>,
) {
    for ev in cursor_evr.iter() {
        let window = windows.get(ev.window).unwrap();
        let (camera, camera_transform) = camera_q.single();

        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            player_target.0 = world_position;
        }
    }
}
