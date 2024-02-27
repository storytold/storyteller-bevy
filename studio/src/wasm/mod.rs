use std::{cell::RefCell, sync::OnceLock};

use parking_lot::ReentrantMutex;
use wasm_bindgen::prelude::wasm_bindgen;

mod conv;
mod init;
#[cfg(feature = "wasm_screenshot")]
mod screenshot;
mod util;

pub use conv::*;
#[cfg(feature = "wasm_screenshot")]
pub use screenshot::ScreenshotPlugin;
pub(crate) use util::*;

static INITIALIZED: OnceLock<ReentrantMutex<RefCell<bool>>> = OnceLock::new();

/// The Studio application must only be initialized once (via `init` or
/// `startViewer`). Multiple calls to either of those functions will cause the
/// application to panic. This function can be used to query whether the app has
/// already been initialized.
#[wasm_bindgen(js_name = isInitialized)]
pub fn is_initialized() -> bool {
    let wrapper = INITIALIZED.get_or_init(|| ReentrantMutex::new(RefCell::new(false)));
    let guard = wrapper.lock();
    let result = *guard.borrow();

    result
}

pub(crate) fn initialize() {
    let wrapper = INITIALIZED.get_or_init(|| ReentrantMutex::new(RefCell::new(false)));
    let guard = wrapper.lock();
    *guard.borrow_mut() = true;
}
