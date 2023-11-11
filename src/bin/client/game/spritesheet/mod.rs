use bevy::prelude::*;

pub struct SpriteSheetPlugin;

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (animate_sprite, set_animation_direction));
    }
}

#[derive(Component)]
pub struct AnimationIndices {
    pub rows: usize,
    pub cols: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

pub fn deg_to_facing(degs: f32) -> Facing {
    if degs > 45. && degs < 135. {
        Facing::Up
    } else if degs > 135. && degs < 225. {
        Facing::Left
    } else if degs > 225. && degs < 315. {
        Facing::Down
    } else {
        Facing::Right
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum MovementState {
    Idle,
    Moving,
}

#[derive(Component)]
pub struct AnimateDirection(pub Facing);

#[derive(Component)]
pub struct AnimateState(pub MovementState);

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        Ref<AnimateDirection>,
        Ref<AnimateState>,
    )>,
) {
    for (indices, mut timer, mut sprite, dir, state) in &mut query {
        timer.tick(time.delta());

        let row_pos = match dir.0 {
            Facing::Down => 0,
            Facing::Left => 1,
            Facing::Right => 2,
            Facing::Up => 3,
        };

        // idle frame is on the first position of each row
        // so you need to use -1 and + 1 to get the walking frames
        let idle_direction = row_pos * (indices.rows - 1) + 1;

        if state.is_changed() || dir.is_changed() {
            sprite.index = idle_direction;

            if state.0 == MovementState::Moving {
                sprite.index -= 1;
            }
        }

        if state.0 == MovementState::Idle {
            continue;
        }

        if timer.just_finished() {
            sprite.index = if sprite.index == idle_direction - 1 {
                idle_direction + 1
            } else {
                idle_direction - 1
            };
        }
    }
}

#[derive(Component)]
pub struct PreviousPos(pub Vec3);

pub fn set_animation_direction(
    mut units: Query<(&mut AnimateDirection, Ref<Transform>, &mut PreviousPos), Changed<Transform>>,
) {
    for (mut direction, transform, mut previous_pos) in units.iter_mut() {
        // measure angle between previous position and current position
        let diff = Vec3::ZERO + transform.translation - previous_pos.0;

        let mut angle = diff.angle_between(Vec3::X).to_degrees();

        if diff.y < 0. {
            angle += 180.;
        }

        // println!("angle: {}", angle);
        *direction = AnimateDirection(deg_to_facing(angle));
        *previous_pos = PreviousPos(transform.translation);
    }
}
