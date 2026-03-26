use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::entities::MainScene;
use bevy::prelude::*;
use rand::prelude::SmallRng;
use rand::{RngExt, SeedableRng};

pub fn spawn_sun(mut commands: Commands, data: Res<CurrentSong>) {
    let mut rng = SmallRng::seed_from_u64(data.frames.len() as u64);
    commands.spawn((
        MainScene,
        DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            illuminance: 600.0,
            ..default()
        },
        Transform::from_xyz(0.0, rng.random_range(1.0..500.0), 20_000.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
