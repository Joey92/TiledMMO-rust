use bevy::prelude::*;
use tiled_game::{
    calc_z_pos,
    components::*,
    network::messages::server::{ServerMessages, Vitals},
};

use crate::game::{
    combat::LeaveCombatEvent,
    map::DespawnEvent,
    npc::{Friendly, NPC},
    player::{Charmed, Player},
    unit::{DeathEvent, Unit},
};

use super::{NetworkClientId, SendServerMessageEvent};

pub fn send_threat(
    threats: Query<(Entity, &Threat, &Parent), (With<NPC>, Changed<Threat>)>,
    mut server_message: EventWriter<SendServerMessageEvent>,
    players: Query<(&NetworkClientId, &Parent), With<Player>>,
) {
    for (entity, threat, creature_map_instance) in threats.iter() {
        for (client_id, player_map_instance) in players.iter() {
            if player_map_instance.get() != creature_map_instance.get() {
                continue;
            }

            server_message.send(SendServerMessageEvent {
                client_id: Some(client_id.0),
                message: ServerMessages::Threat {
                    entity,
                    threat: threat.0.clone(),
                },
            });
        }
    }
}

pub fn filter_players_on_map_instance<'a>(player: &'a Parent) -> impl FnMut(&'a Parent) -> bool {
    |unit: &'a Parent| unit.get() == player.get()
}

pub fn send_exit_combat(
    mut server_message: EventWriter<SendServerMessageEvent>,
    mut exit_combat: EventReader<LeaveCombatEvent>,
    units: Query<&Parent, With<Unit>>,
    players: Query<(&NetworkClientId, &Parent), With<Player>>,
) {
    for leave_combat_event in exit_combat.iter() {
        let unit_exited_combat = units.get(leave_combat_event.entity).ok();

        if unit_exited_combat.is_none() {
            continue;
        }

        let unit = unit_exited_combat.unwrap();

        let clients_to_notify = players
            .iter()
            .filter(|p| filter_players_on_map_instance(unit)(p.1))
            .map(|p| p.0);

        clients_to_notify.for_each(|client| {
            server_message.send(SendServerMessageEvent {
                client_id: Some(client.0),
                message: ServerMessages::CombatState {
                    entity: leave_combat_event.entity,
                    in_combat: false,
                },
            });
        });
    }
}

pub fn send_entered_combat(
    mut server_message: EventWriter<SendServerMessageEvent>,
    entered_combat: Query<(Entity, &Parent), Added<InCombat>>,
    players: Query<(&NetworkClientId, &Parent), With<Player>>,
) {
    for (entity, map_instance) in entered_combat.iter() {
        let clients_to_notify = players
            .iter()
            .filter(|p| filter_players_on_map_instance(map_instance)(p.1))
            .map(|p| p.0);

        clients_to_notify.for_each(|client| {
            server_message.send(SendServerMessageEvent {
                client_id: Some(client.0),
                message: ServerMessages::CombatState {
                    entity,
                    in_combat: true,
                },
            });
        });
    }
}

pub fn send_spawn(
    spawns: Query<(Entity, &Parent), Or<(Added<Parent>, Changed<Parent>)>>,
    players: Query<(Entity, &NetworkClientId, &Parent), With<Player>>,
    mut server_message: EventWriter<SendServerMessageEvent>,
) {
    for (entity, map_instance) in spawns.iter() {
        for (player, client_id, player_map_instance) in players.iter() {
            // Don't send spawn to the player that spawned
            if player == entity {
                continue;
            }

            // Don't send spawn to players on a different map instance
            if player_map_instance.get() != map_instance.get() {
                continue;
            }

            server_message.send(SendServerMessageEvent {
                client_id: Some(client_id.0),
                message: ServerMessages::Spawn { entity },
            });
        }
    }
}

// Sends a despawn message to each player on the map
// When a player or entity leaves the map
pub fn send_despawn(
    mut server_message: EventWriter<SendServerMessageEvent>,
    mut despawn_events: EventReader<DespawnEvent>,
    all_players: Query<(Entity, &NetworkClientId, &Parent), With<Player>>,
) {
    for despawn in despawn_events.iter() {
        // get players on the map
        all_players
            .iter()
            .filter(|(_, _, map_instance)| map_instance.get() == despawn.map)
            .for_each(|(_, client_id, _)| {
                // send despawn event to all players in map
                server_message.send(SendServerMessageEvent {
                    client_id: Some(client_id.0),
                    message: ServerMessages::Despawn {
                        entity: despawn.entity,
                    },
                });
            });
    }
}

