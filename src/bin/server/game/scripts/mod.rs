use bevy::{
    ecs::system::EntityCommands,
    prelude::{App, Plugin},
};

pub struct ScriptsPlugin;

impl Plugin for ScriptsPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn handle_add_script(_script_name: String, _cmd: &mut EntityCommands) {
    // match script_name.as_str() {
    //     "" => {
    //         cmd.insert(FollowerBundle::default());
    //         println!("Added follower script to entity")
    //     }
    //     _ => {
    //         println!("Unknown script: {}", script_name)
    //     }
    // }
}
