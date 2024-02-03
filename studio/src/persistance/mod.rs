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
