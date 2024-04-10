use std::{str::FromStr, time::Duration};

use bevy::{prelude::*, utils::hashbrown::HashMap};

use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotationLens, TransformScaleLens},
    Animator, BoxedTweenable, Delay, EaseMethod, Sequence, Targetable, Tracks, Tween,
    TweenCompleted,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use space_editor::{prelude::PrefabMarker, space_prefab::editor_registry::EditorRegistryExt};

use crate::{
    interaction::{MarkedForDeletion, Selectable},
    scene::SceneElement,
};

use super::Skeleton;

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TimedAnimationClipAsset>()
            .editor_registry::<TimedAnimationClipAsset>()
            .register_type::<AnimationTarget>()
            .editor_registry::<AnimationTarget>()
            .register_type::<TransformKeyframe>()
            .editor_registry::<TransformKeyframe>()
            .register_type::<Keyframe>()
            .editor_registry::<Keyframe>()
            .register_type::<KeyframeOrder>()
            .editor_registry::<KeyframeOrder>()
            .register_type::<ClipRetargeting>()
            .editor_registry::<ClipRetargeting>()
            .register_type::<EaseType>()
            .register_type::<uuid::Uuid>()
            .add_event::<TimelineInteractionEvent>()
            .add_event::<KeyframeInteractionEvent>()
            .init_state::<KeyingMode>();

        app.insert_resource(GlobalSeekTime {
            time: 0.,
            playing: false,
            delta: None,
        })
        .add_event::<TweenCompleted>();

        app.add_systems(
            First,
            (generate_transform_animation, generate_clip_animation),
        );
        app.add_systems(PreUpdate, (process_timeline_events, update_tick).chain());

        app.add_systems(
            Update,
            (
                process_auto_key.run_if(in_state(KeyingMode::Auto)),
                process_keyframe_events,
                timeline_bound_component_animator_system::<Transform>,
                timeline_bound_component_animator_system::<AnimationPlayer>,
                update_timed_animation_clip_duration,
            ),
        );

        app.add_systems(PostUpdate, retarget_timed_animation_clip);

        #[cfg(feature = "wasm")]
        {
            app.add_systems(
                Last,
                (
                    wasm::update_global_seek_time,
                    wasm::update_keyframe_interaction,
                ),
            );
            app.add_systems(
                PostUpdate,
                (
                    wasm::notify_seek_time_changes.map(bevy::utils::error),
                    wasm::notify_keyframe_changes.map(bevy::utils::error),
                    wasm::notify_updated_animation_targets.map(bevy::utils::error),
                    wasm::notify_keying_mode_changed.map(bevy::utils::error),
                ),
            );
        }
    }
}

const MIN_KEYFRAME_DISTANCE: f32 = 0.01;

#[allow(dead_code)]
#[derive(Event, Clone, Copy, Debug)]
pub enum TimelineInteractionEvent {
    Play,
    Pause,
    Seek(f32),
    ToggleAutoKey,
}

#[allow(dead_code)]
#[derive(Event, Clone, Debug)]
pub enum KeyframeInteractionEvent {
    RecordKeyframe {
        target: Entity,
    },
    MoveKeyframeTime {
        target: Entity,
        time: f32,
    },
    AddEntityTrack {
        target: Entity,
    },
    InsertAnimationClip {
        target: Entity,
        clip_path: String,
        display_name: String,
        retargeting_type: ClipRetargeting,
    },
}

#[derive(Resource, Clone, Copy, Debug, Default, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct GlobalSeekTime {
    pub time: f32,
    pub playing: bool,
    pub delta: Option<f32>,
}

#[derive(States, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum KeyingMode {
    #[default]
    Manual,
    Auto,
}

#[derive(Clone, Debug, Copy, Default, Reflect, PartialEq, Serialize, Deserialize)]
pub enum EaseType {
    #[default]
    Linear,
    CubicHermite,
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component)]
pub struct TransformKeyframe {
    incoming_easing: EaseType,
    affect_translation: bool,
    affect_rotation: bool,
    affect_scale: bool,
}

