use bevy::prelude::*;
use big_brain::prelude::*;

#[derive(Debug, Clone, Component, ScorerBuilder)]
pub struct TimePassed(pub Timer);

pub fn timer_scorer_system(
    time: Res<Time>,
    mut query: Query<(&Actor, &mut Score, &mut TimePassed)>,
) {
    for (Actor(actor), mut score, mut timer) in query.iter_mut() {
        if !timer.0.tick(time.delta()).finished() {
            score.set(0.);
            continue;
        }

        score.set(1.);
    }
    return;
}
