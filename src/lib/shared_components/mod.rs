use bevy::prelude::Component;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct NPC;

#[derive(Component, Debug)]
pub struct Name(pub String);
