mod game;
mod helpers;
mod network;

use std::time::Duration;

use bevy::{asset::ChangeWatcher, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

use network::NetworkPlugin;
use tiled_game::components::Threat;

fn main() {
    let mut app = App::new();
    // .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(120)))
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                ..default()
            }),
    )
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
    .add_plugins(game::GamePlugin)
    .add_plugins(NetworkPlugin)
    .add_systems(Startup, setup_graphics)
    .add_systems(
        Update,
        (helpers::camera::movement, bevy::window::close_on_esc),
    );

    #[cfg(debug_assertions)]
    {
        app.register_type::<Threat>()
            .add_plugins(WorldInspectorPlugin::new())
            .add_plugins(RapierDebugRenderPlugin::default());
    }

    app.run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2dBundle::default());
}
