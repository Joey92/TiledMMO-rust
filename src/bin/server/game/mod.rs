use bevy::prelude::default;
use bevy::prelude::{App, Plugin};
use bevy_spatial::KDTreePlugin2D;
pub mod combat;
pub mod map;
pub mod npc;
pub mod player;
pub mod scripts;
pub mod unit;

use self::combat::CombatPlugin;
use self::map::*;
use self::npc::NPCPlugin;
use self::player::*;
use self::scripts::ScriptsPlugin;
use self::unit::{Unit, UnitPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(KDTreePlugin2D::<Unit> { ..default() }) // track everything that has the unit marker in a spatial index
            .add_plugin(UnitPlugin)
            .add_plugin(NPCPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(MapsPlugin)
            .add_plugin(CombatPlugin)
            .add_plugin(ScriptsPlugin);
    }
}
