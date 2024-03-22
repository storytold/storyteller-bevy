use std::sync::Arc;

use bevy::{prelude::*, render::mesh::skinning::SkinnedMesh};
use bevy_mod_outline::{
    AutoGenerateOutlineNormalsPlugin, OutlineBundle, OutlinePlugin, OutlineVolume,
};
use bevy_mod_picking::{prelude::Pointer, selection::Select, PickableBundle};
use space_editor::prelude::space_undo::{self, NewChange, RemovedEntity};

use crate::{
    input::{self, Action, InputAction, Lifecycle},
    scene::{SceneElement, SceneSpawnerSystem, SceneState},
};

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin))
            .add_systems(
                Update,
                (
                    update_selection,
                    process_delete_inputs.run_if(resource_exists::<Selection>()),
                ),
            )
            .add_systems(
                PostUpdate,
                draw_selection_outline.run_if(resource_changed_or_removed::<Selection>()),
            )
            .add_systems(First, finalize_delete)
            .add_systems(
                OnEnter(SceneState::Active),
                (apply_deferred, tag_scene_elements)
                    .chain()
                    .after(SceneSpawnerSystem),
            );

        app.add_systems(Update, tag_scene_elements);

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            wasm::update_selection_from_frontend.run_if(crate::is_editor),
        )
        .add_systems(
            PostUpdate,
            wasm::notify_entity_selection_change
                .map(bevy::utils::error)
                .run_if(resource_changed_or_removed::<Selection>()),
        );

        app.add_systems(PostUpdate, propagate_space_editor_to_studio);
    }
}

#[derive(Component, Clone, Copy, Reflect, Default)]
#[reflect(Component, Default)]
pub struct Selectable;

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct Selection(Entity);

fn propagate_space_editor_to_studio(
    mut cmd: Commands,
    r_selection: Option<Res<Selection>>,
    q_new_selected: Query<Entity, Added<space_editor::prelude::Selected>>,
    mut er_deselected: RemovedComponents<space_editor::prelude::Selected>,
    q_parents: Query<&Parent>,
    q_selectable: Query<&Selectable>,
) {
    let studio_selection_deselected_in_space_editor = r_selection
        .as_ref()
        .map(|selection| {
            er_deselected
                .read()
                .any(|deselected| deselected == ***selection)
        })
        .unwrap_or(false);

    if q_new_selected.is_empty() && studio_selection_deselected_in_space_editor {
        cmd.remove_resource::<Selection>();
    } else {
        let mut new_selected_iter = q_new_selected.iter();
        let Some(selected) = new_selected_iter.next().and_then(|se_selected| {
            nearest_selectable_parent(se_selected, &q_parents, &q_selectable)
        }) else {
            return;
        };

        if r_selection.is_none() || **r_selection.unwrap() != selected {
            cmd.insert_resource(Selection(selected));
        }

        if new_selected_iter.next().is_some() {
            warn!(
                "Multiple new entities were selected in Space Editor, but \
                Storyteller Studio doesn't currently support multiple selections"
            );
        }
    }
}

fn tag_scene_elements(
    mut cmd: Commands,
    q_scene_elements: Query<(Entity, Option<&Children>), (With<SceneElement>, Without<Selectable>)>,
    q_primitives: Query<&Handle<Mesh>>,
) {
    for (ent, children) in q_scene_elements.iter() {
        cmd.entity(ent).insert(Selectable).insert(PickableBundle::default());

        if let Some(children) = children {
            for &ent_child in children
                .iter()
                .filter(|child| q_primitives.contains(**child))
            {
                cmd.entity(ent_child).insert(PickableBundle::default());
            }
        }
    }
}

fn update_selection(
    mut cmd: Commands,
    // FIXME: These events aren't aware of our input actions, so we receive a
    //        selection event when LMB is released after orbiting the camera.
    //        Should drop bevy_mod_picking's Selection plugin/feature and just
    //        use our own conditional click listener.
    mut ev_select: EventReader<Pointer<Select>>,
    mut ev_input: EventReader<InputAction>,
    q_parents: Query<&Parent>,
    q_selectable: Query<&Selectable>,
) {
    if let Some(event) = ev_select.read().last() {
        // Find the nearest Selectable parent of the target entity
        if let Some(ent) = nearest_selectable_parent(event.target, &q_parents, &q_selectable) {
            cmd.insert_resource(Selection(ent));
        } else {
            cmd.remove_resource::<Selection>();
        }
    } else if ev_input.read().any(|action| {
        matches!(
            action,
            InputAction(Lifecycle::Stop, Action::Selection(input::Selection::Clear))
        )
    }) {
        cmd.remove_resource::<Selection>();
    }
}

