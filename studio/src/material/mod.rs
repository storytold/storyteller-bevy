use crate::material::short_material::ShortMaterial;
use bevy::app::{App, Plugin, PostUpdate};
use bevy::asset::{Assets, Handle};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Commands, Entity, Query, ResMut};
use space_editor::prelude::EditorRegistryExt;

pub mod short_material;

pub struct MaterialPlugin;
impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShortMaterial>()
            .editor_registry::<ShortMaterial>();

        app.add_systems(PostUpdate, apply_short_materials);
    }
}

fn apply_short_materials(
    mut commands: Commands,
    q_mats: Query<(Entity, &Handle<StandardMaterial>, &ShortMaterial)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, handle, short) in q_mats.iter() {
        let Some(material) = materials.get_mut(handle) else {
            continue;
        };

        material.base_color = short.base_color;
        material.perceptual_roughness = short.perceptual_roughness;
        material.metallic = short.metallic;
        material.reflectance = short.reflectance;

        commands.entity(entity).remove::<ShortMaterial>();
    }
}
