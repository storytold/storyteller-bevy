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
            wasm::send_change_events::<Transform>.map(bevy::utils::error),
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
    use bevy::{prelude::*, window::PrimaryWindow};
    use wasm_bindgen::prelude::wasm_bindgen;

    use crate::wasm;

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
}
