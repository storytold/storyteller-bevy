use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{InitAsset, InitParams, StudioMode, ViewportSize};

#[wasm_bindgen(typescript_custom_section)]
const WASM_ENTRY_POINTS: &str = r#"
export interface InitOptions {
    mode: StudioMode;
    /** Required for all entry-points except `initFromStorytellerScene()` */
    skyboxId?: string;
    canvasSelector: string;
    canvasAlt: string;
    viewportSize: ViewportSize;
}

export function initFromLibrary(objectId: string, options: InitOptions): void;
export function initFromBvh(animPath: string, options: InitOptions): void;
export function initFromMixamo(animPath: string, options: InitOptions): void;
export function initFromSceneImport(scenePath: string, options: InitOptions): void;
export function initFromStorytellerScene(scenePath: string, options: InitOptions): void;
"#;

#[derive(Serialize, Deserialize)]
struct WasmInitOptions {
    #[serde(rename = "skyboxId")]
    skybox_id: Option<String>,
    mode: StudioMode,
    #[serde(rename = "canvasSelector")]
    canvas_selector: String,
    #[serde(rename = "canvasAlt")]
    canvas_alt: String,
    #[serde(rename = "viewportSize")]
    viewport_size: ViewportSize,
}

#[wasm_bindgen(skip_typescript, js_name = initFromLibrary)]
pub fn init_from_library(object_id: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromLibrary`");

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::LibraryObject(object_id),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromBvh)]
pub fn init_from_bvh(anim_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromBvh`");

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::BvhAnim(anim_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromMixamo)]
pub fn init_from_mixamo(anim_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromMixamo`");

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::MixamoAnim(anim_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromSceneImport)]
pub fn init_from_scene_import(scene_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;
    let skybox_id = options
        .skybox_id
        .expect("`skyboxId` is required for `initFromSceneImport`");

    crate::start(
        InitParams::FromAsset {
            asset: InitAsset::ImportedScene(scene_path),
            skybox_id,
        },
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}

#[wasm_bindgen(skip_typescript, js_name = initFromStorytellerScene)]
pub fn init_from_storyteller_scene(scene_path: String, options: JsValue) -> Result<(), JsValue> {
    let options = serde_wasm_bindgen::from_value::<WasmInitOptions>(options)?;

    crate::start(
        InitParams::FromScene(scene_path),
        options.mode,
        Some(Window {
            resolution: options.viewport_size.into(),
            title: options.canvas_alt,
            canvas: Some(options.canvas_selector),
            ..default()
        }),
    )
    .run();

    Ok(())
}
