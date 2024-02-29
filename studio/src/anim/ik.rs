use bevy::{pbr::NotShadowCaster, prelude::*, render::view::RenderLayers};
use bevy_mod_picking::prelude::*;

use crate::{
    design_system,
    interaction::{gizmos::GizmoMaterial, NoSelectionOutline, Selectable},
    math::geo,
};

pub const IK_RIG_RENDER_LAYER: u8 = 1;

pub struct IkPlugin;

impl Plugin for IkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_ik_rig.map(bevy::utils::error));

        #[cfg(feature = "wasm")]
        app.add_systems(Update, wasm::insert_ik_component);
    }
}

#[derive(Component)]
pub struct IkRig;

#[derive(Component)]
pub struct IkRigCamera;

#[derive(Component, Clone, Copy)]
struct IkBoneChain {
    root: Entity,
    twist_1: Option<Entity>,
    mid: Entity,
    twist_2: Option<Entity>,
    end: Entity,
    len_upper: f32,
    len_lower: f32,
}

#[derive(Component)]
pub struct IkPoleTarget;

#[derive(Component)]
pub struct CopyXformTarget(pub Entity);

#[derive(Default)]
struct IkBoneChainBuilder {
    root: Option<Entity>,
    twist_1: Option<Entity>,
    mid: Option<Entity>,
    twist_2: Option<Entity>,
    end: Option<Entity>,
}

impl IkBoneChain {
    fn build() -> IkBoneChainBuilder {
        IkBoneChainBuilder::default()
    }
}

impl IkBoneChainBuilder {
    fn set_root(&mut self, ent: Entity) {
        self.root = Some(ent);
    }
    fn set_twist_1(&mut self, ent: Entity) {
        self.twist_1 = Some(ent);
    }
    fn set_mid(&mut self, ent: Entity) {
        self.mid = Some(ent);
    }
    fn set_twist_2(&mut self, ent: Entity) {
        self.twist_2 = Some(ent);
    }
    fn set_end(&mut self, ent: Entity) {
        self.end = Some(ent);
    }
    fn try_finish(self, q_xforms: &Query<&GlobalTransform>) -> Result<IkBoneChain, String> {
        let root = self.root.ok_or_else(|| "Root bone missing!".to_string())?;
        let mid = self.mid.ok_or_else(|| "Mid bone missing!".to_string())?;
        let end = self.end.ok_or_else(|| "End bone missing!".to_string())?;

        let root_xform = q_xforms.get(root).map_err(|err| format!("{err}"))?;
        let mid_xform = q_xforms.get(mid).map_err(|err| format!("{err}"))?;
        let end_xform = q_xforms.get(end).map_err(|err| format!("{err}"))?;

        let len_upper = Vec3::distance(root_xform.translation(), mid_xform.translation());
        let len_lower = Vec3::distance(mid_xform.translation(), end_xform.translation());

        Ok(IkBoneChain {
            root,
            twist_1: self.twist_1,
            mid,
            twist_2: self.twist_2,
            end,
            len_upper,
            len_lower,
        })
    }
}

