use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    utils::HashSet,
    window::PrimaryWindow,
};
use space_editor::prelude::space_undo::UndoRedo;

use crate::{is_editor, math::NearlyZero};

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct StudioInputSystem;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct CameraInputSystem;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct EditorInputSystem;

#[derive(SystemSet, Hash, Debug, PartialEq, Eq, Clone)]
pub struct SelectionInputSystem;

pub struct StudioInputPlugin;
impl Plugin for StudioInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CameraAction>()
            .add_event::<SelectionAction>()
            .add_event::<EditorAction>();

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
            (translate_dcc_camera_events, translate_fps_camera_events)
                .in_set(StudioInputSystem)
                .in_set(CameraInputSystem)
                .after(bevy::input::InputSystem)
                .after(read_mouse_delta),
        );

        app.add_systems(
            PreUpdate,
            (translate_debug_toggle, send_undo_event.run_if(is_editor))
                .in_set(StudioInputSystem)
                .in_set(EditorInputSystem)
                .after(bevy::input::InputSystem)
                .after(read_mouse_delta),
        );

        app.add_systems(
            PreUpdate,
            (apply_deferred, translate_selection_events)
                .in_set(StudioInputSystem)
                .in_set(SelectionInputSystem)
                .after(bevy::input::InputSystem)
                .after(CameraInputSystem)
                .after(read_mouse_delta),
        );

        app.add_systems(
            PostUpdate,
            (apply_deferred, log_input_actions).after(StudioInputSystem),
        );
    }
}

// #[derive(Event, Clone, Copy, Debug)]
// pub struct InputAction(pub Lifecycle, pub Action);

#[derive(Clone, Copy, Debug)]
pub enum Lifecycle {
    Start,
    Ongoing,
    Stop,
    Cancel,
}

// #[derive(Clone, Copy, Debug)]
// pub enum Action {
//     Camera(CameraMotion),
//     Selection(Selection),
//     ToggleDebug,
// }

#[derive(Event, Clone, Copy, Debug)]
pub struct CameraAction(pub Lifecycle, pub CameraMotion);

#[derive(Clone, Copy, Debug)]
pub enum CameraMotion {
    Translate(Vec3),
    Pan(Vec2),
    Zoom(f32),
    Orbit(Vec2),
    Rotate(Vec2),
}
// impl From<CameraMotion> for Action {
//     fn from(value: CameraMotion) -> Self {
//         Action::Camera(value)
//     }
// }

#[derive(Event, Clone, Copy, Debug)]
pub struct SelectionAction(pub Lifecycle, pub Selection);

#[derive(Clone, Copy, Debug)]
pub enum Selection {
    /// Clear the current selection
    Clear,
    /// Focus the camera on the currently selected entity
    Focus,
    /// Delete the currently selected entity and its descendants
    Delete,
    /// Select the single target under the cursor position
    Single(Vec2),
    /// Add or remove the target under the cursor position to the current
    /// selection
    Multi(Vec2),
    /// Select all targets that overlap the marquee selection
    Marquee(Rect),
}
// impl From<Selection> for Action {
//     fn from(value: Selection) -> Self {
//         Action::Selection(value)
//     }
// }

#[derive(Event, Clone, Copy, Debug)]
pub struct EditorAction(pub Lifecycle, pub Editor);

#[derive(Clone, Copy, Debug)]
pub enum Editor {
    ToggleDebug,
}

// TODO: Make the bindings of raw device inputs to InputActions configurable

