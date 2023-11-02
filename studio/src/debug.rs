use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::debug::DebugPickingMode;

use crate::{
    camera_controller::{CameraController, CameraMotionSystem},
    input::{Action, InputAction, Lifecycle},
};

#[derive(Resource, Deref, DerefMut)]
pub struct DebugMode(bool);

pub struct DebugPlugin;
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugMode(false))
            .add_plugins(WorldInspectorPlugin::new().run_if(is_debug_mode))
            .insert_resource(State::new(DebugPickingMode::Disabled))
            .insert_resource(GizmoConfig {
                enabled: true,
                depth_bias: -0.1,
                line_perspective: false,
                line_width: 2.,
                ..default()
            })
            .add_systems(
                Update,
                (
                    toggle_debug_mode,
                    update_debug_picking_mode.run_if(resource_changed::<DebugMode>()),
                    update_gizmo_line_thickness,
                    draw_world_axes
                        .after(CameraMotionSystem)
                        .run_if(is_debug_mode),
                ),
            );
    }
}

fn toggle_debug_mode(mut input: EventReader<InputAction>, mut r_debug: ResMut<DebugMode>) {
    if input
        .iter()
        .any(|action| matches!(action, InputAction(Lifecycle::Stop, Action::ToggleDebug)))
    {
        **r_debug = !**r_debug;
    }
}

fn update_debug_picking_mode(
    r_debug: Res<DebugMode>,
    mut s_debug_picking: ResMut<NextState<DebugPickingMode>>,
) {
    s_debug_picking.set(match **r_debug {
        true => DebugPickingMode::Normal,
        false => DebugPickingMode::Disabled,
    });
}

pub fn is_debug_mode(r_debug: Res<DebugMode>) -> bool {
    **r_debug
}

fn update_gizmo_line_thickness(
    mut config: ResMut<GizmoConfig>,
    q_window: Query<&Window, (Changed<Window>, With<PrimaryWindow>)>,
) {
    if let Ok(scale_factor) = q_window.get_single().map(|win| win.scale_factor() as f32) {
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
