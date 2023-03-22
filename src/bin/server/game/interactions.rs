use bevy::prelude::*;
use tiled_game::components::Interactable;

use super::{map::Teleport, player::Player};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EntityInteractionEvent>()
            .add_system(portal_interaction);
    }
}

// Whenever a player interacts with an entity, this event is fired.
pub struct EntityInteractionEvent {
    pub target: Entity,
    pub source: Entity,
}

#[derive(Component)]
pub struct Portal {
    pub map: String,
    pub position: Transform,
}

fn portal_interaction(
    mut map_change_events: EventWriter<Teleport>,
    mut interactions: EventReader<EntityInteractionEvent>,
    portals: Query<(&Portal, &Transform), With<Interactable>>,
    players: Query<(Entity, &Transform), With<Player>>,
) {
    interactions.iter().for_each(|interaction| {
        let portal = portals.get(interaction.target).ok();
        let units = players.get(interaction.source).ok();

        if portal.is_none() || units.is_none() {
            return;
        }

        let (portal, portal_transform) = portal.unwrap();
        let (unit, unit_transform) = units.unwrap();

        if portal_transform
            .translation
            .distance(unit_transform.translation)
            > 32.
        {
            // send a message to the client to tell them they are too far away
            // or handle it on the client side
            // and just return here
            return;
        }

        map_change_events.send(Teleport {
            entity: unit,
            map: portal.map.clone(),
            position: portal.position.clone(),
            prev_map_instance: None,
            map_instance: None,
        });
    });
}
