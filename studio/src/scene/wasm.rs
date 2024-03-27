use bevy::{prelude::*, window::PrimaryWindow};
use wasm_bindgen::prelude::*;

use super::{ImportSceneEvent, LoadingQueue, SceneState};
use crate::wasm::{self, StaticBuffer, WasmMessageBuffer};

static SPAWN_SCENES: WasmMessageBuffer<Vec<String>> = WasmMessageBuffer::new();

#[wasm_bindgen(js_name = spawnScene)]
pub fn spawn_scene(path: String) {
    SPAWN_SCENES.write_message(vec![path]);
}

#[wasm_bindgen(js_name = spawnScenes)]
pub fn spawn_scenes(paths: Vec<String>) {
    SPAWN_SCENES.write_message(paths);
}

pub(super) fn handle_spawn_scene_messages(mut ew_import_scene: EventWriter<ImportSceneEvent>) {
    if let Some(paths) = SPAWN_SCENES.take_message() {
        for path in paths {
            ew_import_scene.send(ImportSceneEvent(path));
        }
    }
}

#[wasm_bindgen(typescript_custom_section)]
const SCENE_STATE_EVENT: &str = r#"
export interface SceneStateEvent extends CustomEvent {
    type: "scene-state";
    detail: SceneState;
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "scene-state": SceneStateEvent;
    }
}
"#;

pub(super) fn notify_state_changes(
    r_state: Res<State<SceneState>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) -> Result<(), String> {
    if let Ok(window) = q_window.get_single() {
        wasm::dispatch_to_js(window, "scene-state", &(**r_state as u32))?;
    }
    Ok(())
}

#[wasm_bindgen(typescript_custom_section)]
const LOADING_QUEUE_EVENT: &str = r#"
export interface LoadingQueueEvent extends CustomEvent {
    type: "loading-queue";
    detail: number;
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "loading-queue": LoadingQueueEvent;
    }
}
"#;

pub(super) fn notify_loading_queue_changes(
    mut l_last_sent: Local<Option<usize>>,
    r_loading: Res<LoadingQueue>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) -> Result<(), String> {
    let queue_len = r_loading.queue.len();
    if let Some(last_sent) = *l_last_sent {
        if last_sent == queue_len {
            return Ok(());
        }
    }

    let Ok(window) = q_window.get_single() else {
        return Ok(());
    };

    wasm::dispatch_to_js(window, "loading-queue", &queue_len)?;
    *l_last_sent = Some(queue_len);

    Ok(())
}
