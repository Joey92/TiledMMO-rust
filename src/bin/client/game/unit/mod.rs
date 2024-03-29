use bevy::{log, prelude::*};
use bevy_renet::renet::{DefaultChannel, RenetClient};
use tiled_game::network::messages::client::ClientMessages;

use crate::network::ServerSideEntity;

use super::{
    components::{Highlighted, MousePointerTarget},
    player::PlayerTarget,
    spritesheet::{deg_to_facing, AnimateDirection},
};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(highlight_name)
            .add_system(unhighlight_name)
            .add_system(highlight_entities)
            .add_system(handle_mouse_rightclick)
            .add_system(handle_mouse_leftclick)
            .add_system(target_name)
            .add_system(untarget_name)
            .add_system(set_unit_previous_pos);
    }
}

pub fn target_name(
    units: Query<Entity, Added<PlayerTarget>>,
    mut names: Query<(&Parent, &mut Text)>,
) {
    for entity in units.iter() {
        for (parent, mut text) in names.iter_mut() {
            if parent.get() != entity {
                continue;
            }

            for section in text.sections.iter_mut() {
                section.style.color = Color::RED;
            }
        }
        log::info!("targeted unit: {:?}", entity);
    }
}

pub fn untarget_name(
    mut unit_unhighlighted: RemovedComponents<PlayerTarget>,

    mut names: Query<(&Parent, &mut Text)>,
) {
    for entity in unit_unhighlighted.iter() {
        for (parent, mut text) in names.iter_mut() {
            if parent.get() != entity {
                continue;
            }

            for section in text.sections.iter_mut() {
                section.style.color = Color::BLACK;
            }
        }
        log::info!("untarget unit: {:?}", entity);
    }
}

/** Highlight the unit when hovering over
 */
pub fn highlight_name(
    units: Query<Entity, Added<Highlighted>>,
    mut names: Query<(&Parent, &mut Text)>,
) {
    for entity in units.iter() {
        for (parent, mut text) in names.iter_mut() {
            if parent.get() != entity {
                continue;
            }

            for section in text.sections.iter_mut() {
                section.style.color = Color::BLUE;
            }
        }
        log::info!("highlighted unit: {:?}", entity);
    }
}

pub fn unhighlight_name(
    mut unit_unhighlighted: RemovedComponents<Highlighted>,

    mut names: Query<(&Parent, &mut Text)>,
) {
    for entity in unit_unhighlighted.iter() {
        for (parent, mut text) in names.iter_mut() {
            if parent.get() != entity {
                continue;
            }

            for section in text.sections.iter_mut() {
                section.style.color = Color::BLACK;
            }
        }
        log::info!("unhighlighted unit: {:?}", entity);
    }
}

/**
 * Highlight entities when the mouse is over them
 */
pub fn highlight_entities(
    mut commands: Commands,
    mouse_position: ResMut<MousePointerTarget>,

    // all entities which are highlighted
    // to compare against the list of entities to highlight
    highlighted_entities: Query<Entity, With<Highlighted>>,

    // all entities which can be highlighted
    // exclude entities being targeted by the player
    // to not overwrite the text styling
    entities_highlightable: Query<
        (Entity, &Transform),
        (With<ServerSideEntity>, Without<PlayerTarget>),
    >,
) {
    // find which entities are in the mouse pointer target
    let entities = entities_highlightable
        .iter()
        .filter(|transform| {
            transform
                .1
                .translation
                .distance(mouse_position.0.extend(0.))
                < 32.
        })
        .collect::<Vec<_>>();

    for entity in highlighted_entities.iter() {
        // don't remove the entity if it's in the list of entities to highlight
        if entities.iter().any(|(e, _)| *e == entity) {
            continue;
        }

        commands.entity(entity).remove::<Highlighted>();
    }

    for (entity, _) in entities.iter() {
        commands.entity(*entity).insert(Highlighted);
    }
}

/**
 *
 */
pub fn handle_mouse_rightclick(
    mouse_button_input: ResMut<Input<MouseButton>>,
    highlighted_entities: Query<&ServerSideEntity, With<Highlighted>>,
    mut client: ResMut<RenetClient>,
) {
    if mouse_button_input.just_pressed(MouseButton::Right) {
        if highlighted_entities.is_empty() {
            return;
        }

        let entity = highlighted_entities.iter().next().unwrap();
        let msg = ClientMessages::Interact { entity: entity.0 };

        let msg = bincode::serialize(&msg).unwrap();

        client.send_message(DefaultChannel::ReliableUnordered, msg);
    }
}

pub fn handle_mouse_leftclick(
    mouse_button_input: ResMut<Input<MouseButton>>,
    highlighted_entities: Query<Entity, With<Highlighted>>,
    targeted_entities: Query<Entity, With<PlayerTarget>>,
    mut cmd: Commands,
) {
    if !mouse_button_input.just_pressed(MouseButton::Left) {
        return;
    }

    // remove previous targets
    for entity in targeted_entities.iter() {
        cmd.entity(entity).remove::<PlayerTarget>();
    }

    if highlighted_entities.is_empty() {
        return;
    }

    // select the first entity as a target
    let client_side_entity = highlighted_entities.iter().next().unwrap();
    cmd.entity(client_side_entity).insert(PlayerTarget);
}

#[derive(Component)]
pub struct PreviousPos(pub Vec3);

pub fn set_unit_previous_pos(
    mut units: Query<(&mut AnimateDirection, Ref<Transform>, &mut PreviousPos), Changed<Transform>>,
) {
    for (mut direction, transform, mut previous_pos) in units.iter_mut() {
        // measure angle between previous position and current position
        let diff = Vec3::ZERO + transform.translation - previous_pos.0;

        let mut angle = diff.angle_between(Vec3::X).to_degrees();

        if diff.y < 0. {
            angle += 180.;
        }

        println!("angle: {}", angle);
        *direction = AnimateDirection(deg_to_facing(angle));
        *previous_pos = PreviousPos(transform.translation);
    }
}
