use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct TrackPoint {
    pub rotation: Quat,
    pub position: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl TrackPoint {
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            rotation: self.rotation.slerp(other.rotation, t),
            position: self.position.lerp(other.position, t),
            forward: self.forward.lerp(other.forward, t),
            right: self.right.lerp(other.right, t),
            up: self.up.lerp(other.up, t),
        }
    }
}

pub fn generate_track_points(
    analysis: &TrackAnalysis,
    frames: &[GameplayFrame],
) -> Vec<TrackPoint> {
    let mut points: Vec<TrackPoint> = Vec::with_capacity(frames.len());
    let bps = analysis.estimated_bpm.unwrap_or(120.0) / 60.;

    let z_step = 9.0;
    let height_scale = 0.06;
    let curve_scale = 0.033;
    let yaw_delta_decay = 0.0001;
    let pitch_delta_decay = 0.012;
    let pitch_recentering_force = 0.0002;
    let pitch_limit = PI / 6.;
    let roll_limit = PI / 12.;
    let damping = 0.99;
    let springiness = 0.01;

    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let mut roll = 0.0;
    let mut rotation = Quat::IDENTITY;
    let mut position = Vec3::ZERO;

    for frame in frames {
        let mut yaw_delta =
            (frame.lane_left - frame.lane_right) * curve_scale * (0.5 + frame.beat_strength * 0.5);
        if ((frame.time_s / bps) % 16.0) as i32 % 2 == 0 {
            yaw_delta = -yaw_delta;
        }
        if yaw_delta > 0. {
            yaw_delta = 0.0_f32.max(yaw_delta - yaw_delta_decay).min(0.15);
        } else if yaw_delta < 0. {
            yaw_delta = 0.0_f32.min(yaw_delta + yaw_delta_decay).max(-0.15);
        }
        let mut pitch_delta = (frame.energy * 0.3 + frame.lane_center * 0.7) * height_scale;
        // - (frame.beat_strength * frame.energy * -0.02);
        if ((frame.time_s / bps) % 12.0) as i32 % 2 == 0 {
            pitch_delta = -pitch_delta;
        }
        if pitch_delta > 0. {
            pitch_delta = 0.0_f32.max(pitch_delta - pitch_delta_decay).min(0.02);
        } else if pitch_delta < 0. {
            pitch_delta = 0.0_f32.min(pitch_delta + pitch_delta_decay).max(-0.02);
        }
        if position.y > 0. {
            pitch_delta += pitch_recentering_force;
        } else {
            pitch_delta -= pitch_recentering_force;
        }
        let roll_delta = yaw_delta * 0.1;
        pitch += pitch_delta;
        yaw += yaw_delta * damping;
        roll += roll_delta;
        pitch = pitch.clamp(-pitch_limit, pitch_limit) * damping;
        pitch += -pitch * springiness;
        roll = roll.clamp(-roll_limit, roll_limit) * damping;
        roll += -roll * springiness;
        let new_rotation = Quat::from_rotation_y(yaw)
            * Quat::from_rotation_x(pitch)
            * Quat::from_rotation_z(-roll);
        rotation = rotation.slerp(new_rotation, damping);

        let forward = rotation * Vec3::Z;
        let right = (rotation * -Vec3::X).normalize();
        let up = right.cross(forward).normalize();

        position += forward
            * (z_step * (frame.beat_strength * 0.3 + frame.energy * 0.7 + frame.lane_center * 0.7));

        points.push(TrackPoint {
            rotation,
            position,
            forward,
            right,
            up,
        });
    }

    info!(
        "maximum Y: {}, minimum Y:{}",
        points
            .iter()
            .map(|p| p.position.y)
            .reduce(f32::max)
            .unwrap_or(0.0),
        points
            .iter()
            .map(|p| p.position.y)
            .reduce(f32::min)
            .unwrap_or(0.0)
    );

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
    extrude_along_track(points, &track_shape)
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
    extrude_along_track(points, &track_shape)
}

pub fn extrude_along_track(frames: &[TrackPoint], shape: &[Vec2]) -> Mesh {
    let mut positions = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];

    let shape_len = shape.len();

    for (i, frame) in frames.iter().enumerate() {
        for (j, p) in shape.iter().enumerate() {
            let world_pos = frame.position + frame.right * p.x + frame.up * p.y;

            positions.push(world_pos.to_array());

            normals.push(frame.up.to_array());

            uvs.push([j as f32 / shape_len as f32, i as f32 / frames.len() as f32]);
        }
    }

    // triangulação
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

pub fn resample_track_equidistant_points(points: &[TrackPoint], distance: f32) -> Vec<TrackPoint> {
    if points.len() < 2 || distance <= 0.0 {
        return points.to_vec();
    }

    let mut result = Vec::new();
    result.push(points[0].clone());

    let mut accumulated_distance = 0.0;
    let mut target_distance = distance;

    for i in 0..points.len() - 1 {
        let p1 = &points[i];
        let p2 = &points[i + 1];
        let segment_len = p1.position.distance(p2.position);

        if segment_len == 0.0 {
            continue;
        }

        while accumulated_distance + segment_len >= target_distance {
            let t = (target_distance - accumulated_distance) / segment_len;
            result.push(p1.lerp(p2, t));
            target_distance += distance;
        }

        accumulated_distance += segment_len;
    }

    result
}
