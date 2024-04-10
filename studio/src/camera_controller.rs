use std::f32::consts;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    input::{self, Action, CameraMotion, InputAction, Lifecycle},
    interaction::Selection,
    math::{
        Easing, InterpPosition, InterpVelocity, Interpolation, InterpolationSystem, NearlyEq,
        NearlyZero, Velocity,
    },
};

pub struct CameraControllerPlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CameraMotionSystem;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                camera_control_kinematic,
                (process_physical_inputs, process_focus_inputs).before(InterpolationSystem),
                (camera_control_physical, interp_camera_focus).after(InterpolationSystem),
            )
                .in_set(CameraMotionSystem),
        );

        #[cfg(not(feature = "wasm"))]
        app.add_systems(PreUpdate, sync_camera_viewport);
    }
}

#[cfg(not(feature = "wasm"))]
fn sync_camera_viewport(
    mut camera: Query<&mut Camera, With<crate::MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    use bevy::render::camera::Viewport;

    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(window) = window.get_single() else {
        return;
    };

    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(0, 0),
        physical_size: UVec2::new(window.physical_width(), window.physical_height()),
        ..default()
    });
}

#[derive(Component)]
pub struct CameraController {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 10.0,
            upside_down: false,
        }
    }
}

#[derive(Bundle)]
pub struct CameraControllerBundle {
    controller: CameraController,
    velocity: Velocity,
}

impl CameraControllerBundle {
    pub fn new(focus: Vec3, position: Vec3) -> Self {
        Self {
            controller: CameraController {
                focus,
                radius: Vec3::distance(position, focus),
                ..default()
            },
            velocity: Velocity(Vec3::ZERO),
        }
    }
}

fn camera_control_kinematic(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut input: EventReader<InputAction>,
    mut q_camera: Query<(&mut CameraController, &mut Transform, &Projection)>,
) {
    if input.is_empty() {
        return;
    }

    let mut any_ongoing = false;
    let mut any_started = false;
    let mut any_stopped = false;

    for action in input.read().filter(|action| {
        !matches!(
            action,
            InputAction(_, Action::Camera(CameraMotion::Translate(_)))
        )
    }) {
        if let InputAction(lifecycle, Action::Camera(motion)) = action {
            let (mut controller, mut xform, proj) = q_camera.single_mut();
            let window = q_window.single();

            match lifecycle {
                Lifecycle::Start => any_started = true,
                Lifecycle::Ongoing => any_ongoing = true,
                Lifecycle::Stop => any_stopped = true,
            }

            use CameraMotion::*;
            match motion {
                Pan(delta) => do_pan(proj, window, *delta, &mut xform, &mut controller),
                Zoom(delta) => do_zoom(window, *delta, &mut xform, &mut controller),
                Orbit(delta) => do_orbit(window, *delta, *lifecycle, &mut xform, &mut controller),
                Rotate(delta) => do_rotate(window, *delta, &mut xform, &mut controller),
                _ => {}
            }
        }
    }

    if any_started {
        capture_pointer(&mut q_window.single_mut());
    } else if any_stopped && !any_ongoing {
        release_pointer(&mut q_window.single_mut());
    }
}

fn process_physical_inputs(
    mut cmd: Commands,
    mut input: EventReader<InputAction>,
    q_camera: Query<(Entity, &Velocity), With<CameraController>>,
    mut q_interp: Query<&mut InterpVelocity>,
) {
    for action in input.read() {
        let &InputAction(_, Action::Camera(CameraMotion::Translate(delta))) = action else {
            continue;
        };

        for (ent, velocity) in q_camera.iter() {
            let target = delta * 0.075;

            if let Ok(mut interp) = q_interp.get_mut(ent) {
                if interp.target.nearly_eq_within(target, 1e-3) {
                    continue;
                }

                interp.initial = **velocity;
                interp.target = target;
                interp.t = 0.;
            } else {
                cmd.entity(ent).insert(InterpVelocity {
                    inner: Interpolation {
                        initial: **velocity,
                        target,
                        easing: Easing::Sine,
                        ..default()
                    },
                    accel_zero2max: 0.333,
                });
            }
        }
    }
}

fn process_focus_inputs(
    mut cmd: Commands,
    r_selection: Option<Res<Selection>>,
    mut input: EventReader<InputAction>,
    mut q_camera: Query<(Entity, &CameraController, &mut Velocity)>,
    q_interp_vel: Query<&InterpVelocity>,
    mut q_interp_pos: Query<&mut InterpPosition>,
    q_global_xforms: Query<&GlobalTransform>,
) {
    let Some(selection) = r_selection else {
        return;
    };

    if input.read().any(|action| {
        matches!(
            action,
            InputAction(Lifecycle::Stop, Action::Selection(input::Selection::Focus))
        )
    }) {
        let Ok(target) = q_global_xforms
            .get(**selection)
            .map(|xform| xform.translation())
        else {
            return;
        };

        for (ent, controller, mut velocity) in q_camera.iter_mut() {
            // Zero out any existing camera velocity or interpolation
            **velocity = Vec3::ZERO;

            info!("Setting focus to {:?}", target);

            if q_interp_vel.get(ent).is_ok() {
                cmd.entity(ent).remove::<InterpVelocity>();
            }

            let initial = controller.focus;
            let easing = Easing::SineOut;
            let duration = 0.25;

            if let Ok(mut interp) = q_interp_pos.get_mut(ent) {
                if interp.target.nearly_eq_within(target, 1e-3) {
                    continue;
                }

                interp.initial = initial;
                interp.target = target;
                interp.easing = easing;
                interp.t = 0.;
                interp.duration = duration;
            } else {
                cmd.entity(ent).insert(InterpPosition {
                    inner: Interpolation {
                        initial,
                        target,
                        easing,
                        ..default()
                    },
                    duration,
                });
            }
        }
    }
}

