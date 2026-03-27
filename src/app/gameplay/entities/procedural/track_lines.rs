use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::track::mesh_generation;
use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::math::Vec2;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues};

pub fn generate_line_meshes(points: &[TrackPoint]) -> Vec<Mesh> {
    let mut line_meshes = Vec::new();
    let (left, right) = generate_edge_line_meshes(points);
    line_meshes.push(left);
    line_meshes.push(right);
    let lane_meshes = generate_lane_line_meshes(points, 0.3, &[3.0, -3.0], 20, 20);
    line_meshes.extend(lane_meshes);
    line_meshes
}

pub fn generate_edge_line_meshes(points: &[TrackPoint]) -> (Mesh, Mesh) {
    let left_track_shape = vec![Vec2::new(-8.6, 0.0), Vec2::new(-8.3, 0.0)];
    let right_track_shape = vec![Vec2::new(8.3, 0.0), Vec2::new(8.6, 0.0)];
    let (left_lengths, _) = CurrentSong::compute_arc_length(points);
    let (right_lengths, _) = CurrentSong::compute_arc_length(points);
    (
        mesh_generation::extrude_along_track(points, &left_track_shape, &left_lengths, Vec2::ONE),
        mesh_generation::extrude_along_track(points, &right_track_shape, &right_lengths, Vec2::ONE),
    )
}

pub fn generate_lane_line_meshes(
    points: &[TrackPoint],
    width: f32,
    x_offsets: &[f32],
    dash_length: usize,  // how many TrackPoints a dash spans
    dash_spacing: usize, // how many TrackPoints dashes are separated by
) -> Vec<Mesh> {
    if points.len() < 2 || width <= 0.0 || dash_length == 0 {
        return Vec::new();
    }

    let mut lane_meshes = Vec::with_capacity(x_offsets.len());
    let half_width = width * 0.5;

    for &x_offset in x_offsets {
        let lane_shape = vec![
            Vec2::new(x_offset - half_width, 0.0),
            Vec2::new(x_offset + half_width, 0.0),
        ];

        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let mut start = 0;
        while start < points.len() {
            let end = (start + dash_length).min(points.len());

            if end - start >= 2 {
                let dash_points = &points[start..end];
                let (lengths, _) = CurrentSong::compute_arc_length(dash_points);
                let dash_mesh =
                    mesh_generation::extrude_along_track(dash_points, &lane_shape, &lengths, Vec2::ONE);

                let Some(VertexAttributeValues::Float32x3(dash_positions)) =
                    dash_mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    start = start.saturating_add(dash_length + dash_spacing);
                    continue;
                };
                let Some(VertexAttributeValues::Float32x3(dash_normals)) =
                    dash_mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
                else {
                    start = start.saturating_add(dash_length + dash_spacing);
                    continue;
                };
                let Some(VertexAttributeValues::Float32x2(dash_uvs)) =
                    dash_mesh.attribute(Mesh::ATTRIBUTE_UV_0)
                else {
                    start = start.saturating_add(dash_length + dash_spacing);
                    continue;
                };

                let vertex_offset = positions.len() as u32;
                positions.extend(dash_positions.iter().copied());
                normals.extend(dash_normals.iter().copied());
                uvs.extend(dash_uvs.iter().copied());

                if let Some(mesh_indices) = dash_mesh.indices() {
                    match mesh_indices {
                        Indices::U16(dash_indices) => {
                            indices.extend(dash_indices.iter().map(|&i| i as u32 + vertex_offset));
                        }
                        Indices::U32(dash_indices) => {
                            indices.extend(dash_indices.iter().map(|&i| i + vertex_offset));
                        }
                    }
                }
            }

            start = start.saturating_add(dash_length + dash_spacing);
        }

        let mut lane_mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
        lane_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        lane_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        lane_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        lane_mesh.insert_indices(Indices::U32(indices));
        lane_meshes.push(lane_mesh);
    }

    lane_meshes
}
