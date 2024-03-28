use bevy::{prelude::*, window::PrimaryWindow};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use super::{ImportSceneEvent, LoadingQueue, SceneState};
use crate::wasm::{self, StaticBuffer, WasmMessageBuffer};

static SPAWN_ASSETS: WasmMessageBuffer<Vec<AssetImport>> = WasmMessageBuffer::new();

#[wasm_bindgen(typescript_custom_section)]
const ASSET_IMPORT_TS: &str = r#"
export interface AssetImport {
    path: string;
    name?: string;
}
"#;
#[derive(Deserialize, Serialize)]
struct AssetImport {
    path: String,
    name: Option<String>,
}

#[wasm_bindgen(typescript_custom_section)]
const SPAWN_ASSET_TS: &str = r#"
export function spawnAsset(asset: AssetImport): void;
"#;
#[wasm_bindgen(js_name = spawnAsset, skip_typescript)]
pub fn spawn_asset(asset: JsValue) -> Result<(), JsValue> {
    let asset: AssetImport = serde_wasm_bindgen::from_value(asset)?;
    SPAWN_ASSETS.write_message(vec![asset]);
    Ok(())
}

#[wasm_bindgen(typescript_custom_section)]
const SPAWN_ASSETS_TS: &str = r#"
export function spawnAssets(assets: AssetImport[]): void;
"#;
#[wasm_bindgen(js_name = spawnAssets, skip_typescript)]
pub fn spawn_assets(assets: JsValue) -> Result<(), JsValue> {
    let assets: Vec<AssetImport> = serde_wasm_bindgen::from_value(assets)?;
    SPAWN_ASSETS.write_message(assets);
    Ok(())
}

pub(super) fn handle_spawn_asset_messages(mut ew_import_scene: EventWriter<ImportSceneEvent>) {
    if let Some(assets) = SPAWN_ASSETS.take_message() {
        for AssetImport { path, name } in assets {
            ew_import_scene.send(ImportSceneEvent { path, name });
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
