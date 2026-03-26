use crate::analysis;
use crate::analysis::gameplay::derive_gameplay;
use crate::analysis::model::{GameplayFrame, TrackAnalysis};
use crate::app::gameplay::current_song::CurrentSong;
use crate::app::playback::SongAsset;
use crate::app::{AppState, Args};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::canonicalize;

pub struct AnalyzePlugin;

impl Plugin for AnalyzePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Analyze), start_analysis);
    }
}

fn start_analysis(args: Res<Args>, mut commands: Commands, assets: Res<AssetServer>) -> Result {
    let file_path = canonicalize(args.input_file_path())?
        .to_string_lossy()
        .to_string();
    let song_asset = assets.add(SongAsset {
        path: file_path.clone(),
    });

    let current_song = CurrentSong::new(&args, song_asset)?;

    commands.insert_resource(current_song);
    commands.set_state(AppState::Gameplay);
    Ok(())
}

pub fn perform_analysis(args: &Args) -> Result<(TrackAnalysis, Vec<GameplayFrame>)> {
    info!("input: {}", args.input_file_path().display());
    let analysis = analysis::analyze_file(args.input_file_path().as_path())?;
    let frames = derive_gameplay(&analysis);
    debug!("analysis complete");

    let out_path = args.output.clone().unwrap_or_else(|| {
        let mut p = args.input_file_path().clone();
        p.set_extension("analysis.json");
        p
    });
    let json = serde_json::to_string_pretty(&PersistedAnalysis::from((&analysis, &frames)))?;
    fs::write(&out_path, json)?;

    info!("wrote analysis: {}", out_path.display());
    Ok((analysis, frames))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PersistedAnalysis {
    pub analysis: TrackAnalysis,
    pub frames: Vec<GameplayFrame>,
}

impl From<(&TrackAnalysis, &Vec<GameplayFrame>)> for PersistedAnalysis {
    fn from(analysis: (&TrackAnalysis, &Vec<GameplayFrame>)) -> Self {
        Self {
            analysis: analysis.0.clone(),
            frames: analysis.1.clone(),
        }
    }
}
