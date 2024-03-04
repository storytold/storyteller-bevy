use std::sync::Arc;

use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use space_editor::prelude::{
    space_undo::{AddedEntity, NewChange},
    *,
};
use wasm_bindgen::prelude::*;

use crate::{
    interaction::Selectable,
    scene::SceneElement,
    wasm::{StaticBuffer, WasmMessageBuffer},
};

pub struct UiBlockingPlugin;

impl Plugin for UiBlockingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, read_command_system);
    }
}

fn read_command_system(mut command: Commands, mut change_event: EventWriter<NewChange>) {
    // info!("Blocking command");
    let Some(cmd) = COMMAND.take_message() else {
        info!("No blocking command");
        return;
    };

    let Some(cmd) = cmd else {
        info!("No blocking command found");
        return;
    };

    let id;
    if cmd == "cube" {
        id = command
            .spawn((
                SpatialBundle::default(),
                MeshPrimitivePrefab::Cube(1.0),
                PrefabMarker,
                Selectable,
                SceneElement::Mesh,
                PickableBundle::default(),
                Name::new("Cube"),
            ))
            .id();
    } else if cmd == "sphere" {
        id = command
            .spawn((
                SpatialBundle::default(),
                MeshPrimitivePrefab::Sphere(SpherePrefab { r: 1.0 }),
                PrefabMarker,
                Selectable,
                SceneElement::Mesh,
                PickableBundle::default(),
                Name::new("Sphere"),
            ))
            .id();
    } else if cmd == "torus" {
        id = command
            .spawn((
                SpatialBundle::default(),
                MeshPrimitivePrefab::Torus(TorusPrefab::default()),
                PrefabMarker,
                Selectable,
                SceneElement::Mesh,
                PickableBundle::default(),
                Name::new("Torus"),
            ))
            .id();
    } else if cmd == "cylinder" {
        id = command
            .spawn((
                SpatialBundle::default(),
                MeshPrimitivePrefab::Cylinder(CylinderPrefab::default()),
                PrefabMarker,
                Selectable,
                SceneElement::Mesh,
                PickableBundle::default(),
                Name::new("Cylinder"),
            ))
            .id();
    } else {
        error!("Unknown blocking command: {}", cmd);
        return;
    }

    change_event.send(NewChange {
        change: Arc::new(AddedEntity { entity: id }),
    });
}

#[derive(Clone, Copy, Debug)]
#[wasm_bindgen]
pub enum BlockingCommand {
    Cube,
    Sphere,
    Torus,
    Cylinder,
}

static COMMAND: WasmMessageBuffer<BlockingCommand> = WasmMessageBuffer::new();

#[wasm_bindgen(js_name = dispatchBlockingCommand)]
pub fn dispatch_command(command: BlockingCommand) {
    info!("Dispatching blocking command: {command:?}");
    COMMAND.write_message(command);
}
