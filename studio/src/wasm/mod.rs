use std::{cell::RefCell, sync::OnceLock};

use bevy::window::Window;
use parking_lot::ReentrantMutex;
use web_sys::{CustomEvent, CustomEventInit, Element};

mod conv;

pub use conv::*;

pub(crate) type WasmMessageBuffer<T> = OnceLock<ReentrantMutex<RefCell<Option<T>>>>;

pub(crate) fn dispatch_to_js(
    window: &Window,
    event_type: &str,
    // NOTE: We use dynamic dispatch here to keep the wasm binary small(-ish)
    value: &dyn AsJsValue,
) -> Result<(), String> {
    let canvas = try_get_canvas(window)?;

    let detail = value.as_js()?;
    let event = CustomEvent::new_with_event_init_dict(
        event_type,
        CustomEventInit::new()
            .bubbles(true)
            .cancelable(true)
            .detail(&detail),
    )
    .map_err(|err| format!("{err:?}"))?;

    canvas
        .dispatch_event(&event)
        .map_err(|err| format!("{err:?}"))?;

    Ok(())
}

fn try_get_canvas(window: &Window) -> Result<Element, String> {
    let canvas_selector = window
        .canvas
        .as_ref()
        .ok_or("Canvas selector not set on Bevy Window".to_string())?;

    let window = web_sys::window().ok_or("web_sys::window() returned None".to_string())?;
    let document = window
        .document()
        .ok_or("window.document() returned None".to_string())?;

    match document.query_selector(canvas_selector) {
        Ok(Some(canvas)) => Ok(canvas),
        Ok(None) => Err(format!(
            "Failed to find element with selector '{canvas_selector}'"
        )),
        Err(err) => Err(format!("JavaScript threw an Error: {err:?}")),
    }
}

macro_rules! get_buffer {
    ($buffer:ident) => {
        $buffer.get_or_init(|| {
            ::parking_lot::ReentrantMutex::new(::std::cell::RefCell::new(
                ::core::option::Option::None,
            ))
        })
    };
}

macro_rules! write_message {
    ($buffer:ident, $expr:expr) => {{
        let buffer = crate::wasm::get_buffer!($buffer);
        if let Some(lock) = buffer.try_lock() {
            *lock.borrow_mut() = Some($expr);
        }
    }};
}

macro_rules! read_buffer {
    ($buffer:ident, |$msg:ident| $stmt:stmt) => {{
        let buffer = crate::wasm::get_buffer!($buffer);
        if let Some(lock) = buffer.try_lock() {
            if let Some($msg) = lock.borrow_mut().take() {
                $stmt
            }
        }
    }};
}

pub(crate) use get_buffer;
pub(crate) use read_buffer;
pub(crate) use write_message;