impl Default for TransformKeyframe {
    fn default() -> Self {
        Self {
            incoming_easing: Default::default(),
            affect_translation: true,
            affect_rotation: true,
            affect_scale: true,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component)]
pub struct Keyframe {
    target: AnimationTarget,
    pub time: f32,
}

#[derive(Component, Clone, Debug, Default, Reflect, PartialEq, Eq, PartialOrd, Ord)]
#[reflect(Component)]
pub struct KeyframeOrder(u32);

impl PartialOrd for Keyframe {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Default for Keyframe {
    fn default() -> Self {
        Self {
            time: Default::default(),
            target: AnimationTarget::Unknown,
        }
    }
}

#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component)]
pub struct TimedAnimationClipAsset {
    clip_path: String,
    #[reflect(ignore)]
    clip: Option<Handle<AnimationClip>>,
    speed: f32,
    cut_in: Option<f32>,
    cut_out: Option<f32>,
}

#[derive(Component, Clone, Debug, PartialEq)]
pub struct TimedAnimationClipDuration {
    duration: f32,
    cut_in: f32,
    full_clip_duration: f32,
}

#[derive(Component, Clone, Debug)]
enum RetargetedClipReference {
    UseOrigin,
    RenamedRoot {
        retargeted: Handle<AnimationClip>,
    },
    #[allow(dead_code)]
    Mixamo {
        retargeted: Handle<AnimationClip>,
    },
    #[allow(dead_code)]
    MocapNet {
        retargeted: Handle<AnimationClip>,
    },
}

#[derive(Component, Clone, Debug, Reflect, PartialEq, Default, Serialize, Deserialize)]
#[reflect(Component)]
#[cfg_attr(all(feature = "wasm"), wasm_bindgen::prelude::wasm_bindgen)]
pub enum ClipRetargeting {
    #[default]
    UseOrigin,
    Mixamo,
    MocapNet,
}

impl PartialOrd for TimedAnimationClipAsset {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.speed.partial_cmp(&other.speed) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        None
    }
}

impl Default for TimedAnimationClipAsset {
    fn default() -> Self {
        Self {
            clip_path: Default::default(),
            clip: Default::default(),
            speed: 1.0,
            cut_in: None,
            cut_out: None,
        }
    }
}

#[derive(
    Component, Clone, Debug, Default, Reflect, PartialEq, Eq, Hash, Copy, Serialize, Deserialize,
)]
#[reflect(Component)]
#[serde(try_from = "&str", into = "String")]
pub enum AnimationTarget {
    #[default]
    Unknown,
    MainCamera,
    Uuid(uuid::Uuid),
}

impl AnimationTarget {
    pub fn new() -> Self {
        Self::Uuid(uuid::Uuid::new_v4())
    }
}

impl ToString for AnimationTarget {
    fn to_string(&self) -> String {
        match self {
            Self::Unknown => "unknown".to_string(),
            Self::MainCamera => "MainCamera".to_string(),
            Self::Uuid(uuid) => uuid.to_string(),
        }
    }
}

impl From<AnimationTarget> for String {
    fn from(val: AnimationTarget) -> Self {
        val.to_string()
    }
}

impl TryFrom<&str> for AnimationTarget {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl FromStr for AnimationTarget {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "unknown" {
            Ok(Self::Unknown)
        } else if s == "MainCamera" {
            Ok(Self::MainCamera)
        } else {
            Ok(Self::Uuid(s.parse()?))
        }
    }
}

fn process_timeline_events(
    mut e_timeline_event: EventReader<TimelineInteractionEvent>,
    mut r_global_seek_time: ResMut<GlobalSeekTime>,
    key_mode: Res<State<KeyingMode>>,
    mut next_keying_mode: ResMut<NextState<KeyingMode>>,
) {
    for event in e_timeline_event.read() {
        match event {
            TimelineInteractionEvent::Play => r_global_seek_time.playing = true,
            TimelineInteractionEvent::Pause => {
                r_global_seek_time.playing = false;
            }
            TimelineInteractionEvent::Seek(time) => {
                r_global_seek_time.time = time.max(0.);
            }
            TimelineInteractionEvent::ToggleAutoKey => {
                next_keying_mode.set(match key_mode.get() {
                    KeyingMode::Manual => KeyingMode::Auto,
                    KeyingMode::Auto => KeyingMode::Manual,
                });
            }
        }

        r_global_seek_time.delta = None;
    }
}

fn update_tick(
    mut r_global_seek_time: ResMut<GlobalSeekTime>,
    time: Res<Time>,
    #[cfg(feature = "headless")] headless_settings: Option<Res<crate::headless::HeadlessSettings>>,
) {
    if r_global_seek_time.playing {
        #[cfg(feature = "headless")]
        let delta = match headless_settings {
            Some(headless_settings) => headless_settings.frame_delta,
            None => time.delta_seconds(),
        };

        #[cfg(not(feature = "headless"))]
        let delta = time.delta_seconds();

        r_global_seek_time.time += delta;
        r_global_seek_time.delta = Some(delta);
    }
}

