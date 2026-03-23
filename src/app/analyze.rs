use crate::analysis;
use crate::analysis::gameplay::derive_gameplay;
use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use crate::app::playback::SongAsset;
use crate::app::{AppState, Args};
use bevy::prelude::*;
use std::fs;
use std::fs::canonicalize;

pub struct AnalyzePlugin;

impl Plugin for AnalyzePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Analyze), start_analysis);
    }
}

fn start_analysis(args: Res<Args>, mut commands: Commands, assets: Res<AssetServer>) -> Result {
    info!("input: {}", args.input.display());
    let analysis = analysis::analyze_file(&args.input)?;
    debug!("analysis complete");

    let out_path = args.output.clone().unwrap_or_else(|| {
        let mut p = args.input.clone();
        p.set_extension("analysis.json");
        p
    });

    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(&out_path, json)?;

    info!("wrote analysis: {}", out_path.display());
    let frames = derive_gameplay(&analysis);
    let file_path = canonicalize(&args.input)?.to_string_lossy().to_string();
    let song_asset = assets.add(SongAsset {
        path: file_path.clone(),
    });
    commands.insert_resource(CurrentSong {
        track_analysis: analysis,
        frames,
        file_path,
        time_seconds: 0.,
        song_asset,
    });
    commands.set_state(AppState::DebugUi);
    Ok(())
}

#[derive(Resource, Clone)]
pub struct CurrentSong {
    pub track_analysis: TrackAnalysis,
    pub frames: Vec<GameplayFrame>,
    pub file_path: String,
    pub time_seconds: f32,
    pub song_asset: Handle<SongAsset>,
}

impl CurrentSong {
    pub fn file_name(&self) -> &str {
        &self.file_path
    }
}