fn camera_control_physical(
    mut q_camera: Query<(Entity, &mut CameraController, &Velocity)>,
    mut q_xforms: Query<&mut Transform>,
) {
    for (ent, mut controller, velocity) in q_camera.iter_mut() {
        if !velocity.nearly_zero() {
            let mut xform = q_xforms.get_mut(ent).unwrap();
            let delta_pos = xform.rotation * (**velocity);
            xform.translation += delta_pos;
            controller.focus += delta_pos;
        }
    }
}

fn interp_camera_focus(
    mut q_camera: Query<(&mut CameraController, &mut Transform, &InterpPosition)>,
) {
    for (mut controller, mut xform, interp) in q_camera.iter_mut() {
        let target_focus = interp.current();
        let delta_pos = target_focus - controller.focus;

        controller.focus = target_focus;
        xform.translation += delta_pos;
    }
}

#[inline]
fn capture_pointer(window: &mut Window) {
    window.cursor.visible = false;
    window.cursor.grab_mode = CursorGrabMode::Confined;
}

#[inline]
fn release_pointer(window: &mut Window) {
    window.cursor.visible = true;
    window.cursor.grab_mode = CursorGrabMode::None;
}

#[inline]
fn do_orbit(
    win: &Window,
    delta: Vec2,
    event_lifecycle: Lifecycle,
    xform: &mut Transform,
    controller: &mut CameraController,
) {
    // Set `upside_down` flag if the event just started or stopped
    if matches!(event_lifecycle, Lifecycle::Start | Lifecycle::Stop) {
        controller.upside_down = xform.up().y <= 0.;
    }

    // Translate to origin
    xform.translation += xform.forward() * controller.radius;

    // Calculate rotation
    let delta_x = {
        let delta = delta.x / (win.physical_width() as f32) * consts::PI * 2.0;
        match controller.upside_down {
            true => -delta,
            false => delta,
        }
    };
    let delta_y = delta.y / (win.physical_height() as f32) * consts::PI;

    let yaw = Quat::from_rotation_y(-delta_x);
    let pitch = Quat::from_rotation_x(-delta_y);

    // Rotate around global Y and local X
    xform.rotation = yaw * xform.rotation;
    xform.rotation *= pitch;

    // Translate back to the original position
    xform.translation += xform.back() * controller.radius;
}

#[inline]
fn do_pan(
    proj: &Projection,
    win: &Window,
    delta: Vec2,
    xform: &mut Transform,
    controller: &mut CameraController,
) {
    // Normalize the pan distance with respect to the window resolution
    let pan = match proj {
        Projection::Perspective(p) => {
            delta
                * Vec2 {
                    x: (p.fov * p.aspect_ratio) / (win.physical_width() as f32),
                    y: p.fov / (win.physical_height() as f32),
                }
        }
        _ => delta,
    };

    // Pan along local XY plane, with the distance scaled by the radius
    let up = xform.up().xyz();
    let left = xform.left().xyz();
    let delta = (pan.y * up + pan.x * left) * controller.radius;

    // Update camera translation and focus point
    xform.translation += delta;
    controller.focus += delta;
}

#[inline]
fn do_zoom(win: &Window, delta: f32, xform: &mut Transform, controller: &mut CameraController) {
    let delta = (delta * 1.5) / (win.physical_height() as f32);
    controller.radius -= controller.radius * delta;
    xform.translation = controller.focus + xform.back() * controller.radius;
}

#[inline]
fn do_rotate(win: &Window, delta: Vec2, xform: &mut Transform, controller: &mut CameraController) {
    // TODO: Move this somewhere else
    let sensitivity = 0.75;

    let delta_x = delta.x / (win.physical_width() as f32) * consts::PI * 2.0 * sensitivity;
    let delta_y = delta.y / (win.physical_height() as f32) * consts::PI * sensitivity;

    let yaw = Quat::from_rotation_y(-delta_x);
    let pitch = Quat::from_rotation_x(-delta_y);

    // Rotate around global Y and local X
    xform.rotation = yaw * xform.rotation;
    xform.rotation *= pitch;

    // Update focus point
    controller.focus = xform.translation + xform.forward() * controller.radius;
}