fn process_keyframe_events(
    mut commands: Commands,
    mut e_timeline_event: EventReader<KeyframeInteractionEvent>,
    r_global_seek_time: Res<GlobalSeekTime>,
    q_animation_targets: Query<(Entity, &Transform, &AnimationTarget)>,
    q_potential_targets: Query<Entity, (With<Transform>, Without<AnimationTarget>)>,
    mut q_keyframes: Query<(Entity, &mut Keyframe)>,
    asset_server: Res<AssetServer>,
) {
    for event in e_timeline_event.read() {
        info!("Processing Keyframe Event {event:?}");
        match event {
            KeyframeInteractionEvent::RecordKeyframe { target } => {
                if let Ok((_target, transform, animation_target)) = q_animation_targets.get(*target)
                {
                    let mut existing = None;
                    for (entity, keyframe) in q_keyframes.iter_mut() {
                        if keyframe.target == *animation_target
                            && (keyframe.time - r_global_seek_time.time).abs()
                                < MIN_KEYFRAME_DISTANCE
                        {
                            existing = Some(entity);
                            break;
                        }
                    }
                    if let Some(existing) = existing {
                        commands.entity(existing).insert((
                            *transform,
                            TransformKeyframe::default(),
                            Keyframe {
                                time: r_global_seek_time.time,
                                target: *animation_target,
                            },
                        ));
                    } else {
                        commands.spawn((
                            PrefabMarker,
                            SpatialBundle {
                                transform: *transform,
                                ..Default::default()
                            },
                            Selectable,
                            Keyframe {
                                time: r_global_seek_time.time,
                                target: *animation_target,
                            },
                            TransformKeyframe::default(),
                        ));
                    }
                }
            }
            KeyframeInteractionEvent::MoveKeyframeTime { target, time } => {
                if let Ok((_, mut keyframe)) = q_keyframes.get_mut(*target) {
                    keyframe.time = *time;
                }
            }
            KeyframeInteractionEvent::AddEntityTrack { target } => {
                if let Ok(target) = q_potential_targets.get(*target) {
                    info!("Adding Animation Target to Entity");
                    commands.entity(target).insert(AnimationTarget::new());
                } else {
                    warn!("Can't add animation target to entity");
                }
            }
            KeyframeInteractionEvent::InsertAnimationClip {
                target,
                clip_path,
                display_name,
                retargeting_type,
            } => {
                if let Ok((_target, transform, animation_target)) = q_animation_targets.get(*target)
                {
                    let clip_path = format!("{clip_path}#Animation0");
                    commands.spawn((
                        Name::new(display_name.to_owned()),
                        PrefabMarker,
                        SpatialBundle {
                            transform: *transform,
                            ..Default::default()
                        },
                        Selectable,
                        Keyframe {
                            time: r_global_seek_time.time,
                            target: *animation_target,
                        },
                        TimedAnimationClipAsset {
                            clip: Some(asset_server.load(&clip_path)),
                            clip_path,
                            speed: 1.0,
                            cut_in: None,
                            cut_out: None,
                        },
                        retargeting_type.clone(),
                    ));
                }
            }
        }
    }
}

fn process_auto_key(
    changed: Query<Entity, (With<AnimationTarget>, Changed<Transform>)>,
    r_global_seek_time: Res<GlobalSeekTime>,
    mut event: EventWriter<KeyframeInteractionEvent>,
) {
    if r_global_seek_time.playing || r_global_seek_time.is_changed() {
        info!("Skipping Auto Key");
        return;
    }

    for entity in &changed {
        info!("Setting Auto Key");
        event.send(KeyframeInteractionEvent::RecordKeyframe { target: entity });
    }
}

fn generate_transform_animation(
    mut commands: Commands,
    keyframes: Query<
        (&Keyframe, &Transform, &TransformKeyframe, Entity),
        Without<MarkedForDeletion>,
    >,
    valid_targets: Query<(
        Entity,
        &AnimationTarget,
        Option<&Name>,
        Option<&SceneElement>,
    )>,
    mut e_timeline_event: EventReader<TimelineInteractionEvent>,
) {
    let mut playing = false;
    for e in e_timeline_event.read() {
        if matches!(
            e,
            TimelineInteractionEvent::Play | TimelineInteractionEvent::Seek(_)
        ) {
            playing = true;
            continue;
        }
    }
    if !playing {
        return;
    }
    info!("Generating Animation From Keyframes");

    let available_targets = valid_targets
        .iter()
        .map(|(e, a, name, element)| (*a, (e, name, element)))
        .collect::<HashMap<_, _>>();

    let keyframes = keyframes
        .iter()
        .into_group_map_by(|(keyframe, _, _, _)| keyframe.target);

    for (target, keyframes) in keyframes.into_iter() {
        info!("Keyframes For Target {target:?}");
        let Some(target) = available_targets.get(&target) else {
            info!("No valid target.");
            continue;
        };
        let (target, target_name, target_scene_element) = *target;

        let keyframes = keyframes.iter();

        let keyframes = keyframes.sorted_by(|a, b| a.0.time.total_cmp(&b.0.time));

        let (_, _, _, translation, rotation, scale) = keyframes.fold(
            (
                Option::<(f32, Vec3)>::None,
                Option::<(f32, Quat)>::None,
                Option::<(f32, Vec3)>::None,
                Vec::<Tween<Transform>>::new(),
                Vec::<Tween<Transform>>::new(),
                Vec::<Tween<Transform>>::new(),
            ),
            |(
                last_translation,
                last_rotation,
                last_scale,
                mut translation_list,
                mut rotation_list,
                mut scale_list,
            ),
             (keyframe, transform, transform_keyframe, _)| {
                let time = keyframe.time;
                let translation = transform.translation;
                let rotation = transform.rotation;
                let scale = transform.scale;

                if transform_keyframe.affect_translation {
                    let (last_time, start) = match last_translation {
                        Some(v) => v,
                        None => (0., translation),
                    };

                    if (time - last_time).abs() > 0.00001 {
                        translation_list.push(Tween::new(
                            EaseMethod::Linear,
                            Duration::from_secs_f32((time - last_time).max(0.00001)),
                            TransformPositionLens {
                                start,
                                end: translation,
                            },
                        ));
                    }
                }

                if transform_keyframe.affect_rotation {
                    let (last_time, start) = match last_rotation {
                        Some(v) => v,
                        None => (0., rotation),
                    };

                    if (time - last_time).abs() > 0.00001 {
                        rotation_list.push(Tween::new(
                            EaseMethod::Linear,
                            Duration::from_secs_f32((time - last_time).max(0.00001)),
                            TransformRotationLens {
                                start,
                                end: rotation,
                            },
                        ));
                    }
                }
                if transform_keyframe.affect_scale {
                    let (last_time, start) = match last_scale {
                        Some(v) => v,
                        None => (0., scale),
                    };

                    if (time - last_time).abs() > 0.00001 {
                        scale_list.push(Tween::new(
                            EaseMethod::Linear,
                            Duration::from_secs_f32((time - last_time).max(0.00001)),
                            TransformScaleLens { start, end: scale },
                        ));
                    }
                }

                let next_translation = if transform_keyframe.affect_translation {
                    Some((time, translation))
                } else {
                    last_translation
                };
                let next_rotation = if transform_keyframe.affect_rotation {
                    Some((time, rotation))
                } else {
                    last_rotation
                };
                let next_scale = if transform_keyframe.affect_scale {
                    Some((time, scale))
                } else {
                    last_scale
                };

                (
                    next_translation,
                    next_rotation,
                    next_scale,
                    translation_list,
                    rotation_list,
                    scale_list,
                )
            },
        );

        let mut tracks = Vec::with_capacity(3);

        if !translation.is_empty() {
            tracks.push(Sequence::new(translation));
        } else {
            info!("No translation");
        }
        if !rotation.is_empty() {
            tracks.push(Sequence::new(rotation));
        } else {
            info!("No rotation");
        }
        if !scale.is_empty() {
            tracks.push(Sequence::new(scale));
        } else {
            info!("No scale");
        }

        if tracks.is_empty() {
            info!("Empty Animation Track for {target_name:?} {target_scene_element:?} {target:?}");
            commands.entity(target).remove::<Animator<Transform>>();
        } else {
            info!("Adding Animation Track for {target_name:?} {target_scene_element:?} {target:?}");
            let tracks = Tracks::new(tracks);
            commands.entity(target).insert(Animator::new(tracks));
        }
    }
}

