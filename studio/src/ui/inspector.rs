use std::{marker::PhantomData, sync::Arc};

use bevy::{
    ecs::world::unsafe_world_cell::UnsafeWorldCell, prelude::*, reflect::{serde::ReflectSerializer, GetTypeRegistration}, transform::commands, utils::HashMap, window::PrimaryWindow
};
use serde::{Deserialize, Serialize};
use serde_json::{self, json, Map as JsonMap, Value as JsonValue};
use space_editor::{
    space_editor_ui::{
        editor_tab::{EditorTab, EditorTabName},
        EditorUiAppExt,
    },
    space_prefab::{component::MeshPrimitivePrefab, editor_registry::EditorRegistryExt, save::SaveState},
};
use wasm_bindgen::prelude::*;

use crate::{
    interaction::Selection,
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
            update_field_by_ui
                .map(bevy::utils::error)
                .in_set(InspectorSystem::WriteChanges),
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

            app.editor_registry::<ShortMaterial>()
                .add_event::<UpdateInspect<Handle<StandardMaterial>>>()
                .add_systems(
                    Update,
                    (
                        update_inspect_by_select_change::<Handle<StandardMaterial>>
                            .run_if(resource_changed_or_removed::<Selection>()),
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

            app.add_systems(
                OnEnter(SaveState::Save),
                (create_short_materials, apply_deferred).chain().before(space_editor::space_prefab::sub_scene::prepare_auto_scene)
            );
            app.add_systems(PostUpdate, apply_short_materials);

            app.world.resource_mut::<InspectGetters>().getters.insert(
                StandardMaterial::get_type_registration()
                    .type_info()
                    .type_path()
                    .to_string(),
                Arc::new(|entity, path, world, f| unsafe {
                    let handle = entity.get::<Handle<StandardMaterial>>().ok_or_else(|| {
                        format!(
                            "Failed to get StandardMaterial for entity {:?}",
                            entity.id()
                        )
                    })?;

                    let mut assets = world
                        .get_resource_mut::<Assets<StandardMaterial>>()
                        .ok_or_else(|| {
                            "Failed to get `Assets<StandadrMaterial>` resource".to_string()
                        })?;

                    let material = assets.get_mut(handle).ok_or_else(|| {
                        format!("Failed to retrieve material from handle {handle:?}")
                    })?;

                    let reflect_field = material
                        .reflect_path_mut(path)
                        .map_err(|err| format!("{err}"))?;

                    (f)(reflect_field)
                }),
            );
        }
    }
}



fn create_short_materials(
    mut commands : Commands,
    q_mats: Query<(Entity, &Handle<StandardMaterial>)>,
    materials: Res<Assets<StandardMaterial>>,
) {
    for (entity, handle) in q_mats.iter() {
        if let Some(material) = materials.get(handle) {
            commands.entity(entity).insert(ShortMaterial::new(material));
        }
    }
}

fn apply_short_materials(
    mut commands : Commands,
    q_mats: Query<(Entity, &Handle<StandardMaterial>, &ShortMaterial)>,
    mut materials: ResMut<Assets<StandardMaterial>>,) 
{
    for (entity, handle, short) in q_mats.iter() {
        let Some(mut material) = materials.get_mut(handle) else {
            continue;
        };

        material.base_color = short.base_color;
        material.perceptual_roughness = short.perceptual_roughness;
        material.metallic = short.metallic;
        material.reflectance = short.reflectance;

        commands.entity(entity).remove::<ShortMaterial>();
    }
}


