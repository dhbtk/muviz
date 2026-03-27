pub mod track_lines;

use crate::app::assets::GlobalAssets;
use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::entities::MainScene;
use crate::app::gameplay::track::mesh_generation::{extrude_along_track, TrackLinePoint};
use crate::app::gameplay::track::procedural_meshes::{
    generate_guard_rail_meshes, generate_track_mesh, generate_viaduct_mesh,
};
use crate::app::gameplay::track::track_generation::resample_track_equidistant_points;
use bevy::math::{Affine2, VectorSpace};
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
    let guardrail_material_low = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 0.0, 0.0),
        emissive: LinearRgba::rgb(0.0, 0.0, 0.0),
        perceptual_roughness: 0.1,
        ..default()
    });
    let guardrail_material_mid = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 0.0, 0.0),
        emissive: LinearRgba::rgb(0.0, 0.0, 0.0),
        perceptual_roughness: 0.1,
        ..default()
    });
    let guardrail_material_high = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.0, 0.0, 0.0),
        emissive: LinearRgba::rgb(0.0, 0.0, 0.0),
        perceptual_roughness: 0.1,
        ..default()
    });
    let (left_rails, right_rails) = generate_guard_rail_meshes(&resampled_distance_points);
    for shapes in [left_rails, right_rails].into_iter() {
        for (i, shape) in shapes.into_iter().enumerate() {
            if i % 2 == 0 {
                let material = [
                    &guardrail_material_low,
                    &guardrail_material_mid,
                    &guardrail_material_high,
                ][(i / 2) % 3];
                commands.spawn((
                    MainScene,
                    GuardRail(i / 2),
                    Mesh3d(meshes.add(shape)),
                    MeshMaterial3d(material.clone()),
                ));
            } else {
                commands.spawn((
                    MainScene,
                    Mesh3d(meshes.add(shape)),
                    MeshMaterial3d(viaduct_material.clone()),
                ));
            }
        }
    }
    let cateye_mesh = meshes.add(Cuboid::from_size(Vec3::splat(0.18)));
    for (i, point) in resample_track_equidistant_points(&data.track_points, 10.0)
        .iter()
        .enumerate()
    {
        if i % 4 != 0 {
            continue;
        }
        let offset = if i % 8 == 0 { 10.0 } else { -10.0 };
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
                        color: Color::WHITE,
                        radius: 0.5,
                        range: 70.0,
                        intensity: 10_000_000.0,
                        outer_angle: PI / 3.0,
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 14.0, -3.5)
                        .looking_at(Vec3::new(0.0, 0.0, -10.0), Vec3::Y),
                ));
            });
        let support_mesh_points: Vec<Vec2> = vec![
            Vec2::new(-7.5, -5.5),
            Vec2::new(-7.5, 5.5),
            Vec2::new(7.5, 5.5),
            Vec2::new(7.5, -5.5),
            Vec2::new(-7.5, -5.5),
        ];
        if point.is_above_other_track {
            continue;
        }
        if i % 8 != 0 {
            continue;
        }
        let mut support_height_points = Vec::new();
        let mut starting_y = 0.0;
        let required_length = point.position.distance(Vec3::new(
            point.position.x,
            track_min_y - 20.0,
            point.position.z,
        ));
        let mut i = 0;
        let transforms = vec![
            Affine2::from_scale(Vec2::new(1.5, 1.2)),
            Affine2::from_scale(Vec2::new(1.5, 1.2)),
        ];
        while starting_y > -required_length {
            let position = Vec3::new(0.0, starting_y, 0.0);
            let rotation = Quat::from_rotation_y(point.yaw) * Quat::from_rotation_x(PI / 2.0);
            support_height_points.push(TrackLinePoint {
                rotation,
                position,
                forward: (rotation * Vec3::Z).normalize(),
                right: (rotation * Vec3::X).normalize(),
                up: (rotation * -Vec3::Y).normalize(),
                transform: *transforms.get(i).unwrap_or(&Affine2::IDENTITY),
            });
            starting_y += -3.0;
            i += 1;
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

#[derive(Component)]
pub struct GuardRail(usize);

pub fn update_track_line_emissive(
    song: Res<CurrentSong>,
    line_materials: Query<&MeshMaterial3d<StandardMaterial>, With<SongTrackLine>>,
    cateye_materials: Query<&MeshMaterial3d<StandardMaterial>, With<Cateye>>,
    guardrail_materials: Query<(&GuardRail, &MeshMaterial3d<StandardMaterial>), With<GuardRail>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let curve = EasingCurve::new(0.0, 1.0, EaseFunction::SmootherStep);
    let frame = song.sample_gameplay_frame(song.current_frame_t());
    let energy = curve.sample_clamped(frame.energy.max(0.0));
    let light_intensity = energy * 1.0;
    for handle in &line_materials {
        if let Some(material) = materials.get_mut(&handle.0) {
            material.emissive = material.emissive.lerp(
                LinearRgba::rgb(light_intensity, light_intensity, light_intensity),
                0.1,
            );
        }
    }
    let beat_color = Color::hsl(energy * 360.0, 1.0, 0.2);
    let beat_intensity = frame.beat_strength * 250.0;
    for handle in &cateye_materials {
        if let Some(material) = materials.get_mut(&handle.0) {
            material.base_color = beat_color;
            material.emissive = material
                .emissive
                .lerp(beat_color.to_linear() * beat_intensity, 0.1);
        }
    }
    let bps = song.track_analysis.estimated_bpm.unwrap_or(120.0) / 60.;
    let beat_frac = (frame.time_s / bps).fract();
    let left_lane = curve.sample_clamped(frame.lane_left);
    let center_lane = curve.sample_clamped(frame.lane_center);
    let right_lane = curve.sample_clamped(frame.lane_right);
    for (index, handle) in &guardrail_materials {
        let (color, intensity) = match index.0 {
            0 => {
                let rail_color = Color::hsl(
                    (left_lane * 60.0 - 30.0 + (360.0 * beat_frac)) % 360.0,
                    1.0,
                    0.2,
                );
                let light_intensity = left_lane * 3.2;
                (rail_color, light_intensity)
            }
            1 => {
                let rail_color = Color::hsl(
                    (center_lane * 60.0 - 30.0 + (360.0 * beat_frac)) % 360.0,
                    0.7,
                    0.6,
                );
                let light_intensity = curve.sample_clamped(frame.lane_center) * 3.2;
                (rail_color, light_intensity)
            }
            2 => {
                let rail_color = Color::hsl(
                    (right_lane * 60.0 - 30.0 + (360.0 * beat_frac)) % 360.0,
                    0.7,
                    0.7,
                );
                let light_intensity = curve.sample_clamped(frame.lane_right) * 3.2;
                (rail_color, light_intensity)
            }
            _ => unreachable!(),
        };
        if let Some(material) = materials.get_mut(&handle.0) {
            material.base_color = color;
            material.emissive = material.emissive.lerp(color.to_linear() * intensity, 0.1);
        }
    }
}