fn translate_selection_events(
    r_mouse_motion: Res<LastFrameMouseDelta>,
    ri_mouse_buttons: Res<ButtonInput<MouseButton>>,
    ri_keyboard: Res<ButtonInput<KeyCode>>,
    mut l_pointer_drag_start: Local<Option<Vec2>>,
    mut l_drag_dist2: Local<f32>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut er_camera_actions: EventReader<CameraAction>,
    mut ew_actions: EventWriter<SelectionAction>,
) {
    // Keyboard actions -- clear, focus, etc.
    if ri_keyboard.just_pressed(KeyCode::Escape) {
        ew_actions.send(SelectionAction(Lifecycle::Start, Selection::Clear));
    } else if ri_keyboard.just_released(KeyCode::Escape) {
        ew_actions.send(SelectionAction(Lifecycle::Stop, Selection::Clear));
    }

    if ri_keyboard.just_pressed(KeyCode::KeyF) {
        ew_actions.send(SelectionAction(Lifecycle::Start, Selection::Focus));
    } else if ri_keyboard.just_released(KeyCode::KeyF) {
        ew_actions.send(SelectionAction(Lifecycle::Stop, Selection::Focus));
    }

    if ri_keyboard.just_pressed(KeyCode::Delete) {
        ew_actions.send(SelectionAction(Lifecycle::Start, Selection::Delete));
    } else if ri_keyboard.just_released(KeyCode::Delete) {
        ew_actions.send(SelectionAction(Lifecycle::Stop, Selection::Delete));
    }

    // Mouse-based actions
    if !er_camera_actions.is_empty() {
        // We don't want to emit any mouse-triggered selection actions if
        // we've already emitted some kind of camera motion this frame. This
        // avoids UX jank where releasing the LMB at the end of e.g.
        // orbiting the camera causes the object under the cursor to become
        // selected as an unintended side-effect.
        er_camera_actions.clear();
        return;
    }

    let Some(mouse_pos) = q_window
        .get_single()
        .ok()
        .and_then(|window| window.cursor_position())
    else {
        return;
    };

    if ri_mouse_buttons.just_pressed(MouseButton::Left) {
        *l_drag_dist2 = 0.;
        *l_pointer_drag_start = Some(mouse_pos);

        ew_actions.send(SelectionAction(
            Lifecycle::Start,
            Selection::Marquee(Rect::from_corners(mouse_pos, mouse_pos)),
        ));
    } else if ri_mouse_buttons.pressed(MouseButton::Left) {
        *l_drag_dist2 += r_mouse_motion.0.length_squared();

        let Some(drag_start) = *l_pointer_drag_start else {
            return;
        };

        ew_actions.send(SelectionAction(
            Lifecycle::Ongoing,
            Selection::Marquee(Rect::from_corners(drag_start, mouse_pos)),
        ));
    } else if ri_mouse_buttons.just_released(MouseButton::Left) {
        *l_drag_dist2 += r_mouse_motion.0.length_squared();

        let Some(drag_start) = l_pointer_drag_start.take() else {
            return;
        };

        // If the total drag distance is nearly zero, treat this as a single-click
        if l_drag_dist2.nearly_zero_within(1e-3) {
            ew_actions.send(SelectionAction(
                Lifecycle::Cancel,
                Selection::Marquee(Rect::from_corners(drag_start, mouse_pos)),
            ));

            if ri_keyboard.any_pressed([
                KeyCode::ShiftLeft,
                KeyCode::ShiftRight,
                KeyCode::ControlLeft,
                KeyCode::ControlRight,
            ]) {
                ew_actions.send(SelectionAction(
                    Lifecycle::Stop,
                    Selection::Multi(mouse_pos),
                ));
            } else {
                ew_actions.send(SelectionAction(
                    Lifecycle::Stop,
                    Selection::Single(mouse_pos),
                ));
            }
        } else {
            ew_actions.send(SelectionAction(
                Lifecycle::Stop,
                Selection::Marquee(Rect::from_corners(drag_start, mouse_pos)),
            ));
        }
    }
}

fn translate_debug_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: EventWriter<EditorAction>,
) {
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if keyboard.just_pressed(KeyCode::Backquote) {
            input.send(EditorAction(Lifecycle::Start, Editor::ToggleDebug));
        } else if keyboard.just_released(KeyCode::Backquote) {
            input.send(EditorAction(Lifecycle::Stop, Editor::ToggleDebug));
        }
    }
}

fn translate_dcc_camera_events(
    mouse_motion: Res<LastFrameMouseDelta>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: EventWriter<CameraAction>,
) {
    let delta = mouse_motion.get();
    // Camera Pan
    if mouse_buttons.pressed(MouseButton::Middle) {
        let lifecycle = if mouse_buttons.just_pressed(MouseButton::Middle) {
            Lifecycle::Start
        } else {
            Lifecycle::Ongoing
        };
        input.send(CameraAction(lifecycle, CameraMotion::Pan(delta)));
    }
    // Stop Camera Panning
    else if mouse_buttons.just_released(MouseButton::Middle) {
        input.send(CameraAction(Lifecycle::Stop, CameraMotion::Pan(Vec2::ZERO)));
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

            input.send(CameraAction(lifecycle, motion));
        }
        // Stop Camera Orbit
        else if mouse_buttons.just_released(MouseButton::Left) {
            input.send(CameraAction(
                Lifecycle::Stop,
                CameraMotion::Orbit(Vec2::ZERO),
            ));
        }
        // Stop Camera Zoom
        else if mouse_buttons.just_released(MouseButton::Right) {
            input.send(CameraAction(Lifecycle::Stop, CameraMotion::Zoom(0.)));
        }
    }
    // Stop Camera Orbit & Zoom
    else if (keyboard.just_released(KeyCode::AltLeft) && !keyboard.pressed(KeyCode::AltRight))
        || (keyboard.just_released(KeyCode::AltRight) && !keyboard.pressed(KeyCode::AltLeft))
    {
        // Stop Camera Orbit
        if mouse_buttons.pressed(MouseButton::Left) {
            input.send(CameraAction(
                Lifecycle::Stop,
                CameraMotion::Orbit(Vec2::ZERO),
            ));
        }
        // Stop Camera Zoom
        else if mouse_buttons.pressed(MouseButton::Right) {
            input.send(CameraAction(Lifecycle::Stop, CameraMotion::Zoom(0.)));
        }
    }
    // Camera Rotation
    else if mouse_buttons.pressed(MouseButton::Right) {
        let lifecycle = if mouse_buttons.just_pressed(MouseButton::Right) {
            Lifecycle::Start
        } else {
            Lifecycle::Ongoing
        };

        input.send(CameraAction(lifecycle, CameraMotion::Rotate(delta)));
    }
    // Stop Camera Rotation
    else if mouse_buttons.just_released(MouseButton::Right) {
        input.send(CameraAction(
            Lifecycle::Stop,
            CameraMotion::Rotate(Vec2::ZERO),
        ));
    }
    // Alternative Camera Rotation (designed for laptop)
    else if keyboard.pressed(KeyCode::ShiftLeft) {
        let lifecycle = if keyboard.just_pressed(KeyCode::ShiftLeft) {
            Lifecycle::Start
        } else {
            Lifecycle::Ongoing
        };

        input.send(CameraAction(lifecycle, CameraMotion::Rotate(delta)));
    }
    // Stop Camera Rotation
    else if keyboard.just_released(KeyCode::ShiftLeft) {
        input.send(CameraAction(
            Lifecycle::Stop,
            CameraMotion::Rotate(Vec2::ZERO),
        ));
    }
}