#[derive(Resource, Default)]
struct InspectGetters {
    getters: HashMap<
        String,
        Arc<
            dyn Fn(
                    &mut EntityWorldMut,
                    &str,
                    &mut UnsafeWorldCell,
                    &dyn Fn(&mut dyn Reflect) -> Result<(), String>,
                ) -> Result<(), String>
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
                    update_inspect_by_select_change::<T>
                        .run_if(resource_changed_or_removed::<Selection>()),
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
                    (f)(reflect_field)
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
                    update_inspect_by_select_change::<Handle<T>>
                        .run_if(resource_changed_or_removed::<Selection>()),
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
                value: JsonValue::from(0.0),
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

#[derive(Serialize, Deserialize)]
struct Rgba {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

fn update_field_by_ui(world: &mut World) -> Result<(), String> {
    unsafe {
        let mut unsafe_world = world.as_unsafe_world_cell();
        let Some(msg) = CHANGE_FROM_UI.take_message() else {
            return Ok(());
        };
        info!("Processing update: {msg:?}");

        let Some(selection) = unsafe_world.world_mut().get_resource::<Selection>() else {
            return Ok(());
        };
        let entity = **selection;

        let getters = unsafe_world.get_resource_mut::<InspectGetters>().unwrap();
        let getter = getters
            .getters
            .get(&msg.component)
            .ok_or_else(|| format!("Failed to find getter for component: {}", msg.component))?;

        // TODO: This is pretty ugly
        let target = 'find_target: {
            if &msg.component[..] == "bevy_pbr::pbr_material::StandardMaterial" {
                let mut q_handles = unsafe_world
                    .world_mut()
                    .query::<&Handle<StandardMaterial>>();

                if q_handles.get(unsafe_world.world(), entity).is_ok() {
                    break 'find_target Some(entity);
                }

                let mut q_children = unsafe_world.world_mut().query::<&Children>();
                let Ok(ent_children) = q_children.get(unsafe_world.world(), entity) else {
                    break 'find_target None;
                };

                break 'find_target ent_children
                    .iter()
                    .copied()
                    .find(|&child| q_handles.get(unsafe_world.world(), child).is_ok());
            } else {
                break 'find_target Some(entity);
            }
        };

        let Some(target) = target else {
            return Err(format!(
                "Failed to find target for component {}",
                &msg.component
            ));
        };

        info!("Applying to entity: {target:?}");

        let mut entity_mut = unsafe_world.world_mut().entity_mut(target);

        getter.as_ref()(
            &mut entity_mut,
            &msg.reflect_path,
            &mut unsafe_world,
            &|reflect_field| {
                let fmt_err = |err: serde_json::Error| {
                    format!(
                        "Failed to parse value {} for path {}: {err}",
                        msg.value, &msg.reflect_path,
                    )
                };

                if let Some(val) = reflect_field.downcast_mut::<f32>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<f64>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<u32>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<u64>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<i32>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<i64>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<bool>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<String>() {
                    let msg_value = msg
                        .value
                        .as_str()
                        .ok_or_else(|| format!("Failed to parse value as string: {}", msg.value))?;
                    *val = msg_value.to_string();
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<Vec3>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<Quat>() {
                    let msg_value = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;
                    *val = msg_value;
                    Ok(())
                } else if let Some(val) = reflect_field.downcast_mut::<Color>() {
                    let Rgba {
                        red,
                        green,
                        blue,
                        alpha,
                    } = serde_json::from_value(msg.value.clone()).map_err(fmt_err)?;

                    match *val {
                        Color::RgbaLinear { .. } => {
                            *val = Color::RgbaLinear {
                                red,
                                green,
                                blue,
                                alpha,
                            };
                        }
                        Color::Rgba { .. } => {
                            *val = Color::Rgba {
                                red,
                                green,
                                blue,
                                alpha,
                            };
                        }
                        other => {
                            return Err(format!("Unsupported color format: {other:?}"));
                        }
                    }
                    Ok(())
                } else {
                    Err(format!("Unsupported type {}!", &msg.reflect_path))
                }
            },
        )
    }
}

#[derive(Resource, Serialize, Deserialize, Default, Clone)]
pub struct EntityDescription {
    pub components: HashMap<String, JsonValue>,
}

fn update_inspect_by_select_change<T: Component + Reflect + GetTypeRegistration>(
    mut ev_inspect: EventWriter<UpdateInspect<T>>,
) {
    ev_inspect.send(UpdateInspect::<T>::default());
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
    r_selection: Option<Res<Selection>>,
    mut ev_inspect: EventWriter<UpdateInspect<T>>,
    q_t_changed: Query<&T, Changed<T>>,
) {
    if let Some(selection) = r_selection {
        if q_t_changed.contains(**selection) {
            ev_inspect.send(UpdateInspect::<T>::default());
        }
    }
}

fn auto_serialize_system<T: Component + Reflect + GetTypeRegistration>(
    r_selection: Option<Res<Selection>>,
    q_t: Query<&T>,
    mut description: ResMut<EntityDescription>,
    registry: Res<AppTypeRegistry>,
    mut ev_inspect: EventReader<UpdateInspect<T>>,
    mut ev_update_description: EventWriter<UpdateEntityDescription>,
) {
    if ev_inspect.is_empty() {
        return;
    }

    ev_inspect.clear();

    if let Some(t) = r_selection.and_then(|selection| q_t.get(**selection).ok()) {
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
    r_selection: Option<Res<Selection>>,
    mut ev_inspect: EventWriter<UpdateInspect<Handle<T>>>,
    mut asset_changing: EventReader<AssetEvent<T>>,
    q_handles: Query<&Handle<T>>,
    q_children: Query<&Children>,
) {
    for ev in asset_changing.read() {
        if let Some(handle) = r_selection
            .as_ref()
            .and_then(|selection| {
                find_handle_in_self_or_children(***selection, &q_children, &q_handles)
            })
            .and_then(|target| q_handles.get(target).ok())
        {
            if ev.is_modified(handle.id()) {
                ev_inspect.send(UpdateInspect::<Handle<T>>::default());
            }
        }
    }
}

fn auto_serialize_asset<T: Asset + Reflect + GetTypeRegistration>(
    r_selection: Option<Res<Selection>>,
    q_t: Query<&Handle<T>>,
    q_children: Query<&Children>,
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

    if let Some(handle) = r_selection
        .as_ref()
        .and_then(|selection| find_handle_in_self_or_children(***selection, &q_children, &q_t))
        .and_then(|target| q_t.get(target).ok())
    {
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

#[derive(Reflect, Default, Clone, Component)]
#[reflect(Default, Component)]
struct ShortMaterial {
    pub base_color: Color,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
}

impl ShortMaterial {
    pub fn new(material: &StandardMaterial) -> Self {
        Self {
            base_color: material.base_color,
            perceptual_roughness: material.perceptual_roughness,
            metallic: material.metallic,
            reflectance: material.reflectance,
        }
    }
}


fn serialize_standart_materal(
    r_selection: Option<Res<Selection>>,
    q_m: Query<&Handle<StandardMaterial>>,
    q_children: Query<&Children>,
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

    if let Some((target, handle)) = r_selection
        .as_ref()
        .and_then(|selection| find_handle_in_self_or_children(***selection, &q_children, &q_m))
        .and_then(|target| q_m.get(target).ok().map(|h| (target, h.clone())))
    {
        if let Some(material) = assets.get(handle.clone()) {
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
            let mut value = JsonValue::Object(JsonMap::new());
            value
                .as_object_mut()
                .unwrap()
                .insert("name".to_string(), JsonValue::String(name.to_string()));
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

/// Find a `Handle<T>` component on this entity or one of its direct children
fn find_handle_in_self_or_children<T: Asset>(
    target: Entity,
    q_children: &Query<&Children>,
    q_handles: &Query<&Handle<T>>,
) -> Option<Entity> {
    if q_handles.contains(target) {
        return Some(target);
    }

    let Ok(children) = q_children.get(target) else {
        return None;
    };

    children
        .iter()
        .copied()
        .find(|&child| q_handles.contains(child))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateField {
    pub component: String,
    pub reflect_path: String,
    pub value: JsonValue,
}

#[wasm_bindgen(typescript_custom_section)]
const EVENT_DECLARATION: &str = r#"

export interface InspectorChangedEvent extends CustomEvent {
    detail: string | null;
}

declare global {
    export interface HTMLElementEventMap {
        "inspector-changed": InspectorChangedEvent;
    }
}

export interface UpdateField<T> {
    component: string;
    reflectPath: string;
    value: T;
}

export function updateComponentField<T>(params: UpdateField<T>): void;
"#;

pub(super) fn notify_changed(q_window: &Window, msg: String) {
    let res = wasm::dispatch_to_js(q_window, "inspector-changed", &msg);
    if let Err(e) = res {
        error!("Failed to dispatch inspector-changed: {}", e);
    }
}

static CHANGE_FROM_UI: WasmMessageBuffer<UpdateField> = WasmMessageBuffer::new();

#[wasm_bindgen(js_name = updateComponentField, skip_typescript)]
pub fn update_component_field(msg: JsValue) -> Result<(), JsValue> {
    let msg = serde_wasm_bindgen::from_value(msg)?;
    info!("Received update: {msg:?}");
    CHANGE_FROM_UI.write_message(msg);
    Ok(())
}
