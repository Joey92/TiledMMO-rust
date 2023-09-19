mod game;
mod network;

use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*, MinimalPlugins};

use game::GamePlugin;

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_millis(16))));

    app.add_plugins(network::NetworkPlugin)
        .add_plugins(GamePlugin);

    app.run();
}
