use bevy::pbr::StandardMaterial;
use bevy::prelude::{Color, Component, Reflect};

use bevy::ecs::reflect::ReflectComponent;
use bevy::reflect::std_traits::ReflectDefault;

#[derive(Reflect, Default, Clone, Component)]
#[reflect(Default, Component)]
pub struct ShortMaterial {
    pub base_color: Color,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
}

impl ShortMaterial {
    pub fn new(material: &StandardMaterial) -> Self {
        Self {
            base_color: material.base_color,
            perceptual_roughness: material.perceptual_roughness,
            metallic: material.metallic,
            reflectance: material.reflectance,
        }
    }
}
