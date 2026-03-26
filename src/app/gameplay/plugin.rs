use crate::app::gameplay::entities::camera::spawn_camera;
use crate::app::gameplay::entities::ocean::spawn_water;
use crate::app::gameplay::entities::ocean::Water;
use crate::app::gameplay::entities::procedural::spawn_track;
use crate::app::gameplay::entities::song_player::spawn_song_player;
use crate::app::gameplay::entities::sun::spawn_sun;
use crate::app::gameplay::runtime::{update_camera, update_playback, update_streetlights};
use crate::app::gameplay::teardown::despawn_entities;
use crate::app::AppState;
use bevy::app::{App, Plugin, Update};
use bevy::camera::ClearColor;
use bevy::color::Color;
use bevy::light::GlobalAmbientLight;
use bevy::pbr::{DefaultOpaqueRendererMethod, ExtendedMaterial, MaterialPlugin, StandardMaterial};
use bevy::prelude::{in_state, IntoScheduleConfigs, OnEnter, OnExit};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DefaultOpaqueRendererMethod::deferred())
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(GlobalAmbientLight::NONE)
            .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
            .add_systems(
                OnEnter(AppState::Gameplay),
                (
                    spawn_track,
                    spawn_camera,
                    spawn_sun,
                    spawn_water,
                    spawn_song_player,
                ),
            )
            .add_systems(OnExit(AppState::Gameplay), despawn_entities)
            .add_systems(
                Update,
                (update_playback, update_camera, update_streetlights)
                    .run_if(in_state(AppState::Gameplay)),
            );
    }
}
