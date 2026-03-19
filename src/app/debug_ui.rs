use bevy::prelude::*;
use crate::app::analyze::CurrentSong;
use crate::app::AppState;

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::DebugUi), start_debug_ui);
    }
}

fn start_debug_ui(mut commands: Commands, current_song: Res<CurrentSong>) {
    info!("starting ui for {}", current_song.file_name());
}
