use std::fs;
use bevy::ecs::storage::Resources;
use bevy::prelude::*;
use crate::analysis;
use crate::analysis::model::TrackAnalysis;
use crate::app::{AppState, Args};

pub struct AnalyzePlugin;

impl Plugin for AnalyzePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Analyze), start_analysis);
    }
}

fn start_analysis(args: Res<Args>, mut commands: Commands,) -> Result {
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
    commands.insert_resource(CurrentSong { track_analysis: analysis, file_name: args.input.file_name().unwrap().to_string_lossy().to_string() });
    commands.set_state(AppState::DebugUi);
    Ok(())
}

#[derive(Resource, Clone)]
pub struct CurrentSong {
    track_analysis: TrackAnalysis,
    file_name: String,
}

impl CurrentSong {
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}
