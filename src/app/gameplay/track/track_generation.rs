use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::math::{Curve, Quat, Vec3};
use bevy::prelude::{EaseFunction, EasingCurve};
use rand::prelude::SmallRng;
use rand::{RngExt, SeedableRng};
use std::f32::consts::PI;
use tracing::{debug, info};
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
    let pitch_scale = 0.16;
    let yaw_scale = 0.044;
    let yaw_delta_decay = 0.0001;
    let yaw_delta_limit = 0.15;
    let yaw_recentering_force = 0.002;
    let pitch_delta_decay = 0.012;
    let pitch_delta_limit = 0.02;
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
        ) * yaw_scale;
        if ((frame.time_s / bps) % yaw_flip_interval as f32) as i32 % 2 == 0 {
            yaw_delta = -yaw_delta;
            let prev = yaw_flip_interval;
            while prev == yaw_flip_interval {
                yaw_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len() - 1)];
            }
            debug!("yaw flip interval: {} -> {}", prev, yaw_flip_interval);
        }
        if yaw_delta > 0. {
            yaw_delta = 0.0_f32
                .max(yaw_delta - yaw_delta_decay)
                .min(yaw_delta_limit);
        } else if yaw_delta < 0. {
            yaw_delta = 0.0_f32
                .min(yaw_delta + yaw_delta_decay)
                .max(-yaw_delta_limit);
        }
        if yaw > 0. {
            yaw_delta -= yaw_recentering_force * rng.random::<f32>();
        } else if yaw < 0. {
            yaw_delta += yaw_recentering_force * rng.random::<f32>();
        }
        if yaw_delta > 0. {
            yaw_delta = curve.sample_clamped(yaw_delta / yaw_delta_limit) * yaw_delta_limit;
        } else {
            yaw_delta = -curve.sample_clamped(-yaw_delta / yaw_delta_limit) * yaw_delta_limit;
        }
        let mut pitch_delta = (frame.energy * 0.3 + frame.lane_center * 0.7) * pitch_scale;
        // - (frame.beat_strength * frame.energy * -0.02);
        if ((frame.time_s / bps) % pitch_flip_interval as f32) as i32 % 2 == 0 {
            pitch_delta = -pitch_delta;
            let prev = pitch_flip_interval;
            while prev == pitch_flip_interval {
                pitch_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len() - 1)];
            }
            debug!("pitch flip interval: {} -> {}", prev, pitch_flip_interval);
        }
        if pitch_delta > 0. {
            pitch_delta = 0.0_f32
                .max(pitch_delta - pitch_delta_decay)
                .min(pitch_delta_limit);
        } else if pitch_delta < 0. {
            pitch_delta = 0.0_f32
                .min(pitch_delta + pitch_delta_decay)
                .max(-pitch_delta_limit);
        }
        if position.y > 0. {
            pitch_delta += pitch_recentering_force * rng.random::<f32>();
        } else {
            pitch_delta -= pitch_recentering_force * rng.random::<f32>();
        }
        if pitch_delta > 0. {
            pitch_delta = curve.sample_clamped(pitch_delta / pitch_delta_limit) * pitch_delta_limit;
        } else {
            pitch_delta =
                -curve.sample_clamped(-pitch_delta / pitch_delta_limit) * pitch_delta_limit;
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