struct ComponentTarget<'a, T: Component>(&'a mut T);

impl<'a, T: Component> ComponentTarget<'a, T> {
    fn new(target: Mut<'a, T>) -> Self {
        Self(target.into_inner())
    }
}

impl<'a, T: Component> Targetable<T> for ComponentTarget<'a, T> {
    fn target_mut(&mut self) -> &mut T {
        self.0
    }
}

pub fn timeline_bound_component_animator_system<T: Component>(
    time: Res<GlobalSeekTime>,
    mut query: Query<(Entity, &mut T, &mut Animator<T>)>,
    events: ResMut<Events<TweenCompleted>>,
) {
    if time.is_changed() && time.time >= 0.0 {
        let mut events: Mut<Events<TweenCompleted>> = events.into();
        for (entity, target, mut animator) in query.iter_mut() {
            if animator.tweenable().duration() <= Duration::from_secs_f32(0.0001) {
                continue;
            }
            let mut target = ComponentTarget::new(target);
            if let Some(delta) = time.delta {
                animator.tweenable_mut().tick(
                    Duration::from_secs_f32(delta),
                    &mut target,
                    entity,
                    &mut events,
                );
            } else {
                let tween = animator.tweenable_mut();
                tween.set_elapsed(Duration::from_secs_f32(time.time.max(0.)));
                tween.tick(Duration::ZERO, &mut target, entity, &mut events);
            }
        }
    }
}

#[derive(Component)]
struct TimedAnimationClipLens {
    clip: Handle<AnimationClip>,
    duration: f32,
    inner_start: f32,
}

impl bevy_tweening::lens::Lens<AnimationPlayer> for TimedAnimationClipLens {
    fn lerp(&mut self, target: &mut AnimationPlayer, ratio: f32) {
        let inner_time = self.duration * ratio + self.inner_start;
        if target.animation_clip() != &self.clip {
            target.play(self.clip.clone());
            target.pause();
            target.seek_to(inner_time);
        } else {
            target.seek_to(inner_time);
        }
    }
}

