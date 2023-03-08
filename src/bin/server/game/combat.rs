/**
 * This file contains all the systems that are related to combat
 * Here we manage which entities are InCombat by measuring Threat
 *
 */
use bevy::{prelude::*, time::Time};

use tiled_game::components::*;

use super::{
    npc::{Evading, Home, NPC},
    unit::{AttackSpeed, Target, Unit},
};

const COMBAT_RANGE: f32 = 20.0;

pub struct DoDamageEvent {
    pub origin: Entity,
    pub receiver: Entity,
    pub damage: i32,
}

impl DoDamageEvent {
    pub fn new(origin: Entity, receiver: Entity, damage: i32) -> Self {
        Self {
            origin,
            receiver,
            damage,
        }
    }
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DoDamageEvent>()
            .add_event::<LeaveCombatEvent>()
            .add_system(auto_attack_system.before("damage"))
            .add_system(do_damage_system.label("damage"))
            .add_system(remove_unreachable_targets_from_threat.after("damage"))
            .add_system(remove_dead_from_threat.after("damage"))
            .add_system(increase_threat_on_damage.after("damage"))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                leave_combat.after("damage").label("combat"),
            );
    }
}

// auto attack system
// attacks the target
fn auto_attack_system(
    // Everyone in combat with a target that is not dead
    mut attackers: Query<
        (Entity, &Target, &Transform, &mut AttackSpeed),
        (With<InCombat>, Without<Dead>),
    >,

    // query for every unit so that we can query attackers target
    targets: Query<(Entity, &Transform), (With<Unit>, Without<Dead>)>,

    mut damage_event: EventWriter<DoDamageEvent>,
    time: Res<Time>,
) {
    for (attacker, target, position, mut attack_speed) in attackers.iter_mut() {
        let target = targets.get(target.0).ok();

        if let Some((enemy, t_position)) = target {
            if !attack_speed.0.tick(time.delta()).just_finished() {
                continue;
            }

            let distance = position.translation.distance(t_position.translation);

            if distance > COMBAT_RANGE {
                // to far away for attack
                continue;
            }

            let damage = 1;

            damage_event.send(DoDamageEvent::new(attacker, enemy, damage));
        }
    }
}

fn do_damage_system(
    mut cmd: Commands,
    mut damage_events: EventReader<DoDamageEvent>,

    targets: Query<(&Health, Option<&Evading>)>,
) {
    for evt in damage_events.iter() {
        let target_entity = targets.get(evt.receiver).ok();
        if let Some((health, evading)) = target_entity {
            if evading.is_some() {
                // send message to player that creature is evading
                // and is immune to all damage
                continue;
            }

            let mut target = cmd.entity(evt.receiver);

            println!("{:?} took {} damage", evt.receiver, evt.damage);

            target.insert(Health(health.0 - evt.damage));
        }
    }
}

pub struct LeaveCombatEvent {
    pub entity: Entity,
}

fn leave_combat(
    mut cmd: Commands,
    in_combat: Query<(Entity, &Threat), With<InCombat>>,
    death_events: Query<Entity, Added<Dead>>,
    mut events: EventWriter<LeaveCombatEvent>,
) {
    for (entity, threat) in in_combat.iter() {
        if threat.0.is_empty() {
            println!("Unit left combat: {:?}", entity);
            cmd.entity(entity).remove::<InCombat>().remove::<Threat>();
            events.send(LeaveCombatEvent { entity });
        }
    }

    // dead units should also leave combat
    for entity in death_events.iter() {
        println!("Unit left combat: {:?}", entity);
        cmd.entity(entity).remove::<InCombat>().remove::<Threat>();
        events.send(LeaveCombatEvent { entity });
    }
}

const MAX_DISTANCE: f32 = 300.;
/**
 * Removes units from threat if they are further away than HOME_ZONE
 * This is to prevent units from chasing the player forever
 * todo: Add more reasons to leave combat
 */
fn remove_unreachable_targets_from_threat(
    mut creatures: Query<(&Home, &Target, &Parent, &mut Threat), With<InCombat>>,
    targets: Query<(&Transform, &Parent), (With<Unit>, Without<Dead>)>,
) {
    for (home_zone, target, map_instance, mut threat) in creatures.iter_mut() {
        if let Ok((target_transform, target_map_instance)) = targets.get(target.0) {
            // remove combat if target is not in the same map instance anymore
            if map_instance.get() != target_map_instance.get() {
                println!(
                    "removing {:?} from threat because it is not in the same map instance",
                    target.0
                );
                threat.0.remove(&target.0);
                continue;
            }

            // remove combat if target is further away than HOME_ZONE
            let distance = target_transform.translation.distance(home_zone.0);

            if distance > MAX_DISTANCE {
                println!(
                    "removing {:?} from threat because it is too far away",
                    target.0
                );
                threat.0.remove(&target.0);
                continue;
            }
        }
    }
}

/**
 * Removes dead entities from the threat map
 */
fn remove_dead_from_threat(
    mut units_in_combat: Query<&mut Threat, With<InCombat>>,
    died: Query<Entity, Added<Dead>>,
) {
    for died_entity in died.iter() {
        for mut threat in units_in_combat.iter_mut() {
            if !threat.0.contains_key(&died_entity) {
                continue;
            }
            println!("removing {:?} from threat because it died", died_entity);

            threat.0.remove(&died_entity);
        }
    }
}

/**
* Increases threat on damage
* This should only be applied to creatures that are in combat
*/
fn increase_threat_on_damage(
    mut damage_events: EventReader<DoDamageEvent>,
    mut targets: Query<&mut Threat, (With<NPC>, With<InCombat>)>,
) {
    for evt in damage_events.iter() {
        let target_entity = targets.get_mut(evt.receiver).ok();

        if let Some(mut threat) = target_entity {
            threat.add(evt.origin, evt.damage);
        }
    }
}
