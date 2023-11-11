use bevy::prelude::*;
use big_brain::BigBrainSet;

use self::{chat::chat_action_system, spell::spell_action_system};

pub mod chat;
pub mod spell;

pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (spell_action_system, chat_action_system).in_set(BigBrainSet::Actions),
        );
    }
}
