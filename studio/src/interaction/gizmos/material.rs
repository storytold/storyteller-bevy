use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GizmoMaterial {
    #[uniform(0)]
    pub color: Color,

    pub blend_mode: AlphaMode,
}

impl From<Color> for GizmoMaterial {
    fn from(color: Color) -> Self {
        GizmoMaterial {
            color,
            blend_mode: AlphaMode::Opaque,
        }
    }
}

impl Material for GizmoMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/gizmo.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.blend_mode
    }
}
