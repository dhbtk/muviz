use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GlobalAssets {
    #[asset(path = "models/streetlight.glb#Scene0")]
    pub streetlight_scene: Handle<Scene>,

    #[asset(path = "fonts/Audiowide-Regular.ttf")]
    pub ui_font: Handle<Font>,
}
