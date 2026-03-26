use crate::app::gameplay::components::{MainScene, RailCamera, SongPlayer, SongTrack, Streetlight};
use crate::app::gameplay::model::{
    extrude_along_track, generate_edge_line_meshes, generate_track_mesh, generate_viaduct_mesh,
    resample_track_equidistant_points, TrackPoint,
};
use crate::app::gameplay::ocean::{spawn_water, Water};
use crate::app::gameplay::CurrentSong;
use crate::app::AppState;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::audio::PlaybackMode;
use bevy::camera::Exposure;
use bevy::light::AtmosphereEnvironmentMapLight;
use bevy::pbr::{
    Atmosphere, AtmosphereSettings, ExtendedMaterial, ScatteringMedium, ScreenSpaceReflections,
};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use rand::prelude::SmallRng;
use rand::{RngExt, SeedableRng};
use std::f32::consts::PI;

pub fn spawn_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    data: Res<CurrentSong>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = SmallRng::seed_from_u64(data.frames.len() as u64);
    let track_min_y = data
        .track_points
        .iter()
        .map(|p| p.position.y)
        .reduce(f32::min)
        .unwrap();
    let resampled_distance_points = resample_track_equidistant_points(&data.track_points, 1.0);
    let mesh = generate_track_mesh(&resampled_distance_points);
    commands
        .spawn((
            MainScene,
            SongTrack,
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.05, 0.05, 0.05),
                perceptual_roughness: 0.9,
                ..default()
            })),
        ))
        .with_children(|builder| {
            let (left, right) = generate_edge_line_meshes(&resampled_distance_points);
            builder.spawn((
                Mesh3d(meshes.add(left)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.95, 0.95, 0.95),
                    perceptual_roughness: 0.6,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.05, 0.0),
            ));
            builder.spawn((
                Mesh3d(meshes.add(right)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.95, 0.95, 0.95),
                    perceptual_roughness: 0.6,
                    ..default()
                })),
                Transform::from_xyz(0.0, 0.05, 0.0),
            ));
        });
    let viaduct_mesh = generate_viaduct_mesh(&resampled_distance_points);
    let viaduct_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.3),
        perceptual_roughness: 0.7,
        ..default()
    });
    commands.spawn((
        MainScene,
        SongTrack,
        Mesh3d(meshes.add(viaduct_mesh)),
        MeshMaterial3d(viaduct_material.clone()),
    ));
    let streetlight_mesh =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/streetlight.glb"));
    let debug_cube = meshes.add(Cuboid::new(1., 1., 1.0));
    let debug_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.5,
        ..default()
    });
    for (i, point) in resample_track_equidistant_points(&data.track_points, 40.0)
        .iter()
        .enumerate()
    {
        let offset = if i % 2 == 0 { 10.5 } else { -10.5 };
        commands
            .spawn((
                MainScene,
                SceneRoot(streetlight_mesh.clone()),
                Transform::from_translation(point.position + point.right * offset)
                    .looking_at(point.position, Vec3::Y),
            ))
            .with_children(|builder| {
                builder.spawn((
                    Streetlight,
                    SpotLight {
                        color: Color::srgb(1.0, 0.71, 0.29),
                        radius: 0.5,
                        range: 50.0,
                        intensity: 10_000_000.0,
                        outer_angle: PI / 2.0,
                        shadows_enabled: false,
                        ..default()
                    },
                    // Mesh3d(debug_cube.clone()),
                    // MeshMaterial3d(debug_material.clone()),
                    Transform::from_xyz(0.0, 14.0, -3.5)
                        .looking_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y),
                ));
            });
        let support_mesh_points: Vec<Vec2> = vec![
            Vec2::new(-2.5, -1.5),
            Vec2::new(-2.5, 1.5),
            Vec2::new(2.5, 1.5),
            Vec2::new(2.5, -1.5),
            Vec2::new(-2.5, -1.5),
        ];
        let mut support_height_points = Vec::new();
        let mut starting_y = 0.0;
        let required_length = point.position.distance(Vec3::new(
            point.position.x,
            track_min_y - 20.0,
            point.position.z,
        ));
        while starting_y > -required_length {
            let position = Vec3::new(0.0, starting_y, 0.0);
            let rotation = Quat::from_rotation_x(PI / 2.0);
            support_height_points.push(TrackPoint {
                rotation,
                position,
                forward: (rotation * Vec3::Z).normalize(),
                right: (rotation * Vec3::X).normalize(),
                up: Vec3::Y,
            });
            starting_y += -3.0;
        }
        let (lengths, _) = CurrentSong::compute_arc_length(&support_height_points);
        let pillar_mesh =
            extrude_along_track(&support_height_points, &support_mesh_points, &lengths);
        commands.spawn((
            MainScene,
            SongTrack,
            Mesh3d(meshes.add(pillar_mesh)),
            MeshMaterial3d(viaduct_material.clone()),
            Transform::from_translation(point.position - 3.0 * Vec3::Y),
        ));
    }

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
        // Can be adjusted to change the scene scale and rendering quality
        AtmosphereSettings::default(),
        // The directional light illuminance used in this scene
        // (the one recommended for use with this feature) is
        // quite bright, so raising the exposure compensation helps
        // bring the scene to a nicer brightness range.
        Exposure { ev100: 10.0 },
        // Bloom gives the sun a much more natural look.
        Bloom::OLD_SCHOOL,
        // Enables the atmosphere to drive reflections and ambient lighting (IBL) for this view
        AtmosphereEnvironmentMapLight::default(),
        // DistanceFog {
        //     color: Color::srgba(0.35, 0.48, 0.66, 1.0),
        //     directional_light_color: Color::srgba(1.0, 0.95, 0.85, 0.5),
        //     directional_light_exponent: 30.0,
        //     falloff: FogFalloff::from_visibility_colors(
        //         500.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
        //         Color::srgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
        //         Color::srgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
        //     ),
        // },
        Msaa::Off,
        Fxaa::default(),
        ScreenSpaceReflections::default(),
    ));

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

    spawn_water(
        &mut commands,
        &asset_server,
        &mut meshes,
        &mut water_materials,
        track_min_y,
    );

    commands.spawn((
        MainScene,
        SongPlayer,
        AudioPlayer(data.song_asset.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            paused: true,
            ..default()
        },
    ));
}

