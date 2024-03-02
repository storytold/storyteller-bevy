use bevy::prelude::{Entity, GlobalTransform, Quat, Transform, Vec3};
use serde::ser::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

pub type JsonValue = serde_json::Value;

#[wasm_bindgen(typescript_custom_section)]
const BEVY_TYPES: &str = r#"
export interface BevyObject<Tag extends string, T> {
    type: Tag;
    value: T;
};

export type Vec3 = BevyObject<"Vec3", [number, number, number]>;
export type Quat = BevyObject<"Quat", [number, number, number, number]>;

export interface ITransform {
    translation: Vec3;
    rotation: Quat;
    scale: Vec3;
}
export type Transform = BevyObject<"Transform", ITransform>;
export type GlobalTransform = BevyObject<"GlobalTransform", ITransform>;


"#;

pub(crate) const SERIALIZER: Serializer = Serializer::new()
    .serialize_missing_as_null(true)
    .serialize_maps_as_objects(true)
    .serialize_large_number_types_as_bigints(true);

pub trait AsJsValue {
    fn as_js(&self) -> Result<JsValue, String>;
}

pub trait AsJson {
    fn as_json(&self) -> JsonValue;
}

impl AsJson for Vec3 {
    fn as_json(&self) -> JsonValue {
        serde_json::json! {{
            "type": "Vec3",
            "value": [self.x, self.y, self.z],
        }}
    }
}

impl AsJson for Quat {
    fn as_json(&self) -> JsonValue {
        serde_json::json! {{
            "type": "Quat",
            "value": [self.x, self.y, self.z, self.w],
        }}
    }
}

impl AsJson for Transform {
    fn as_json(&self) -> JsonValue {
        serde_json::json! {{
            "type": "Transform",
            "value": {
                "translation": self.translation.as_json(),
                "rotation": self.rotation.as_json(),
                "scale": self.scale.as_json(),
            },
        }}
    }
}

impl AsJson for GlobalTransform {
    fn as_json(&self) -> JsonValue {
        let (scale, rotation, translation) = self.to_scale_rotation_translation();

        serde_json::json! {{
            "type": "GlobalTransform",
            "value": {
                "translation": translation.as_json(),
                "rotation": rotation.as_json(),
                "scale": scale.as_json(),
            },
        }}
    }
}

impl AsJson for serde_json::Map<String, JsonValue> {
    fn as_json(&self) -> JsonValue {
        JsonValue::Object(self.clone())
    }
}

impl<T> AsJsValue for T
where
    T: AsJson,
{
    fn as_js(&self) -> Result<JsValue, String> {
        self.as_json()
            .serialize(&SERIALIZER)
            .map_err(|err| format!("{err}"))
    }
}

impl<T> AsJsValue for Vec<T>
where
    T: Serialize,
{
    fn as_js(&self) -> Result<JsValue, String> {
        self.serialize(&SERIALIZER).map_err(|err| format!("{err}"))
    }
}

impl AsJsValue for JsonValue {
    fn as_js(&self) -> Result<JsValue, String> {
        self.serialize(&SERIALIZER).map_err(|err| format!("{err}"))
    }
}

impl AsJsValue for Entity {
    fn as_js(&self) -> Result<JsValue, String> {
        Ok(JsValue::from_str(&format!("{self:?}")))
    }
}

impl AsJsValue for String {
    fn as_js(&self) -> Result<JsValue, String> {
        Ok(JsValue::from_str(self))
    }
}

impl<T> AsJsValue for Option<T>
where
    T: AsJsValue,
{
    fn as_js(&self) -> Result<JsValue, String> {
        match self.as_ref() {
            Some(value) => Ok(value.as_js()?),
            None => Ok(JsValue::NULL),
        }
    }
}

impl AsJsValue for u32 {
    fn as_js(&self) -> Result<JsValue, String> {
        Ok(JsValue::from_f64(*self as f64))
    }
}

impl AsJsValue for usize {
    fn as_js(&self) -> Result<JsValue, String> {
        Ok(JsValue::from_f64(*self as f64))
    }
}

/// A crate-local version of `std::str::FromStr` that can be implemented for
/// foreign types
pub trait FromStr: Sized {
    type Err;

    fn from_str(s: &str) -> Result<Self, Self::Err>;
}

impl FromStr for Entity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fmt_err =
            || format!("Expected a string with format '<integer>v<integer>', found '{s}'");

        let mut split = s.split('v');

        let index = split
            .next()
            .unwrap()
            .parse::<u32>()
            .map_err(|_| fmt_err())?;

        let generation = split
            .next()
            .unwrap()
            .parse::<u32>()
            .map_err(|_| fmt_err())?;

        let bits = (generation as u64) << 32 | index as u64;

        Ok(Entity::from_bits(bits))
    }
}
