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

    mesh_generation::extrude_along_track(points, &track_shape, &lengths, Vec2::ONE * 0.3)
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
    mesh_generation::extrude_along_track(points, &track_shape, &lengths, Vec2::ONE)
}

// shapes are, in order:
// 1. lower guard rail
// 2. mid spacer
// 3. mid guard rail
// 4. upper spacer
// 5. upper guard rail
// 6. back piece
pub fn generate_guard_rail_meshes(points: &[TrackPoint]) -> (Vec<Mesh>, Vec<Mesh>) {
    let shapes = [
        vec![
            Vec2::new(-9.0, 1.2),
            Vec2::new(-8.8, 1.2),
            Vec2::new(-8.8, 0.7),
            Vec2::new(-9.0, 0.7),
        ],
        vec![Vec2::new(-9.0, 1.5), Vec2::new(-9.0, 1.2)],
        vec![
            Vec2::new(-9.0, 2.0),
            Vec2::new(-8.8, 2.0),
            Vec2::new(-8.8, 1.5),
            Vec2::new(-9.0, 1.5),
        ],
        vec![Vec2::new(-9.0, 2.3), Vec2::new(-9.0, 2.0)],
        vec![
            Vec2::new(-9.0, 2.8),
            Vec2::new(-8.8, 2.8),
            Vec2::new(-8.8, 2.3),
            Vec2::new(-9.0, 2.3),
        ],
        vec![
            Vec2::new(-9.0, 0.7),
            Vec2::new(-9.5, 0.0),
            Vec2::new(-9.5, 3.1),
            Vec2::new(-9.0, 3.1),
            Vec2::new(-9.0, 2.8),
        ],
    ];
    let (lengths, _) = CurrentSong::compute_arc_length(points);
    let left_side = shapes
        .iter()
        .map(|shape| mesh_generation::extrude_along_track(points, shape, &lengths, Vec2::ONE))
        .collect();
    let right_side = shapes
        .iter()
        .map(|shape| {
            let mirrored_shape = shape
                .iter()
                .map(|point| point * Vec2::new(-1.0, 1.0))
                .rev()
                .collect::<Vec<_>>();
            mesh_generation::extrude_along_track(points, &mirrored_shape, &lengths, Vec2::ONE)
        })
        .collect();
    (left_side, right_side)
}
