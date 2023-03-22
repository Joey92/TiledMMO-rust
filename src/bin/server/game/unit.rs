use bevy::core::Name;
/**
 * Everything that creatures and players have in common
 */
use bevy::prelude::*;
use bevy_spatial::KDTreeAccess2D;
use tiled_game::components::*;

use super::combat::DoDamageEvent;
use super::SystemLabels;

#[derive(Component, Default)]
pub struct Unit;

// A vector that the movement system will try to get to
#[derive(Component)]
pub struct MoveDestination(pub Vec3);

// Allows a unit to follow another unit
#[derive(Component)]
pub struct Follow(pub Entity);

// Speed on foot
#[derive(Component)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(1.0)
    }
}

// spatial index for fast lookup of nearby entities
// All units with the Name component are tracked by the spatial index
pub type UnitsNearby = KDTreeAccess2D<Unit>;

#[derive(Component)]
pub struct Faction(pub String);

#[derive(Component, Debug)]
pub struct AttackSpeed(pub Timer);

#[derive(Bundle)]
pub struct UnitBundle {
    pub unit: Unit, // marker component
    pub name: Name,
    pub speed: Speed,
    pub health: Health,
    pub max_health: MaxHealth,
    pub mana: Mana,
    pub max_mana: MaxMana,
    pub attack_speed: AttackSpeed,
    pub spatial: SpatialBundle,
}

impl UnitBundle {
    pub fn new(name: String, transform: Transform) -> Self {
        Self {
            name: Name::new(name),
            speed: Speed(1.0),
            health: Health(5),
            max_health: MaxHealth(5),
            mana: Mana(5),
            max_mana: MaxMana(5),
            attack_speed: AttackSpeed(Timer::from_seconds(1., TimerMode::Repeating)),
            spatial: SpatialBundle {
                transform,
                ..Default::default()
            },
            unit: Unit,
        }
    }
}

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        // Updates movement of entities
        app.add_system(movement_system)
            .add_system(follow_system)
            .insert_resource(HealthRecoveryTimer(Timer::from_seconds(
                2.,
                TimerMode::Repeating,
            )))
            .add_system(health_recovery_system)
            .add_event::<DeathEvent>()
            .add_system(death_system)
            .add_system(target_on_damage.after(SystemLabels::Damage));
    }
}

// Moves entities towards their destination
fn movement_system(
    mut commands: Commands,
    mut movements: Query<(Entity, &mut Transform, &MoveDestination, &Speed)>,
) {
    for (entity, mut transform, destination, speed) in movements.iter_mut() {
        let distance = transform.translation.distance(destination.0);
        let direction = (destination.0 - transform.translation).normalize();

        if distance > 1. {
            transform.translation += direction * speed.0;
        } else {
            // reached our destination
            commands.entity(entity).remove::<MoveDestination>();
        }
    }
}

// Follows entities
// If the entity to follow is not found, the follow component is removed
// continuously sets the MoveDestination of the entity
fn follow_system(
    mut commands: Commands,
    follower: Query<(Entity, &Follow)>,
    entities: Query<&Transform, With<Unit>>,
) {
    for (entity, follow) in follower.iter() {
        match entities.get(follow.0) {
            Ok(target_transform) => {
                commands
                    .entity(entity)
                    .insert(MoveDestination(target_transform.translation));
            }
            Err(_) => {
                commands.entity(entity).remove::<Follow>();
            }
        }
    }
}

#[derive(Resource)]
pub struct HealthRecoveryTimer(pub Timer);

// Slowly recovers health of units
// if they are out of combat
fn health_recovery_system(
    mut healths: Query<(&mut Health, &MaxHealth), (Without<Dead>, Without<InCombat>)>,
    mut recovery_timer: ResMut<HealthRecoveryTimer>,
    time: Res<Time>,
) {
    if !recovery_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for (mut health, max_health) in healths.iter_mut() {
        if health.0 >= max_health.0 {
            continue;
        }

        health.0 += 1;
    }
}

pub struct DeathEvent {
    pub entity: Entity,
}

fn death_system(
    mut commands: Commands,
    mut events: EventWriter<DeathEvent>,
    healths: Query<(Entity, &Health), Without<Dead>>,
) {
    for (entity, health) in healths.iter() {
        if health.0 <= 0 {
            println!("{:?} died", entity);
            commands.entity(entity).insert(Dead);
            events.send(DeathEvent { entity });
        }
    }
}

/**
 * Automatically target the attack if no target is set
 */
fn target_on_damage(
    mut cmd: Commands,
    mut damage_event: EventReader<DoDamageEvent>,
    unsuspecting_unit: Query<Entity, (With<Unit>, Without<Target>)>,
) {
    for evt in damage_event.iter() {
        // target should not have been in combat when damage occurred
        let unit = unsuspecting_unit.get(evt.receiver).ok();

        // Set the target
        if let Some(npc) = unit {
            cmd.entity(npc).insert(Target(evt.origin));
        }
    }
}
