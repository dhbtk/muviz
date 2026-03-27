use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::track::mesh_generation;
use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::math::Vec2;
use bevy::mesh::Mesh;

pub fn generate_track_mesh(points: &[TrackPoint]) -> Mesh {
    let track_shape = vec![
        Vec2::new(-9.0, 0.0),
        Vec2::new(-3.0, 0.0),
        Vec2::new(3.0, 0.0),
        Vec2::new(9.0, 0.0),
        Vec2::new(9.0, -0.5),
        Vec2::new(0.0, -0.5),
        Vec2::new(-9.0, -0.5),
        Vec2::new(-9.0, 0.0),
    ];
    let (lengths, _) = CurrentSong::compute_arc_length(points);

    mesh_generation::extrude_along_track(points, &track_shape, &lengths)
}

pub fn generate_viaduct_mesh(points: &[TrackPoint]) -> Mesh {
    let track_shape = vec![
        Vec2::new(9.0, 0.0),
        Vec2::new(9.0, 0.5),
        Vec2::new(12.0, 0.0),
        Vec2::new(12.0, 0.5),
        Vec2::new(12.0, -4.0),
        Vec2::new(0.0, -6.0),
        Vec2::new(-12.0, -4.0),
        Vec2::new(-12.0, 0.5),
        Vec2::new(-9.0, 0.5),
        Vec2::new(-9.0, 0.0),
    ];
    let (lengths, _) = CurrentSong::compute_arc_length(points);
    mesh_generation::extrude_along_track(points, &track_shape, &lengths)
}

pub fn generate_guard_rail_meshes(points: &[TrackPoint]) -> Vec<Mesh> {
    let shapes = vec![
        vec![
            Vec2::new(-9.0, 0.7),
            Vec2::new(-9.9, 0.7),
            Vec2::new(-9.9, 1.2),
            Vec2::new(-9.0, 1.2),
            Vec2::new(-9.0, 0.7),
        ],
        vec![
            Vec2::new(9.9, 0.7),
            Vec2::new(9.0, 0.7),
            Vec2::new(9.0, 1.2),
            Vec2::new(9.9, 1.2),
            Vec2::new(9.9, 0.7),
        ],
    ];
    let (lengths, _) = CurrentSong::compute_arc_length(points);
    shapes
        .iter()
        .map(|shape| mesh_generation::extrude_along_track(points, shape, &lengths))
        .collect()
}
