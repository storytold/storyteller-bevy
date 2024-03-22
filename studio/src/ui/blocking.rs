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

fn read_command_system(mut commands: Commands, mut change_event: EventWriter<NewChange>) {
    let Some(cmd) = COMMAND.take_message() else {
        return;
    };

    let mut new_entity = commands.spawn((
        SpatialBundle::default(),
        PrefabMarker,
        SceneElement::Mesh,
    ));

    match cmd {
        BlockingCommand::Cube => {
            new_entity
                .insert(MeshPrimitivePrefab::Cube(1.0))
                .insert(Name::new("Cube"));
        }
        BlockingCommand::Sphere => {
            new_entity
                .insert(MeshPrimitivePrefab::Sphere(SpherePrefab { r: 1.0 }))
                .insert(Name::new("Sphere"));
        }
        BlockingCommand::Torus => {
            new_entity
                .insert(MeshPrimitivePrefab::Torus(TorusPrefab::default()))
                .insert(Name::new("Torus"));
        }
        BlockingCommand::Cylinder => {
            new_entity
                .insert(MeshPrimitivePrefab::Cylinder(CylinderPrefab::default()))
                .insert(Name::new("Cylinder"));
        }
    };

    change_event.send(NewChange {
        change: Arc::new(AddedEntity {
            entity: new_entity.id(),
        }),
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
