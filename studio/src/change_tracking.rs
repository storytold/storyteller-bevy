use bevy::prelude::*;

pub struct ChangeTrackingPlugin;

impl Plugin for ChangeTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EntityChangedEvent>();

        // Add more of these systems for each component that we want to track
        app.add_systems(Update, send_change_events::<Transform>);

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            (
                wasm::send_change_events::<Transform>.map(bevy::utils::error),
                wasm::take_screenshots.map(bevy::utils::error),
                wasm::send_screenshots.map(bevy::utils::error),
            ),
        );
    }
}

#[derive(Component, Debug)]
pub struct ChangeTracker;

fn send_change_events<T: Component>(
    query: Query<Entity, (Changed<T>, With<ChangeTracker>)>,
    mut entity_change_writer: EventWriter<EntityChangedEvent>,
) {
    for entity in query.iter() {
        entity_change_writer.send(EntityChangedEvent(entity));
    }
}

#[derive(Event, Deref)]
pub struct EntityChangedEvent(pub Entity);

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, render::view::screenshot::ScreenshotManager, window::PrimaryWindow};
    use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
    use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

    use crate::wasm::{self, AsJsValue, StaticBuffer, WasmMessageBuffer};

    use super::ChangeTracker;

    #[wasm_bindgen(typescript_custom_section)]
    const ENTITY_CHANGES_EVENT: &str = r#"
export interface EntityChangesEvent extends CustomEvent {
    type: "entity-changes";
    detail: string[];
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "entity-changes": EntityChangesEvent;
    }
}
    "#;

    pub(super) fn send_change_events<T: Component>(
        q_changed_ents: Query<Entity, (Changed<T>, With<ChangeTracker>)>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        if q_changed_ents.is_empty() {
            return Ok(());
        }

        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        let entities = q_changed_ents
            .iter()
            .map(|ent| format!("{ent:?}"))
            .collect::<Vec<_>>();

        wasm::dispatch_to_js(window, "entity-changes", &entities)
    }

    // FIXME: Everything below this line most definitely does not belong in this
    //        module

    // Outline of the flow of events here:
    //
    // 1. Change-tracking system sends event to frontend to notify of changes
    // 2. Frontend calls `request_screenshot` on a debounce, writing an empty
    //    message to `REQUEST_SCREENSHOT`
    // 3. `take_screenshots` watches the `REQUEST_SCREENSHOT` buffer and
    //    populates `SCREENSHOT` buffer with PNG-encoded Vec<u8>
    // 4. `send_screenshots` watches the `SCREENSHOT` buffer and sends an event
    //    to the frontend by converting the Vec<u8> to a Uint8Array

    static SCREENSHOT_REQUEST: WasmMessageBuffer<()> = WasmMessageBuffer::new();
    static SCREENSHOT: WasmMessageBuffer<ScreenshotNative> = WasmMessageBuffer::new();

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

    pub(super) fn take_screenshots(
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

    pub(super) fn send_screenshots(
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        let Some(screenshot) = SCREENSHOT.take_message() else {
            return Ok(());
        };

        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        let screenshot = Screenshot::from(screenshot);

        wasm::dispatch_to_js(window, "screenshot-ready", &screenshot)
    }
}
