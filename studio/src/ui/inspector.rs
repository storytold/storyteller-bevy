use std::marker::PhantomData;

use bevy::{
    prelude::*,
    reflect::{serde::ReflectSerializer, GetTypeRegistration},
    utils::HashMap,
    window::PrimaryWindow,
};
use serde::{Deserialize, Serialize};
use serde_json::*;
use space_editor::space_prefab::component::MeshPrimitivePrefab;
use wasm_bindgen::prelude::*;

use crate::{interaction::Selected, wasm};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityDescription>();
        app.add_event::<UpdateEntityDescription>();

        app.configure_sets(
            Update,
            (
                InspectorSystem::DetectChanges,
                InspectorSystem::PerComponentCollect,
                InspectorSystem::CollectChanges,
            )
                .chain(),
        );

        app.add_systems(
            Update,
            update_description.in_set(InspectorSystem::CollectChanges),
        );

        app.register_inspect::<Transform>();
        app.register_inspect::<Name>();
        app.register_inspect::<MeshPrimitivePrefab>();
    }
}

trait AppInspectable {
    fn register_inspect<T: Component + Reflect + GetTypeRegistration>(&mut self);
}

impl AppInspectable for App {
    fn register_inspect<T: Component + Reflect + GetTypeRegistration>(&mut self) {
        self.register_type::<T>()
            .add_event::<UpdateInspect<T>>()
            .add_systems(
                Update,
                (
                    update_inspect_by_select_change::<T>,
                    update_inspect_by_remove::<T>,
                    update_inspect_by_change::<T>,
                )
                    .in_set(InspectorSystem::DetectChanges),
            )
            .add_systems(
                Update,
                auto_serialize_system::<T>.in_set(InspectorSystem::PerComponentCollect),
            );
    }
}

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InspectorSystem {
    DetectChanges,
    PerComponentCollect,
    CollectChanges,
}

#[derive(Event, Clone, Copy)]
pub struct UpdateInspect<T> {
    _phantom: PhantomData<T>,
}

#[derive(Event)]
pub struct UpdateEntityDescription;

impl<T> Default for UpdateInspect<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Default, Clone)]
pub struct EntityDescription {
    pub components: HashMap<String, Value>,
}

fn update_inspect_by_select_change<T: Component + Reflect + GetTypeRegistration>(
    mut ev_inspect: EventWriter<UpdateInspect<T>>,
    q_added: Query<(), Added<Selected>>,
    q_removed: RemovedComponents<Selected>,
) {
    if !q_added.is_empty() {
        ev_inspect.send(UpdateInspect::<T>::default());
    }

    if !q_removed.is_empty() {
        ev_inspect.send(UpdateInspect::<T>::default());
    }
}

fn update_inspect_by_remove<T: Component + Reflect + GetTypeRegistration>(
    mut ev_inspect: EventWriter<UpdateInspect<T>>,
    mut q_t_removed: RemovedComponents<T>,
) {
    if let Some(_t) = q_t_removed.read().next() {
        ev_inspect.send(UpdateInspect::<T>::default());
    }
}

fn update_inspect_by_change<T: Component + Reflect + GetTypeRegistration>(
    mut ev_inspect: EventWriter<UpdateInspect<T>>,
    q_t_changed: Query<&T, (With<Selected>, Changed<T>)>,
) {
    if q_t_changed.iter().next().is_some() {
        ev_inspect.send(UpdateInspect::<T>::default());
    }
}

fn auto_serialize_system<T: Component + Reflect + GetTypeRegistration>(
    q_t: Query<&T, With<Selected>>,
    mut description: ResMut<EntityDescription>,
    registry: Res<AppTypeRegistry>,
    mut ev_inspect: EventReader<UpdateInspect<T>>,
    mut ev_update_description: EventWriter<UpdateEntityDescription>,
) {
    if ev_inspect.is_empty() {
        return;
    }

    ev_inspect.clear();

    if let Some(t) = q_t.iter().next() {
        let registry = registry.read();
        let json = serde_json::to_value(ReflectSerializer::new(t, &registry)).unwrap();
        info!("json: {:?}", json.to_string());
        description.components.insert(
            T::get_type_registration()
                .type_info()
                .type_path()
                .to_string(),
            json,
        );
    } else {
        description
            .components
            .remove(T::get_type_registration().type_info().type_path());
    }

    ev_update_description.send(UpdateEntityDescription);
}

fn update_description(
    description: ResMut<EntityDescription>,
    mut ev_update_description: EventReader<UpdateEntityDescription>,
    mut q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    if ev_update_description.is_empty() {
        return;
    }

    ev_update_description.clear();

    let values = description
        .components
        .iter()
        .map(|(name, v)| {
            let mut value = Value::Object(Map::new());
            value
                .as_object_mut()
                .unwrap()
                .insert("name".to_string(), Value::String(name.to_string()));
            value
                .as_object_mut()
                .unwrap()
                .insert("value".to_string(), v[name].clone());
            value
        })
        .collect::<Vec<_>>();

    //Write pretty json
    // info!("description: {}", serde_json::to_string_pretty(&values).unwrap());

    if let Some(window) = q_windows.iter_mut().next() {
        notify_changed(window, serde_json::to_string_pretty(&values).unwrap());
    }
}

#[wasm_bindgen(typescript_custom_section)]
const EVENT_DECLARATION: &str = r#"

export interface InspectorChanged extends CustomEvent {
    detail: string | null;
}

declare global {
export interface HTMLElementEventMap {
    "inspector-changed": InspectorChanged;
}
}
"#;

pub(super) fn notify_changed(q_window: &Window, msg: String) {
    let res = wasm::dispatch_to_js(q_window, "inspector-changed", &msg);
    if let Err(e) = res {
        error!("Failed to dispatch inspector-changed: {}", e);
    }
}
