use std::fmt;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_mod_picking::debug::DebugPickingMode;
use space_editor::prelude::space_undo::AppAutoUndo;
use space_editor::{
    prelude::*,
    space_editor_ui::{tools::gizmo::GizmoToolPlugin, EditorPluginGroup},
};

use crate::StudioMode;
use crate::{
    anim::SkeletalJoint,
    camera_controller::{CameraController, CameraMotionSystem},
    input::{Action, InputAction, Lifecycle},
    interaction::Selectable,
    is_editor,
    scene::SceneElement,
    MainCamera,
};

#[derive(Resource, Deref, DerefMut)]
pub struct DebugMode(bool);

#[derive(Component, Clone, Copy, Debug)]
pub struct DebugPoint {
    pub point: Vec3,
    pub color: Color,
}

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugMode(false))
            .insert_resource(DebugPickingMode::Disabled)
            .add_systems(Startup, |mut config_store: ResMut<GizmoConfigStore>| {
                let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
                *config = GizmoConfig {
                    enabled: true,
                    depth_bias: -0.1,
                    line_perspective: false,
                    line_width: 2.,
                    ..default()
                };
            })
            .add_systems(
                Update,
                (
                    toggle_debug_mode,
                    update_debug_picking_mode
                        .run_if(is_editor)
                        .run_if(resource_changed::<DebugMode>),
                    update_gizmo_line_thickness,
                    (draw_world_axes, draw_debug_points, draw_skeletal_joints)
                        .after(CameraMotionSystem)
                        .run_if(is_debug_mode),
                ),
            );

        let editor_ui_group = EditorPluginGroup
            .build()
            .disable::<EditorUiCore>()
            .add_before::<GameViewPlugin, _>(EditorUiCore {
                disable_no_editor_cams: false,
            })
            // .disable::<EguiPlugin>()
            .disable::<DefaultInspectorConfigPlugin>()
            .disable::<EditorPickingPlugin>()
            .disable::<EditorDefaultCameraPlugin>()
            // .disable::<BottomMenuPlugin>()
            .disable::<GizmoToolPlugin>()
            // .disable::<EditorGizmoConfigPlugin>()
            // .disable::<MeshlessVisualizerPlugin>()
            .disable::<space_editor::prelude::SelectedPlugin>();

        let mut only_core = false;

        if let Some(state) = app.world.get_resource::<StudioMode>() {
            if *state == StudioMode::Headless {
                only_core = true;
            }
        }

        if only_core {
            app.add_plugins(PrefabPlugin).add_plugins(EditorCore);
        } else {
            app.add_plugins(editor_ui_group);
            app.auto_undo::<Handle<StandardMaterial>>();
            app.auto_undo::<SkeletalJoint>();
            app.auto_undo::<Handle<Mesh>>();
            app.auto_undo::<SceneElement>();
            app.auto_undo::<Selectable>();

            #[cfg(feature = "wasm")]
            {
                app.add_systems(
                    PostUpdate,
                    wasm::notify_debug_mode_toggle.map(bevy::utils::error),
                );
            }

            app.add_systems(Update, sync_editor_camera);
            app.add_systems(
                Update,
                update_editor_ui_mode.run_if(resource_changed::<DebugMode>),
            );
        }
    }
}

fn toggle_debug_mode(mut input: EventReader<InputAction>, mut r_debug: ResMut<DebugMode>) {
    if input
        .read()
        .any(|action| matches!(action, InputAction(Lifecycle::Stop, Action::ToggleDebug)))
    {
        **r_debug = !**r_debug;
    }
}

fn update_editor_ui_mode(
    r_debug: Res<DebugMode>,
    mut s_show_editor_ui: ResMut<NextState<ShowEditorUi>>,
) {
    s_show_editor_ui.set(match **r_debug {
        true => ShowEditorUi::Show,
        false => ShowEditorUi::Hide,
    });
}

fn update_debug_picking_mode(
    r_debug: Res<DebugMode>,
    mut s_debug_picking: ResMut<DebugPickingMode>,
) {
    *s_debug_picking = match **r_debug {
        true => DebugPickingMode::Normal,
        false => DebugPickingMode::Disabled,
    };
}

pub fn is_debug_mode(r_debug: Res<DebugMode>) -> bool {
    **r_debug
}

fn update_gizmo_line_thickness(
    mut config_store: ResMut<GizmoConfigStore>,
    q_window: Query<&Window, (Changed<Window>, With<PrimaryWindow>)>,
) {
    if let Ok(scale_factor) = q_window.get_single().map(|win| win.scale_factor()) {
        let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
        config.line_width = 2. * scale_factor;
    }
}

