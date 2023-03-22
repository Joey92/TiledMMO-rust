// everything that is a NPC

use bevy::prelude::*;
use bevy_spatial::SpatialAccess;
use tiled_game::components::*;

use super::{
    combat::{DoDamageEvent, LeaveCombatEvent},
    player::Player,
    unit::{Follow, MoveDestination, Speed, UnitBundle, UnitsNearby},
};

// A vector that represents the NPCs home position
// Usually the spawn point
#[derive(Component, Debug)]
pub struct Home(pub Vec3);

// friendly to players
#[derive(Component)]
pub struct Friendly;

#[derive(Component)]
pub struct NPC;

// A NPC that is currently evading combat to return to its home position
#[derive(Component)]
pub struct Evading;

#[derive(Bundle)]
pub struct NPCBundle {
    npc: NPC,
    unit: UnitBundle,
    home: Home,
}

impl NPCBundle {
    pub fn new(name: String, transform: Transform) -> Self {
        let mut npc = Self {
            npc: NPC,
            unit: UnitBundle::new(name, transform),
            home: Home(transform.translation),
        };

        npc.unit.speed = Speed(0.9);
        npc
    }
}

pub struct NPCPlugin;

impl Plugin for NPCPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(npc_evade_system)
            .add_system(return_to_home_system.in_base_set(CoreSet::PostUpdate))
            .add_system(
                npc_evaded
                    .in_base_set(CoreSet::PostUpdate)
                    .after(return_to_home_system),
            )
            .add_system(aggro_by_range_system)
            .add_system(aggro_by_damage_system)
            .add_system(target_and_follow_highest_threat);
    }
}

// Returns the NPC to its home position if it is not in combat anymore
fn return_to_home_system(
    mut cmd: Commands,
    npcs: Query<(Entity, &Home), With<NPC>>,
    mut out_of_combat: EventReader<LeaveCombatEvent>,
) {
    for evt in out_of_combat.iter() {
        if let Ok((npc_entity, spawn_point)) = npcs.get(evt.entity) {
            println!("NPC returning home: {:?}", spawn_point.0);
            cmd.entity(npc_entity)
                .remove::<Target>()
                .remove::<Follow>()
                .insert((Evading, MoveDestination(spawn_point.0)));
        }
    }
}

// Runs after the NPC stops evading
fn npc_evaded(mut out_of_combat: RemovedComponents<Evading>) {
    // no need to check if entity is a NPC since only NPCs can evade
    for entity in out_of_combat.iter() {
        println!("NPC returned home: {:?}", entity);
    }
}

// Check if the NPC has arrived at his home position
fn npc_evade_system(
    mut cmd: Commands,
    evading_npcs: Query<(Entity, &Transform, &Home), (With<NPC>, With<Evading>)>,
) {
    for (npc_entity, npc_transform, home) in evading_npcs.iter() {
        if npc_transform.translation.distance(home.0) < 1.0 {
            cmd.entity(npc_entity).remove::<Evading>();
        }
    }
}

// Aggro is the entry point for combat

// Aggros a NPC to the player if the player is in range
// Sets the target of NPCs that are not friendly to the player
// and sets the move destination to the target
fn aggro_by_range_system(
    mut cmd: Commands,
    // Who can be aggroed
    aggressors: Query<
        (Entity, &Transform, &Parent),
        (With<NPC>, Without<Friendly>, Without<InCombat>),
    >,

    // possible targets that can pull the aggressor
    entities_that_can_aggro: Query<(Entity, &Transform, &Parent), (With<Player>, Without<Dead>)>, // currently only players can aggro

                                                                                                  // todo: re-enable this when bevy_spatial is updated
                                                                                                  // units: Res<UnitsNearby>,
) {
    for (aggro_entity, aggro_transform, map_instance) in aggressors.iter() {
        let targets_in_range: Vec<_> = entities_that_can_aggro
            .iter()
            .filter(|(_, _, nearby_map_instance)| nearby_map_instance.get() == map_instance.get())
            .filter(|(_, target_transform, _)| {
                aggro_transform
                    .translation
                    .distance(target_transform.translation)
                    < 100.0
            })
            .collect();
        // // find all targets in range
        // let targets_in_range = units.within_distance(aggro_transform.translation, 100.0);

        // // if there are no targets, remove the aggro component
        if targets_in_range.is_empty() {
            // do nothing
            continue;
        }

        // find the first target that is allowed
        for (target, _, _) in targets_in_range {
            // let target = entities_that_can_aggro.get(*target_entity).ok();

            let mut threat = ThreatMap::new();
            threat.insert(target, 100);

            cmd.entity(aggro_entity)
                .insert(Target(target))
                .insert(Threat(threat))
                .insert(InCombat)
                .insert(Follow(target));
            break;
        }
    }
}

/**
 * Aggro a NPC if damage is done to it
 */
fn aggro_by_damage_system(
    mut cmd: Commands,
    mut damage_event: EventReader<DoDamageEvent>,
    npcs: Query<Entity, (With<NPC>, Without<InCombat>)>,
) {
    for evt in damage_event.iter() {
        // target should not have been in combat when damage occurred
        let npc = npcs.get(evt.receiver).ok();

        // If the target is not in combat, set it to combat
        if let Some(npc) = npc {
            let mut npc_commands = cmd.entity(npc);

            let mut threat = ThreatMap::new();
            threat.insert(evt.origin, 100 + evt.damage);

            npc_commands.insert((
                InCombat,
                Threat(threat),
                Follow(evt.origin),
                Target(evt.origin),
            ));
        }
    }
}

/**
 * Targets and follows the highest threat
 * If there is no threat, the NPC will stop targeting and following
 */
fn target_and_follow_highest_threat(
    mut cmd: Commands,
    npcs_in_combat: Query<(Entity, &Threat), (With<NPC>, With<InCombat>)>,
) {
    for (npc, threat) in npcs_in_combat.iter() {
        let mut highest_threat = 0;
        let mut highest_threat_entity = None;

        for (entity, threat) in threat.0.iter() {
            if *threat > highest_threat {
                highest_threat = *threat;
                highest_threat_entity = Some(*entity);
            }
        }

        if let Some(highest_threat_entity) = highest_threat_entity {
            cmd.entity(npc)
                .insert(Target(highest_threat_entity))
                .insert(Follow(highest_threat_entity));
            continue;
        }
    }
}
