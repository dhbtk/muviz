use bevy::prelude::*;
#[derive(Component)]
pub struct RailCamera {
    pub height: f32,
    pub distance: f32,
    pub look_ahead: f32,
    pub target_pos: Vec3,
    pub target_looking_at: Vec3,
    pub smoothing: f32,
}

#[derive(Component)]
pub struct SongTrack;
