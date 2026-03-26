use crate::app::gameplay::current_song::CurrentSong;
use crate::app::gameplay::entities::camera::RailCamera;
use crate::app::gameplay::entities::song_player::SongPlayer;
use crate::app::gameplay::entities::track::Streetlight;
use crate::app::AppState;
use bevy::audio::{AudioSink, AudioSinkPlayback};
use bevy::input::ButtonInput;
use bevy::light::SpotLight;
use bevy::math::{Curve, Vec3};
use bevy::prelude::{
    Commands, CommandsStatesExt, EaseFunction, EasingCurve, GlobalTransform, KeyCode, Query, Res,
    ResMut, Time, Transform, With,
};

pub fn update_camera(
    song: Res<CurrentSong>,
    time: Res<Time>,
    mut query: Query<(&mut RailCamera, &mut Transform)>,
) {
    for (mut cam, mut transform) in query.iter_mut() {
        let t = song.current_frame_t();

        let current = song.sample_track_point(t);
        let ahead = song.sample_track_point(t + cam.look_ahead);

        let cam_pos = current.position - current.forward * cam.distance + current.up * cam.height;
        let target = ahead.position;

        if cam.target_pos == Vec3::ZERO {
            cam.target_pos = cam_pos;
        }
        if cam.target_looking_at == Vec3::ZERO {
            cam.target_looking_at = target;
        }
        if cam.target_up == Vec3::Y && current.up != Vec3::Y {
            cam.target_up = current.up;
        }

        let dt = time.delta_secs();
        // Smoothing factor (the fraction to move towards the target each frame, independent of framerate)
        let factor = 1.0 - cam.smoothing.powf(dt);

        // Smoothly update the camera position
        cam.target_pos = cam.target_pos.lerp(cam_pos, factor);
        transform.translation = cam.target_pos;

        // Smoothly update where the camera is looking
        cam.target_looking_at = cam.target_looking_at.lerp(target, factor);
        // Smoothly update the up vector
        cam.target_up = cam.target_up.lerp(current.up, factor).normalize();
        transform.look_at(cam.target_looking_at, cam.target_up);
    }
}

pub fn update_playback(
    mut commands: Commands,
    mut song: ResMut<CurrentSong>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    sink_query: Query<&AudioSink, With<SongPlayer>>,
) {
    let Ok(sink) = sink_query.single() else {
        return;
    };
    if keyboard_input.just_pressed(KeyCode::Space) {
        song.paused = !song.paused;
        if song.paused {
            sink.pause();
        } else {
            sink.play();
        }
    }
    if !song.paused {
        song.time_seconds += time.delta_secs();
    }
    if song.time_seconds > song.frames.last().unwrap().time_s
        || keyboard_input.just_pressed(KeyCode::Escape)
    {
        commands.set_state(AppState::FilePicker);
    }
}

pub fn update_streetlights(
    mut query: Query<(&GlobalTransform, &mut SpotLight), With<Streetlight>>,
    song: Res<CurrentSong>,
) {
    let curve = EasingCurve::new(0.0, 1.0, EaseFunction::ExponentialInOut);
    let player_coordinates = song.sample_track_point(song.current_frame_t()).position;
    // streetlights far from the player should be off.
    // streetlight intensity should be proportional to beat strength at that point times a falloff for lamps away from
    // the player.
    for (transform, mut light) in query.iter_mut() {
        let distance = player_coordinates.distance(transform.translation());
        let cutoff = 500.0;
        if distance > cutoff {
            light.intensity = 0.0;
            continue;
        }
        let falloff = curve.sample_clamped(1.0 - (distance - cutoff).clamp(0.0, cutoff) / cutoff);
        let features = song.nearest_frame(transform.translation());
        light.intensity = 50_000_000.0
            * (1.0 - falloff).clamp(0.2, 1.0)
            * curve.sample_clamped((features.beat_strength + features.lane_left))
            * 2.0;
    }
}
