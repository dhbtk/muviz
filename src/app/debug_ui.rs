use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::app::analyze::CurrentSong;
use crate::app::AppState;

use crate::app::playback::SongAsset;

pub struct DebugUiPlugin;

#[derive(Component)]
pub struct SecondsCounter;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::DebugUi), start_debug_ui)
            .add_systems(Update, (update_timing, draw_graphs).run_if(in_state(AppState::DebugUi)));
    }
}

fn start_debug_ui(
    mut commands: Commands,
    current_song: Res<CurrentSong>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    info!("starting ui for {}", current_song.file_name());
    commands.spawn(Camera2d);

    commands.spawn(AudioPlayer(current_song.song_asset.clone()));

    commands.spawn((Text::new(current_song.file_name()), Node {
        position_type: PositionType::Absolute,
        top: px(10),
        left: px(10),
        ..default()
    }));

    commands.spawn((
        SecondsCounter,
        Text::new(format!("{:.2}", current_song.time_seconds)),
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            right: px(10),
            ..default()
        },
        ));
}

fn update_timing(
    mut current_song: ResMut<CurrentSong>,
    time: Res<Time>,
    mut time_label_query: Query<&mut Text, With<SecondsCounter>>,
) -> Result {
    current_song.time_seconds += time.delta().as_secs_f32();
    if let Ok(mut label) = time_label_query.single_mut() {
        label.0 = format!("{:.2}", current_song.time_seconds);
    }
    Ok(())
}

fn draw_graphs(
    mut gizmos: Gizmos,
    data: Res<CurrentSong>,
    windows: Query<&Window, With<PrimaryWindow>>,
) -> Result {
    let app_window = windows.single()?;
    let x_offset = -app_window.width() / 2.0 + 10.0;
    let y_offset = -app_window.height() / 2.0 + 40.0;
    let window = 5.0; // segundos visíveis
    let futureness = 0.5;
    let start_time = data.time_seconds - (window - futureness);
    let end_time = data.time_seconds + futureness;
    if start_time > data.track_analysis.duration_s {
        return Ok(());
    }

    let scale_x = (app_window.width() - 20.0) / window;
    // let scale_y = 50.0;
    let placement_scale_y = (app_window.height() - 50.0) / 7.0;
    let scale_y = placement_scale_y - 10.0;

    for i in 1..data.frames.len() {
        let f0 = &data.frames[i - 1];
        let f1 = &data.frames[i];

        if f1.time_s < start_time {
            continue;
        }
        if f1.time_s > end_time {
            break;
        }

        let x0 = x_offset + (f0.time_s - start_time) * scale_x;
        let x1 = x_offset + (f1.time_s - start_time) * scale_x;

        // lane_left
        gizmos.line_2d(
            Vec2::new(x0, f0.lane_left * scale_y - y_offset - scale_y),
            Vec2::new(x1, f1.lane_left * scale_y - y_offset - scale_y),
            Color::linear_rgb(0.0, 0.0, 1.0),
        );

        // lane_center
        gizmos.line_2d(
            Vec2::new(x0, f0.lane_center * scale_y - y_offset - (placement_scale_y * 2.0)),
            Vec2::new(x1, f1.lane_center * scale_y  - y_offset - (placement_scale_y * 2.0)),
            Color::linear_rgb(0.0, 1.0, 0.0),
        );

        // lane_right
        gizmos.line_2d(
            Vec2::new(x0, f0.lane_right * scale_y - y_offset - (placement_scale_y * 3.0)),
            Vec2::new(x1, f1.lane_right * scale_y - y_offset - (placement_scale_y * 3.0)),
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        // energy
        gizmos.line_2d(
            Vec2::new(x0, f0.energy * scale_y - y_offset - (placement_scale_y * 4.0)),
            Vec2::new(x1, f1.energy * scale_y - y_offset - (placement_scale_y * 4.0)),
            Color::WHITE,
        );

        // event (picos)
        gizmos.line_2d(
            Vec2::new(x0, f0.event * scale_y - y_offset - (placement_scale_y * 5.0)),
            Vec2::new(x1, f1.event * scale_y - y_offset - (placement_scale_y * 5.0)),
            Color::linear_rgb(1.0, 1.0, 0.0),
        );

        // texture
        gizmos.line_2d(
            Vec2::new(x0, f0.texture * scale_y - y_offset - (placement_scale_y * 6.0)),
            Vec2::new(x1, f1.texture * scale_y - y_offset - (placement_scale_y * 6.0)),
            Color::linear_rgb(0.0, 1.0, 1.0),
        );

        // beat strength
        gizmos.line_2d(
            Vec2::new(x0, f0.beat_strength * scale_y - y_offset - (placement_scale_y * 7.0)),
            Vec2::new(x1, f1.beat_strength * scale_y - y_offset - (placement_scale_y * 7.0)),
            Color::linear_rgb(1.0, 0.0, 1.0),
        );

    }

    for &beat_time in &data.track_analysis.beat_times_s {
        if beat_time < start_time || beat_time > end_time {
            continue;
        }

        let x = (beat_time - start_time) * scale_x + x_offset;

        gizmos.line_2d(
            Vec2::new(x, -y_offset),
            Vec2::new(x, -(app_window.height() - 60.0) - y_offset),
            Color::srgba(1.0, 1.0, 1.0, 0.25),
        );
    }

    let playhead_x = x_offset + (window - futureness) * scale_x;
    gizmos.line_2d(
        Vec2::new(playhead_x, -y_offset),
        Vec2::new(playhead_x, -(app_window.height() - 50.0) - y_offset),
        Color::WHITE,
    );
    Ok(())
}
