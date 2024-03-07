use std::{marker::PhantomData, sync::Arc};

use bevy::{
    ecs::world::unsafe_world_cell::UnsafeWorldCell,
    prelude::*,
    reflect::{serde::ReflectSerializer, GetTypeRegistration},
    utils::HashMap,
    window::PrimaryWindow,
};
use serde::{Deserialize, Serialize};
use serde_json::*;
use space_editor::{
    space_editor_ui::{
        editor_tab::{EditorTab, EditorTabName},
        EditorUiAppExt,
    },
    space_prefab::component::MeshPrimitivePrefab,
};
use wasm_bindgen::prelude::*;

use crate::{
    interaction::Selected,
    wasm::{self, StaticBuffer, WasmMessageBuffer},
};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityDescription>();
        app.add_event::<UpdateEntityDescription>();

        app.configure_sets(
            Update,
            (
                InspectorSystem::WriteChanges,
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

        app.add_systems(
            Update,
            update_field_by_ui.in_set(InspectorSystem::WriteChanges),
        );

        app.editor_tab_by_trait(
            EditorTabName::Other("Inspector tester".to_string()),
            TestInspectorTab::default(),
        );

        app.init_resource::<InspectGetters>();

        app.register_inspect::<Transform>();
        app.register_inspect::<Name>();
        app.register_inspect::<MeshPrimitivePrefab>();

        //register standart material
        {
            app.register_type::<ShortMaterial>()
                .add_event::<UpdateInspect<Handle<StandardMaterial>>>()
                .add_systems(
                    Update,
                    (
                        update_inspect_by_select_change::<Handle<StandardMaterial>>,
                        update_inspect_by_remove::<Handle<StandardMaterial>>,
                        update_inspect_by_change::<Handle<StandardMaterial>>,
                        notify_asset_change_system::<StandardMaterial>,
                    )
                        .in_set(InspectorSystem::DetectChanges),
                )
                .add_systems(
                    Update,
                    serialize_standart_materal.in_set(InspectorSystem::PerComponentCollect),
                );

            app.world.resource_mut::<InspectGetters>().getters.insert(
                StandardMaterial::get_type_registration()
                    .type_info()
                    .type_path()
                    .to_string(),
                Arc::new(|entity, path, world, f| unsafe {
                    let handle = entity.get::<Handle<StandardMaterial>>().unwrap();
                    let mut assets = world
                        .get_resource_mut::<Assets<StandardMaterial>>()
                        .unwrap();
                    let material = assets.get_mut(handle).unwrap();

                    let reflect_field = material.reflect_path_mut(path).unwrap();
                    (f)(reflect_field);
                }),
            );
        }
    }
}

#[derive(Resource, Default)]
struct InspectGetters {
    getters: HashMap<
        String,
        Arc<
            dyn Fn(&mut EntityWorldMut, &str, &mut UnsafeWorldCell, &dyn Fn(&mut dyn Reflect))
                + Send
                + Sync,
        >,
    >,
}

trait AppInspectable {
    fn register_inspect<T: Component + Reflect + GetTypeRegistration>(&mut self);
    fn register_asset_inspect<T: Asset + Reflect + GetTypeRegistration>(&mut self);
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

        {
            let mut getters = self.world.get_resource_mut::<InspectGetters>().unwrap();
            getters.getters.insert(
                T::get_type_registration()
                    .type_info()
                    .type_path()
                    .to_string(),
                Arc::new(|entity, path, _, f| {
                    let mut component = entity.get_mut::<T>().unwrap();
                    let reflect_field = component.reflect_path_mut(path).unwrap();
                    (f)(reflect_field);
                }),
            );
        }
    }

    fn register_asset_inspect<T: Asset + Reflect + GetTypeRegistration>(&mut self) {
        self.register_type::<T>()
            .add_event::<UpdateInspect<Handle<T>>>()
            .add_systems(
                Update,
                (
                    update_inspect_by_select_change::<Handle<T>>,
                    update_inspect_by_remove::<Handle<T>>,
                    update_inspect_by_change::<Handle<T>>,
                    notify_asset_change_system::<T>,
                )
                    .in_set(InspectorSystem::DetectChanges),
            )
            .add_systems(
                Update,
                auto_serialize_asset::<T>.in_set(InspectorSystem::PerComponentCollect),
            );
    }
}

#[derive(Default, Resource)]
pub struct TestInspectorTab {}

