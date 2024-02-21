use bevy::prelude::*;
use itertools::Itertools;
use space_editor::space_prefab::editor_registry::EditorRegistryExt;

use crate::math::{InvLerp, Lerp};

use super::GlobalSeekTime;

pub struct CameraAnimPlugin;
impl Plugin for CameraAnimPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraKeyframe>()
            .editor_registry::<CameraKeyframe>()
            .register_type::<CameraTargetKeyframe>()
            .editor_registry::<CameraTargetKeyframe>();

        app.add_systems(Update, draw_camera_keyframes);

        #[cfg(feature = "wasm")]
        app.add_systems(
            Update,
            (
                wasm::spawn_camera_keyframes,
                wasm::notify_keyframe_changes.map(bevy::utils::error),
                wasm::process_update_keyframe,
            ),
        );
    }
}

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct CameraKeyframe(pub f32);

#[derive(Component, Clone, Copy, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component)]
pub struct CameraTargetKeyframe(pub f32);

fn draw_camera_keyframes(
    mut gizmos: Gizmos,
    r_seek_time: Res<GlobalSeekTime>,
    q_cam_keyframe_xforms: Query<&GlobalTransform, With<CameraKeyframe>>,
    q_cam_target_keyframe_xforms: Query<&GlobalTransform, With<CameraTargetKeyframe>>,
    q_cam_keyframes: Query<(&CameraKeyframe, &GlobalTransform)>,
) {
    if q_cam_keyframes.is_empty() {
        return;
    }

    for world_xform in q_cam_keyframe_xforms
        .iter()
        .chain(q_cam_target_keyframe_xforms.iter())
    {
        let origin = world_xform.translation();
        let x_axis = world_xform.right() * 0.25;
        let y_axis = world_xform.up() * 0.25;
        let z_axis = world_xform.back() * 0.25;

        gizmos.line(origin, origin + x_axis, Color::RED);
        gizmos.line(origin, origin + y_axis, Color::GREEN);
        gizmos.line(origin, origin + z_axis, Color::BLUE);
    }

    let seek_time = **r_seek_time;
    let keyframes_sorted = q_cam_keyframes
        .iter()
        .sorted_by(|(a, _), (b, _)| a.partial_cmp(&b.0).unwrap())
        .collect_vec();

    let (timestamp_first, xform_first) = keyframes_sorted.first().unwrap();
    let (timestamp_last, xform_last) = keyframes_sorted.last().unwrap();

    if seek_time <= timestamp_first.0 {
        let (_, rot, pos) = xform_first.to_scale_rotation_translation();
        gizmos.sphere(pos, rot, 0.25, Color::WHITE);
    } else if seek_time >= timestamp_last.0 {
        let (_, rot, pos) = xform_last.to_scale_rotation_translation();
        gizmos.sphere(pos, rot, 0.25, Color::WHITE);
    } else if let Some(((timestamp_a, xform_a), (timestamp_b, xform_b))) = keyframes_sorted
        .iter()
        .tuple_windows()
        .find(|((a, _), (b, _))| a.0 <= seek_time && b.0 >= seek_time)
    {
        let t = seek_time.inv_lerp(timestamp_a.0, timestamp_b.0);
        let (_, rot_a, pos_a) = xform_a.to_scale_rotation_translation();
        let (_, rot_b, pos_b) = xform_b.to_scale_rotation_translation();
        let pos = t.lerp(pos_a, pos_b);
        let rot = Quat::slerp(rot_a, rot_b, t);

        gizmos.sphere(pos, rot, 0.25, Color::WHITE);
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use std::sync::Arc;

    use bevy::{prelude::*, window::PrimaryWindow};
    use space_editor::prelude::space_undo::{self, AddedEntity};
    use wasm_bindgen::prelude::*;

    use crate::{
        camera_controller::CameraController,
        wasm::{self, FromStr, StaticBuffer, WasmMessageBuffer},
    };

    use super::{CameraKeyframe, CameraTargetKeyframe};

    static ADD_KEYFRAME: WasmMessageBuffer<f32> = WasmMessageBuffer::new();
    static UPDATE_KEYFRAME: WasmMessageBuffer<(Entity, f32)> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = addCameraKeyframe)]
    pub fn add_camera_keyframe(timestamp: f32) {
        ADD_KEYFRAME.write_message(timestamp);
    }

    pub(super) fn spawn_camera_keyframes(
        mut cmd: Commands,
        mut ew_changes: EventWriter<space_undo::NewChange>,
        q_camera: Query<(&CameraController, &GlobalTransform)>,
    ) {
        if let Some(timestamp) = ADD_KEYFRAME.take_message() {
            let Ok((cam_controller, world_xform)) = q_camera.get_single() else {
                return;
            };

            let entity = cmd
                .spawn((
                    Name::new("Camera Target Keyframe"),
                    TransformBundle::from_transform(Transform::from_translation(
                        cam_controller.focus,
                    )),
                    CameraTargetKeyframe(timestamp),
                ))
                .id();
            ew_changes.send(space_undo::NewChange {
                change: Arc::new(AddedEntity { entity }),
            });

            let entity = cmd
                .spawn((
                    Name::new("Camera Keyframe"),
                    TransformBundle::from_transform(Transform::from(*world_xform)),
                    CameraKeyframe(timestamp),
                ))
                .id();
            ew_changes.send(space_undo::NewChange {
                change: Arc::new(AddedEntity { entity }),
            });
        }
    }

    #[wasm_bindgen(js_name = updateKeyframe)]
    pub fn update_keyframe(entity: &str, timestamp: f32) {
        let entity = Entity::from_str(entity).unwrap();
        UPDATE_KEYFRAME.write_message((entity, timestamp));
    }

    pub(super) fn process_update_keyframe(
        mut q_camera_keyframes: Query<&mut CameraKeyframe>,
        mut q_camera_target_keyframes: Query<&mut CameraTargetKeyframe>,
    ) {
        if let Some((ent, timestamp)) = UPDATE_KEYFRAME.take_message() {
            if let Ok(mut keyframe) = q_camera_keyframes.get_mut(ent) {
                **keyframe = timestamp;
            } else if let Ok(mut keyframe) = q_camera_target_keyframes.get_mut(ent) {
                **keyframe = timestamp;
            } else {
                warn!("No matching keyframe found for entity: {ent:?}");
            }
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const CAMERA_KEYFRAMES_EVENT: &str = r#"
export interface CameraKeyframesEvent extends CustomEvent {
    type: "camera-keyframes";
    detail: Record<string, number>;
}

declare global {
    export interface GlobalEventHandlersEventMap {
        "camera-keyframes": CameraKeyframesEvent;
    }
}
    "#;

    pub(super) fn notify_keyframe_changes(
        mut l_notified: Local<serde_json::Map<String, serde_json::Value>>,
        q_keyframes: Query<(Entity, &CameraKeyframe)>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        if l_notified.len() == q_keyframes.iter().count() {
            return Ok(());
        }

        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        l_notified.clear();
        for (ent, timestamp) in q_keyframes.iter() {
            l_notified.insert(format!("{ent:?}"), timestamp.0.into());
        }

        wasm::dispatch_to_js(window, "camera-keyframes", &*l_notified)
    }
}