pub fn despawn_entities(mut commands: Commands, query: Query<Entity, With<MainScene>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn update_camera(
    song: Res<CurrentSong>,
    time: Res<Time>,
    mut query: Query<(&mut RailCamera, &mut Transform)>,
) {
    for (mut cam, mut transform) in query.iter_mut() {
        let t = song.current_frame_t();

        let current = song.sample_track_point(t);
        let ahead = song.sample_track_point(t + cam.look_ahead);

        let cam_pos = current.position - current.forward * cam.distance + current.up * cam.height;
        let target = ahead.position;

        if cam.target_pos == Vec3::ZERO {
            cam.target_pos = cam_pos;
        }
        if cam.target_looking_at == Vec3::ZERO {
            cam.target_looking_at = target;
        }
        if cam.target_up == Vec3::Y && current.up != Vec3::Y {
            cam.target_up = current.up;
        }

        let dt = time.delta_secs();
        // Smoothing factor (the fraction to move towards the target each frame, independent of framerate)
        let factor = 1.0 - cam.smoothing.powf(dt);

        // Smoothly update the camera position
        cam.target_pos = cam.target_pos.lerp(cam_pos, factor);
        transform.translation = cam.target_pos;

        // Smoothly update where the camera is looking
        cam.target_looking_at = cam.target_looking_at.lerp(target, factor);
        // Smoothly update the up vector
        cam.target_up = cam.target_up.lerp(current.up, factor).normalize();
        transform.look_at(cam.target_looking_at, cam.target_up);
    }
}

pub fn update_playback(
    mut commands: Commands,
    mut song: ResMut<CurrentSong>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    sink_query: Query<&AudioSink, With<SongPlayer>>,
) {
    let Ok(sink) = sink_query.single() else {
        return;
    };
    if keyboard_input.just_pressed(KeyCode::Space) {
        song.paused = !song.paused;
        if song.paused {
            sink.pause();
        } else {
            sink.play();
        }
    }
    if !song.paused {
        song.time_seconds += time.delta_secs();
    }
    if song.time_seconds > song.frames.last().unwrap().time_s
        || keyboard_input.just_pressed(KeyCode::Escape)
    {
        commands.set_state(AppState::FilePicker);
    }
}

pub fn update_streetlights(
    mut query: Query<(&GlobalTransform, &mut SpotLight), With<Streetlight>>,
    song: Res<CurrentSong>,
) {
    let curve = EasingCurve::new(0.0, 1.0, EaseFunction::ExponentialInOut);
    let player_coordinates = song.sample_track_point(song.current_frame_t()).position;
    // streetlights far from the player should be off.
    // streetlight intensity should be proportional to beat strength at that point times a falloff for lamps away from
    // the player.
    for (transform, mut light) in query.iter_mut() {
        let distance = player_coordinates.distance(transform.translation());
        let cutoff = 500.0;
        if distance > cutoff {
            light.intensity = 0.0;
            continue;
        }
        let falloff = curve.sample_clamped(1.0 - (distance - cutoff).clamp(0.0, cutoff) / cutoff);
        let features = song.nearest_frame(transform.translation());
        light.intensity = 50_000_000.0
            * (1.0 - falloff).clamp(0.2, 1.0)
            * curve.sample_clamped((features.beat_strength + features.lane_left))
            * 2.0;
    }
}
