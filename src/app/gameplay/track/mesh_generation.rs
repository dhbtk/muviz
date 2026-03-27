use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::math::{Affine2, Quat, Vec2, Vec3};
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};

#[derive(Debug, Clone, Copy)]
pub struct TrackLinePoint {
    pub rotation: Quat,
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub transform: Affine2,
}

impl From<&TrackPoint> for TrackLinePoint {
    fn from(value: &TrackPoint) -> Self {
        TrackLinePoint {
            rotation: value.rotation,
            position: value.position,
            forward: value.forward,
            right: value.right,
            up: value.up,
            transform: Affine2::IDENTITY,
        }
    }
}

impl From<TrackPoint> for TrackLinePoint {
    fn from(value: TrackPoint) -> Self {
        (&value).into()
    }
}

pub fn extrude_along_track(
    frames: &[impl Into<TrackLinePoint> + Clone],
    shape: &[Vec2],
    cumulative_lengths: &[f32],
    uv_scale: Vec2,
) -> Mesh {
    let frame_points: Vec<TrackLinePoint> = frames.iter().cloned().map(Into::into).collect();
    let shape_len = shape.len();
    let frame_len = frame_points.len();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    if frame_len < 2 || shape_len < 2 {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
        mesh.insert_indices(Indices::U32(Vec::new()));
        return mesh;
    }

    let lengths = if cumulative_lengths.len() == frame_len {
        cumulative_lengths.to_vec()
    } else {
        let mut fallback = Vec::with_capacity(frame_len);
        let mut total = 0.0;
        fallback.push(0.0);
        for i in 1..frame_len {
            total += frame_points[i]
                .position
                .distance(frame_points[i - 1].position);
            fallback.push(total);
        }
        fallback
    };

    let mut profile_lengths = Vec::with_capacity(shape_len);
    let mut profile_total = 0.0;
    profile_lengths.push(0.0);
    for i in 1..shape_len {
        profile_total += shape[i].distance(shape[i - 1]);
        profile_lengths.push(profile_total);
    }

    let mut positions = Vec::with_capacity(frame_len * shape_len);
    let mut uvs = Vec::with_capacity(frame_len * shape_len);
    let mut indices = vec![];

    for (i, frame) in frame_points.iter().enumerate() {
        for (j, p) in shape.iter().enumerate() {
            let p = frame.transform.transform_vector2(*p);
            let world = frame.position + frame.right * p.x + frame.up * p.y;

            positions.push(world.to_array());
            uvs.push([lengths[i] * uv_scale.x, profile_lengths[j] * uv_scale.y]);
        }
    }

    // strip
    for i in 0..frame_len - 1 {
        for j in 0..shape_len - 1 {
            let a = (i * shape_len + j) as u32;
            let b = a + 1;
            let c = a + shape_len as u32;
            let d = c + 1;

            indices.extend_from_slice(&[a, b, c]);
            indices.extend_from_slice(&[b, d, c]);
        }
    }

    let mut normals = vec![Vec3::ZERO; positions.len()];
    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;

        let pa = Vec3::from_array(positions[a]);
        let pb = Vec3::from_array(positions[b]);
        let pc = Vec3::from_array(positions[c]);

        let face_normal = (pb - pa).cross(pc - pa);
        if face_normal.length_squared() > 0.0 {
            normals[a] += face_normal;
            normals[b] += face_normal;
            normals[c] += face_normal;
        }
    }

    let normals: Vec<[f32; 3]> = normals
        .into_iter()
        .map(|normal| {
            let normal = if normal.length_squared() > 0.0 {
                normal.normalize()
            } else {
                Vec3::Y
            };
            normal.to_array()
        })
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
