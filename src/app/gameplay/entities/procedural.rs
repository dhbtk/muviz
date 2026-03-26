pub mod track_lines;

use crate::app::assets::GlobalAssets;
use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::entities::MainScene;
use crate::app::gameplay::track::mesh_generation::extrude_along_track;
use crate::app::gameplay::track::procedural_meshes::{generate_track_mesh, generate_viaduct_mesh};
use crate::app::gameplay::track::track_generation::resample_track_equidistant_points;
use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use std::f32::consts::PI;
use track_lines::generate_line_meshes;

pub fn spawn_track(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    data: Res<CurrentSong>,
    assets: Res<GlobalAssets>,
) {
    let track_min_y = data.track_min_y();
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
            let mesh_list = generate_line_meshes(&resampled_distance_points);
            let line_material = materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.95, 0.95),
                perceptual_roughness: 0.6,
                ..default()
            });
            mesh_list.iter().for_each(|mesh| {
                builder.spawn((
                    SongTrackLine,
                    Mesh3d(meshes.add(mesh.clone())),
                    MeshMaterial3d(line_material.clone()),
                    Transform::from_xyz(0.0, 0.01, 0.0),
                ));
            })
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
    let cateye_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.0, 0.0),
        emissive: LinearRgba::rgb(1.0, 0.0, 0.0),
        perceptual_roughness: 0.1,
        ..default()
    });
    let cateye_mesh = meshes.add(Cuboid::from_size(Vec3::splat(0.18)));
    for (i, point) in resample_track_equidistant_points(&data.track_points, 40.0)
        .iter()
        .enumerate()
    {
        let offset = if i % 2 == 0 { 10.0 } else { -10.0 };
        commands
            .spawn((
                MainScene,
                SceneRoot(assets.streetlight_scene.clone()),
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
    resampled_distance_points
        .iter()
        .enumerate()
        .skip(30)
        .for_each(|(i, point)| {
            if (10 + i) % 40 == 0 {
                [3.0, -3.0].into_iter().for_each(|offset: f32| {
                    commands.spawn((
                        MainScene,
                        Cateye,
                        Mesh3d(cateye_mesh.clone()),
                        MeshMaterial3d(cateye_material.clone()),
                        Transform::from_translation(
                            point.position + point.right * offset + point.up * -0.06,
                        )
                        .with_rotation(point.rotation),
                    ));
                });
            }
            if (30 + i) % 40 == 0 {
                [8.45, -8.45].into_iter().for_each(|offset: f32| {
                    commands.spawn((
                        MainScene,
                        Cateye,
                        Mesh3d(cateye_mesh.clone()),
                        MeshMaterial3d(cateye_material.clone()),
                        Transform::from_translation(
                            point.position + point.right * offset + point.up * -0.06,
                        )
                        .with_rotation(point.rotation),
                    ));
                });
            }
        });
}

#[derive(Component)]
pub struct SongTrack;

#[derive(Component)]
pub struct Streetlight;

#[derive(Component)]
pub struct SongTrackLine;

#[derive(Component)]
pub struct Cateye;

pub fn update_track_line_emissive(
    song: Res<CurrentSong>,
    line_materials: Query<&MeshMaterial3d<StandardMaterial>, With<SongTrackLine>>,
    cateye_materials: Query<&MeshMaterial3d<StandardMaterial>, With<Cateye>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let frame = song.sample_gameplay_frame(song.current_frame_t());
    let energy = EasingCurve::new(0.0, 1.0, EaseFunction::SmootherStep)
        .sample_clamped(frame.energy.max(0.0));
    let light_intensity = energy * 1.0;
    for handle in &line_materials {
        if let Some(material) = materials.get_mut(&handle.0) {
            material.emissive = material.emissive.lerp(
                LinearRgba::rgb(light_intensity, light_intensity, light_intensity),
                0.1,
            );
        }
    }
    let beat_color = Color::hsl(energy * 360.0, frame.beat_strength.max(0.2), 0.5);
    let beat_intensity = frame.beat_strength * 150.0;
    for handle in &cateye_materials {
        if let Some(material) = materials.get_mut(&handle.0) {
            material.base_color = beat_color;
            material.emissive = material
                .emissive
                .lerp(beat_color.to_linear() * beat_intensity, 0.1);
        }
    }
}