impl EditorTab for TestInspectorTab {
    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        _commands: &mut Commands,
        _world: &mut World,
    ) {
        if ui.button("Test set transform x to zero").clicked() {
            CHANGE_FROM_UI.write_message(UpdateField {
                component: "bevy_transform::components::transform::Transform".to_string(),
                reflect_path: ".translation.x".to_string(),
                value: serde_json::Value::from(0.0),
            });
        }
    }

    fn title(&self) -> bevy_inspector_egui::egui::WidgetText {
        "Inspector tester".into()
    }
}

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InspectorSystem {
    WriteChanges,
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

fn update_field_by_ui(world: &mut World) {
    unsafe {
        let mut unsafe_world = world.as_unsafe_world_cell();
        let Some(msg) = CHANGE_FROM_UI.take_message() else {
            return;
        };

        let mut selected_query = unsafe_world
            .world_mut()
            .query_filtered::<Entity, With<Selected>>();
        let Ok(entity) = selected_query.get_single(unsafe_world.world_mut()) else {
            return;
        };

        let getters = unsafe_world.get_resource_mut::<InspectGetters>().unwrap();

        let mut entity_mut = unsafe_world.world_mut().entity_mut(entity);

        let Some(getter) = getters.getters.get(&msg.component) else {
            error!("Failed to find getter for component: {}", msg.component);
            return;
        };

        getter.as_ref()(
            &mut entity_mut,
            &msg.reflect_path,
            &mut unsafe_world,
            &|reflect_field| {
                if let Some(val) = reflect_field.downcast_mut::<f32>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<f64>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<u32>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<u64>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<i32>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<i64>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<bool>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<String>() {
                    *val = msg.value.as_str().unwrap().to_string();
                } else if let Some(val) = reflect_field.downcast_mut::<Vec3>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else if let Some(val) = reflect_field.downcast_mut::<Quat>() {
                    let Ok(msg_value) = serde_json::from_value(msg.value.clone()) else {
                        error!("Failed to parse value: {}", msg.value);
                        return;
                    };
                    *val = msg_value;
                } else {
                    error!("Unsupported type {}!", &msg.reflect_path);
                    return;
                }
            },
        );
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

fn notify_asset_change_system<T: Asset>(
    mut ev_inspect: EventWriter<UpdateInspect<Handle<T>>>,
    mut asset_changing: EventReader<AssetEvent<T>>,
    mut selected: Query<&Handle<T>, With<Selected>>,
) {
    for ev in asset_changing.read() {
        for handle in selected.iter_mut() {
            if ev.is_modified(handle.id()) {
                ev_inspect.send(UpdateInspect::<Handle<T>>::default());
            }
        }
    }
}

fn auto_serialize_asset<T: Asset + Reflect + GetTypeRegistration>(
    q_t: Query<&Handle<T>, With<Selected>>,
    mut description: ResMut<EntityDescription>,
    registry: Res<AppTypeRegistry>,
    mut ev_inspect: EventReader<UpdateInspect<Handle<T>>>,
    mut ev_update_description: EventWriter<UpdateEntityDescription>,
    assets: Res<Assets<T>>,
) {
    if ev_inspect.is_empty() {
        return;
    }

    ev_inspect.clear();

    if let Some(handle) = q_t.iter().next() {
        if let Some(asset) = assets.get(handle) {
            let registry = registry.read();
            info!(
                "type: {:?}",
                T::get_type_registration()
                    .type_info()
                    .type_path()
                    .to_string()
            );

            let json = serde_json::to_value(ReflectSerializer::new(asset, &registry)).unwrap();
            info!("json: {:?}", json.to_string());
            description.components.insert(
                T::get_type_registration()
                    .type_info()
                    .type_path()
                    .to_string(),
                json,
            );
        }
    } else {
        description
            .components
            .remove(T::get_type_registration().type_info().type_path());
    }

    ev_update_description.send(UpdateEntityDescription);
}

#[derive(Reflect, Default)]
#[reflect(Default)]
struct ShortMaterial {
    pub base_color: Color,
}

impl ShortMaterial {
    pub fn new(material: &StandardMaterial) -> Self {
        Self {
            base_color: material.base_color,
        }
    }
}

fn serialize_standart_materal(
    q_m: Query<&Handle<StandardMaterial>>,
    mut description: ResMut<EntityDescription>,
    registry: Res<AppTypeRegistry>,
    mut ev_inspect: EventReader<UpdateInspect<Handle<StandardMaterial>>>,
    mut ev_update_description: EventWriter<UpdateEntityDescription>,
    assets: Res<Assets<StandardMaterial>>,
) {
    if ev_inspect.is_empty() {
        return;
    }

    ev_inspect.clear();

    if let Some(handle) = q_m.iter().next() {
        if let Some(material) = assets.get(handle) {
            let registry = registry.read();
            let short_material = ShortMaterial::new(material);
            let json =
                serde_json::to_value(ReflectSerializer::new(&short_material, &registry)).unwrap();
            let json = json[ShortMaterial::get_type_registration()
                .type_info()
                .type_path()
                .to_string()]
            .clone();
            let json = json!(
            {
                StandardMaterial::get_type_registration()
                .type_info()
                .type_path()
                .to_string(): json
            });
            info!("json: {:?}", json.to_string());
            description.components.insert(
                StandardMaterial::get_type_registration()
                    .type_info()
                    .type_path()
                    .to_string(),
                json,
            );
        }
    } else {
        description.components.remove(
            StandardMaterial::get_type_registration()
                .type_info()
                .type_path(),
        );
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

#[derive(Deserialize, Serialize)]
pub struct UpdateField {
    pub component: String,
    pub reflect_path: String,
    pub value: serde_json::Value,
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

static CHANGE_FROM_UI: WasmMessageBuffer<UpdateField> = WasmMessageBuffer::new();

#[wasm_bindgen(js_name = changeValue)]
pub fn change_value(msg: JsValue) {
    let Ok(msg) = serde_wasm_bindgen::from_value(msg) else {
        error!("Failed to parse message");
        return;
    };
    CHANGE_FROM_UI.write_message(msg);
}
