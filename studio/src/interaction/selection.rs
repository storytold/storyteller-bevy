use bevy::{
    asset::AssetPath,
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
    utils::HashSet,
};
use bevy_mod_outline::{
    AutoGenerateOutlineNormalsPlugin, OutlineBundle, OutlinePlugin, OutlineVolume,
};
use bevy_mod_picking::{prelude::Pointer, selection::Select, PickableBundle};

use crate::{
    input::{self, Action, InputAction, Lifecycle},
    scene::{ActiveScene, SceneElement, SceneSpawnerSystem, SceneState},
};

pub struct SelectionPlugin;
impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin))
            .add_systems(Update, update_selection)
            .add_systems(
                PostUpdate,
                draw_selection_outline.run_if(resource_changed_or_removed::<Selection>()),
            )
            .add_systems(
                OnEnter(SceneState::Active),
                (apply_deferred, tag_scene_elements)
                    .chain()
                    .after(SceneSpawnerSystem),
            );

        #[cfg(feature = "wasm")]
        app.add_systems(
            PostUpdate,
            wasm::notify_entity_selection_change
                .map(bevy::utils::error)
                .run_if(resource_changed_or_removed::<Selection>()),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct Selectable;

#[derive(Component, Clone, Copy)]
pub struct Selected;

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct Selection(Entity);

fn tag_scene_elements<'a>(
    mut cmd: Commands,
    r_active_scene: Res<ActiveScene>,
    ra_gltfs: Res<Assets<Gltf>>,
    ra_gltf_nodes: Res<Assets<GltfNode>>,
    ra_gltf_meshes: Res<Assets<GltfMesh>>,
    q_elements: Query<(Entity, &Name, &Children), (With<SceneElement>, Without<Selectable>)>,
    q_primitives: Query<&Handle<Mesh>>,
    // Local data structures to avoid excessive reallocations
    mut l_prims_from_gltf: Local<HashSet<AssetPath<'a>>>,
    mut l_prims_from_ecs: Local<HashSet<AssetPath<'a>>>,
) {
    let gltf = ra_gltfs.get(&r_active_scene.source).unwrap();

    for (ent, name, children) in q_elements.iter() {
        // See if this name matches any of the meshes or nodes in our glTF scene
        let Some(gltf_mesh) = gltf
            .named_meshes
            .get(&**name)
            .cloned()
            .or_else(|| {
                gltf.named_nodes.get(&**name).and_then(|node_handle| {
                    ra_gltf_nodes
                        .get(node_handle)
                        .and_then(|node| node.mesh.clone())
                })
            })
            .and_then(|handle| ra_gltf_meshes.get(&handle))
        else {
            continue;
        };

        // Collect all of the mesh handles in the glTF mesh
        l_prims_from_gltf.clear();
        l_prims_from_gltf.extend(
            gltf_mesh
                .primitives
                .iter()
                .filter_map(|prim| prim.mesh.path())
                .cloned(),
        );

        // Collect all of the mesh handles in this entity's children
        l_prims_from_ecs.clear();
        l_prims_from_ecs.extend(
            children
                .iter()
                .filter_map(|ent_child| {
                    q_primitives
                        .get(*ent_child)
                        .ok()
                        .and_then(|handle| handle.path())
                })
                .cloned(),
        );

        // If the sets match, this is the entity that represents the glTF mesh
        if *l_prims_from_gltf == *l_prims_from_ecs && !l_prims_from_gltf.is_empty() {
            cmd.entity(ent).insert(Selectable);

            for &ent_child in children
                .iter()
                .filter(|child| q_primitives.contains(**child))
            {
                cmd.entity(ent_child).insert(PickableBundle::default());
            }
        }
    }

    // Avoid holding any long-term references
    l_prims_from_gltf.clear();
    l_prims_from_ecs.clear();
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
    q_selected: Query<Entity, With<Selected>>,
) {
    if let Some(event) = ev_select.read().last() {
        // Clear any existing `Selected` components
        for ent in q_selected.iter() {
            cmd.entity(ent).remove::<Selected>();
        }

        // Find the nearest Selectable parent of the target entity
        let mut ent = event.target;
        let ent_selected = loop {
            if q_selectable.get(ent).is_ok() {
                break Some(ent);
            }

            if let Ok(parent) = q_parents.get(ent) {
                ent = parent.get();
            } else {
                break None;
            }
        };

        if let Some(ent) = ent_selected {
            cmd.insert_resource(Selection(ent));
            cmd.entity(ent).insert(Selected);
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

        for ent in q_selected.iter() {
            cmd.entity(ent).remove::<Selected>();
        }
    }
}

fn draw_selection_outline(
    mut cmd: Commands,
    q_selected: Query<&Children, With<Selected>>,
    q_meshes: Query<Entity, (With<Handle<Mesh>>, Without<OutlineVolume>)>,
    q_outlines: Query<Entity, With<OutlineVolume>>,
) {
    // Remove existing outlines
    for ent in q_outlines.iter() {
        cmd.entity(ent).remove::<OutlineBundle>();
    }

    for children in q_selected.iter() {
        for &child in children.iter() {
            if q_meshes.contains(child) {
                cmd.entity(child).insert(OutlineBundle {
                    outline: OutlineVolume {
                        visible: true,
                        width: 4.,
                        colour: Color::YELLOW,
                    },
                    ..default()
                });
            }
        }
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{
        prelude::{Query, Res, With},
        window::{PrimaryWindow, Window},
    };
    use wasm_bindgen::prelude::*;

    use super::Selection;
    use crate::wasm;

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
    export interface HTMLElementEventMap {
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
}
