use bevy::prelude::*;
use tiled_game::calc_z_pos;

use crate::network::ServerSideEntity;

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
