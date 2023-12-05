use std::{cell::RefCell, sync::OnceLock};

use bevy::window::Window;
use parking_lot::ReentrantMutex;
use web_sys::{CustomEvent, CustomEventInit, Element};

use super::AsJsValue;

pub(crate) type WasmMessageBuffer<T> = OnceLock<ReentrantMutex<RefCell<Option<T>>>>;

pub(crate) trait StaticBuffer<T> {
    type Inner;

    fn inner(&self) -> &Self::Inner;
    fn write_message(&self, value: T);
    fn take_message(&self) -> Option<T>;
}

impl<T> StaticBuffer<T> for WasmMessageBuffer<T> {
    type Inner = ReentrantMutex<RefCell<Option<T>>>;

    #[inline]
    fn inner(&self) -> &Self::Inner {
        self.get_or_init(|| ReentrantMutex::new(RefCell::new(None)))
    }

    #[inline]
    fn write_message(&self, value: T) {
        let mutex = self.inner();
        if let Some(guard) = mutex.try_lock() {
            *guard.borrow_mut() = Some(value);
        }
    }

    #[inline]
    fn take_message(&self) -> Option<T> {
        let mutex = self.inner();
        mutex.try_lock().and_then(|guard| guard.borrow_mut().take())
    }
}

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
