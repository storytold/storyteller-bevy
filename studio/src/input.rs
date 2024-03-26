use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    utils::HashSet,
};
use space_editor::prelude::space_undo::UndoRedo;

use crate::is_editor;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct StudioInputSystem;

pub struct StudioInputPlugin;
impl Plugin for StudioInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InputAction>();

        // Reading each frame of the mouse delta allows
        // to get rid of the bug that delta is counted from the last reading,
        // which leads to camera jumps if you move the mouse between clicks
        app.add_systems(
            PreUpdate,
            read_mouse_delta
                .in_set(StudioInputSystem)
                .after(bevy::input::InputSystem),
        );
        app.init_resource::<LastFrameMouseDelta>();

        app.add_systems(
            PreUpdate,
            (
                translate_selection_events,
                translate_debug_toggle,
                translate_dcc_camera_events,
                translate_fps_camera_events,
                send_undo_event.run_if(is_editor),
            )
                .in_set(StudioInputSystem)
                .after(bevy::input::InputSystem)
                .after(read_mouse_delta),
        );

        app.add_systems(
            PostUpdate,
            (apply_deferred, log_input_actions).after(StudioInputSystem),
        );
    }
}

/// Logs all `InputAction`s emitted, along with the input state at the time they
/// were fired, with `debug` verbosity.
fn log_input_actions(
    ri_keyboard: Res<Input<KeyCode>>,
    ri_mouse_buttons: Res<Input<MouseButton>>,
    mut er_input_actions: EventReader<InputAction>,
) {
    if er_input_actions.is_empty() {
        return;
    }

    let lmb = if ri_mouse_buttons.pressed(MouseButton::Left) {
        'X'
    } else {
        '-'
    };
    let mmb = if ri_mouse_buttons.pressed(MouseButton::Middle) {
        'X'
    } else {
        '-'
    };
    let rmb = if ri_mouse_buttons.pressed(MouseButton::Right) {
        'X'
    } else {
        '-'
    };

    use KeyCode::*;
    let mod_keys = [AltLeft, AltRight, ShiftLeft, ControlLeft, ControlRight]
        .into_iter()
        .collect::<HashSet<_>>();
    let action_keys = [W, A, S, D, E, Q, Escape, Grave, Delete, F]
        .into_iter()
        .collect::<HashSet<_>>();

    let keys_pressed = ri_keyboard.get_pressed().copied().collect::<HashSet<_>>();
    let keys_display = keys_pressed
        .intersection(&mod_keys)
        .map(|key| format!("{key:?}"))
        .chain(
            keys_pressed
                .intersection(&action_keys)
                .map(|key| format!("{key:?}")),
        )
        .collect::<Vec<_>>()
        .join("+");

    for action in er_input_actions.read() {
        debug!(
            "Mouse: [{lmb}{mmb}{rmb}]  \
            Keyboard: [{keys_display}]  \
            Action: [{action:.2?}]"
        );
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct InputAction(pub Lifecycle, pub Action);

#[derive(Clone, Copy, Debug)]
pub enum Lifecycle {
    Start,
    Ongoing,
    Stop,
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Camera(CameraMotion),
    Selection(Selection),
    ToggleDebug,
}

#[derive(Clone, Copy, Debug)]
pub enum CameraMotion {
    Translate(Vec3),
    Pan(Vec2),
    Zoom(f32),
    Orbit(Vec2),
    Rotate(Vec2),
}
impl From<CameraMotion> for Action {
    fn from(value: CameraMotion) -> Self {
        Action::Camera(value)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Selection {
    /// Clear the current selection
    Clear,
    /// Focus the camera on the currently selected entity
    Focus,
    /// Delete the currently selected entity and its descendants
    Delete,
}
impl From<Selection> for Action {
    fn from(value: Selection) -> Self {
        Action::Selection(value)
    }
}

// TODO: Make the bindings of raw device inputs to InputActions configurable

fn translate_selection_events(keyboard: Res<Input<KeyCode>>, mut input: EventWriter<InputAction>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        input.send(InputAction(Lifecycle::Start, Selection::Clear.into()));
    } else if keyboard.just_released(KeyCode::Escape) {
        input.send(InputAction(Lifecycle::Stop, Selection::Clear.into()));
    }

    if keyboard.just_pressed(KeyCode::F) {
        input.send(InputAction(Lifecycle::Start, Selection::Focus.into()));
    } else if keyboard.just_released(KeyCode::F) {
        input.send(InputAction(Lifecycle::Stop, Selection::Focus.into()));
    }

    if keyboard.just_pressed(KeyCode::Delete) {
        input.send(InputAction(Lifecycle::Start, Selection::Delete.into()));
    } else if keyboard.just_released(KeyCode::Delete) {
        input.send(InputAction(Lifecycle::Stop, Selection::Delete.into()));
    }
}

fn translate_debug_toggle(keyboard: Res<Input<KeyCode>>, mut input: EventWriter<InputAction>) {
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::Grave) {
            input.send(InputAction(Lifecycle::Start, Action::ToggleDebug));
        } else if keyboard.just_released(KeyCode::Grave) {
            input.send(InputAction(Lifecycle::Stop, Action::ToggleDebug));
        }
    }
}

fn translate_dcc_camera_events(
    mouse_motion: Res<LastFrameMouseDelta>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut input: EventWriter<InputAction>,
) {
    let delta = mouse_motion.get();
    // Camera Pan
    if mouse_buttons.pressed(MouseButton::Middle) {
        let lifecycle = if mouse_buttons.just_pressed(MouseButton::Middle) {
            Lifecycle::Start
        } else {
            Lifecycle::Ongoing
        };
        input.send(InputAction(lifecycle, CameraMotion::Pan(delta).into()));
    }
    // Stop Camera Panning
    else if mouse_buttons.just_released(MouseButton::Middle) {
        input.send(InputAction(
            Lifecycle::Stop,
            CameraMotion::Pan(Vec2::ZERO).into(),
        ));
    }
    // Camera Orbit/Zoom
    else if keyboard.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
        if mouse_buttons.any_pressed([MouseButton::Left, MouseButton::Right]) {
            let lifecycle =
                if mouse_buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
                    Lifecycle::Start
                } else {
                    Lifecycle::Ongoing
                };

            let motion = if mouse_buttons.pressed(MouseButton::Left) {
                CameraMotion::Orbit(delta)
            } else {
                CameraMotion::Zoom(delta.y)
            };

            input.send(InputAction(lifecycle, motion.into()));
        }
        // Stop Camera Orbit
        else if mouse_buttons.just_released(MouseButton::Left) {
            input.send(InputAction(
                Lifecycle::Stop,
                CameraMotion::Orbit(Vec2::ZERO).into(),
            ));
        }
        // Stop Camera Zoom
        else if mouse_buttons.just_released(MouseButton::Right) {
            input.send(InputAction(Lifecycle::Stop, CameraMotion::Zoom(0.).into()));
        }
    }
    // Stop Camera Orbit & Zoom
    else if (keyboard.just_released(KeyCode::AltLeft) && !keyboard.pressed(KeyCode::AltRight))
        || (keyboard.just_released(KeyCode::AltRight) && !keyboard.pressed(KeyCode::AltLeft))
    {
        // Stop Camera Orbit
        if mouse_buttons.pressed(MouseButton::Left) {
            input.send(InputAction(
                Lifecycle::Stop,
                CameraMotion::Orbit(Vec2::ZERO).into(),
            ));
        }
        // Stop Camera Zoom
        else if mouse_buttons.pressed(MouseButton::Right) {
            input.send(InputAction(Lifecycle::Stop, CameraMotion::Zoom(0.).into()));
        }
    }
    // Camera Rotation
    else if mouse_buttons.pressed(MouseButton::Right) {
        let lifecycle = if mouse_buttons.just_pressed(MouseButton::Right) {
            Lifecycle::Start
        } else {
            Lifecycle::Ongoing
        };

        input.send(InputAction(lifecycle, CameraMotion::Rotate(delta).into()));
    }
    // Stop Camera Rotation
    else if mouse_buttons.just_released(MouseButton::Right) {
        input.send(InputAction(
            Lifecycle::Stop,
            CameraMotion::Rotate(Vec2::ZERO).into(),
        ));
    }
    // Alternative Camera Rotation (designed for laptop)
    else if keyboard.pressed(KeyCode::ShiftLeft) {
        let lifecycle = if keyboard.just_pressed(KeyCode::ShiftLeft) {
            Lifecycle::Start
        } else {
            Lifecycle::Ongoing
        };

        input.send(InputAction(lifecycle, CameraMotion::Rotate(delta).into()));
    }
    // Stop Camera Rotation
    else if keyboard.just_released(KeyCode::ShiftLeft) {
        input.send(InputAction(
            Lifecycle::Stop,
            CameraMotion::Rotate(Vec2::ZERO).into(),
        ));
    }
}

