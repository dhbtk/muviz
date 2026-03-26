use crate::app::assets::GlobalAssets;
use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::entities::MainScene;
use crate::app::gameplay::track::mesh_generation::extrude_along_track;
use crate::app::gameplay::track::procedural_meshes::{
    generate_edge_line_meshes, generate_track_mesh, generate_viaduct_mesh,
};
use crate::app::gameplay::track::track_generation::resample_track_equidistant_points;
use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::prelude::*;
use std::f32::consts::PI;

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
    for (i, point) in resample_track_equidistant_points(&data.track_points, 40.0)
        .iter()
        .enumerate()
    {
        let offset = if i % 2 == 0 { 10.5 } else { -10.5 };
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
}

#[derive(Component)]
pub struct SongTrack;

#[derive(Component)]
pub struct Streetlight;
