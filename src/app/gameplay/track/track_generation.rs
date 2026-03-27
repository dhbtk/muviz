use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use crate::app::gameplay::track::track_point::TrackPoint;
use bevy::math::{Curve, Quat, Vec3};
use bevy::prelude::{EaseFunction, EasingCurve};
use rand::prelude::SmallRng;
use rand::{RngExt, SeedableRng};
use std::f32::consts::PI;
use std::ops::Div;
use tracing::{debug, info};

fn find_overlap_with_previous_points(
    candidate: Vec3,
    points: &[TrackPoint],
    ignore_recent_points: usize,
    horizontal_radius: f32,
    vertical_clearance: f32,
) -> Option<usize> {
    if points.len() <= ignore_recent_points {
        return None;
    }

    let search_end = points.len() - ignore_recent_points;
    let radius_sq = horizontal_radius * horizontal_radius;

    points[..search_end]
        .iter()
        .enumerate()
        .find_map(|(i, point)| {
            let dx = candidate.x - point.position.x;
            let dz = candidate.z - point.position.z;
            let horizontal_dist_sq = dx * dx + dz * dz;

            if horizontal_dist_sq <= radius_sq
                && (candidate.y - point.position.y).abs() < vertical_clearance
            {
                Some(i)
            } else {
                None
            }
        })
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

pub fn generate_track_points(
    analysis: &TrackAnalysis,
    frames: &[GameplayFrame],
) -> Vec<TrackPoint> {
    let mut rng = SmallRng::seed_from_u64(frames.len() as u64);
    let mut points: Vec<TrackPoint> = Vec::with_capacity(frames.len());
    let bps = analysis.estimated_bpm.unwrap_or(120.0) / 60.;

    let beat_intervals = vec![1, 2, 2, 3, 3, 4, 4];

    let mut yaw_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len())];
    let mut pitch_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len())];
    let curve = EasingCurve::new(0.0, 1.0, EaseFunction::SmootherStep);
    let pitch_scale = 0.0096;
    let yaw_scale = 0.044;
    let yaw_delta_decay = 0.0001;
    let yaw_delta_limit = 0.15;
    let yaw_recentering_force = 0.006;
    let pitch_delta_decay = 0.00012;
    let pitch_delta_limit = 0.02;
    let pitch_recentering_force = 0.0002;
    let pitch_limit = PI / 8.;
    let roll_limit = PI / 12.;
    let damping = 0.95;
    let springiness = 0.03;
    let acceleration_decay = 0.0015;
    let acceleration_scale = 0.002;
    let acceleration_limit = 0.01;
    let speed_decay = 0.001;
    let min_speed = 1.0;
    let max_speed = 1.5;
    let overlap_horizontal_radius = 24.0;
    let overlap_vertical_clearance = 24.0;
    let overlap_ignore_recent_points = 24;
    let overlap_max_backtrack_points = 720;

    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let mut roll = 0.0;
    let mut speed = 0.0;
    let mut rotation = Quat::IDENTITY;
    let mut position = Vec3::ZERO;
    let mut acceleration = 0.0;
    let mut previous_beat_index: i32 = -1;
    let mut yaw_sign = 1.0;
    let mut pitch_sign = 1.0;

    for frame in frames {
        let current_beat = frame.time_s / bps;
        let beat_index = current_beat.floor() as i32;
        let beat_changed = beat_index != previous_beat_index;
        previous_beat_index = beat_index;

        let mut yaw_delta = curve.sample_clamped(if frame.lane_left + frame.lane_right > 1.0 {
            frame.lane_left - frame.lane_right
        } else {
            frame.lane_left + frame.lane_right
        }) * yaw_scale;
        if beat_changed && beat_index > 0 && beat_index % yaw_flip_interval == 0 {
            yaw_sign = -yaw_sign;
            let prev = yaw_flip_interval;
            while prev == yaw_flip_interval {
                yaw_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len())];
            }
            debug!(
                "[{:03.2}] yaw flip interval: {} -> {}",
                frame.time_s, prev, yaw_flip_interval
            );
        }
        yaw_delta *= yaw_sign;
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
        if beat_changed && beat_index > 0 && beat_index % pitch_flip_interval == 0 {
            pitch_sign = -pitch_sign;
            let prev = pitch_flip_interval;
            while prev == pitch_flip_interval {
                pitch_flip_interval = beat_intervals[rng.random_range(0..beat_intervals.len())];
            }
            debug!(
                "[{:03.2}] pitch flip interval: {} -> {}",
                frame.time_s, prev, pitch_flip_interval
            );
        }
        pitch_delta *= pitch_sign;
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
        if pitch < 0. {
            acceleration -= acceleration_decay * (pitch.abs() / pitch_limit);
        } else if pitch > 0. {
            acceleration += acceleration_decay * (pitch.abs() / pitch_limit) * 0.3;
        }
        if acceleration > 0. {
            acceleration = 0.0_f32
                .max(acceleration - acceleration_decay)
                .min(acceleration_limit);
        } else if acceleration < 0. {
            acceleration = 0.0_f32
                .min(acceleration + acceleration_decay)
                .max(-acceleration_limit);
        }
        speed = (speed + (acceleration - speed_decay) * 0.2).clamp(min_speed, max_speed);

        position += forward * speed;

        let mut is_above_other_track = false;
        if let Some(overlap_index) = find_overlap_with_previous_points(
            position,
            &points,
            overlap_ignore_recent_points,
            overlap_horizontal_radius,
            overlap_vertical_clearance,
        ) {
            let overlap_y = points[overlap_index].position.y;
            let y_delta = position.y - overlap_y;
            if y_delta >= overlap_vertical_clearance {
                is_above_other_track = true;
            }
            let required_clearance = overlap_vertical_clearance - y_delta.abs();

            if required_clearance > 0.0 {
                let direction = 1.0; // always go up
                let max_vertical_step = (speed * pitch_limit.div(5.0).sin()).abs().max(0.001);
                let desired_steps = (required_clearance / max_vertical_step).ceil() as usize;
                let backtrack_steps = desired_steps.clamp(1, overlap_max_backtrack_points);
                let available_steps = points.len().min(backtrack_steps);
                let start = points.len().saturating_sub(available_steps);
                let per_step = direction * (required_clearance / available_steps as f32);

                for (offset, point) in points[start..].iter_mut().enumerate() {
                    point.position.y += per_step * (offset as f32 + 1.0);
                    point.is_above_other_track = true;
                }

                position.y += per_step * available_steps as f32;
                is_above_other_track = true;
            }
        }

        points.push(TrackPoint {
            rotation,
            position,
            forward,
            right,
            up,
            pitch,
            yaw,
            roll,
            speed,
            acceleration,
            current_beat,
            yaw_flip_interval: yaw_flip_interval as f32,
            pitch_flip_interval: pitch_flip_interval as f32,
            pitch_delta,
            yaw_delta,
            roll_delta,
            is_above_other_track,
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
