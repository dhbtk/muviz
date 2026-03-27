use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use crate::app::gameplay::track::track_point::{smooth_positions, TrackPoint};
use crate::app::gameplay::track::{track_generation, track_point};
use crate::app::playback::SongAsset;
use crate::app::{analyze, Args};
use crate::{HOP_SIZE, SAMPLE_RATE};
use bevy::asset::Handle;
use bevy::camera::primitives::Aabb;
use bevy::math::{Quat, Vec3};
use bevy::prelude;
use bevy::prelude::Resource;
use std::fs::canonicalize;
use std::path::Path;
use tracing::debug;

#[derive(Resource, Clone)]
pub struct CurrentSong {
    pub track_analysis: TrackAnalysis,
    pub frames: Vec<GameplayFrame>,
    pub track_points: Vec<TrackPoint>,
    pub file_path: String,
    pub time_seconds: f32,
    pub song_asset: Handle<SongAsset>,
    pub paused: bool,
    pub track_bounding_box: Aabb,
    pub cumulative_lengths: Vec<f32>,
    pub total_length: f32,
}

impl CurrentSong {
    pub fn new(args: &Args, song_asset: Handle<SongAsset>) -> prelude::Result<Self> {
        let (track_analysis, frames) = analyze::perform_analysis(args)?;
        let file_path = canonicalize(args.input_file_path())?
            .to_string_lossy()
            .to_string();
        let mut track_points = track_generation::generate_track_points(&track_analysis, &frames);
        smooth_positions(&mut track_points, 0.3);
        let (arc_lengths, total_length) = Self::compute_arc_length(&track_points);
        debug!("total length: {}", total_length);

        Ok(Self {
            track_analysis,
            frames,
            file_path,
            time_seconds: 0.,
            song_asset,
            track_bounding_box: Self::track_bounding_box(&track_points),
            track_points,
            paused: true,
            cumulative_lengths: arc_lengths,
            total_length,
        })
    }
    pub fn compute_arc_length(points: &[TrackPoint]) -> (Vec<f32>, f32) {
        let mut lengths = Vec::with_capacity(points.len());
        let mut total = 0.0;

        lengths.push(0.0);

        for i in 1..points.len() {
            let d = points[i].position.distance(points[i - 1].position);
            total += d;
            lengths.push(total);
        }

        (lengths, total)
    }

    pub fn file_name(&self) -> &str {
        Path::new(&self.file_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    }

    pub fn sample_track_point(&self, t: f32) -> TrackPoint {
        let i = t.floor() as usize;
        let frac = t.fract();

        let len = self.track_points.len();
        if len == 0 {
            return TrackPoint {
                rotation: Quat::IDENTITY,
                position: Vec3::ZERO,
                forward: Vec3::Z,
                right: Vec3::X,
                up: Vec3::Y,
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
                speed: 0.0,
                acceleration: 0.0,
                current_beat: 0.0,
                yaw_flip_interval: 0.0,
                pitch_flip_interval: 0.0,
                pitch_delta: 0.0,
                yaw_delta: 0.0,
                roll_delta: 0.0,
                is_above_other_track: false,
            };
        }

        let i0 = i.saturating_sub(1).min(len - 1);
        let i1 = i.min(len - 1);
        let i2 = (i + 1).min(len - 1);
        let i3 = (i + 2).min(len - 1);

        let p0 = &self.track_points[i0];
        let p1 = &self.track_points[i1];
        let p2 = &self.track_points[i2];
        let p3 = &self.track_points[i3];

        TrackPoint::catmull_rom(p0, p1, p2, p3, frac)
    }

    pub fn sample_gameplay_frame(&self, t: f32) -> GameplayFrame {
        let i = t.floor() as usize;
        let frac = t.fract();
        let frame = self.frames.get(i).unwrap_or(&self.frames[0]);
        let next = self.frames.get(i + 1).unwrap_or(&self.frames[1]);
        frame.lerp(next, frac)
    }

    pub fn current_frame_t(&self) -> f32 {
        (self.time_seconds * SAMPLE_RATE as f32) / HOP_SIZE as f32
    }

    pub fn nearest_frame(&self, pos: Vec3) -> &GameplayFrame {
        let mut min_dist = f32::MAX;
        let mut min_index = 0;
        for (i, _frame) in self.frames.iter().enumerate() {
            let dist = self.track_points[i].position.distance(pos);
            if dist < min_dist {
                min_dist = dist;
                min_index = i;
            }
        }
        &self.frames[min_index]
    }

    pub fn track_min_y(&self) -> f32 {
        self.track_bounding_box.center.y - self.track_bounding_box.half_extents.y
    }

    fn track_bounding_box(track_points: &[TrackPoint]) -> Aabb {
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for point in track_points {
            min = min.min(point.position);
            max = max.max(point.position);
        }

        Aabb::from_min_max(min, max)
    }
}
