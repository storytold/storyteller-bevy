use core::fmt;

use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, asset_loaded);
    }
}

#[derive(Resource)]
pub struct Cubemap {
    is_loaded: bool,
    image_handle: Handle<Image>,
    env_diffuse: Handle<Image>,
    env_specular: Handle<Image>,
}

impl Cubemap {
    fn new(name: impl fmt::Display, assets: &Res<AssetServer>) -> Self {
        let paths = SkyboxPaths::new(name);

        Self {
            is_loaded: false,
            image_handle: assets.load(paths.main),
            env_diffuse: assets.load(paths.env_diffuse),
            env_specular: assets.load(paths.env_specular),
        }
    }
}

pub struct SkyboxPaths {
    pub main: String,
    pub env_diffuse: String,
    pub env_specular: String,
}

impl SkyboxPaths {
    fn new(name: impl fmt::Display) -> Self {
        Self {
            main: format!("skyboxes/{name}.hdr"),
            env_diffuse: format!("skyboxes/{name}_diffuse.zstd.ktx2"),
            env_specular: format!("skyboxes/{name}_specular.zstd.ktx2"),
        }
    }
}

fn setup(mut cmd: Commands, assets: Res<AssetServer>) {
    cmd.insert_resource(Cubemap::new("test_scene", &assets));
}

fn asset_loaded(
    assets: Res<AssetServer>,
    camera: Query<Entity, With<Camera3d>>,
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
) {
    if !cubemap.is_loaded
        && assets.get_load_state(cubemap.image_handle.clone_weak()) == LoadState::Loaded
    {
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(6);
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        cmd.entity(camera.single()).insert((
            Skybox(cubemap.image_handle.clone()),
            EnvironmentMapLight {
                diffuse_map: cubemap.env_diffuse.clone(),
                specular_map: cubemap.env_specular.clone(),
            },
        ));

        cubemap.is_loaded = true;
    }
}
