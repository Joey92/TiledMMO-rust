mod game;
mod helpers;
mod network;

use bevy::{log, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use bevy_renet::{renet::RenetClient, run_if_client_connected};

use network::NetworkPlugin;
use tiled_game::components::Threat;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Connecting,
    Running,
}

fn main() {
    App::new()
        // .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(120)))
        .register_type::<Threat>()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: String::from("Tiled Map Editor Example"),
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: true,
                    asset_folder: String::from(""),
                    ..default()
                }),
        )
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(game::GamePlugin)
        .add_plugin(NetworkPlugin)
        .add_state(AppState::Connecting)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(run_if_client_connected)
                .with_system(set_running_state),
        )
        .add_startup_system(setup_graphics)
        .add_system(helpers::camera::movement)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn set_running_state(mut state: ResMut<State<AppState>>, client: Res<RenetClient>) {
    if state.current() == &AppState::Running {
        return;
    }
    log::info!("Connected to server, client ID is {}", client.client_id());
    state.set(AppState::Running).unwrap();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2dBundle::default());
}
