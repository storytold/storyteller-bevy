use bevy::prelude::*;

use crate::math::{NearlyEq, NearlyZero};

use super::{CurveSin, Easing, Lerp};

pub struct InterpolationPlugin;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct InterpolationSystem;

impl Plugin for InterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (interp_velocity, interp_position).in_set(InterpolationSystem),
        )
        .add_systems(PostUpdate, cleanup_finished_interps);
    }
}

/// A `newtype` component for a `t` interpolation parameter.
#[derive(Component, Deref, DerefMut, Debug, Clone, Copy)]
pub struct Interp(pub f32);

pub struct Interpolation<T>
where
    f32: Lerp<T>,
{
    pub initial: T,
    pub target: T,
    pub t: f32,
    pub easing: Easing,
}

impl<T> Interpolation<T>
where
    f32: Lerp<T>,
    T: Copy,
{
    fn tick(&mut self, dt: f32) -> &mut Self {
        self.t = (self.t + dt).clamp(0., 1.);
        self
    }

    pub fn current(&self) -> T {
        if self.t.nearly_zero() {
            self.initial
        } else if self.t.nearly_eq(1.) {
            self.target
        } else {
            let t = match self.easing {
                Easing::Linear => self.t,
                Easing::Sine => self.t.curve_sin(),
                Easing::SineIn => self.t.curve_sin_in(),
                Easing::SineOut => self.t.curve_sin_out(),
            };

            t.lerp(self.initial, self.target)
        }
    }
}

impl<T> Default for Interpolation<T>
where
    f32: Lerp<T>,
    T: Default,
{
    fn default() -> Self {
        Self {
            initial: default(),
            target: default(),
            t: 0.,
            easing: Easing::Linear,
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct InterpVelocity {
    #[deref]
    pub inner: Interpolation<Vec3>,
    pub accel_zero2max: f32,
}

// TODO: Does this belong here?
#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec3);

#[derive(Component, Deref, DerefMut)]
pub struct InterpPosition {
    #[deref]
    pub inner: Interpolation<Vec3>,
    pub duration: f32,
}

fn interp_velocity(time: Res<Time>, mut q_interp: Query<(&mut Velocity, &mut InterpVelocity)>) {
    let delta_seconds = time.delta_seconds();

    for (mut velocity, mut interp) in q_interp.iter_mut() {
        let dt = delta_seconds / interp.accel_zero2max;
        **velocity = interp.tick(dt).current();
    }
}

fn interp_position(time: Res<Time>, mut q_interp: Query<&mut InterpPosition>) {
    let delta_seconds = time.delta_seconds();

    for mut interp in q_interp.iter_mut() {
        let dt = delta_seconds / interp.duration;
        interp.tick(dt);
    }
}

fn cleanup_finished_interps(
    mut cmd: Commands,
    q_interp_velocity: Query<(Entity, &InterpVelocity)>,
    q_interp_position: Query<(Entity, &InterpPosition)>,
) {
    for (ent, interp) in q_interp_velocity.iter() {
        if interp.t.nearly_eq(1.) {
            cmd.entity(ent).remove::<InterpVelocity>();
        }
    }

    for (ent, interp) in q_interp_position.iter() {
        if interp.t.nearly_eq(1.) {
            cmd.entity(ent).remove::<InterpPosition>();
        }
    }
}
