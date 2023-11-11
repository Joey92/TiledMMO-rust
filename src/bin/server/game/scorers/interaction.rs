use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct InteractedWith;

// Whenever a player interacts with an entity, this event is fired.
#[derive(Event)]
pub struct InteractEvent {
    pub target: Entity,
    pub source: Entity,
}

pub fn interaction_scorer_system(
    mut interactions: EventReader<InteractEvent>,
    mut query: Query<(&Actor, &mut Score), With<InteractedWith>>,
) {
    if interactions.is_empty() {
        for (Actor(actor), mut score) in query.iter_mut() {
            score.set(0.);
        }
        return;
    }

    for interaction in interactions.iter() {
        for (Actor(actor), mut score) in query.iter_mut() {
            if interaction.target != *actor {
                continue;
            }
            score.set(1.);
        }
    }
}
