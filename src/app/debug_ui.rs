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

#[derive(Resource, PartialEq)]
pub struct DebugUiVisible(pub bool);

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugUiVisible(true))
            .add_systems(OnEnter(AppState::Gameplay), start_debug_ui)
            .add_systems(OnExit(AppState::Gameplay), teardown_debug_ui)
            .add_systems(
                Update,
                (
                    toggle_debug_ui,
                    (update_timing, update_debug_info, draw_graphs, draw_mini_map)
                        .run_if(resource_equals(DebugUiVisible(true))),
                    sync_debug_ui_visibility,
                )
                    .run_if(in_state(AppState::Gameplay)),
            );
    }
}

fn start_debug_ui(
    mut commands: Commands,
    current_song: Res<CurrentSong>,
    _windows: Query<&Window, With<PrimaryWindow>>,
    debug_ui_visible: Res<DebugUiVisible>,
) {
    info!("starting ui for {}", current_song.file_name());
    commands.spawn((
        DebugUi,
        if debug_ui_visible.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        Node {
            display: Display::Grid,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
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

fn toggle_debug_ui(input: Res<ButtonInput<KeyCode>>, mut debug_ui_visible: ResMut<DebugUiVisible>) {
    if input.just_pressed(KeyCode::KeyH) {
        debug_ui_visible.0 = !debug_ui_visible.0;
    }
}

fn sync_debug_ui_visibility(
    debug_ui_visible: Res<DebugUiVisible>,
    mut query: Query<&mut Visibility, With<DebugUi>>,
) {
    if debug_ui_visible.is_changed() {
        for mut visibility in query.iter_mut() {
            *visibility = if debug_ui_visible.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
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
    let Some(frame) = current_song.frames.get(frame_index) else {
        return Ok(());
    };
    let previous_frame = current_song
        .frames
        .get(frame_index.saturating_sub(1))
        .unwrap();
    let track_point = &current_song.track_points[frame_index];
    let previous_track_point = &current_song.track_points[frame_index.saturating_sub(1)];
    let world_speed = (track_point.position.distance(previous_track_point.position)
        / (frame.time_s - previous_frame.time_s))
        * 3.6;

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

fn draw_mini_map(
    mut gizmos: Gizmos,
    data: Res<CurrentSong>,
    camera_query: Query<&Transform, With<Camera3d>>,
    _input: Res<ButtonInput<KeyCode>>,
) -> Result {
    let camera_transform = if let Some(t) = camera_query.iter().next() {
        t
    } else {
        return Ok(());
    };

    let width = 10.0;
    let height = 10.0;
    let distance = 30.0;

    let (min_x, max_x) = data
        .track_points
        .iter()
        .map(|p| p.position.x)
        .fold((f32::MAX, f32::MIN), |(min, max), x| {
            (min.min(x), max.max(x))
        });

    let (min_z, max_z) = data
        .track_points
        .iter()
        .map(|p| p.position.z)
        .fold((f32::MAX, f32::MIN), |(min, max), z| {
            (min.min(z), max.max(z))
        });

    let dx = max_x - min_x;
    let dz = max_z - min_z;
    if dx == 0. || dz == 0. {
        return Ok(());
    }

    // Proportional scaling to fit within width/height and maintain aspect ratio
    let scale = (width / dx).min(height / dz);

    let center_x = (min_x + max_x) / 2.0;
    let center_z = (min_z + max_z) / 2.0;

    let map_center = camera_transform.translation
        + camera_transform.forward() * distance
        + camera_transform.right() * (width * 0.5 + 10.0)
        + camera_transform.down() * (height * 0.5 - 10.0);

    let right = camera_transform.right();
    let up = camera_transform.up();

    let to_3d = |x: f32, z: f32| -> Vec3 {
        let lx = -(x - center_x) * scale;
        let ly = (z - center_z) * scale;
        map_center + right * lx + up * ly
    };

    for i in 0..(data.track_points.len() - 1) {
        let f0 = &data.track_points[i];
        let f1 = &data.track_points[i + 1];

        let p0 = to_3d(f0.position.x, f0.position.z);
        let p1 = to_3d(f1.position.x, f1.position.z);

        gizmos.line(p0, p1, Color::WHITE);
    }

    let current_position = data.sample_track_point(data.current_frame_t()).position;
    let current_p = to_3d(current_position.x, current_position.z);

    let size = 0.2;
    gizmos.line(
        current_p - right * size,
        current_p + right * size,
        Color::linear_rgb(1.0, 0.0, 0.0),
    );
    gizmos.line(
        current_p - up * size,
        current_p + up * size,
        Color::linear_rgb(1.0, 0.0, 0.0),
    );
    // gizmos.line(
    //     current_p - camera_transform.forward() * size,
    //     current_p + camera_transform.forward() * size,
    //     Color::linear_rgb(1.0, 0.0, 0.0),
    // );

    Ok(())
}
