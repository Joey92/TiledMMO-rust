use bevy::prelude::*;
use big_brain::BigBrainSet;

use self::{
    interaction::{interaction_scorer_system, InteractEvent},
    timer::timer_scorer_system,
};

pub mod interaction;
pub mod timer;

pub struct ScorerPlugin;

impl Plugin for ScorerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractEvent>().add_systems(
            PreUpdate,
            (interaction_scorer_system, timer_scorer_system).in_set(BigBrainSet::Scorers),
        );
    }
}
