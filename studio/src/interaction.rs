use bevy::prelude::*;
use bevy_mod_picking::{prelude::Pointer, selection::Select};

use crate::camera_controller::CameraMotionSystem;

#[derive(Component, Clone, Copy)]
pub struct Selectable;

#[derive(Resource, Deref, DerefMut)]
pub struct Selection(Entity);

pub struct InteractionPlugin;
impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_selection,
                draw_selection_axes
                    .run_if(resource_exists::<Selection>())
                    .after(CameraMotionSystem),
            ),
        );
    }
}

fn update_selection(
    mut cmd: Commands,
    mut ev_select: EventReader<Pointer<Select>>,
    q_parents: Query<&Parent>,
    q_selectable: Query<&Selectable>,
) {
    let Some(event) = ev_select.iter().last() else {
        return;
    };

    // Find the nearest Selectable parent of the target entity
    let mut ent = event.target;
    let selected = loop {
        if q_selectable.get(ent).is_ok() {
            break Some(ent);
        }

        if let Ok(parent) = q_parents.get(ent) {
            ent = parent.get();
        } else {
            break None;
        }
    };

    if let Some(ent) = selected {
        cmd.insert_resource(Selection(ent));
    } else {
        cmd.remove_resource::<Selection>();
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