fn translate_fps_camera_events(
    mut mouse_wheel: EventReader<MouseWheel>,
    keyboard: Res<Input<KeyCode>>,
    mut input: EventWriter<InputAction>,
) {
    // Camera Translation (FPS-like)
    use KeyCode::*;
    let wasd_keys = [W, A, S, D, E, Q];
    let wasd_keys_set = wasd_keys.into_iter().collect::<HashSet<_>>();

    // A little unfortunate that `get_pressed()` doesn't just return a reference
    // to the original HashSet >_< Maybe the compiler will be nice to us and
    // optimize away the redundant clone?
    let keys_pressed = keyboard.get_pressed().cloned().collect::<HashSet<_>>();
    let wasd_keys_pressed = wasd_keys_set
        .intersection(&keys_pressed)
        .collect::<HashSet<_>>();

    if !wasd_keys_pressed.is_empty() {
        let mut mvmt = Vec3::ZERO;

        #[rustfmt::skip] {
        if wasd_keys_pressed.contains(&W) { mvmt.z -= 1.; }
        if wasd_keys_pressed.contains(&A) { mvmt.x -= 1.; }
        if wasd_keys_pressed.contains(&S) { mvmt.z += 1.; }
        if wasd_keys_pressed.contains(&D) { mvmt.x += 1.; }
        if wasd_keys_pressed.contains(&E) { mvmt.y += 1.; }
        if wasd_keys_pressed.contains(&Q) { mvmt.y -= 1.; }
        };

        let mvmt = mvmt.normalize_or_zero();

        // Compare WASD keys pressed with WASD keys _just_ pressed to determine lifecycle
        let keys_just_pressed = keyboard.get_just_pressed().cloned().collect::<HashSet<_>>();
        let wasd_keys_just_pressed = keys_just_pressed
            .intersection(&wasd_keys_set)
            .collect::<HashSet<_>>();

        let lifecycle = if wasd_keys_pressed.len() == wasd_keys_just_pressed.len() {
            Lifecycle::Start
        } else {
            Lifecycle::Ongoing
        };

        input.send(InputAction(lifecycle, CameraMotion::Translate(mvmt).into()));
    }
    // Stop Translation
    else if keyboard.any_just_released(wasd_keys) {
        input.send(InputAction(
            Lifecycle::Stop,
            CameraMotion::Translate(Vec3::ZERO).into(),
        ));
    }

    // Wheel zoom
    for &MouseWheel { y, .. } in mouse_wheel.read() {
        // info!("wheel: ({x}, {y}) [{unit:?}]");
        // let rem_x = (100. / x).abs() % 1.;
        // let rem_y = (100. / y).abs() % 1.;

        // if (rem_x.is_nan() || rem_x < 1e-5_f32) && (rem_y.is_nan() || rem_y < 1e-5_f32) {
        input.send(InputAction(
            Lifecycle::Ongoing,
            CameraMotion::Zoom(y).into(),
        ));
        // } else {
        // FIXME: I had high hopes for this, but the input translation from
        //        Bevy / winit is herky-jerky-janky af. It seems to be
        //        trying to force the wheel to only emit one axis at a time,
        //        unless the user makes a really deliberate diagonal motion
        //        to start. But even then, it makes bizarre jumps
        //        intermittently instead of just translating the input
        //        motion directly.
        //
        //        I Was hoping to achieve something like what Blender does
        //        with touchpad input to pan, but it doesn't look like
        //        that's going to be possible without bypassing bevy::input
        //        and reading the system events directly. We may want to do
        //        that eventually, but not today.

        // input.send(InputAction(
        //     Lifecycle::Ongoing,
        //     CameraMotion::Pan(vec2(x, y)).into(),
        // ));
        // }
    }
}

#[derive(Resource, Default)]
pub struct LastFrameMouseDelta(pub Vec2);

impl LastFrameMouseDelta {
    fn get(&self) -> Vec2 {
        self.0
    }
}

fn read_mouse_delta(
    mut mouse_motion: EventReader<MouseMotion>,
    mut last_delta: ResMut<LastFrameMouseDelta>,
) {
    let delta = mouse_motion
        .read()
        .fold(Vec2::ZERO, |accum, current| accum + current.delta);
    last_delta.0 = delta;
}

// TODO: Make it configurable
fn send_undo_event(mut events: EventWriter<UndoRedo>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.just_pressed(KeyCode::Z)
        && !keyboard.pressed(KeyCode::ShiftLeft)
    {
        events.send(UndoRedo::Undo);
        info!("Undo event sent");
    }

    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.just_pressed(KeyCode::Z)
        && keyboard.pressed(KeyCode::ShiftLeft)
    {
        events.send(UndoRedo::Redo);
        info!("Redo event sent");
    }
}
