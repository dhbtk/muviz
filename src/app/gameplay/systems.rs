use crate::app::gameplay::components::{RailCamera, SongTrack};
use crate::app::gameplay::model::generate_track_mesh;
use crate::app::gameplay::CurrentSong;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;

pub fn spawn_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    data: Res<CurrentSong>,
) {
    let mesh = generate_track_mesh(&data.track_points, 18.0);
    commands.spawn((
        SongTrack,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.2, 0.25),
            perceptual_roughness: 0.9,
            ..default()
        })),
        Wireframe,
    ));

    commands.spawn((
        RailCamera {
            height: 15.0,
            distance: 50.0,
            look_ahead: 5.0,
            target_pos: Vec3::ZERO,
            target_looking_at: Vec3::ZERO,
            smoothing: 5.0,
        },
        Camera3d::default(),
        Transform::default(),
        DistanceFog {
            color: Color::srgba(0.35, 0.48, 0.66, 1.0),
            directional_light_color: Color::srgba(1.0, 0.95, 0.85, 0.5),
            directional_light_exponent: 30.0,
            falloff: FogFalloff::from_visibility_colors(
                15.0, // distance in world units up to which objects retain visibility (>= 5% contrast)
                Color::srgb(0.35, 0.5, 0.66), // atmospheric extinction color (after light is lost due to absorption by atmospheric particles)
                Color::srgb(0.8, 0.844, 1.0), // atmospheric inscattering color (light gained due to scattering from the sun)
            ),
        },
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
    ));

    commands.spawn(AudioPlayer(data.song_asset.clone()));
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

        // Smoothly update the camera position
        cam.target_pos = cam
            .target_pos
            .lerp(cam_pos, time.delta_secs() * cam.smoothing);
        transform.translation = cam.target_pos;

        // Smoothly update where the camera is looking
        cam.target_looking_at = cam
            .target_looking_at
            .lerp(target, time.delta_secs() * cam.smoothing);
        transform.look_at(cam.target_looking_at, current.up);
    }
}

pub fn update_playback(
    mut song: ResMut<CurrentSong>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if !keyboard_input.pressed(KeyCode::Space) {
        song.time_seconds += time.delta().as_secs_f32();
    }
}
