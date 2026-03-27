pub mod analyze;
pub mod assets;
pub mod colors;
pub mod debug_ui;
pub mod file_picker;
pub mod gameplay;
pub mod playback;

use crate::app::analyze::AnalyzePlugin;
use crate::app::assets::GlobalAssets;
use crate::app::debug_ui::DebugUiPlugin;
use crate::app::file_picker::resources::FilePicker;
use crate::app::playback::PlaybackPlugin;
use bevy::asset::UnapprovedPathMode;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_asset_loader::prelude::*;
use clap::Parser;
use file_picker::plugin::FilePickerPlugin;
use gameplay::plugin::GameplayPlugin;
use std::path::PathBuf;

pub fn run_app(args: Args) {
    App::new()
        .insert_resource(args.clone())
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    level: Level::DEBUG,
                    filter:
                        "info,symphonia_core=warn,symphonia_bundle_mp3=warn,wgpu=error,muviz=debug"
                            .into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "muviz".to_string(),
                        resolution: (1280, 720).into(),
                        mode: if args.fullscreen {
                            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                        } else {
                            WindowMode::Windowed
                        },
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    unapproved_path_mode: UnapprovedPathMode::Deny,
                    ..default()
                }),
        )
        .init_state::<AppState>()
        .add_loading_state(
            LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::Initial)
                .load_collection::<GlobalAssets>(),
        )
        .add_plugins(FilePickerPlugin)
        .add_plugins(AnalyzePlugin)
        .add_plugins(DebugUiPlugin)
        .add_plugins(PlaybackPlugin)
        .add_plugins(GameplayPlugin)
        .add_systems(OnEnter(AppState::Initial), read_args)
        .run();
}

#[derive(Debug, Parser, Resource, Clone, Default)]
pub struct Args {
    pub input: Option<PathBuf>,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(short, long)]
    pub analyze_only: bool,

    #[arg(short, long)]
    pub fullscreen: bool,
}

impl Args {
    pub fn input_file_path(&self) -> PathBuf {
        self.input.clone().unwrap()
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Loading,
    Initial,
    FilePicker,
    Analyze,
    DebugUi,
    Gameplay,
}

fn read_args(args: Res<Args>, mut file_picker: ResMut<FilePicker>, mut commands: Commands) {
    if let Some(path) = &args.input {
        let path = path.to_owned().canonicalize().unwrap();
        file_picker.current_dir = path.parent().unwrap().to_owned();
        file_picker.refresh().unwrap();
        file_picker.select_file(path);
        commands.set_state(AppState::Analyze);
    } else {
        commands.set_state(AppState::FilePicker);
    }
}
