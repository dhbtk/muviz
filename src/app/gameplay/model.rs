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

    let z_step = 3.0;
    let height_scale = 0.06;
    let curve_scale = 0.033;
    let yaw_delta_decay = 0.0001;
    let pitch_delta_decay = 0.025;
    let pitch_recentering_force = 0.0005;
    let pitch_limit = PI / 4.;
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
            yaw_delta = 0.0_f32.max(yaw_delta - yaw_delta_decay).min(0.25);
        } else if yaw_delta < 0. {
            yaw_delta = 0.0_f32.min(yaw_delta + yaw_delta_decay).max(-0.25);
        }
        let mut pitch_delta = (frame.energy * 0.3 + frame.lane_center * 0.7) * height_scale;
        // - (frame.beat_strength * frame.energy * -0.02);
        if ((frame.time_s / bps) % 12.0) as i32 % 2 == 0 {
            pitch_delta = -pitch_delta;
        }
        if pitch_delta > 0. {
            pitch_delta = 0.0_f32.max(pitch_delta - pitch_delta_decay).min(0.05);
        } else if pitch_delta < 0. {
            pitch_delta = 0.0_f32.min(pitch_delta + pitch_delta_decay).max(-0.05);
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
            * (z_step * ((frame.beat_strength.max(0.2) * 0.3) + (frame.energy.max(0.2) * 0.7)));

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
