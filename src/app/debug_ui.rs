use crate::app::gameplay::CurrentSong;
use crate::app::AppState;
use anyhow::anyhow;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct DebugUiPlugin;

#[derive(Component)]
pub struct SecondsCounter;

#[derive(Component)]
pub struct DebugInfoLabel;

#[derive(Component)]
pub struct DebugUi;

#[derive(Component)]
pub struct SongPlayer;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Gameplay), start_debug_ui)
            .add_systems(OnExit(AppState::Gameplay), teardown_debug_ui)
            .add_systems(
                Update,
                (update_timing, update_debug_info, draw_graphs)
                    .run_if(in_state(AppState::Gameplay)),
            );
    }
}

fn start_debug_ui(
    mut commands: Commands,
    current_song: Res<CurrentSong>,
    _windows: Query<&Window, With<PrimaryWindow>>,
) {
    info!("starting ui for {}", current_song.file_name());
    commands.spawn((
        DebugUi,
        Node {
            display: Display::Grid,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        children![
            (
                Text::new(current_song.file_name()),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
            ),
            (
                SecondsCounter,
                Text::new(format!("{:.2}", current_song.time_seconds)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..default()
                }
            ),
            (
                DebugInfoLabel,
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                }
            )
        ],
    ));
}

fn teardown_debug_ui(mut commands: Commands, query: Query<Entity, With<DebugUi>>) -> Result {
    let entity = query.single()?;
    commands.entity(entity).despawn();
    Ok(())
}

fn update_timing(
    current_song: Res<CurrentSong>,
    mut time_label_query: Query<&mut Text, (With<SecondsCounter>, Without<DebugInfoLabel>)>,
) -> Result {
    let mut label = time_label_query.single_mut()?;
    label.0 = format!("{:.2}", current_song.time_seconds);
    Ok(())
}

fn update_debug_info(
    current_song: Res<CurrentSong>,
    mut debug_label_query: Query<&mut Text, With<DebugInfoLabel>>,
) -> Result {
    let t = current_song.current_frame_t();
    let frame_index = t.floor() as usize;
    let frame = current_song
        .frames
        .get(frame_index)
        .ok_or_else(|| anyhow!("frame index out of bounds"))?;
    let track_point = current_song.sample_track_point(t);
    let previous_track_point = current_song.sample_track_point(t - 0.05);
    let world_speed = track_point.position.distance(previous_track_point.position) / 0.05;

    let mut label = debug_label_query.single_mut()?;
    let euler = track_point.rotation.to_euler(EulerRot::YXZ);

    let mut info = format!(
        "time: {:.2}\n\
            world_speed: {:.2}\n\
            pos: {:.1}\n\
            yaw: {:.1} roll: {:.1} pitch: {:.1}\n\
            rot: {:.1}\n\
            forward: {}\n\
            right: {}\n\
            up: {}\n\
            lane_left: {:.2}\n\
            lane_center: {:.2}\n\
            lane_right: {:.2}\n\
            energy: {:.2}\n\
            event: {:.2}\n\
            texture: {:.2}\n\
            beat_strength: {:.2}\n\
            rms: {:.2}\n\
            spectral_flux: {:.2}\n\
            spectral_flatness: {:.2}",
        frame.time_s,
        world_speed,
        track_point.position,
        euler.0.to_degrees(),
        euler.1.to_degrees(),
        euler.2.to_degrees(),
        track_point.rotation,
        track_point.forward,
        track_point.right,
        track_point.up,
        frame.lane_left,
        frame.lane_center,
        frame.lane_right,
        frame.energy,
        frame.event,
        frame.texture,
        frame.beat_strength,
        frame.frame.rms,
        frame.frame.spectral_flux,
        frame.frame.spectral_flatness,
    );

    if !frame.frame.band_energy.is_empty() {
        info.push_str("\nband_energy: [");
        for (i, e) in frame.frame.band_energy.iter().enumerate() {
            if i > 0 {
                info.push_str(", ");
            }
            info.push_str(&format!("{:.2}", e));
        }
        info.push(']');
    }

    if !frame.frame.band_flux.is_empty() {
        info.push_str("\nband_flux: [");
        for (i, fl) in frame.frame.band_flux.iter().enumerate() {
            if i > 0 {
                info.push_str(", ");
            }
            info.push_str(&format!("{:.2}", fl));
        }
        info.push(']');
    }

    label.0 = info;
    Ok(())
}

