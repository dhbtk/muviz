use crate::app::gameplay::entities::MainScene;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::camera::Exposure;
use bevy::light::AtmosphereEnvironmentMapLight;
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections};
use bevy::post_process::bloom::{Bloom, BloomCompositeMode, BloomPrefilter};
use bevy::prelude::*;

pub fn spawn_camera(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    commands.spawn((
        MainScene,
        RailCamera {
            height: 9.0,
            distance: 30.0,
            look_ahead: 2.0,
            target_pos: Vec3::ZERO,
            target_looking_at: Vec3::ZERO,
            target_up: Vec3::Y,
            smoothing: 0.05,
        },
        Camera3d { ..default() },
        Transform::default(),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
        Exposure { ev100: 10.0 },
        Bloom {
            intensity: 0.15,
            low_frequency_boost: 0.7,
            low_frequency_boost_curvature: 0.95,
            high_pass_frequency: 1.0,
            prefilter: BloomPrefilter {
                threshold: 0.0,
                threshold_softness: 0.0,
            },
            composite_mode: BloomCompositeMode::EnergyConserving,
            max_mip_dimension: 1024,
            scale: Vec2::new(6.0, 1.5),
        },
        AtmosphereEnvironmentMapLight::default(),
        Msaa::Off,
        Fxaa::default(),
        ScreenSpaceReflections::default(),
    ));
}

#[derive(Component)]
pub struct RailCamera {
    pub height: f32,
    pub distance: f32,
    pub look_ahead: f32,
    pub target_pos: Vec3,
    pub target_looking_at: Vec3,
    pub target_up: Vec3,
    pub smoothing: f32,
}