fn setup_ik_rig(
    mut cmd: Commands,
    mut ra_meshes: ResMut<Assets<Mesh>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
    q_target: Query<Entity, Added<IkRig>>,
    q_children: Query<&Children>,
    q_names: Query<&Name>,
    q_world_xforms: Query<&GlobalTransform>,
) -> Result<(), String> {
    let Ok(target) = q_target.get_single() else {
        return Ok(());
    };

    let root_xform = q_world_xforms.get(target).map_err(|err| format!("{err}"))?;

    let mut leg_chain_l = IkBoneChain::build();
    let mut leg_chain_r = IkBoneChain::build();

    let mut arm_chain_l = IkBoneChain::build();
    let mut arm_chain_r = IkBoneChain::build();

    let mut spine = None;
    let mut spine1 = None;

    for ent in q_children.iter_descendants(target) {
        let Ok(name) = q_names.get(ent) else {
            continue;
        };

        match name.as_str() {
            // Spine entities
            "DEF-spine" => spine = Some(ent),
            "DEF-spine.001" => spine1 = Some(ent),
            // Left Leg
            "DEF-thigh.L" => leg_chain_l.set_root(ent),
            "DEF-thigh.L.001" => leg_chain_l.set_twist_1(ent),
            "DEF-shin.L" => leg_chain_l.set_mid(ent),
            "DEF-shin.L.001" => leg_chain_l.set_twist_2(ent),
            "DEF-foot.L" => leg_chain_l.set_end(ent),
            // Right Leg
            "DEF-thigh.R" => leg_chain_r.set_root(ent),
            "DEF-thigh.R.001" => leg_chain_r.set_twist_1(ent),
            "DEF-shin.R" => leg_chain_r.set_mid(ent),
            "DEF-shin.R.001" => leg_chain_r.set_twist_2(ent),
            "DEF-foot.R" => leg_chain_r.set_end(ent),
            // Left Arm
            "DEF-upper_arm.L" => arm_chain_l.set_root(ent),
            "DEF-upper_arm.L.001" => arm_chain_l.set_twist_1(ent),
            "DEF-forearm.L" => arm_chain_l.set_mid(ent),
            "DEF-forearm.L.001" => arm_chain_l.set_twist_2(ent),
            "DEF-hand.L" => arm_chain_l.set_end(ent),
            // Right Arm
            "DEF-upper_arm.R" => arm_chain_r.set_root(ent),
            "DEF-upper_arm.R.001" => arm_chain_r.set_twist_1(ent),
            "DEF-forearm.R" => arm_chain_r.set_mid(ent),
            "DEF-forearm.R.001" => arm_chain_r.set_twist_2(ent),
            "DEF-hand.R" => arm_chain_r.set_end(ent),
            _ => {}
        }
    }

    if let Some(spine) = spine {
        if let Some(spine1) = spine1 {
            let spine_name = q_names.get(spine).unwrap();
            let spine_xform = q_world_xforms.get(spine).unwrap();
            let spine1_xform = q_world_xforms.get(spine1).unwrap();

            cmd.entity(target).with_children(|builder| {
                builder
                    .spawn((
                        Selectable,
                        CopyXformTarget(spine),
                        SpatialBundle {
                            transform: spine_xform.reparented_to(root_xform),
                            ..default()
                        },
                    ))
                    .with_children(|builder| {
                        builder.spawn((
                            Name::new(format!("TGT_{spine_name}")),
                            NotShadowCaster,
                            NoSelectionOutline,
                            RenderLayers::layer(IK_RIG_RENDER_LAYER),
                            MaterialMeshBundle {
                                transform: spine1_xform.reparented_to(spine_xform),
                                mesh: ra_meshes.add(Mesh::from(shape::Cube { size: 0.25 })),
                                material: ra_mats.add(GizmoMaterial {
                                    color: design_system::Color::YELLOW.with_a(0.125),
                                    blend_mode: AlphaMode::Blend,
                                }),
                                ..default()
                            },
                            PickableBundle::default(),
                            On::<Pointer<Over>>::run(on_ik_gizmo_hover),
                            On::<Pointer<Out>>::run(on_ik_gizmo_unhover),
                        ));
                    });
            });
        }
    }

    for (color, bone_chain) in [
        (design_system::Color::RED, leg_chain_l),
        (design_system::Color::BLUE, leg_chain_r),
        (design_system::Color::RED, arm_chain_l),
        (design_system::Color::BLUE, arm_chain_r),
    ] {
        let bone_chain = bone_chain.try_finish(&q_world_xforms)?;
        let end_xform = q_world_xforms.get(bone_chain.end).unwrap();
        let mid_xform = q_world_xforms.get(bone_chain.mid).unwrap();

        let end_name = q_names.get(bone_chain.end).unwrap();
        let mid_name = q_names.get(bone_chain.mid).unwrap();

        cmd.entity(target).with_children(|builder| {
            builder.spawn((
                bone_chain,
                Name::new(format!("IK_{end_name}")),
                NotShadowCaster,
                NoSelectionOutline,
                RenderLayers::layer(IK_RIG_RENDER_LAYER),
                MaterialMeshBundle {
                    transform: end_xform.reparented_to(root_xform),
                    mesh: ra_meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                    material: ra_mats.add(GizmoMaterial {
                        color: color.with_a(0.4),
                        blend_mode: AlphaMode::Blend,
                    }),
                    ..default()
                },
                PickableBundle::default(),
                Selectable,
                On::<Pointer<Over>>::run(on_ik_gizmo_hover),
                On::<Pointer<Out>>::run(on_ik_gizmo_unhover),
            ));

            builder
                .spawn((
                    Selectable,
                    IkPoleTarget,
                    SpatialBundle {
                        transform: mid_xform.reparented_to(root_xform),
                        ..default()
                    },
                ))
                .with_children(|builder| {
                    let transform = Transform::from_xyz(0., 0., -0.1).with_rotation(
                        Quat::from_axis_angle(Vec3::NEG_X, std::f32::consts::FRAC_PI_2),
                    );

                    builder.spawn((
                        Name::new(format!("IK_{mid_name}")),
                        NotShadowCaster,
                        NoSelectionOutline,
                        RenderLayers::layer(IK_RIG_RENDER_LAYER),
                        MaterialMeshBundle {
                            transform,
                            mesh: ra_meshes.add(Mesh::from(geo::Cone {
                                radius: 0.05,
                                length: 0.15,
                                resolution: 12,
                            })),
                            material: ra_mats.add(GizmoMaterial {
                                color: color.with_a(0.4),
                                blend_mode: AlphaMode::Blend,
                            }),
                            ..default()
                        },
                        PickableBundle::default(),
                        On::<Pointer<Over>>::run(on_ik_gizmo_hover),
                        On::<Pointer<Out>>::run(on_ik_gizmo_unhover),
                    ));
                });
        });
    }

    Ok(())
}

fn on_ik_gizmo_hover(
    ev: Listener<Pointer<Over>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
    q_target: Query<&Handle<GizmoMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.color *= 1.5;
}

fn on_ik_gizmo_unhover(
    ev: Listener<Pointer<Out>>,
    mut ra_mats: ResMut<Assets<GizmoMaterial>>,
    q_target: Query<&Handle<GizmoMaterial>>,
) {
    let mat = ra_mats.get_mut(q_target.get(ev.target).unwrap()).unwrap();
    mat.color *= 2. / 3.;
}

#[cfg(feature = "wasm")]
mod wasm {
    use bevy::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::wasm::{FromStr, StaticBuffer, WasmMessageBuffer};

    use super::IkRig;

    static ATTACH_IK_RIG: WasmMessageBuffer<Entity> = WasmMessageBuffer::new();

    #[wasm_bindgen(js_name = attachIkRig)]
    pub fn attach_ik_rig(entity: &str) {
        let entity = Entity::from_str(entity).unwrap();
        ATTACH_IK_RIG.write_message(entity);
    }

    pub(super) fn insert_ik_component(mut cmd: Commands) {
        if let Some(entity) = ATTACH_IK_RIG.take_message() {
            cmd.entity(entity).insert(IkRig);
        }
    }
}
