use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use bevy::math::{Curve, Quat, Vec3};
use bevy::prelude::{EaseFunction, EasingCurve};
use rand::prelude::SmallRng;
use rand::{RngExt, SeedableRng};
use std::f32::consts::PI;
use tracing::info;

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

    pub fn catmull_rom(p0: &Self, p1: &Self, p2: &Self, p3: &Self, t: f32) -> Self {
        let t2 = t * t;
        let t3 = t2 * t;

        fn cr(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, t: f32, t2: f32, t3: f32) -> Vec3 {
            0.5 * (2.0 * v1
                + (-v0 + v2) * t
                + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2
                + (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3)
        }

        Self {
            rotation: p1.rotation.slerp(p2.rotation, t),
            position: cr(
                p0.position,
                p1.position,
                p2.position,
                p3.position,
                t,
                t2,
                t3,
            ),
            forward: cr(p0.forward, p1.forward, p2.forward, p3.forward, t, t2, t3).normalize(),
            right: cr(p0.right, p1.right, p2.right, p3.right, t, t2, t3).normalize(),
            up: cr(p0.up, p1.up, p2.up, p3.up, t, t2, t3).normalize(),
        }
    }
}

pub fn generate_track_points(
    analysis: &TrackAnalysis,
    frames: &[GameplayFrame],
) -> Vec<TrackPoint> {
    let mut rng = SmallRng::seed_from_u64(frames.len() as u64);
    let mut points: Vec<TrackPoint> = Vec::with_capacity(frames.len());
    let bps = analysis.estimated_bpm.unwrap_or(120.0) / 60.;

    let beat_intervals = vec![2, 4, 8, 12, 16, 24, 32];

    let mut yaw_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len() - 1)];
    let mut pitch_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len() - 1)];
    let curve = EasingCurve::new(0.0, 1.0, EaseFunction::SmootherStep);
    let height_scale = 0.06;
    let curve_scale = 0.022;
    let yaw_delta_decay = 0.0001;
    let yaw_recentering_force = 0.0002;
    let pitch_delta_decay = 0.012;
    let pitch_recentering_force = 0.0002;
    let pitch_limit = PI / 8.;
    let roll_limit = PI / 12.;
    let damping = 0.95;
    let springiness = 0.03;
    let acceleration_decay = 0.005;
    let acceleration_scale = 0.01;
    let acceleration_limit = 0.02;
    let speed_decay = 0.001;
    let min_speed = 1.0;
    let max_speed = 1.5;

    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let mut roll = 0.0;
    let mut speed = 0.0;
    let mut rotation = Quat::IDENTITY;
    let mut position = Vec3::ZERO;
    let mut acceleration = 0.0;

    for frame in frames {
        let mut yaw_delta = curve.sample_clamped(
            (frame.lane_left - frame.lane_right) * (0.5 + frame.beat_strength * 0.5),
        ) * curve_scale;
        if ((frame.time_s / bps) % yaw_flip_interval as f32) as i32 % 2 == 0 {
            yaw_delta = -yaw_delta;
            yaw_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len() - 1)];
        }
        if yaw_delta > 0. {
            yaw_delta = 0.0_f32.max(yaw_delta - yaw_delta_decay).min(0.15);
        } else if yaw_delta < 0. {
            yaw_delta = 0.0_f32.min(yaw_delta + yaw_delta_decay).max(-0.15);
        }
        if yaw > 0. {
            yaw_delta -= yaw_recentering_force * rng.random::<f32>();
        } else if yaw < 0. {
            yaw_delta += yaw_recentering_force * rng.random::<f32>();
        }
        let mut pitch_delta = (frame.energy * 0.3 + frame.lane_center * 0.7) * height_scale;
        // - (frame.beat_strength * frame.energy * -0.02);
        if ((frame.time_s / bps) % pitch_flip_interval as f32) as i32 % 2 == 0 {
            pitch_delta = -pitch_delta;
            pitch_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len() - 1)];
        }
        if pitch_delta > 0. {
            pitch_delta = 0.0_f32.max(pitch_delta - pitch_delta_decay).min(0.02);
        } else if pitch_delta < 0. {
            pitch_delta = 0.0_f32.min(pitch_delta + pitch_delta_decay).max(-0.02);
        }
        if position.y > 0. {
            pitch_delta += pitch_recentering_force * rng.random::<f32>();
        } else {
            pitch_delta -= pitch_recentering_force * rng.random::<f32>();
        }
        let roll_delta = yaw_delta * 0.1;
        pitch += pitch_delta;
        yaw += yaw_delta * damping;
        roll += roll_delta;
        pitch = pitch.clamp(-pitch_limit, pitch_limit) * damping;
        pitch += -pitch * springiness;
        roll = roll.clamp(-roll_limit, roll_limit) * damping;
        roll += -roll * springiness;
        let new_rotation =
            Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        rotation = rotation.slerp(new_rotation, damping);

        let forward = rotation * Vec3::Z;
        let right = (rotation * -Vec3::X).normalize();
        let up = right.cross(forward).normalize();
        acceleration += acceleration_scale
            * curve.sample_clamped(
                frame.beat_strength * 0.2 + frame.energy * 0.4 + frame.lane_center * 0.4,
            );
        acceleration -= acceleration_decay;
        acceleration = acceleration.clamp(-acceleration_limit, acceleration_limit);
        speed = (speed + acceleration - speed_decay).clamp(min_speed, max_speed);

        position += forward * speed;

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
