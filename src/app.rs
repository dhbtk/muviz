pub mod debug_ui;
pub mod analyze;
pub mod playback;

use std::path::PathBuf;
use bevy::asset::UnapprovedPathMode;
use bevy::audio::{AddAudioSource, AudioPlugin};
use crate::analysis::model;
use bevy::prelude::*;
use clap::Parser;
use crate::app::analyze::AnalyzePlugin;
use crate::app::debug_ui::DebugUiPlugin;
use crate::app::playback::{PlaybackPlugin, SongAsset};

pub fn run_app(args: Args) {
    let window_title = format!("muviz - {}", args.input_file_path().file_name().unwrap_or_default().to_string_lossy());
   App::new()
       .insert_resource(args)
       .add_plugins(DefaultPlugins.set(WindowPlugin {
           primary_window: Some(Window {
               title: window_title,
               resolution: (1280, 720).into(),
               ..default()
           }),
          ..default()
       }).set(AssetPlugin {
           unapproved_path_mode: UnapprovedPathMode::Deny,
           ..default()
       }))
       .init_state::<AppState>()
       .add_audio_source::<SongAsset>()
       .add_plugins(AnalyzePlugin)
       .add_plugins(DebugUiPlugin)
       .add_plugins(PlaybackPlugin)
       .run();
}

#[derive(Debug, Parser, Resource, Clone)]
pub struct Args {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

impl Args {
    pub fn input_file_path(&self) -> &PathBuf {
        &self.input
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Analyze,
    DebugUi
}
