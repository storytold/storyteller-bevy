use core::fmt;

use bevy::window::WindowResolution;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ViewportSize {
    pub width: f32,
    pub height: f32,
}

#[wasm_bindgen]
impl ViewportSize {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl Default for ViewportSize {
    fn default() -> Self {
        WindowResolution::default().into()
    }
}

impl From<ViewportSize> for WindowResolution {
    fn from(value: ViewportSize) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<WindowResolution> for ViewportSize {
    fn from(value: WindowResolution) -> Self {
        Self {
            width: value.width(),
            height: value.height(),
        }
    }
}

impl fmt::Debug for ViewportSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("({:.2}, {:.2})", self.width, self.height))
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use bevy::{prelude::*, render::camera::Viewport};
    use wasm_bindgen::prelude::*;

    use crate::{
        wasm::{StaticBuffer, WasmMessageBuffer},
        MainCamera,
    };

    use super::ViewportSize;

    static RESIZE_BUF: WasmMessageBuffer<ViewportSize> = WasmMessageBuffer::new();

    #[wasm_bindgen]
    pub fn resize(viewport_size: ViewportSize) {
        RESIZE_BUF.write_message(viewport_size);
    }

    pub struct ViewportPlugin;

    impl Plugin for ViewportPlugin {
        fn build(&self, app: &mut App) {
            app.add_event::<ViewportResizeEvent>().add_systems(
                PreUpdate,
                (dispatch_resize_events, sync_window_to_viewport).chain(),
            );
        }
    }

    #[derive(Event, Clone, Copy)]
    struct ViewportResizeEvent(ViewportSize);

    fn sync_window_to_viewport(
        mut events: EventReader<ViewportResizeEvent>,
        mut window: Query<&mut Window>,
        mut q_camera: Query<&mut Camera, With<MainCamera>>,
    ) {
        if let Some(event) = events.read().last() {
            let ViewportSize { width, height } = event.0;
            let mut window = window.single_mut();
            let scale_factor = window.scale_factor() as f32;
            window
                .resolution
                .set(width * scale_factor, height * scale_factor);
            q_camera.single_mut().viewport = Some(Viewport {
                physical_size: UVec2::new(
                    (width * scale_factor) as u32,
                    (height * scale_factor) as u32,
                ),
                ..Default::default()
            });
            info!("Viewport size: ({:.2}, {:.2})", width, height);
        }
    }

    fn dispatch_resize_events(mut events: EventWriter<ViewportResizeEvent>) {
        if let Some(resize) = RESIZE_BUF.take_message() {
            events.send(ViewportResizeEvent(resize));
            info!(
                "Dispatch Viewport size: ({:.2}, {:.2})",
                resize.width, resize.height
            );
        }
    }
}