fn nearest_selectable_parent(
    mut target: Entity,
    q_parents: &Query<&Parent>,
    q_selectable: &Query<&Selectable>,
) -> Option<Entity> {
    loop {
        if q_selectable.get(target).is_ok() {
            break Some(target);
        }

        if let Ok(parent) = q_parents.get(target) {
            target = parent.get();
        } else {
            break None;
        }
    }
}

#[derive(Component)]
pub struct MarkedForDeletion;

fn process_delete_inputs(
    mut cmd: Commands,
    mut er_input: EventReader<InputAction>,
    mut ew_changes: EventWriter<space_undo::NewChange>,
    r_selection: Res<Selection>,
) {
    if er_input.read().any(|action| {
        matches!(
            action,
            InputAction(Lifecycle::Stop, Action::Selection(input::Selection::Delete))
        )
    }) {
        let entity = **r_selection;

        cmd.entity(entity).insert(MarkedForDeletion);
        cmd.remove_resource::<Selection>();
        ew_changes.send(NewChange {
            change: Arc::new(RemovedEntity { entity }),
        });
    }
}

fn finalize_delete(mut cmd: Commands, q: Query<Entity, With<MarkedForDeletion>>) {
    for entity in &q {
        cmd.entity(entity).despawn_recursive();
    }
}

fn draw_selection_outline(
    mut cmd: Commands,
    r_selection: Option<Res<Selection>>,
    q_children: Query<&Children>,
    q_meshes: Query<
        Entity,
        (
            With<Handle<Mesh>>,
            Without<OutlineVolume>,
            // FIXME: It shouldn't be necessary to filter out SkinnedMesh here
            Without<SkinnedMesh>,
        ),
    >,
    q_outlines: Query<Entity, With<OutlineVolume>>,
) {
    // Remove existing outlines
    for ent in q_outlines.iter() {
        cmd.entity(ent).remove::<OutlineBundle>();
    }

    let Some(selection) = r_selection else {
        return;
    };

    let target = if q_meshes.contains(**selection) {
        Some(**selection)
    } else {
        q_children.get(**selection).ok().and_then(|children| {
            children
                .iter()
                .copied()
                .find(|&child| q_meshes.contains(child))
        })
    };

    if let Some(target) = target {
        cmd.entity(target).insert(OutlineBundle {
            outline: OutlineVolume {
                visible: true,
                width: 4.,
                colour: Color::YELLOW,
            },
            ..default()
        });
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{
        prelude::*,
        window::{PrimaryWindow, Window},
    };
    use wasm_bindgen::prelude::*;

    use super::Selection;
    use crate::wasm::{self, FromStr, StaticBuffer, WasmMessageBuffer};

    #[wasm_bindgen(typescript_custom_section)]
    const EVENT_DECLARATION: &str = r#"
/**
 * An event indicating that an entity has been selected or that the selection
 * has been cleared.
 */
export interface EntitySelectEvent extends CustomEvent {
    /** The selected entity ID. If `null`, the selection has been cleared. */
    detail: string | null;
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "entity-select": EntitySelectEvent;
    }
}
    "#;

    pub(super) fn notify_entity_selection_change(
        r_selection: Option<Res<Selection>>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        let entity = r_selection.map(|selection| **selection);
        wasm::dispatch_to_js(q_window.single(), "entity-select", &entity)
    }

    static SELECTION_REQUEST: WasmMessageBuffer<Entity> = WasmMessageBuffer::new();

    #[wasm_bindgen]
    pub fn select(entity: &str) {
        let entity = Entity::from_str(entity).unwrap();
        SELECTION_REQUEST.write_message(entity);
    }

    pub(super) fn update_selection_from_frontend(mut cmd: Commands) {
        if let Some(entity) = SELECTION_REQUEST.take_message() {
            cmd.insert_resource(Selection(entity));
        }
    }
}
