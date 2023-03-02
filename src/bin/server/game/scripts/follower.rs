use bevy::prelude::{Query, Transform, With};

use super::Script;

#[derive(Default)]
pub struct Follower;

pub fn follower_script(mut query: Query<&mut Transform, With<Script<Follower>>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 0.1;
    }
}
