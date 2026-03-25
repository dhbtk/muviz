use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct TrackPoint {
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

pub fn generate_track_points(
    analysis: &TrackAnalysis,
    frames: &[GameplayFrame],
) -> Vec<TrackPoint> {
    let mut points: Vec<TrackPoint> = Vec::with_capacity(frames.len());
    let bps = analysis.estimated_bpm.unwrap_or(120.0) / 60.;

    let z_step = 5.0;
    let height_scale = 0.06;
    let curve_scale = 0.023;
    let curve_decay = 0.001;
    let y_decay = 0.01;

    let mut rotation = Quat::IDENTITY;
    let mut position = Vec3::ZERO;

    for frame in frames {
        let mut x_delta = (frame.lane_left + frame.lane_right) * curve_scale * frame.beat_strength;
        if ((frame.time_s / bps) % 16.0) as i32 % 2 == 0 {
            x_delta = -x_delta;
        }
        if x_delta > 0. {
            x_delta = 0.0_f32.max(x_delta - curve_decay).min(0.15);
        } else if x_delta < 0. {
            x_delta = 0.0_f32.min(x_delta + curve_decay).max(-0.15);
        }
        let mut y_delta = -(frame.energy * 0.3 + frame.lane_center * 0.7) * height_scale
            + height_scale * 0.5
            - (frame.beat_strength * frame.energy * -0.02);
        if y_delta > 0. {
            y_delta = 0.0_f32.max(y_delta - y_decay).min(0.1);
        } else if y_delta < 0. {
            y_delta = 0.0_f32.min(y_delta + y_decay).max(-0.1);
        }
        rotation = rotation.lerp(rotation * Quat::from_rotation_y(x_delta), 0.5)
            * Quat::from_rotation_z(y_delta);
        let forward = rotation * Vec3::Z;
        position += forward
            * (z_step * ((frame.beat_strength.max(0.2) * 0.3) + (frame.energy.max(0.2) * 0.7)));

        let forward = rotation * Vec3::Z;
        let right = rotation * Vec3::X;
        let up = rotation * Vec3::Y;

        points.push(TrackPoint {
            position,
            forward,
            right,
            up,
        });
    }

    for i in 0..points.len() {
        let forward = if i < points.len() - 1 {
            (points[i + 1].position - points[i].position).normalize()
        } else {
            (points[i].position - points[i - 1].position).normalize()
        };

        let right = forward.cross(Vec3::Y).normalize();
        let up = right.cross(forward).normalize();

        points[i].forward = forward;
        points[i].right = right;
        points[i].up = up;
    }

    points
}

pub fn smooth_positions(points: &mut [TrackPoint], strength: f32) {
    for i in 1..points.len() - 1 {
        let prev = points[i - 1].position;
        let next = points[i + 1].position;

        let avg = (prev + next) * 0.5;
        points[i].position = points[i].position.lerp(avg, strength);
    }
}

pub fn generate_track_mesh(points: &[TrackPoint], width: f32) -> Mesh {
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let mut indices = Vec::<u32>::new();

    for (i, p) in points.iter().enumerate() {
        let left = p.position - p.right * width * 0.5;
        let right = p.position + p.right * width * 0.5;

        positions.push(left.into());
        positions.push(right.into());

        normals.push(p.up.into());
        normals.push(p.up.into());

        let v = i as f32 / points.len() as f32;

        uvs.push([0.0, v]);
        uvs.push([1.0, v]);
    }

    for i in 0..points.len() - 1 {
        let base = (i * 2) as u32;

        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);

        indices.push(base + 1);
        indices.push(base + 3);
        indices.push(base + 2);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
