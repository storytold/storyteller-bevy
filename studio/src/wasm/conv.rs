use bevy::prelude::Entity;
use wasm_bindgen::JsValue;

pub trait AsJsValue {
    fn as_js(&self) -> JsValue;
}

// TODO: Try a blanket impl<T> AsJsValue for T where T: serde::Serialize

impl AsJsValue for Entity {
    fn as_js(&self) -> JsValue {
        self.to_bits().into()
    }
}

impl<T> AsJsValue for Option<T>
where
    T: AsJsValue,
{
    fn as_js(&self) -> JsValue {
        match self.as_ref() {
            Some(value) => value.as_js(),
            None => JsValue::NULL,
        }
    }
}
