pub mod components;
pub mod model;
pub mod ocean;
pub mod systems;

use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use crate::app::gameplay::model::smooth_positions;
use crate::app::gameplay::ocean::Water;
use crate::app::gameplay::systems::{update_camera, update_playback};
use crate::app::playback::SongAsset;
use crate::app::{analyze, AppState, Args};
use crate::{HOP_SIZE, SAMPLE_RATE};
use bevy::camera::primitives::Aabb;
use bevy::pbr::{DefaultOpaqueRendererMethod, ExtendedMaterial};
use bevy::prelude::*;
use model::TrackPoint;
use std::fs::canonicalize;
use systems::{despawn_entities, spawn_entities};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DefaultOpaqueRendererMethod::deferred())
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(GlobalAmbientLight::NONE)
            .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
            .add_systems(OnEnter(AppState::Gameplay), spawn_entities)
            .add_systems(OnExit(AppState::Gameplay), despawn_entities)
            .add_systems(
                Update,
                (update_playback, update_camera).run_if(in_state(AppState::Gameplay)),
            );
    }
}

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
}

impl CurrentSong {
    pub fn new(args: &Args, song_asset: Handle<SongAsset>) -> Result<Self> {
        let (track_analysis, frames) = analyze::perform_analysis(args)?;
        let file_path = canonicalize(args.input_file_path())?
            .to_string_lossy()
            .to_string();
        let mut track_points = model::generate_track_points(&track_analysis, &frames);
        smooth_positions(&mut track_points, 0.3);

        Ok(Self {
            track_analysis,
            frames,
            file_path,
            time_seconds: 0.,
            song_asset,
            track_bounding_box: Self::track_bounding_box(&track_points),
            track_points,
            paused: false,
        })
    }

    pub fn file_name(&self) -> &str {
        &self.file_path
    }

    pub fn sample_track_point(&self, t: f32) -> TrackPoint {
        let i = t.floor() as usize;
        let frac = t.fract();

        let i0 = i.min(self.track_points.len() - 1);
        let i1 = (i + 1).min(self.track_points.len() - 1);

        let p0 = &self.track_points[i0];
        let p1 = &self.track_points[i1];

        p0.lerp(p1, frac)
    }

    pub fn current_frame_t(&self) -> f32 {
        (self.time_seconds * SAMPLE_RATE as f32) / HOP_SIZE as f32
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
