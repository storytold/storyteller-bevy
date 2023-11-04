use core::fmt;

use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
};

use crate::MainCamera;

pub struct SkyboxPlugin;

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                load_skybox.run_if(resource_exists_and_changed::<SkyboxId>()),
                spawn_skybox
                    .run_if(resource_exists::<SkyboxAssets>())
                    .run_if(skybox_assets_loaded),
            ),
        );
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct SkyboxId(pub String);

#[derive(Resource)]
struct SkyboxAssets {
    cubemap: Handle<Image>,
    env_diffuse: Handle<Image>,
    env_specular: Handle<Image>,
}

fn load_skybox(mut cmd: Commands, assets: Res<AssetServer>, id: Res<SkyboxId>) {
    let paths = SkyboxPaths::new(&**id);
    cmd.insert_resource(SkyboxAssets {
        cubemap: assets.load(paths.cubemap),
        env_diffuse: assets.load(paths.env_diffuse),
        env_specular: assets.load(paths.env_specular),
    });
}

fn skybox_assets_loaded(
    assets: Res<AssetServer>,
    skybox_assets: Option<Res<SkyboxAssets>>,
) -> bool {
    let Some(skybox_assets) = skybox_assets else {
        return false;
    };

    assets.get_load_state(skybox_assets.cubemap.clone_weak()) == LoadState::Loaded
        && assets.get_load_state(skybox_assets.env_diffuse.clone_weak()) == LoadState::Loaded
        && assets.get_load_state(skybox_assets.env_specular.clone_weak()) == LoadState::Loaded
}

fn spawn_skybox(
    mut cmd: Commands,
    mut images: ResMut<Assets<Image>>,
    skybox_assets: Res<SkyboxAssets>,
    q_camera: Query<Entity, With<MainCamera>>,
) {
    // Configure the skybox texture as a cubemap
    let image = images.get_mut(&skybox_assets.cubemap).unwrap();
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(6);
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }

    // Add components to the scene camera
    cmd.entity(q_camera.single()).insert((
        Skybox(skybox_assets.cubemap.clone()),
        EnvironmentMapLight {
            diffuse_map: skybox_assets.env_diffuse.clone(),
            specular_map: skybox_assets.env_specular.clone(),
        },
    ));

    // Remove the intermediate resource
    cmd.remove_resource::<SkyboxAssets>();
}

struct SkyboxPaths {
    cubemap: String,
    env_diffuse: String,
    env_specular: String,
}

impl SkyboxPaths {
    fn new(name: impl fmt::Display) -> Self {
        Self {
            cubemap: format!("skyboxes/{name}.hdr"),
            env_diffuse: format!("skyboxes/{name}_diffuse.zstd.ktx2"),
            env_specular: format!("skyboxes/{name}_specular.zstd.ktx2"),
        }
    }
}
