use bevy::{ecs::query::{ReadOnlyWorldQuery}, prelude::*};
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "wasm")]
use crate::wasm;


#[wasm_bindgen(typescript_custom_section)]
const SCENE_STATE_EVENT: &str = r#"

export interface EntityChange extends CustomEvent {
    type: "entity-change";
    detail: string;
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "entity-change": EntityChange;
    }
}

"#;

#[derive(Debug)]
pub struct ChangeTrackingPlugin;

impl Plugin for ChangeTrackingPlugin {
    fn build(&self, app: &mut App) {
        //Add more of these systems for each component that we want to track
        app.add_systems(Update, send_change_events::<Transform>);
    }
}

#[derive(Component, Debug)]
pub struct ChangeTracker;

fn send_change_events<T: Component>(
    query : Query<Entity, (Changed<T>, With<ChangeTracker>)>,
    mut entity_change_writer : EventWriter<EntityChangedEvent>
) {
    for entity in query.iter() { 
        entity_change_writer.send(EntityChangedEvent(entity));
        
        #[cfg(feature = "wasm")]
        wasm::dispatch_to_js(window, "scene-state", &(**r_state as u32))?;
    }
}

#[derive(Event)]
pub struct EntityChangedEvent(Entity);