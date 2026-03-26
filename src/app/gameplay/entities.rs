use bevy::prelude::Component;

pub mod camera;
pub mod ocean;
pub mod procedural;
pub mod song_player;
pub mod sun;

#[derive(Component)]
pub struct MainScene;