fn generate_clip_animation(
    mut commands: Commands,
    keyframes: Query<
        (
            &Keyframe,
            &TimedAnimationClipAsset,
            &TimedAnimationClipDuration,
            &RetargetedClipReference,
            Entity,
        ),
        Without<MarkedForDeletion>,
    >,
    valid_targets: Query<(
        Entity,
        &AnimationTarget,
        Option<&Name>,
        Option<&SceneElement>,
        &Skeleton,
    )>,
    mut e_timeline_event: EventReader<TimelineInteractionEvent>,
) {
    let mut playing = false;
    for e in e_timeline_event.read() {
        if matches!(
            e,
            TimelineInteractionEvent::Play | TimelineInteractionEvent::Seek(_)
        ) {
            playing = true;
            continue;
        }
    }
    if !playing {
        return;
    }
    info!("Generating Animation From Keyframes");

    let available_targets = valid_targets
        .iter()
        .map(|(e, a, name, element, _skeleton)| (*a, (e, name, element)))
        .collect::<HashMap<_, _>>();

    let keyframes = keyframes
        .iter()
        .into_group_map_by(|(keyframe, _, _, _, _)| keyframe.target);

    for (target, keyframes) in keyframes.into_iter() {
        info!("Keyframes For Target {target:?}");
        let Some(target) = available_targets.get(&target) else {
            info!("No valid target.");
            continue;
        };
        let (target, target_name, target_scene_element) = *target;

        let keyframes = keyframes.iter();

        let keyframes = keyframes.sorted_by(|a, b| a.0.time.total_cmp(&b.0.time));

        let (last, mut clip_timeline) = keyframes.fold(
            (
                Option::<(f32, f32, f32, Handle<AnimationClip>)>::None,
                Vec::<BoxedTweenable<AnimationPlayer>>::new(),
            ),
            |(last_clip, mut clip_timeline), (keyframe, asset, duration, reference, _entity)| {
                let time = keyframe.time;

                let (last_start, last_duration, inner_start, last_clip) = match last_clip {
                    Some((start, duration, inner_start, clip)) => {
                        (start, duration, inner_start, Some(clip))
                    }
                    None => (0., 0., 0., None),
                };

                let mut reached_now = false;
                let mut clip_end = last_start + last_duration;
                if clip_end > time {
                    reached_now = true;
                    clip_end = time;
                }

                if let Some(last_clip) = last_clip {
                    let duration = clip_end - last_start;

                    if duration > 0f32 {
                        clip_timeline.push(
                            Tween::new(
                                EaseMethod::Linear,
                                Duration::from_secs_f32(duration),
                                TimedAnimationClipLens {
                                    clip: last_clip,
                                    duration,
                                    inner_start,
                                },
                            )
                            .into(),
                        );
                    }
                }

                if !reached_now {
                    let duration = time - clip_end;
                    if duration > 0f32 {
                        clip_timeline.push(Delay::new(Duration::from_secs_f32(duration)).into());
                    }
                }

                let inner_start = duration.cut_in;
                let duration = duration.duration;
                let clip = match reference {
                    RetargetedClipReference::UseOrigin => asset.clip.clone(),
                    RetargetedClipReference::RenamedRoot { retargeted } => Some(retargeted.clone()),
                    RetargetedClipReference::Mixamo { retargeted } => Some(retargeted.clone()),
                    RetargetedClipReference::MocapNet { retargeted } => Some(retargeted.clone()),
                };

                (
                    clip.map(|clip| (time, duration, inner_start, clip)),
                    clip_timeline,
                )
            },
        );

        if let Some((_, duration, inner_start, clip)) = last {
            if duration > 0f32 {
                clip_timeline.push(
                    Tween::new(
                        EaseMethod::Linear,
                        Duration::from_secs_f32(duration),
                        TimedAnimationClipLens {
                            clip,
                            duration,
                            inner_start,
                        },
                    )
                    .into(),
                );
            }
        }

        if clip_timeline.is_empty() {
            commands
                .entity(target)
                .remove::<Animator<AnimationPlayer>>()
                .remove::<AnimationPlayer>();
            continue;
        }

        let clip_timeline = Sequence::new(clip_timeline);
        info!(
            "Adding Animation Clip Track for {target_name:?} {target_scene_element:?} {target:?}"
        );
        commands
            .entity(target)
            .insert((AnimationPlayer::default(), Animator::new(clip_timeline)));
    }
}

pub fn update_timed_animation_clip_duration(
    mut commands: Commands,
    query: Query<(Entity, &TimedAnimationClipAsset), Without<TimedAnimationClipDuration>>,
    clips: Res<Assets<AnimationClip>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, clip_asset) in &query {
        if let Some(clip) = &clip_asset.clip {
            if let Some(clip) = clips.get(clip) {
                let full_clip_duration = clip.duration();
                let cut_in = clip_asset.cut_in.unwrap_or_default();
                let duration = if let Some(cut_out) = clip_asset.cut_out {
                    full_clip_duration.min(cut_out)
                } else {
                    full_clip_duration
                };
                commands.entity(entity).insert(TimedAnimationClipDuration {
                    duration,
                    cut_in,
                    full_clip_duration,
                });
            }
        } else {
            let mut clip_asset = clip_asset.clone();
            clip_asset.clip = Some(asset_server.load(&clip_asset.clip_path));
            commands.entity(entity).insert(clip_asset);
        }
    }
}

