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
