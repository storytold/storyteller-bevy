use std::{f32::consts, mem};

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

use crate::input::{Action, CameraMotion, InputAction, Lifecycle};

pub struct OrbitCamPlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CameraMotionSystem;

impl Plugin for OrbitCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera);
        app.add_systems(
            Update,
            (
                camera_control_kinematic.in_set(CameraMotionSystem),
                update_velocity
                    .in_set(CameraMotionSystem)
                    .after(camera_control_kinematic),
                camera_control_physical
                    .in_set(CameraMotionSystem)
                    .after(update_velocity),
            ),
        );
    }
}

// TODO: These don't really belong in this module
#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct TargetVelocity(pub Vec3);

#[derive(Component)]
pub struct Interp(pub f32);

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
        TargetVelocity(Vec3::ZERO),
        Interp(1.),
    ));
}

fn camera_control_kinematic(
    mut q_window: Query<&mut Window, With<PrimaryWindow>>,
    mut input: EventReader<InputAction>,
    mut q_camera: Query<(
        &mut OrbitCam,
        &mut TargetVelocity,
        &mut Interp,
        &mut Transform,
        &Projection,
    )>,
) {
    if input.is_empty() {
        return;
    }

    let (mut orbit_cam, mut target_vel, mut t, mut xform, proj) = q_camera.single_mut();
    let mut window = q_window.single_mut();

    let mut any_ongoing = false;
    let mut any_started = false;
    let mut any_stopped = false;

    for action in input.iter() {
        #[allow(irrefutable_let_patterns)] // Will eventually be more variants
        if let InputAction(lifecycle, Action::Camera(motion)) = action {
            use CameraMotion::*;

            // Ignore translation actions for the sake of pointer capture
            if mem::discriminant(motion) != mem::discriminant(&Translate(Vec3::ZERO)) {
                match lifecycle {
                    Lifecycle::Start => any_started = true,
                    Lifecycle::Ongoing => any_ongoing = true,
                    Lifecycle::Stop => any_stopped = true,
                }
            }

            match motion {
                Translate(delta) => do_translate(*delta, &mut target_vel, &mut t),
                Pan(delta) => do_pan(proj, &window, *delta, &mut xform, &mut orbit_cam),
                Zoom(delta) => do_zoom(&window, *delta, &mut xform, &mut orbit_cam),
                Orbit(delta) => do_orbit(&window, *delta, *lifecycle, &mut xform, &mut orbit_cam),
                Rotate(delta) => do_rotate(&window, *delta, &mut xform, &mut orbit_cam),
            }
        }
    }

    if any_started {
        capture_pointer(&mut window);
    } else if any_stopped && !any_ongoing {
        release_pointer(&mut window);
    }
}

// TODO: This doesn't really belong in this module
fn update_velocity(
    time: Res<Time>,
    mut q_velocity: Query<(&mut Velocity, &TargetVelocity, &mut Interp)>,
) {
    let dt = time.delta_seconds();
    // TODO: Move these somewhere else -- component maybe?
    let accel_zero2max = 0.333;
    let max_speed = 0.075;

    for (mut velocity, target_vel, mut t) in q_velocity.iter_mut() {
        let target_vel = target_vel.0 * max_speed;
        t.0 = (t.0 + dt / accel_zero2max).clamp(0., 1.);

        velocity.0 = if t.0.nearly_eq(0.) {
            Vec3::ZERO
        } else if t.0.nearly_eq(1.) {
            target_vel
        } else {
            Vec3::lerp(velocity.0, target_vel, curve_sin(t.0))
        };
    }
}

fn camera_control_physical(mut q_camera: Query<(&mut OrbitCam, &mut Transform, &Velocity)>) {
    let (mut orbit_cam, mut xform, velocity) = q_camera.single_mut();
    let delta_pos = xform.rotation * velocity.0;
    xform.translation += delta_pos;
    orbit_cam.focus += delta_pos;
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
fn do_translate(delta: Vec3, target_vel: &mut TargetVelocity, t: &mut Interp) {
    if !delta.nearly_eq(target_vel.0) {
        target_vel.0 = delta;
        t.0 = 0.;
    }
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

/// Fit a [0, 1] `t` interpolation parameter to a sine curve
///
/// https://www.desmos.com/calculator/k9x4c4cr5y
#[inline]
fn curve_sin(t: f32) -> f32 {
    -1. * (f32::cos(t * consts::PI) / 2.) + 0.5
}

trait NearlyEq {
    type Rhs;

    fn nearly_eq(self, rhs: Self::Rhs) -> bool;
}

impl NearlyEq for f32 {
    type Rhs = Self;

    fn nearly_eq(self, rhs: Self::Rhs) -> bool {
        (self - rhs).abs() < Self::EPSILON
    }
}

impl NearlyEq for Vec3 {
    type Rhs = Self;

    fn nearly_eq(self, rhs: Self::Rhs) -> bool {
        self.x.nearly_eq(rhs.x) && self.y.nearly_eq(rhs.y) && self.z.nearly_eq(rhs.z)
    }
}