fn retarget_timed_animation_clip(
    mut commands: Commands,
    clip_keys: Query<
        (
            Entity,
            &TimedAnimationClipAsset,
            &ClipRetargeting,
            &Keyframe,
        ),
        (
            With<TimedAnimationClipDuration>,
            Without<RetargetedClipReference>,
        ),
    >,
    animation_targets: Query<(Entity, &AnimationTarget, &Name), With<Skeleton>>,
    children: Query<(&Name, Option<&Children>)>,
    mut clips: ResMut<Assets<AnimationClip>>,
) {
    let targets = animation_targets
        .iter()
        .map(|(entity, target, name)| (*target, (entity, name)))
        .collect::<HashMap<_, _>>();

    for (entity, asset, retarget, keyframe) in &clip_keys {
        info!("Added Animation: {asset:?} with retargeting {retarget:?}");
        let Some(clip) = &asset.clip else { continue };
        let Some(clip) = clips.get(clip) else {
            continue;
        };
        info!("Animation exists");
        let Some(paths) = clip.field("paths") else {
            continue;
        };
        info!("Parsing Paths");
        let Some(paths): Option<HashMap<EntityPath, usize>> =
            HashMap::<EntityPath, usize>::from_reflect(paths)
        else {
            continue;
        };

        let Some((target_entity, target_name)) = targets.get(&keyframe.target) else {
            error!("Animation has no target: {keyframe:?}");
            continue;
        };

        let Some((_, target_children)) = children.get(*target_entity).ok() else {
            continue;
        };

        match &retarget {
            ClipRetargeting::UseOrigin => {
                info!("Use Origin");
                if clip.compatible_with(target_name) {
                    info!("Animation Already Compatible");
                    commands
                        .entity(entity)
                        .insert(RetargetedClipReference::UseOrigin);
                } else {
                    let mut new_clip = AnimationClip::default();
                    let mut empty = true;
                    for (path, bone_id) in paths.iter() {
                        let Some(new_path) =
                            rebuild_entity_path(target_name, target_children, path, &children)
                        else {
                            continue;
                        };

                        let Some(curves) = clip.get_curves(*bone_id) else {
                            continue;
                        };
                        for curve in curves {
                            new_clip.add_curve_to_path(new_path.clone(), curve.clone());
                            empty = false;
                        }
                    }
                    if empty {
                        error!("Failed to add any curves to animation");
                        commands.entity(entity).remove::<ClipRetargeting>();
                        continue;
                    }
                    info!("Inserted Animation With Renamed Root");
                    let retargeted = clips.add(new_clip);
                    commands
                        .entity(entity)
                        .insert(RetargetedClipReference::RenamedRoot { retargeted });
                }
            }
            ClipRetargeting::Mixamo => {
                commands
                    .entity(entity)
                    .insert(RetargetedClipReference::UseOrigin);
            }
            ClipRetargeting::MocapNet => {
                commands
                    .entity(entity)
                    .insert(RetargetedClipReference::UseOrigin);
            }
        }
    }
}

