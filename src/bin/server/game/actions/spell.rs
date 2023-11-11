use bevy::prelude::*;
use big_brain::prelude::*;
use tiled_game::components::Target;

use crate::game::{combat::DoDamageEvent, map::Teleport};

#[derive(Component)]
pub struct Casting;

#[derive(Debug, Clone)]
pub enum SpellCastType {
    Damage { amount: i32, mana: i32 },
    Teleport { map: String, coordinates: Vec3 },
}

#[derive(Debug, Clone, Component, ActionBuilder)]
pub struct CastSpell {
    pub cast_type: SpellCastType,
    pub cast_duration: Timer,
    pub channeling: bool,
    pub force_self_target: bool,
}

impl Default for CastSpell {
    fn default() -> Self {
        Self {
            cast_type: SpellCastType::Damage { amount: 0, mana: 0 },
            cast_duration: Default::default(),
            channeling: Default::default(),
            force_self_target: Default::default(),
        }
    }
}

pub fn spell_action_system(
    mut cmd: Commands,
    time: Res<Time>,
    units: Query<(Entity, &Transform, Option<&Target>)>,
    mut dmg_event: EventWriter<DoDamageEvent>,
    mut teleport_event: EventWriter<Teleport>,
    mut query: Query<(&Actor, &mut ActionState, &mut CastSpell)>,
) {
    for (Actor(actor), mut state, mut spell) in query.iter_mut() {
        if let Ok((me, pos, target)) = units.get(*actor) {
            match *state {
                ActionState::Requested => {
                    cmd.entity(*actor).insert(Casting);
                    *state = ActionState::Executing;
                }
                ActionState::Executing => {
                    if !spell.cast_duration.tick(time.delta()).finished() {
                        // still casting
                        continue;
                    }

                    match &spell.cast_type {
                        SpellCastType::Damage { amount, mana } => dmg_event.send(DoDamageEvent {
                            origin: me,
                            receiver: target.map_or(me, |t| t.0),
                            damage: *amount,
                        }),
                        SpellCastType::Teleport { map, coordinates } => {
                            teleport_event.send(Teleport {
                                entity: me,
                                map: map.clone(),
                                map_instance: None,
                                prev_map_instance: None,
                                position: *coordinates,
                            })
                        }
                    }

                    *state = ActionState::Success;
                }
                ActionState::Cancelled => {
                    *state = ActionState::Failure;
                }
                _ => {}
            }
        }
    }
}