// Send entity movements to relevant players
pub fn send_movement(
    mut server_messages: EventWriter<SendServerMessageEvent>,
    moved_entities: Query<(Entity, &Transform, &Parent), Changed<Transform>>,
    players: Query<(Entity, &Parent, &NetworkClientId, Option<&Charmed>)>,
) {
    for (moved_entity, transform, map_instance) in moved_entities.iter() {
        for (player_entity, player_map_instance, client_id, charmed) in players.iter() {
            // If player and entity parents match, they are on the same map instance
            if player_map_instance.get() == map_instance.get()
            // Don't send movement to the entity that moved
            // unless it is charmed
            && (player_entity != moved_entity || charmed.is_some())
            {
                server_messages.send(SendServerMessageEvent {
                    client_id: Some(client_id.0),
                    message: ServerMessages::Move {
                        entity: moved_entity,
                        pos: transform.translation,
                    },
                });
            }
        }
    }
}

pub struct SendEntityInfoEvent {
    pub client_id: u64,
    pub entity: Entity,
}

pub fn send_entity_info(
    mut events: EventReader<SendEntityInfoEvent>,
    mut server_message: EventWriter<SendServerMessageEvent>,
    entities: Query<(
        &Name,
        &Transform,
        &Health,
        &MaxHealth,
        &Mana,
        &MaxMana,
        Option<&Player>,
        Option<&Friendly>,
        Option<&Threat>,
    )>,
) {
    for event in events.iter() {
        let entity = event.entity;
        let entity_ref = entities.get(entity).ok();

        let event = match entity_ref {
            Some((name, transform, health, max_health, mana, max_mana, player, friend, threat)) => {
                SendServerMessageEvent {
                    client_id: Some(event.client_id),
                    message: ServerMessages::EntityInfo {
                        entity,
                        pos: transform.translation,
                        name: name.to_string(),
                        is_player: player.is_some(),
                        friendly: friend.is_some(),
                        health: health.0,
                        max_health: max_health.0,
                        mana: mana.0,
                        max_mana: max_mana.0,
                        threat: threat.map(|t| Some(t.0.clone())).unwrap_or(None),
                    },
                }
            }
            _ => {
                // entity doesn't exist.. send a despawn message
                SendServerMessageEvent {
                    client_id: Some(event.client_id),
                    message: ServerMessages::Despawn { entity },
                }
            }
        };

        server_message.send(event);
    }
}

pub fn send_death_events(
    mut server_messages: EventWriter<SendServerMessageEvent>,
    mut death_events: EventReader<DeathEvent>,
) {
    for death_event in death_events.iter() {
        server_messages.send(SendServerMessageEvent {
            client_id: None,
            message: ServerMessages::Vitals {
                entity: death_event.entity,
                vital: Vitals::Dead(true),
            },
        });
    }
}

// Send entity health, mana, etc to relevant players
pub fn send_vitals_changed(
    mut server_messages: EventWriter<SendServerMessageEvent>,
    vitals_changed: Query<
        (
            Entity,
            &Parent,
            &Health,
            ChangeTrackers<Health>,
            &Mana,
            ChangeTrackers<Mana>,
        ),
        Or<(Changed<Health>, Changed<Mana>)>,
    >,
    players: Query<(&Parent, &NetworkClientId)>,
) {
    for (entity, map_instance, health, health_tracker, mana, mana_tracker) in vitals_changed.iter()
    {
        for (player_map_instance, client_id) in players.iter() {
            // If player and entity parents match, they are on the same map instance
            if player_map_instance.get() == map_instance.get() {
                if health_tracker.is_changed() {
                    server_messages.send(SendServerMessageEvent {
                        client_id: Some(client_id.0),
                        message: ServerMessages::Vitals {
                            entity,
                            vital: Vitals::Health(health.0),
                        },
                    });
                }

                if mana_tracker.is_changed() {
                    server_messages.send(SendServerMessageEvent {
                        client_id: Some(client_id.0),
                        message: ServerMessages::Vitals {
                            entity,
                            vital: Vitals::Mana(mana.0),
                        },
                    });
                }
            }
        }
    }
}