fn rebuild_entity_path(
    new_root_name: &Name,
    root_children: Option<&Children>,
    original: &EntityPath,
    named: &Query<(&Name, Option<&Children>)>,
) -> Option<EntityPath> {
    if original.parts.is_empty() {
        error!("Empty Entity Path");
        return None;
    }

    let mut current_children = root_children;

    let mut new_parts = Vec::with_capacity(original.parts.len());
    let mut parts = original.parts.iter();

    new_parts.push(new_root_name.clone());

    let _ = parts.next(); // Skipping the first one

    for part in parts {
        let children = current_children?;
        let mut found = false;
        for child in children.iter() {
            if let Ok((name, children)) = named.get(*child) {
                info!("Checking {name:?}");
                if name.as_str() == part.as_str() {
                    found = true;
                    current_children = children;
                    new_parts.push(name.clone());
                    break;
                }
            }
        }
        if !found {
            warn!(
                r#"Couldn't find Path: {original:?}
            Failed at: {part:?}
            Children are: {children:?}"#
            );
            return None;
        }
    }
    let new_path = EntityPath { parts: new_parts };
    info!("Retargeting: Replaced {original:?} with {new_path:?}");
    Some(new_path)
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::{prelude::*, utils::hashbrown::HashMap, window::PrimaryWindow};
    use serde::{Deserialize, Serialize};
    use wasm_bindgen::prelude::*;

    use crate::{
        interaction::MarkedForDeletion,
        scene::SceneElement,
        wasm::{FromStr, StaticBuffer, WasmMessageBuffer},
    };

    use super::{
        AnimationTarget, ClipRetargeting, EaseType, GlobalSeekTime, Keyframe,
        KeyframeInteractionEvent, KeyingMode, TimedAnimationClipAsset, TimedAnimationClipDuration,
        TimelineInteractionEvent, TransformKeyframe,
    };

    static SEEK_TIME: WasmMessageBuffer<TimelineInteractionEvent> = WasmMessageBuffer::new();
    static KEYFRAME_EVENT: WasmMessageBuffer<Vec<KeyframeInteractionEvent>> =
        WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = seekTime)]
    pub fn seek_time(timestamp: f32) {
        SEEK_TIME.write_message(TimelineInteractionEvent::Seek(timestamp));
    }

    #[wasm_bindgen(js_name = play)]
    pub fn play() {
        SEEK_TIME.write_message(TimelineInteractionEvent::Play);
    }

    #[wasm_bindgen(js_name = pause)]
    pub fn pause() {
        SEEK_TIME.write_message(TimelineInteractionEvent::Pause);
    }

    #[wasm_bindgen(js_name = toggleAutoKey)]
    pub fn toggle_auto_key() {
        SEEK_TIME.write_message(TimelineInteractionEvent::ToggleAutoKey);
    }

    #[wasm_bindgen(js_name = recordKeyframe)]
    pub fn record_keyframe(entities: Vec<String>) {
        KEYFRAME_EVENT.write_message(
            entities
                .iter()
                .filter_map(|entity| Entity::from_str(entity).ok())
                .map(|target| KeyframeInteractionEvent::RecordKeyframe { target })
                .collect(),
        );
    }

    #[wasm_bindgen(js_name = insertClip)]
    pub fn insert_clip(
        entity: String,
        clip_path: String,
        display_name: String,
        retargeting_type: ClipRetargeting,
    ) {
        let Ok(target) = Entity::from_str(&entity) else {
            return;
        };
        KEYFRAME_EVENT.write_message(vec![KeyframeInteractionEvent::InsertAnimationClip {
            target,
            clip_path,
            display_name,
            retargeting_type,
        }]);
    }

    #[wasm_bindgen(js_name = moveKeyframeTime)]
    pub fn move_keyframe_time(entity: String, time: f32) {
        let Ok(target) = Entity::from_str(&entity) else {
            return;
        };
        KEYFRAME_EVENT.write_message(vec![KeyframeInteractionEvent::MoveKeyframeTime {
            target,
            time,
        }]);
    }

    #[wasm_bindgen(js_name = addEntityTrack)]
    pub fn add_entity_track(entity: String) {
        let Ok(target) = Entity::from_str(&entity) else {
            return;
        };
        KEYFRAME_EVENT.write_message(vec![KeyframeInteractionEvent::AddEntityTrack { target }]);
    }

    pub(super) fn update_global_seek_time(
        mut e_timeline_event: EventWriter<TimelineInteractionEvent>,
    ) {
        if let Some(timestamp) = SEEK_TIME.take_message() {
            info!("Updating Seek Time {timestamp:?}");
            e_timeline_event.send(timestamp);
        }
    }

    pub(super) fn update_keyframe_interaction(
        mut e_keyframe_event: EventWriter<KeyframeInteractionEvent>,
    ) {
        if let Some(keyframes) = KEYFRAME_EVENT.take_message() {
            for keyframe in keyframes.into_iter() {
                e_keyframe_event.send(keyframe);
            }
        }
    }

    #[wasm_bindgen(typescript_custom_section)]
    const TIMELINE_TYPES: &str = r#"
    export interface SeekTimeChanged extends CustomEvent {
        type: "seek-time-changed";
        detail: {
            time: number,
            playing: boolean
        };
    }

    export interface AnimationTrack {
        name?: string;
        targetType: SceneElement;
        animationTarget: AnimationTarget;
    }

    export interface UpdateAnimationTargets extends CustomEvent {
        type: "update-animation-targets",
        detail: {
            added: Record<string, AnimationTrack>,
            removed: string[]
        }
    }

    export interface TransformKeyframeInfo {
        target: AnimationTarget,
        time: number,
        entity: string,
        incoming_easing: "linear" | "cubic-hermite",
        affect_translation: boolean,
        affect_rotation: boolean,
        affect_scale: boolean
    }

    export interface TimedAnimationClipInfo {
        target: AnimationTarget,
        time: number,
        entity: string,
        type: 'animation-clip',
        clip: string,
        duration: number,
        speed: number
    }

    export type Keyframe = TransformKeyframeInfo | TimedAnimationClipInfo;

    export type AnimationTarget = string;

    export interface UpdateKeyframes extends CustomEvent {
        type: "update-keyframes",
        detail: {
            added: Record<string, Keyframe>,
            removed: string[]
        }
    }

    export interface UpdateKeyingMode extends CustomEvent {
        type: "keying-mode-changed",
        detail: {
            automatic: boolean
        }
    }

    declare global {
        export interface GlobalEventHandlersEventMap {
            "seek-time-changed": CameraKeyframesEvent;
            "update-animation-targets": UpdateAnimationTargets;
            "update-keyframes": UpdateKeyframes;
        }
    }
    "#;

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AddedAnimationTargetRecordItem {
        name: Option<String>,
        target_type: SceneElement,
        animation_target: AnimationTarget,
    }

    #[derive(Serialize, Deserialize)]
    struct UpdatedAnimationTargetsEvent {
        added: HashMap<String, AddedAnimationTargetRecordItem>,
        removed: Vec<String>,
    }

    #[derive(Serialize, Deserialize)]
    struct UpdateKeyingMode {
        automatic: bool,
    }

    #[derive(Serialize, Deserialize)]
    struct TimedAnimationClipInfo {
        entity: String,
        target: AnimationTarget,
        time: f32,
        clip: String,
        speed: f32,
        duration: f32,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TransformKeframeInfo {
        entity: String,
        target: AnimationTarget,
        time: f32,
        incoming_easing: EaseType,
        affect_translation: bool,
        affect_rotation: bool,
        affect_scale: bool,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct UpdateKeyframes {
        added: HashMap<String, serde_json::Value>,
        removed: Vec<String>,
    }

    pub(super) fn notify_updated_animation_targets(
        q_window: Query<&Window, With<PrimaryWindow>>,
        q_cameras: Query<
            (Entity, Option<&Name>, &AnimationTarget),
            (Added<AnimationTarget>, With<Camera>),
        >,
        q_scene_elements: Query<
            (Entity, Option<&Name>, &SceneElement, &AnimationTarget),
            (Added<AnimationTarget>, Without<Camera>),
        >,
        q_transforms: Query<
            (Entity, Option<&Name>, &AnimationTarget),
            (
                Added<AnimationTarget>,
                With<Transform>,
                Without<Camera>,
                Without<SceneElement>,
            ),
        >,
        mut removed: bevy::prelude::RemovedComponents<AnimationTarget>,
    ) -> Result<(), String> {
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        let added = q_transforms
            .iter()
            .map(|(entity, name, target)| {
                (
                    format!("{entity:?}"),
                    AddedAnimationTargetRecordItem {
                        name: name.map(|v| v.to_string()),
                        target_type: SceneElement::Generic,
                        animation_target: *target,
                    },
                )
            })
            .chain(q_cameras.iter().map(|(entity, name, target)| {
                (
                    format!("{entity:?}"),
                    AddedAnimationTargetRecordItem {
                        name: name.map(|v| v.to_string()),
                        target_type: SceneElement::Camera,
                        animation_target: *target,
                    },
                )
            }))
            .chain(
                q_scene_elements
                    .iter()
                    .map(|(entity, name, element, target)| {
                        (
                            format!("{entity:?}"),
                            AddedAnimationTargetRecordItem {
                                name: name.map(|v| v.to_string()),
                                target_type: *element,
                                animation_target: *target,
                            },
                        )
                    }),
            )
            .collect::<HashMap<_, _>>();

        let removed = removed.read().map(|v| format!("{v:?}")).collect::<Vec<_>>();

        if added.is_empty() && removed.is_empty() {
            return Ok(());
        }

        let event = UpdatedAnimationTargetsEvent { added, removed };

        crate::wasm::dispatch_to_js(
            window,
            "update-animation-targets",
            &serde_json::to_value(&event).map_err(|e| format!("{e}"))?,
        )
    }

    pub(super) fn notify_keyframe_changes(
        q_window: Query<&Window, With<PrimaryWindow>>,
        q_transform_keyframes: Query<
            (Entity, &Keyframe, &TransformKeyframe),
            Or<(Changed<Keyframe>, Changed<TransformKeyframe>)>,
        >,
        q_clips: Query<
            (
                Entity,
                &Keyframe,
                &TimedAnimationClipAsset,
                &Name,
                Option<&TimedAnimationClipDuration>,
            ),
            Or<(
                Changed<Keyframe>,
                Changed<TimedAnimationClipAsset>,
                Changed<TimedAnimationClipDuration>,
            )>,
        >,
        removed: Query<Entity, With<MarkedForDeletion>>,
    ) -> Result<(), String> {
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        let added = q_transform_keyframes
            .iter()
            .map(|(entity, keyframe, transform_keyframe)| {
                match serde_json::to_value(TransformKeframeInfo {
                    entity: format!("{entity:?}"),
                    target: keyframe.target,
                    time: keyframe.time,
                    incoming_easing: transform_keyframe.incoming_easing,
                    affect_translation: transform_keyframe.affect_translation,
                    affect_rotation: transform_keyframe.affect_rotation,
                    affect_scale: transform_keyframe.affect_scale,
                })
                .map_err(|e| format!("{e}"))
                {
                    Ok(value) => Ok((format!("{entity:?}"), value)),
                    Err(e) => Err(e),
                }
            })
            .chain(
                q_clips
                    .iter()
                    .map(|(entity, keyframe, clip, name, duration)| {
                        match serde_json::to_value(TimedAnimationClipInfo {
                            entity: format!("{entity:?}"),
                            target: keyframe.target,
                            time: keyframe.time,
                            clip: name.to_string(),
                            speed: clip.speed,
                            duration: duration.map(|v| v.duration).unwrap_or_default(),
                        })
                        .map_err(|e| format!("{e}"))
                        {
                            Ok(value) => Ok((format!("{entity:?}"), value)),
                            Err(e) => Err(e),
                        }
                    }),
            )
            .collect::<Result<HashMap<_, _>, String>>()?;

        let removed = removed.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>();

        if added.is_empty() && removed.is_empty() {
            return Ok(());
        }

        let event = UpdateKeyframes { added, removed };

        info!("Updating keyframes: {event:?}");

        crate::wasm::dispatch_to_js(
            window,
            "update-keyframes",
            &serde_json::to_value(&event).map_err(|e| format!("{e}"))?,
        )
    }

    pub(super) fn notify_seek_time_changes(
        r_global_seek_time: Res<GlobalSeekTime>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        if !r_global_seek_time.is_changed() {
            return Ok(());
        }
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };
        info!("Current Seek Time: {r_global_seek_time:?}");

        crate::wasm::dispatch_to_js(
            window,
            "seek-time-changed",
            &serde_json::to_value(r_global_seek_time.as_ref()).map_err(|e| format!("{e}"))?,
        )
    }

    pub(super) fn notify_keying_mode_changed(
        r_keying_mode: Res<State<KeyingMode>>,
        q_window: Query<&Window, With<PrimaryWindow>>,
    ) -> Result<(), String> {
        if !r_keying_mode.is_changed() {
            return Ok(());
        }
        let Ok(window) = q_window.get_single() else {
            return Ok(());
        };

        crate::wasm::dispatch_to_js(
            window,
            "keying-mode-changed",
            &serde_json::to_value(UpdateKeyingMode {
                automatic: match r_keying_mode.get() {
                    KeyingMode::Manual => false,
                    KeyingMode::Auto => true,
                },
            })
            .map_err(|e| format!("{e}"))?,
        )
    }
}
