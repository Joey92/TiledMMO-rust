use bevy::{prelude::*, window::PrimaryWindow};
use bevy_renet::renet::RenetClient;
use tiled_game::{
    calc_z_pos,
    components::Target,
    network::{messages::client::ClientMessages, ClientChannel},
};

use crate::network::ServerSideEntity;

use super::{
    components::{Highlighted, MousePointerTarget},
    player::Player,
};

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

pub fn highlight_entities(
    mut commands: Commands,
    mouse_position: ResMut<MousePointerTarget>,
    highlighted_entities: Query<Entity, With<Highlighted>>,
    unhighlighted_entities: Query<
        (Entity, &Transform),
        (With<ServerSideEntity>, Without<Highlighted>),
    >,
) {
    // find which entities are in the mouse pointer target
    let entities = unhighlighted_entities
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
        commands.entity(entity).remove::<Highlighted>();
    }

    for (entity, _) in entities.iter() {
        commands.entity(*entity).insert(Highlighted);
    }
}

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

        client.send_message(ClientChannel::Input, msg);
    }
}

pub fn handle_mouse_leftclick(
    mouse_button_input: ResMut<Input<MouseButton>>,
    highlighted_entities: Query<&ServerSideEntity, With<Highlighted>>,
    mut client: ResMut<RenetClient>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if highlighted_entities.is_empty() {
            let msg = ClientMessages::Target { target: None };

            let msg = bincode::serialize(&msg).unwrap();

            client.send_message(ClientChannel::Input, msg);
            return;
        }

        let entity = highlighted_entities.iter().next().unwrap();
        let msg = ClientMessages::Target {
            target: Some(entity.0),
        };

        let msg = bincode::serialize(&msg).unwrap();

        client.send_message(ClientChannel::Input, msg);
    }
}
