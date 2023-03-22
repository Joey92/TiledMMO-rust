mod game;
mod helpers;
mod network;

use bevy::{ prelude::*};
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
                watch_for_changes: true,
                asset_folder: String::from(""),
                ..default()
            }),
    )
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
    .add_plugin(game::GamePlugin)
    .add_plugin(NetworkPlugin)
    .add_startup_system(setup_graphics)
    .add_system(helpers::camera::movement)
    .add_system(bevy::window::close_on_esc);

    #[cfg(debug_assertions)]
    {
        app.register_type::<Threat>()
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(RapierDebugRenderPlugin::default());
    }

    app.run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2dBundle::default());
}
