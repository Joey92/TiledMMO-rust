use bevy::prelude::*;

pub struct SpriteSheetPlugin;

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate_sprite);
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
