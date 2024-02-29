use std::collections::VecDeque;

use bevy::{prelude::*, utils::HashSet, window::PrimaryWindow};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{scene::SceneElement, wasm};

#[wasm_bindgen(typescript_custom_section)]
const TYPE_DECLARATIONS: &str = r#"
export interface SceneObject {
    /** The entity ID. */
    id: string;
    /** A user-friendly name for displaying in the UI */
    name: string;
    type: SceneElement;
    /** The IDs of this object's children in the hierarchy */
    children: string[];
    /** The ID of this object's parent in the hierarchy, if it has one */
    parent: string | null;
    /** The local-space transform */
    transform: Transform;
    /** The world-space transform */
    globalTransform: GlobalTransform;
}

/**
 * An event indicating that a new entity has spawned.
 */
export interface EntitySpawnEvent extends CustomEvent {
    type: "entity-spawn";
    detail: SceneObject;
}

/**
 * An event indicating that an entity has been de-spawned.
 */
export interface EntityDespawnEvent extends CustomEvent {
    type: "entity-despawn";
    /** The entity ID */
    detail: string;
}

/**
 * An event indicating that multiple entities have spawned.
 */
export interface EntityMultiSpawnEvent extends CustomEvent {
    type: "entity-multi-spawn";
    detail: SceneObject[];
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "entity-spawn": EntitySpawnEvent;
        "entity-despawn": EntityDespawnEvent;
        "entity-multi-spawn": EntityMultiSpawnEvent;
    }
}
    "#;

#[derive(Serialize)]
pub(super) struct SceneObject {
    id: String,
    name: String,
    #[serde(rename = "type")]
    type_: SceneElement,
    children: Vec<String>,
    parent: Option<String>,
}

struct SceneObjectBuilder {
    id: Entity,
    name: Option<String>,
    type_: Option<SceneElement>,
    children: Option<Vec<Entity>>,
    parent: Option<Entity>,
}

impl SceneObjectBuilder {
    fn new(id: Entity) -> Self {
        Self {
            id,
            name: None,
            type_: None,
            children: None,
            parent: None,
        }
    }
    fn finish(self) -> SceneObject {
        SceneObject {
            id: format!("{:?}", self.id),
            name: self.name.unwrap(),
            type_: self.type_.unwrap(),
            children: self
                .children
                .map(|children| children.iter().map(|ent| format!("{ent:?}")).collect())
                .unwrap_or_default(),
            parent: self.parent.map(|ent| format!("{ent:?}")),
        }
    }
    fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    fn with_type(mut self, type_: SceneElement) -> Self {
        self.type_ = Some(type_);
        self
    }
    fn with_children(mut self, children: Vec<Entity>) -> Self {
        self.children = Some(children);
        self
    }
    fn with_parent(mut self, parent: Entity) -> Self {
        self.parent = Some(parent);
        self
    }
}

impl SceneObject {
    fn build(id: Entity) -> SceneObjectBuilder {
        SceneObjectBuilder::new(id)
    }
}

pub(super) fn notify_entity_spawn_changes(
    mut l_notified: Local<HashSet<Entity>>,
    // Note: We use locals for these data structures purely to reuse
    // existing allocations across multiple invocations of this system
    mut l_new_objects: Local<Vec<SceneObject>>,
    mut l_despawned_entities: Local<Vec<Entity>>,
    mut l_missing_parents: Local<VecDeque<Entity>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_inspectables: Query<(Entity, &SceneElement)>,
    q_element_changes: Query<Entity, Changed<SceneElement>>,
    q_others: Query<Entity, Without<SceneElement>>,
    q_names: Query<&Name>,
    q_parents: Query<&Children>,
    q_children: Query<&Parent>,
) -> Result<(), String> {
    l_missing_parents.clear();

    l_new_objects.clear();
    l_new_objects.extend(q_inspectables.iter().filter_map(|(ent, &type_)| {
        build_scene_object(
            &mut l_notified,
            &mut l_missing_parents,
            ent,
            type_,
            &q_inspectables,
            &q_element_changes,
            &q_names,
            &q_parents,
            &q_children,
        )
    }));

    while let Some(parent) = l_missing_parents.pop_front() {
        let Ok(ent) = q_others.get(parent) else {
            error!("Failed to find missing parent: {parent:?}");
            l_notified.insert(parent);

            continue;
        };

        if let Some(object) = build_scene_object(
            &mut l_notified,
            &mut l_missing_parents,
            ent,
            SceneElement::Generic,
            &q_inspectables,
            &q_element_changes,
            &q_names,
            &q_parents,
            &q_children,
        ) {
            l_new_objects.push(object);
        }
    }

    l_despawned_entities.clear();
    l_despawned_entities.extend(
        l_notified
            .iter()
            .filter(|ent| !q_inspectables.contains(**ent) && !q_others.contains(**ent))
            .cloned(),
    );

    let win = q_window.single();

    for ent in l_despawned_entities.iter() {
        // TODO: Make this take an array like the multi-spawn event
        wasm::dispatch_to_js(win, "entity-despawn", ent)?;
        l_notified.remove(ent);
    }

    if !l_new_objects.is_empty() {
        // TODO: Remove the single-spawn event
        wasm::dispatch_to_js(win, "entity-multi-spawn", &*l_new_objects)?;
    }

    Ok(())
}

fn build_scene_object(
    notified: &mut HashSet<Entity>,
    missing_parents: &mut VecDeque<Entity>,
    ent: Entity,
    type_: SceneElement,
    q_inspectables: &Query<(Entity, &SceneElement)>,
    q_element_changes: &Query<Entity, Changed<SceneElement>>,
    q_names: &Query<&Name>,
    q_parents: &Query<&Children>,
    q_children: &Query<&Parent>,
) -> Option<SceneObject> {
    if notified.contains(&ent) && !q_element_changes.contains(ent) {
        return None;
    }

    notified.insert(ent);

    let mut result = SceneObject::build(ent).with_type(type_);

    if let Ok(children) = q_parents.get(ent) {
        result = result.with_children(children.iter().cloned().collect());
    }
    if let Ok(parent) = q_children.get(ent) {
        let parent_ent = **parent;
        result = result.with_parent(parent_ent);

        if !notified.contains(&parent_ent) && !q_inspectables.contains(parent_ent) {
            missing_parents.push_back(parent_ent);
        }
    }

    result = result.with_name(match q_names.get(ent) {
        Ok(name) => name.into(),
        Err(_) => format!("{ent:?}"),
    });

    Some(result.finish())
}