fn draw_world_axes(mut gizmos: Gizmos, q: Query<&CameraController>) {
    let &CameraController { focus, radius, .. } = q.single();

    let mut draw_axis = |direction: Vec3, color: Color| {
        let end = focus + direction * radius / 15.;
        gizmos.line(focus, end, color);
    };

    draw_axis(Vec3::X, Color::RED);
    draw_axis(Vec3::Y, Color::GREEN);
    draw_axis(Vec3::Z, Color::BLUE);
}

fn draw_debug_points(
    mut gizmos: Gizmos,
    q_debug_points: Query<&DebugPoint>,
    q_cam_xform: Query<&GlobalTransform, With<MainCamera>>,
) {
    let Ok(cam_xform) = q_cam_xform.get_single() else {
        return;
    };
    for &DebugPoint { point, color } in q_debug_points.iter() {
        let dist = cam_xform.translation().distance(point);
        gizmos.cuboid(
            Transform::from_translation(point).with_scale(Vec3::splat(dist / 300.)),
            color,
        );
    }
}

fn draw_skeletal_joints(
    mut gizmos: Gizmos,
    q_joints: Query<(&GlobalTransform, Option<&Parent>, &SkeletalJoint)>,
) {
    for (world_xform, parent, joint) in q_joints.iter() {
        let (_, rotation, xlation) = world_xform.to_scale_rotation_translation();

        if let Some((parent_xform, _, _)) = parent.and_then(|parent| q_joints.get(**parent).ok()) {
            let parent_xlation = parent_xform.translation();
            let bone_len = Vec3::distance(xlation, parent_xlation);
            let joint_radius = f32::min(0.01, bone_len / 6.);

            gizmos.sphere(xlation, rotation, joint_radius, joint.0);
            gizmos.line(xlation, parent_xlation, joint.0);

            let local_x = world_xform.right() * 0.05;
            let local_y = world_xform.up() * 0.05;
            let local_z = world_xform.back() * 0.05;

            gizmos.line(xlation, xlation + local_x, Color::RED);
            gizmos.line(xlation, xlation + local_y, Color::GREEN);
            gizmos.line(xlation, xlation + local_z, Color::BLUE);
        } else {
            gizmos.sphere(xlation, rotation, 0.01, joint.0);
        }
    }
}

#[rustfmt::skip]
#[allow(dead_code)]
pub fn write_mat3(w: &mut impl fmt::Write, m: &Mat3) -> fmt::Result {
    writeln!(w, "  {:+.3} {:+.3} {:+.3}", m.x_axis.x, m.x_axis.y, m.x_axis.z)?;
	writeln!(w, "  {:+.3} {:+.3} {:+.3}", m.y_axis.x, m.y_axis.y, m.y_axis.z)?;
	writeln!(w, "  {:+.3} {:+.3} {:+.3}", m.z_axis.x, m.z_axis.y, m.z_axis.z)?;
	Ok(())
}

#[allow(dead_code)]
pub fn print_mat3(m: &Mat3, label: impl fmt::Display) -> fmt::Result {
    let mut s = String::new();
    write_mat3(&mut s, m)?;
    info!("\n{label}\n{}", s.replace('+', " "));

    Ok(())
}

fn sync_editor_camera(
    mut cmd: Commands,
    mut q_cam: Query<Entity, (With<MainCamera>, Without<EditorCameraMarker>)>,
) {
    for ent in q_cam.iter_mut() {
        cmd.entity(ent).insert(EditorCameraMarker);
    }
}

#[cfg(feature = "wasm")]
mod wasm {
    use super::DebugMode;
    use bevy::{prelude::*, window::PrimaryWindow};
    use serde_json::json;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(typescript_custom_section)]
    const DEBUG_TYPES: &str = r#"
    export interface DebugModeToggled extends CustomEvent {
        type: "debug-mode-toggled",
        detail: {
            debugModeActive: boolean
        }
    }
    
    declare global {
        export interface GlobalEventHandlersEventMap {
            "debug-mode-toggled": DebugModeToggled
        }
    }
    "#;

    pub fn notify_debug_mode_toggle(
        q_window: Query<&Window, With<PrimaryWindow>>,
        r_debug_mode: Res<DebugMode>,
    ) -> Result<(), String> {
        if !r_debug_mode.is_changed() {
            return Ok(());
        }
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        crate::wasm::dispatch_to_js(
            window,
            "debug-mode-toggled",
            &json!({ "debugModeActive": r_debug_mode.0 }),
        )
    }
}
