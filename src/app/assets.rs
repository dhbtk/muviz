use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct GlobalAssets {
    #[asset(path = "models/streetlight.glb#Scene0")]
    pub streetlight_scene: Handle<Scene>,

    #[asset(path = "fonts/Audiowide-Regular.ttf")]
    pub ui_font: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct TrackMaterialImages {
    #[asset(path = "textures/asphalt/base_color.jpg")]
    pub asphalt_base_color: Handle<Image>,
    #[asset(path = "textures/asphalt/ambient_occlusion.jpg")]
    pub asphalt_ambient_occlusion: Handle<Image>,
    #[asset(path = "textures/asphalt/displacement.jpg")]
    pub asphalt_displacement: Handle<Image>,
    #[asset(path = "textures/asphalt/normal.jpg")]
    pub asphalt_normal: Handle<Image>,
    #[asset(path = "textures/asphalt/roughness.jpg")]
    pub asphalt_roughness: Handle<Image>,
    #[asset(path = "textures/asphalt/metallic.jpg")]
    pub asphalt_metallic: Handle<Image>,

    #[asset(path = "textures/concrete/base_color.jpg")]
    pub concrete_base_color: Handle<Image>,
    #[asset(path = "textures/concrete/ambient_occlusion.jpg")]
    pub concrete_ambient_occlusion: Handle<Image>,
    #[asset(path = "textures/concrete/displacement.jpg")]
    pub concrete_displacement: Handle<Image>,
    #[asset(path = "textures/concrete/normal.png")]
    pub concrete_normal: Handle<Image>,
    #[asset(path = "textures/concrete/roughness.jpg")]
    pub concrete_roughness: Handle<Image>,
    #[asset(path = "textures/concrete/metallic.jpg")]
    pub concrete_metallic: Handle<Image>,
}

#[derive(Resource)]
pub struct TrackMaterials {
    pub asphalt_material: Handle<StandardMaterial>,
    pub concrete_material: Handle<StandardMaterial>,
}

impl FromWorld for TrackMaterials {
    fn from_world(world: &mut World) -> Self {
        let images = world.resource::<TrackMaterialImages>();

        let asphalt_base_color = images.asphalt_base_color.clone();
        let asphalt_ambient_occlusion = images.asphalt_ambient_occlusion.clone();
        let asphalt_normal = images.asphalt_normal.clone();
        let asphalt_metallic = images.asphalt_metallic.clone();

        let concrete_base_color = images.concrete_base_color.clone();
        let concrete_ambient_occlusion = images.concrete_ambient_occlusion.clone();
        let concrete_normal = images.concrete_normal.clone();
        let concrete_roughness = images.concrete_roughness.clone();
        let concrete_metallic = images.concrete_metallic.clone();

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let asphalt_material = materials.add(StandardMaterial {
            base_color_texture: Some(asphalt_base_color),
            occlusion_texture: Some(asphalt_ambient_occlusion),
            normal_map_texture: Some(asphalt_normal),
            metallic: 0.0,
            perceptual_roughness: 0.75,
            reflectance: 0.04,
            ..default()
        });
        let concrete_material = materials.add(StandardMaterial {
            base_color_texture: Some(concrete_base_color),
            occlusion_texture: Some(concrete_ambient_occlusion),
            normal_map_texture: Some(concrete_normal),
            metallic_roughness_texture: Some(concrete_roughness),
            perceptual_roughness: 1.0,
            ..default()
        });

        let _ = asphalt_metallic;
        let _ = concrete_metallic;

        Self {
            asphalt_material,
            concrete_material,
        }
    }
}
