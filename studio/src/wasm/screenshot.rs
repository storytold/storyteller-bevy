//! Outline of the flow of events here:
//!
//! 1. [`crate::change_tracking`] system sends events to frontend to notify of
//!    tracked changes
//! 2. Frontend calls [`request_screenshot`] on a debounce, writing an empty
//!    message to [`SCREENSHOT_REQUEST`]
//! 3. [`take_screenshots`] watches the [`SCREENSHOT_REQUEST`] buffer and
//!    populates the [`SCREENSHOT`] buffer with a PNG-encoded `Vec<u8>`
//! 4. [`send_screenshots`] watches the [`SCREENSHOT`] buffer and sends an event
//!    to the frontend by converting the `Vec<u8>` to a [`js_sys::Uint8Array`]

use bevy::{prelude::*, render::view::screenshot::ScreenshotManager, window::PrimaryWindow};
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

use super::{AsJsValue, StaticBuffer, WasmMessageBuffer};

static SCREENSHOT_REQUEST: WasmMessageBuffer<()> = WasmMessageBuffer::new();
static SCREENSHOT: WasmMessageBuffer<ScreenshotNative> = WasmMessageBuffer::new();

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                take_screenshots.map(bevy::utils::error),
                send_screenshots.map(bevy::utils::error),
            ),
        );
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Screenshot {
    data_pvt: js_sys::Uint8Array,
    pub width: usize,
    pub height: usize,
}

// This intermediate type is necessary because the `Screenshot` struct
// cannot be safely stored in a `WasmMessageBuffer`
struct ScreenshotNative {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

impl From<ScreenshotNative> for Screenshot {
    fn from(value: ScreenshotNative) -> Self {
        Self {
            data_pvt: js_sys::Uint8Array::from(&value.data[..]),
            width: value.width,
            height: value.height,
        }
    }
}

#[wasm_bindgen]
impl Screenshot {
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> js_sys::Uint8Array {
        self.data_pvt.clone()
    }
}

impl AsJsValue for Screenshot {
    fn as_js(&self) -> Result<JsValue, String> {
        Ok(JsValue::from(self.clone()))
    }
}

#[wasm_bindgen(js_name = requestScreenshot)]
pub fn request_screenshot() {
    SCREENSHOT_REQUEST.write_message(());
}

fn take_screenshots(
    mut r_screenshot_mgr: ResMut<ScreenshotManager>,
    q_window: Query<Entity, With<PrimaryWindow>>,
) -> Result<(), String> {
    if SCREENSHOT_REQUEST.take_message().is_none() {
        return Ok(());
    }

    let Ok(window_ent) = q_window.get_single() else {
        return Ok(());
    };

    r_screenshot_mgr
        .take_screenshot(window_ent, |img| {
            let img = match img.try_into_dynamic().map(|img| img.to_rgb8()) {
                Ok(img) => img,
                Err(err) => {
                    error!("{err}");
                    return;
                }
            };

            let channel_count = <image::Rgb<u8> as image::Pixel>::CHANNEL_COUNT as usize;
            let buffer_len = match Some(channel_count)
                .and_then(|size| size.checked_mul(img.width() as usize))
                .and_then(|size| size.checked_mul(img.height() as usize))
            {
                Some(size) => size,
                None => {
                    error!("Failed to compute buffer length for image!");
                    return;
                }
            };
            let src_buffer = &img.as_raw()[..buffer_len];

            let mut png_bytes = vec![];
            if let Err(err) = PngEncoder::new(&mut png_bytes).write_image(
                src_buffer,
                img.width(),
                img.height(),
                ColorType::Rgb8,
            ) {
                error!("{err}");
                return;
            }

            SCREENSHOT.write_message(ScreenshotNative {
                data: png_bytes,
                width: img.width() as usize,
                height: img.height() as usize,
            });
        })
        .map_err(|err| format!("{err}"))
}

#[wasm_bindgen(typescript_custom_section)]
const SCREENSHOT_READY_EVENT: &str = r#"
export interface ScreenshotReadyEvent extends CustomEvent {
    type: "screenshot-ready";
    detail: Screenshot;
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "screenshot-ready": ScreenshotReadyEvent;
    }
}
    "#;

fn send_screenshots(q_window: Query<&Window, With<PrimaryWindow>>) -> Result<(), String> {
    let Some(screenshot) = SCREENSHOT.take_message() else {
        return Ok(());
    };

    let Ok(window) = q_window.get_single() else {
        return Ok(());
    };

    let screenshot = Screenshot::from(screenshot);

    super::dispatch_to_js(window, "screenshot-ready", &screenshot)
}
