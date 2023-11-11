use std::time::Duration;

use bevy::prelude::*;
use big_brain::{
    prelude::{FirstToScore, Steps},
    thinker::Thinker,
};

use crate::game::{actions::chat::Chat, scorers::timer::TimePassed};

pub fn get_talker_thinker() -> big_brain::thinker::ThinkerBuilder {
    let talk = Steps::build()
        .label("Talk about stuff")
        .step(Chat::say("Hello there".to_string()))
        .step(Chat::say("Welcome to my world".to_string()))
        .step(Chat::say("WTF is happening?".to_string()));

    Thinker::build()
        .picker(FirstToScore { threshold: 0.8 })
        .when(
            TimePassed(Timer::new(Duration::from_secs(10), TimerMode::Repeating)),
            talk,
        )
}
