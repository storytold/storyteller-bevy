use std::f32::consts;

use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
        fxaa::Fxaa,
        tonemapping::Tonemapping,
    },
    math::vec3,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    input::{Action, CameraMotion, InputAction, Lifecycle},
    math::{Easing, InterpVelocity, Interpolation, InterpolationSystem, NearlyEq, Velocity},
};

pub struct OrbitCamPlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CameraMotionSystem;

impl Plugin for OrbitCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(
            Update,
            (
                camera_control_kinematic,
                process_physical_inputs.before(InterpolationSystem),
                camera_control_physical.after(InterpolationSystem),
            )
                .in_set(CameraMotionSystem),
        );
    }
}

#[derive(Component)]
pub struct OrbitCam {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for OrbitCam {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 10.0,
            upside_down: false,
        }
    }
}

fn spawn_camera(mut cmd: Commands) {
    let focus = vec3(0.35011, 0.5, -0.99576);
    let camera_xform = Transform::from_xyz(-2.5, 4.5, 5.0).looking_at(focus, Vec3::Y);

    cmd.spawn((
        // FIXME: Come up with a good way to separate the camera controller from
        //        all the render settings here (all of these components need to
        //        be paired with the camera)
        Camera3dBundle {
            transform: camera_xform,
            tonemapping: Tonemapping::BlenderFilmic,
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        BloomSettings {
            composite_mode: BloomCompositeMode::EnergyConserving,
            intensity: 0.25,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 0.8,
                threshold_softness: 0.5,
            },
            ..default()
        },
        Fxaa::default(),
        OrbitCam {
            focus,
            radius: Vec3::distance(camera_xform.translation, focus),
            ..default()
        },
        Velocity(Vec3::ZERO),
    ));
}

fn camera_control_kinematic(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut input: EventReader<InputAction>,
    mut q_camera: Query<(&mut OrbitCam, &mut Transform, &Projection)>,
) {
    if input.is_empty() {
        return;
    }

    let (mut orbit_cam, mut xform, proj) = q_camera.single_mut();
    let mut window = q_window.single_mut();

    let mut any_ongoing = false;
    let mut any_started = false;
    let mut any_stopped = false;

    for action in input.iter().filter(|action| {
        !matches!(
            action,
            InputAction(_, Action::Camera(CameraMotion::Translate(_)))
        )
    }) {
        #[allow(irrefutable_let_patterns)] // Will eventually be more variants
        if let InputAction(lifecycle, Action::Camera(motion)) = action {
            use CameraMotion::*;

            match lifecycle {
                Lifecycle::Start => any_started = true,
                Lifecycle::Ongoing => any_ongoing = true,
                Lifecycle::Stop => any_stopped = true,
            }

            match motion {
                Pan(delta) => do_pan(proj, &window, *delta, &mut xform, &mut orbit_cam),
                Zoom(delta) => do_zoom(&window, *delta, &mut xform, &mut orbit_cam),
                Orbit(delta) => do_orbit(&window, *delta, *lifecycle, &mut xform, &mut orbit_cam),
                Rotate(delta) => do_rotate(&window, *delta, &mut xform, &mut orbit_cam),
                _ => {}
            }
        }
    }

    if any_started {
        capture_pointer(&mut window);
    } else if any_stopped && !any_ongoing {
        release_pointer(&mut window);
    }
}

fn process_physical_inputs(
    mut cmd: Commands,
    mut input: EventReader<InputAction>,
    q_camera: Query<(Entity, &Velocity), With<OrbitCam>>,
    mut q_interp: Query<&mut InterpVelocity>,
) {
    for action in input.iter() {
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

fn camera_control_physical(mut q_camera: Query<(&mut OrbitCam, &mut Transform, &Velocity)>) {
    for (mut orbit_cam, mut xform, velocity) in q_camera.iter_mut() {
        let delta_pos = xform.rotation * (**velocity);
        xform.translation += delta_pos;
        orbit_cam.focus += delta_pos;
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
    orbit_cam: &mut OrbitCam,
) {
    // Set `upside_down` flag if the event just started or stopped
    if matches!(event_lifecycle, Lifecycle::Start | Lifecycle::Stop) {
        orbit_cam.upside_down = xform.up().y <= 0.;
    }

    // Translate to origin
    xform.translation += xform.forward() * orbit_cam.radius;

    // Calculate rotation
    let delta_x = {
        let delta = delta.x / (win.physical_width() as f32) * consts::PI * 2.0;
        match orbit_cam.upside_down {
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
    xform.translation += xform.back() * orbit_cam.radius;
}

#[inline]
fn do_pan(
    proj: &Projection,
    win: &Window,
    delta: Vec2,
    xform: &mut Transform,
    orbit_cam: &mut OrbitCam,
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
    let up = xform.up();
    let left = xform.left();
    let delta = (pan.y * up + pan.x * left) * orbit_cam.radius;

    // Update camera translation and focus point
    xform.translation += delta;
    orbit_cam.focus += delta;
}

#[inline]
fn do_zoom(win: &Window, delta: f32, xform: &mut Transform, orbit_cam: &mut OrbitCam) {
    let delta = (delta * 1.5) / (win.physical_height() as f32);
    orbit_cam.radius -= orbit_cam.radius * delta;
    xform.translation = orbit_cam.focus + xform.back() * orbit_cam.radius;
}

#[inline]
fn do_rotate(win: &Window, delta: Vec2, xform: &mut Transform, orbit_cam: &mut OrbitCam) {
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
    orbit_cam.focus = xform.translation + xform.forward() * orbit_cam.radius;
}
