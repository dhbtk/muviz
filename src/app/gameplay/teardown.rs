use crate::app::gameplay::entities::MainScene;
use bevy::prelude::{Commands, Entity, Query, With};

pub fn despawn_entities(mut commands: Commands, query: Query<Entity, With<MainScene>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
