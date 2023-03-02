mod game;
mod network;

use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, prelude::App, MinimalPlugins};

use game::GamePlugin;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(16)))
        .add_plugins(MinimalPlugins)
        .add_plugin(network::NetworkPlugin)
        .add_plugin(GamePlugin)
        .run();
}
