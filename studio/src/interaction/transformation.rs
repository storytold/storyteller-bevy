use core::fmt;

use bevy::prelude::*;
use bitflags::bitflags;
use wasm_bindgen::prelude::wasm_bindgen;

use super::gizmos::GizmoPlugin;

pub struct TransformationPlugin;

impl Plugin for TransformationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TransformMode::default());
        app.add_plugins(GizmoPlugin);
        #[cfg(target_arch = "wasm32")]
        app.add_systems(PreUpdate, wasm::update_transform_mode);
    }
}

#[derive(Resource)]
pub struct TransformMode {
    pub ttype: TransformType,
    pub space: TransformSpace,
}

impl Default for TransformMode {
    fn default() -> Self {
        Self {
            ttype: TransformType::TRANSLATE,
            space: default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[wasm_bindgen]
pub enum TransformSpace {
    #[default]
    Local,
    World,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct TransformType : u8 {
        const TRANSLATE = 0b001;
        const ROTATE    = 0b010;
        const SCALE     = 0b100;
    }
}

impl fmt::Debug for TransformType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TransformType(")?;

        let mut written = false;
        if self.contains(TransformType::TRANSLATE) {
            if written {
                write!(f, " | ")?;
            }
            write!(f, "TRANSLATE")?;
            written = true;
        }
        if self.contains(TransformType::ROTATE) {
            if written {
                write!(f, " | ")?;
            }
            write!(f, "ROTATE")?;
            written = true;
        }
        if self.contains(TransformType::SCALE) {
            if written {
                write!(f, " | ")?;
            }
            write!(f, "SCALE")?;
        }

        if self.bits() == 0 {
            write!(f, "None")?;
        }

        write!(f, ")")
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use bevy::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::wasm::{self, WasmMessageBuffer};

    use super::{TransformMode, TransformSpace, TransformType};

    #[wasm_bindgen(js_name = TransformType)]
    #[rustfmt::skip]
    pub enum JsTransformType {
        Translate = 0b001,
        Rotate    = 0b010,
        Scale     = 0b100,
    }

    impl From<u8> for TransformType {
        fn from(value: u8) -> Self {
            TransformType::from_bits_retain(value)
        }
    }

    static TRANSFORM_TYPE_BUF: WasmMessageBuffer<TransformType> = WasmMessageBuffer::new();
    static TRANSFORM_SPACE_BUF: WasmMessageBuffer<TransformSpace> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = setTransformType)]
    pub fn set_transform_type(flags: u8) {
        wasm::write_message!(TRANSFORM_TYPE_BUF, flags.into());
    }

    #[wasm_bindgen(js_name = setTransformSpace)]
    pub fn set_transform_space(space: TransformSpace) {
        wasm::write_message!(TRANSFORM_SPACE_BUF, space);
    }

    pub(super) fn update_transform_mode(mut xform_mode: ResMut<TransformMode>) {
        wasm::read_buffer!(TRANSFORM_TYPE_BUF, |ttype| {
            info!("Setting transform type: {ttype:?}");
            xform_mode.ttype = ttype;
        });
        wasm::read_buffer!(TRANSFORM_SPACE_BUF, |space| {
            info!("Setting transform space: {space:?}");
            xform_mode.space = space;
        });
    }
}
