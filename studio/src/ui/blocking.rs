use bevy::prelude::*;
use bevy_mod_picking::PickableBundle;
use space_editor::prelude::*;
use wasm_bindgen::prelude::*;

use crate::{interaction::Selectable, scene::SceneElement, wasm::{StaticBuffer, WasmMessageBuffer}};

pub struct UiBlockingPlugin;

impl Plugin for UiBlockingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, read_command_system);
    }
}

fn read_command_system(
    mut command: Commands
) {
    // info!("Blocking command");
    let Some(cmd) = COMMAND.take_message() else {
        info!("No blocking command");
        return;
    };

    let Some(cmd) = cmd else {
        info!("No blocking command found");
        return;
    };

    if cmd == "cube" {
        command.spawn((
            SpatialBundle::default(),
            MeshPrimitivePrefab::Cube(1.0),
            PrefabMarker,
            Selectable,
            SceneElement::Mesh,
            PickableBundle::default()

        ));
    } else if cmd == "sphere" {
        command.spawn((
            SpatialBundle::default(),
            MeshPrimitivePrefab::Sphere(SpherePrefab { r: 1.0 }),
            PrefabMarker,
            Selectable,
            SceneElement::Mesh,
            PickableBundle::default()
        ));
    } else if cmd == "torus" {
        command.spawn((
            SpatialBundle::default(),
            MeshPrimitivePrefab::Torus(TorusPrefab::default()),
            PrefabMarker,
            Selectable,
            SceneElement::Mesh,
            PickableBundle::default()
        ));
    } else if cmd == "cylinder" {
        command.spawn((
            SpatialBundle::default(),
            MeshPrimitivePrefab::Cylinder(CylinderPrefab::default()),
            PrefabMarker,
            Selectable,
            SceneElement::Mesh,
            PickableBundle::default()
        ));
    } else {
        error!("Unknown blocking command: {}", cmd);
    }
}

static COMMAND: WasmMessageBuffer<Option<String>> = WasmMessageBuffer::new();

#[wasm_bindgen(js_name = dispatchBlockingCommand)]
pub fn dispatch_command(command: String) {
    info!("Dispatching blocking command: {}", &command);
    COMMAND.write_message(Some(command));
}