fn translate_fps_camera_events(
    mut mouse_wheel: EventReader<MouseWheel>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: EventWriter<CameraAction>,
) {
    // Camera Translation (FPS-like)
    use KeyCode::*;
    let wasd_keys = [KeyW, KeyA, KeyS, KeyD, KeyE, KeyQ];
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
        if wasd_keys_pressed.contains(&KeyW) { mvmt.z -= 1.; }
        if wasd_keys_pressed.contains(&KeyA) { mvmt.x -= 1.; }
        if wasd_keys_pressed.contains(&KeyS) { mvmt.z += 1.; }
        if wasd_keys_pressed.contains(&KeyD) { mvmt.x += 1.; }
        if wasd_keys_pressed.contains(&KeyE) { mvmt.y += 1.; }
        if wasd_keys_pressed.contains(&KeyQ) { mvmt.y -= 1.; }
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

        input.send(CameraAction(lifecycle, CameraMotion::Translate(mvmt)));
    }
    // Stop Translation
    else if keyboard.any_just_released(wasd_keys) {
        input.send(CameraAction(
            Lifecycle::Stop,
            CameraMotion::Translate(Vec3::ZERO),
        ));
    }

    // Wheel zoom
    for &MouseWheel { y, .. } in mouse_wheel.read() {
        // info!("wheel: ({x}, {y}) [{unit:?}]");
        // let rem_x = (100. / x).abs() % 1.;
        // let rem_y = (100. / y).abs() % 1.;

        // if (rem_x.is_nan() || rem_x < 1e-5_f32) && (rem_y.is_nan() || rem_y < 1e-5_f32) {
        input.send(CameraAction(Lifecycle::Ongoing, CameraMotion::Zoom(y)));
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
fn send_undo_event(mut events: EventWriter<UndoRedo>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.just_pressed(KeyCode::KeyZ)
        && !keyboard.pressed(KeyCode::ShiftLeft)
    {
        events.send(UndoRedo::Undo);
        info!("Undo event sent");
    }

    if keyboard.pressed(KeyCode::ControlLeft)
        && keyboard.just_pressed(KeyCode::KeyZ)
        && keyboard.pressed(KeyCode::ShiftLeft)
    {
        events.send(UndoRedo::Redo);
        info!("Redo event sent");
    }
}

/// Logs all `InputAction`s emitted, along with the input state at the time they
/// were fired, with `debug` verbosity.
fn log_input_actions(
    ri_keyboard: Res<ButtonInput<KeyCode>>,
    ri_mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut er_camera_actions: EventReader<CameraAction>,
    mut er_selection_actions: EventReader<SelectionAction>,
    mut er_editor_actions: EventReader<EditorAction>,
) {
    #[derive(Clone, Copy, Debug)]
    #[allow(dead_code)] // Spurious warning; these fields _are_ being read by the `Debug` impl
    enum InputAction {
        Camera(CameraAction),
        Selection(SelectionAction),
        Editor(EditorAction),
    }

    let input_actions = er_camera_actions
        .read()
        .map(|&action| InputAction::Camera(action))
        .chain(
            er_selection_actions
                .read()
                .map(|&action| InputAction::Selection(action)),
        )
        .chain(
            er_editor_actions
                .read()
                .map(|&action| InputAction::Editor(action)),
        )
        .collect::<Vec<_>>();

    if input_actions.is_empty() {
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
    let action_keys = [
        KeyW, KeyA, KeyS, KeyD, KeyE, KeyQ, Escape, Backquote, Delete, KeyF,
    ]
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

    for action in input_actions {
        debug!(
            "Mouse: [{lmb}{mmb}{rmb}]  \
            Keyboard: [{keys_display}]  \
            Action: [{action:.2?}]"
        );
    }
}
