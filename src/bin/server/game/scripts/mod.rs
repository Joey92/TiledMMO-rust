use std::marker::PhantomData;

use bevy::{
    ecs::system::EntityCommands,
    prelude::{App, Component, Plugin},
};

use self::follower::follower_script;

mod follower;

#[derive(Debug, Component, Default)]
pub struct Script<T> {
    script: PhantomData<T>,
}

pub struct ScriptsPlugin;

impl Plugin for ScriptsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(follower_script);
    }
}

pub fn handle_add_script(script_name: String, cmd: &mut EntityCommands) {
    match script_name.as_str() {
        "follower" => {
            cmd.insert(Script::<follower::Follower>::default());
            println!("Added follower script to entity")
        }
        _ => {
            println!("Unknown script: {}", script_name)
        }
    }
}