fn draw_graphs(
    mut gizmos: Gizmos,
    data: Res<CurrentSong>,
    windows: Query<&Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
) -> Result {
    let Ok(app_window) = windows.single() else {
        return Ok(());
    };
    if !input.pressed(KeyCode::ShiftLeft) {
        return Ok(());
    }
    let x_offset = -app_window.width() / 2.0 + 10.0;
    let y_offset = -app_window.height() / 2.0 + 40.0;
    let window = 5.0; // segundos visíveis
    let futureness = 0.;
    let start_time = data.time_seconds - (window - futureness);
    let end_time = data.time_seconds + futureness;
    if start_time > data.track_analysis.duration_s {
        return Ok(());
    }

    let scale_x = (app_window.width() - 20.0) / window;
    // let scale_y = 50.0;
    let placement_scale_y = (app_window.height() - 50.0) / 10.0;
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
            Vec2::new(
                x0,
                f0.lane_center * scale_y - y_offset - (placement_scale_y * 2.0),
            ),
            Vec2::new(
                x1,
                f1.lane_center * scale_y - y_offset - (placement_scale_y * 2.0),
            ),
            Color::linear_rgb(0.0, 1.0, 0.0),
        );

        // lane_right
        gizmos.line_2d(
            Vec2::new(
                x0,
                f0.lane_right * scale_y - y_offset - (placement_scale_y * 3.0),
            ),
            Vec2::new(
                x1,
                f1.lane_right * scale_y - y_offset - (placement_scale_y * 3.0),
            ),
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        // energy
        gizmos.line_2d(
            Vec2::new(
                x0,
                f0.energy * scale_y - y_offset - (placement_scale_y * 4.0),
            ),
            Vec2::new(
                x1,
                f1.energy * scale_y - y_offset - (placement_scale_y * 4.0),
            ),
            Color::WHITE,
        );

        // event (picos)
        gizmos.line_2d(
            Vec2::new(
                x0,
                f0.event * scale_y - y_offset - (placement_scale_y * 5.0),
            ),
            Vec2::new(
                x1,
                f1.event * scale_y - y_offset - (placement_scale_y * 5.0),
            ),
            Color::linear_rgb(1.0, 1.0, 0.0),
        );

        // texture
        gizmos.line_2d(
            Vec2::new(
                x0,
                f0.texture * scale_y - y_offset - (placement_scale_y * 6.0),
            ),
            Vec2::new(
                x1,
                f1.texture * scale_y - y_offset - (placement_scale_y * 6.0),
            ),
            Color::linear_rgb(0.0, 1.0, 1.0),
        );

        // beat strength
        gizmos.line_2d(
            Vec2::new(
                x0,
                f0.beat_strength * scale_y - y_offset - (placement_scale_y * 7.0),
            ),
            Vec2::new(
                x1,
                f1.beat_strength * scale_y - y_offset - (placement_scale_y * 7.0),
            ),
            Color::linear_rgb(1.0, 0.0, 1.0),
        );

        // RMS
        gizmos.line_2d(
            Vec2::new(
                x0,
                f0.frame.rms * scale_y - y_offset - (placement_scale_y * 8.0),
            ),
            Vec2::new(
                x1,
                f1.frame.rms * scale_y - y_offset - (placement_scale_y * 8.0),
            ),
            Color::linear_rgb(0.5, 0.5, 0.5),
        );

        // spectral_flux
        gizmos.line_2d(
            Vec2::new(x0, 0. * scale_y - y_offset - (placement_scale_y * 9.0)),
            Vec2::new(x1, 0. * scale_y - y_offset - (placement_scale_y * 9.0)),
            Color::linear_rgb(1.0, 0.5, 0.0),
        );

        // spectral_flatness
        gizmos.line_2d(
            Vec2::new(
                x0,
                f0.frame.spectral_flatness * scale_y - y_offset - (placement_scale_y * 10.0),
            ),
            Vec2::new(
                x1,
                f1.frame.spectral_flatness * scale_y - y_offset - (placement_scale_y * 10.0),
            ),
            Color::linear_rgb(0.5, 1.0, 0.5),
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
