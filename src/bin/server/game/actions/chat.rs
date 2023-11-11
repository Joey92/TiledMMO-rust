use std::time::Duration;

use bevy::prelude::*;
use big_brain::prelude::*;
use tiled_game::components::Saying;

#[derive(Debug, Clone)]
pub enum ChatType {
    Say,
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct Chat {
    pub chat_type: ChatType,
    pub message: String,
    uptime: Timer,
}

impl Chat {
    pub fn say(message: String) -> Self {
        let duration = 3000 + message.len() * 100;
        Self {
            chat_type: ChatType::Say,
            message,
            uptime: Timer::new(Duration::from_millis(duration as u64), TimerMode::Once),
        }
    }
}

pub fn chat_action_system(
    mut cmd: Commands,
    time: Res<Time>,

    mut query: Query<(&Actor, &mut ActionState, &mut Chat)>,
) {
    for (Actor(actor), mut state, mut chat) in query.iter_mut() {
        match *state {
            ActionState::Requested => {
                cmd.entity(*actor).insert(Saying(chat.message.clone()));
                *state = ActionState::Executing;
            }
            ActionState::Executing => {
                if chat.uptime.tick(time.delta()).finished() {
                    *state = ActionState::Success;
                    cmd.entity(*actor).remove::<Saying>();
                }
            }
            ActionState::Cancelled => {
                chat.uptime.reset();
                cmd.entity(*actor).remove::<Saying>();
                *state = ActionState::Failure;
            }
            _ => {}
        }
    }
}
