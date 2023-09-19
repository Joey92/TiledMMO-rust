use std::time::Duration;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_spatial::{AutomaticUpdate, SpatialStructure};
use tiled_game::components::Unit;

pub mod combat;
pub mod interactions;
pub mod map;
pub mod npc;
pub mod player;
pub mod scripts;
pub mod unit;

use self::combat::CombatPlugin;
use self::interactions::InteractionPlugin;
use self::map::*;
use self::npc::NPCPlugin;
use self::player::*;
use self::scripts::ScriptsPlugin;
use self::unit::UnitPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            AutomaticUpdate::<Unit>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(Duration::from_millis(300)),
        ) // track everything that has the unit marker in a spatial index
        .add_plugins(UnitPlugin)
        .add_plugins(NPCPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MapsPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(ScriptsPlugin)
        .add_plugins(InteractionPlugin);
    }
}
