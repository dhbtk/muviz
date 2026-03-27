use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::math::Vec2;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};

pub fn extrude_along_track(
    frames: &[TrackPoint],
    shape: &[Vec2],
    cumulative_lengths: &[f32],
) -> Mesh {
    let mut positions = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];

    let shape_len = shape.len();
    let total_length = cumulative_lengths.last().copied().unwrap_or(1.0);

    let min_x = shape.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = shape.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);

    for (i, frame) in frames.iter().enumerate() {
        let v = cumulative_lengths[i] / total_length;

        for p in shape {
            let world = frame.position + frame.right * p.x + frame.up * p.y;

            positions.push(world.to_array());

            normals.push(frame.up.to_array());

            let u = (p.x - min_x) / (max_x - min_x);

            uvs.push([u, v]);
        }
    }

    // strip
    for i in 0..frames.len() - 1 {
        for j in 0..shape_len - 1 {
            let a = (i * shape_len + j) as u32;
            let b = a + 1;
            let c = a + shape_len as u32;
            let d = c + 1;

            indices.extend_from_slice(&[a, b, c]);
            indices.extend_from_slice(&[b, d, c]);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
