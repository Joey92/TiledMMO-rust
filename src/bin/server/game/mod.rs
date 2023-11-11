use std::time::Duration;

use bevy::prelude::*;
use bevy_spatial::kdtree::KDTree2;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use big_brain::BigBrainPlugin;
use tiled_game::components::Unit;

pub mod actions;
pub mod combat;
pub mod interactions;
pub mod map;
pub mod npc;
pub mod player;
pub mod scorers;

pub mod unit;
mod thinkers;

use crate::network::NetworkClientId;

use self::actions::ActionsPlugin;
use self::combat::CombatPlugin;
use self::interactions::InteractionPlugin;
use self::map::*;
use self::npc::NPCPlugin;
use self::player::*;

use self::scorers::ScorerPlugin;
use self::unit::UnitPlugin;

pub struct GamePlugin;

// spatial index for fast lookup of nearby entities
// All units with the Name component are tracked by the spatial index
pub type UnitsNearby = KDTree2<Unit>; // this includes clients
pub type ClientsNearby = KDTree2<NetworkClientId>;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BigBrainPlugin::new(PreUpdate))
            // track everything that has the unit marker in a spatial index
            .add_plugins(
                AutomaticUpdate::<Unit>::new()
                    .with_spatial_ds(SpatialStructure::KDTree2)
                    .with_transform(bevy_spatial::TransformMode::GlobalTransform)
                    .with_frequency(Duration::from_millis(300)),
            )
            // all clients
            .add_plugins(
                AutomaticUpdate::<NetworkClientId>::new()
                    .with_spatial_ds(SpatialStructure::KDTree2)
                    .with_transform(bevy_spatial::TransformMode::GlobalTransform)
                    .with_frequency(Duration::from_secs_f32(1.)),
            )
            .add_plugins(UnitPlugin)
            .add_plugins(NPCPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(MapsPlugin)
            .add_plugins(CombatPlugin)
            .add_plugins(InteractionPlugin)
            .add_plugins(ScorerPlugin)
            .add_plugins(ActionsPlugin);
    }
}
