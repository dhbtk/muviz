use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::entities::MainScene;
use bevy::audio::PlaybackMode;
use bevy::prelude::*;

pub fn spawn_song_player(mut commands: Commands, data: Res<CurrentSong>) {
    commands.spawn((
        MainScene,
        SongPlayer,
        AudioPlayer(data.song_asset.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            paused: data.paused,
            ..default()
        },
    ));
}

#[derive(Component)]
pub struct SongPlayer;
