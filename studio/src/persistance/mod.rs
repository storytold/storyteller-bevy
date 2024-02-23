use bevy::prelude::*;
use space_editor::prelude::*;

pub struct PersistancePlugin;

impl Plugin for PersistancePlugin {
    fn build(&self, app: &mut App) {
        info!("PersistancePlugin::build");
        app.editor_tab_by_trait(
            EditorTabName::Other("Persistance".to_string()),
            DebugPersistanceTab::default(),
        );

        #[cfg(feature = "wasm")]
        app.add_systems(Update, wasm::forward_save_scene_events)
            .add_systems(
                OnEnter(SaveState::Idle),
                wasm::on_scene_saved.map(bevy::utils::warn),
            );
    }
}

#[derive(Resource, Default)]
pub struct DebugPersistanceTab {}

impl EditorTab for DebugPersistanceTab {
    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        commands: &mut Commands,
        world: &mut World,
    ) {
        if ui.button("Save").clicked() {
            world.send_event(EditorEvent::Save(EditorPrefabPath::MemoryCahce));
        }

        if ui.button("Load").clicked() {
            world.send_event(EditorEvent::Load(EditorPrefabPath::MemoryCahce));
        }

        if ui.button("Clear level").clicked() {
            let mut query = world.query_filtered::<Entity, With<PrefabMarker>>();
            for e in query.iter(world) {
                commands.entity(e).despawn_recursive();
            }
        }
    }

    fn title(&self) -> bevy_inspector_egui::egui::WidgetText {
        "Persistance".into()
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, window::PrimaryWindow};
    use space_editor::prelude::{EditorEvent, EditorPrefabPath, PrefabMemoryCache};
    use wasm_bindgen::prelude::*;

    use crate::wasm::{self, StaticBuffer, WasmMessageBuffer};

    static SAVE_SCENE: WasmMessageBuffer<()> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = saveScene)]
    pub fn save_scene() {
        SAVE_SCENE.write_message(());
    }

    pub(super) fn forward_save_scene_events(mut ew_editor_events: EventWriter<EditorEvent>) {
        if SAVE_SCENE.take_message().is_some() {
            ew_editor_events.send(EditorEvent::Save(EditorPrefabPath::MemoryCahce));
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const SCENE_SAVED_EVENT: &str = r#"
export interface SceneSavedEvent extends CustomEvent {
    type: "scene-saved";
    detail: string;
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "scene-saved": SceneSavedEvent;
    }
}
    "#;

    pub(super) fn on_scene_saved(
        r_scene_cache: Res<PrefabMemoryCache>,
        r_type_registry: Res<AppTypeRegistry>,
        ra_scenes: Res<Assets<DynamicScene>>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        let handle = r_scene_cache
            .scene
            .as_ref()
            .ok_or_else(|| "Scene cache is empty!".to_string())?;

        let serialized = match ra_scenes.get(handle) {
            Some(scene) => scene
                .serialize_ron(&r_type_registry)
                .map_err(|err| format!("{err}")),
            None => Err("No scene found for cached handle!".into()),
        }?;

        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        wasm::dispatch_to_js(window, "scene-saved", &serialized)
    }
}
