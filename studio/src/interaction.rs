use bevy::prelude::*;
use bevy_mod_outline::{
    AutoGenerateOutlineNormalsPlugin, OutlineBundle, OutlinePlugin, OutlineVolume,
};
use bevy_mod_picking::{prelude::Pointer, selection::Select};

use crate::{
    camera_controller::CameraMotionSystem,
    debug,
    input::{self, Action, InputAction, Lifecycle},
};

#[derive(Component, Clone, Copy)]
pub struct Selectable;

#[derive(Component, Clone, Copy)]
pub struct Selected;

#[derive(Resource, Deref, DerefMut)]
pub struct Selection(Entity);

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((OutlinePlugin, AutoGenerateOutlineNormalsPlugin));
        app.add_systems(Update, update_selection);
        app.add_systems(
            PostUpdate,
            (
                draw_selection_outline.run_if(resource_changed_or_removed::<Selection>()),
                draw_selection_axes
                    .run_if(resource_exists::<Selection>())
                    .run_if(debug::is_debug_mode)
                    .after(CameraMotionSystem),
            ),
        );
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
    q_selected: Query<Entity, With<Selected>>,
) {
    if let Some(event) = ev_select.iter().last() {
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
    } else if ev_input.iter().any(|action| {
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

fn draw_selection_axes(
    mut gizmos: Gizmos,
    selection: Res<Selection>,
    q_transform: Query<&GlobalTransform>,
) {
    let Ok(xform) = q_transform.get(**selection) else {
        return;
    };

    let origin = xform.translation();

    let mut draw_axis = |direction: Vec3, color: Color| {
        let end = origin + direction * 0.25;
        gizmos.line(origin, end, color);
    };

    draw_axis(xform.right(), Color::RED);
    draw_axis(xform.up(), Color::GREEN);
    draw_axis(xform.back(), Color::BLUE);
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
