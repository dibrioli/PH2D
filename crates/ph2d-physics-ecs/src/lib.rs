#![forbid(unsafe_code)]
//! ph2d-physics-ecs — the bridge that wires the dormant `ph2d-physics`
//! rapier wrapper (M10) into the ECS as a **global, runtime-truth** rigid
//! world (ADR-0131 W1).
//!
//! - [`RigidBody`] / [`Collider`] are ECS components (config only) that an
//!   entity carries to become a physics body.
//! - [`PhysicsBridge`] owns the transient rapier world + the entity↔handle
//!   map and is driven once per frame at the `Playhead` tick.
//! - [`register_physics_components`] plugs the components into the shared
//!   [`ComponentRegistry`] so they persist in the `WorldSnapshot` (undo +
//!   save) for free — call it at boot next to `register_render_components`.
//!
//! rapier stays confined to `ph2d-physics`; this crate holds only the
//! re-exported handle types, so the `enhanced-determinism` feature pin
//! lives in exactly one Cargo.toml (HR-5).

pub mod bake;
mod bridge;
mod components;
pub mod interaction;
mod joint;
mod joint_group;
pub mod joint_tool;
pub mod name_refs;
mod parts;
/// ⭐ **O REMAP identidade → identidade** (ADR-0164 / F4.2) — o que faz a junta de uma
/// cópia prender os corpos DELA. Irmão do [`name_refs`], que traduz NOME → identidade.
pub mod ref_remap;
mod rig;
mod scale;
mod seam;
pub mod settings;

pub use bake::{
    BakedTrajectory, PoseChannel, RecordedRun, bake_trajectories, bake_trajectories_with_scene,
    bake_trajectories_with_scene_and_tape,
};
pub use bridge::anchors::JointSide;
pub use bridge::contacts::{
    BodyContact, CONTACT_FLASH_TICKS, ContactEvent, ContactFlash, ContactPhase,
};
pub use bridge::joint_break::JointBreakEvent;
pub use bridge::signals::SignalEvent;
pub use bridge::triggers::TriggerEvent;
pub use name_refs::{ResolvedRefs, resolve_body_names};
pub use ref_remap::{remap_joint_refs, remap_wheel_refs};
// O par de números que um readout de joint mostra. Re-exportado porque a shell
// não depende de `ph2d-physics` direto.
pub use bridge::Launch;
pub use bridge::fk::FkSession;
pub use bridge::ik::{IkPlan, IkSession};
pub use bridge::joints::joint_desc;
pub use bridge::player_view::{ProbeKind, ProbeMark, ProbeShape, ProbeState};
// **O que a lei do player de facto lê deste corpo** — a resposta que a §14 do
// Inspector pinta. Ela sai da MESMA porta que decide quem escreve a pose; a
// shell re-derivá-la do `PlayerMode` foi o que fazia o painel mentir sobre um
// player assado.
pub use bridge::pose_owner::PlayerLiveness;
pub use bridge::rope::pulley_rig;
// A geometria da corda de uma polia. Re-exportada porque a shell **não depende
// de `ph2d-physics`** — a mesma contenção que mantém o rapier confinado — e o
// desenho tem de rodar a MESMA rota que o solver roda.
pub use bridge::views::JointView;
pub use bridge::{
    FrozenScene, HeldInput, InputTape, PhysicsBridge, PlayerInputAtTick, SceneAtTick, TapeWire,
};
pub use components::{
    AreaBuoyancy, AreaDrag, AreaEffector, AreaFalloff, AreaForceWorldAxes, AreaFormDrag,
    AreaTorque, BodyKind, Ccd, Collider, ColliderShape, CombineRule, DampMode, DampingOverride,
    Dominance, GravityScale, InitialVelocity, LockPositionX, LockPositionY, LockRotation,
    MassOverride, MaterialCombine, NoWallCling, OneWayPlatform, PlatformLift, PlatformPlayer,
    PlayerMode, PlayerSignals, PulleyWheel, RigidBody, RopeStops, SignalOnHit, SignalOnLeave,
    WalkSurface, WestonAxle, WrapSide, reseat_mounted_axle, reseat_wheel_geometry, rope_joint_of,
};
pub use interaction::{
    HoldMode, InteractionSettings, InteractionTool, MAX_ATTRACT_FORCE, MAX_BLAST_IMPULSE,
    MAX_HOLD_DAMPING_RATIO, MAX_HOLD_STIFFNESS, MIN_HOLD_STIFFNESS, WORLD_REACH_M,
};
pub use joint::{
    AxisMode, AxisSpec, CustomAxes, CustomAxis, JointKind, JointWorldAnchor, LengthField,
    MotorMode, PhysicsJoint,
};
pub use joint_group::{jointed_by, jointed_group, jointed_rig};
pub use joint_tool::{DragReach, JointGesture, JointTool};
pub use parts::{auto_mass_with_parts, count_parts, governing_kind, is_part, owner_body};
/// A geometria do LIMITADOR (W-RopeStop) — a marca, e a inversa dela.
///
/// Re-exportada ao lado da `rope_route` e pelo mesmo motivo: o desenho e o
/// arrasto do shell autoram sobre a corda, e têm de fazê-lo pelas MESMAS funções
/// que o solver usa para decidir onde a trava age.
pub use ph2d_physics::world::pulley::{StopLeg, stop_at_point, stop_mark};
pub use ph2d_physics::world::rope_route;
pub use ph2d_physics::{IkOptions, JointLoad};
/// A entrada de um player, re-exportada da lei pura.
///
/// ⚠️ Re-exportada em vez de re-declarada: quem escreve a entrada (a shell) e
/// quem a consome (a lei) têm de falar do MESMO tipo — um espelho na ponte
/// seria a segunda porta que diverge no dia em que o pulo (W4) entra num lado só.
pub use ph2d_platformer::PlayerInput;
// ⚠️ Re-exportado para o OVERLAY (`W-Probes`): quem desenha o perfil do teto tem
// de perguntar os deslocamentos à porta da LEI, e a shell fala com esta crate —
// não com a `ph2d-platformer`. Re-exportar em vez de acrescentar uma aresta de
// `Cargo.toml` mantém a porta ÚNICA sem alargar o grafo de dependências.
/// **A config EFETIVA do agachar** (`W-Brink`) — pela MESMA porta acima, e pelo
/// mesmo motivo: quem pinta a row da trava de beirada agachado precisa de saber
/// o que a lei vai de facto ler, e uma segunda cópia do `&&` na shell seria a
/// segunda resposta a *"o agachar aperta ou solta?"*.
pub use ph2d_platformer::walk_for;
pub use ph2d_platformer::{
    CORNER_SAMPLES, MAX_CORNER_SAMPLES, MAX_WALL_SAMPLES, WALL_SAMPLES, corner_offsets,
    odd_samples, wall_offsets,
};
/// A SAÍDA da lei, re-exportada pela MESMA razão dos vizinhos acima: quem a lê é
/// o Inspector, e a shell **não depende da `ph2d-platformer`**. Re-exportar em
/// vez de alargar o grafo de dependências mantém a contenção e a porta única.
pub use ph2d_platformer::{FootingKind, JumpKind, PlayerEvent, PlayerView};
/// A config da lei, re-exportada pela MESMA razão do `rope_route` e do
/// `ShapeDesc`: a shell **não depende da `ph2d-platformer`** (a contenção que
/// mantém o rapier e a lei confinados), e o Inspector precisa do ponto de
/// partida e do piso geométrico da altura de flutuação. Uma segunda cópia deles
/// na shell seria a segunda resposta a *"com que números um player nasce?"*.
pub use ph2d_platformer::{PlayerConfig, RideConfig, WalkConfig};
pub use rig::{RIG_LIMIT_DEG, rig_edges, rig_limits, subtree_parts};
pub use scale::scaled_shape;
pub use seam::{ColliderPose, seam_between, seam_point};
// `ShapeDesc` + the ellipse tessellation are re-exported so the overlay (in
// the shell, which only deps this crate) draws the SAME resolved shape the
// bridge simulates — one import path, one answer. `zone_force_world_at` is there
// for the same reason and it matters more: the arrow IS the only place the wind's
// direction is ever read by a person, so a second answer would be a picture of a
// blow that does not happen, and no gate reads a screenshot.
pub use ph2d_physics::{
    CAPSULE_CAP_SEGS, ELLIPSE_SEGS, LayerMatrix, MAX_LAYERS, PhysicsWorld, ShapeDesc,
    capsule_vertices, ellipse_vertices, zone_force_world_at, zone_spin_sign,
};
pub use settings::{
    DEFAULT_SOLVER_ITERATIONS, GRAVITY_LIMIT, MAX_AIR_DRAG, MAX_CONTACT_HZ, MAX_DAMPING,
    MAX_SLEEP_THRESHOLD, MAX_SOLVER_ITERATIONS, MAX_SUBSTEPS, MAX_TIME_UNTIL_SLEEP, MIN_CONTACT_HZ,
    PhysicsSettings,
};

use ph2d_ecs::scene::ComponentRegistry;

/// Register the components owned by `ph2d-physics-ecs` against the shared
/// [`ComponentRegistry`]. The shell calls this once at boot alongside
/// `register_ecs_components` and `register_render_components`.
///
/// Without it the `WorldSnapshot` **silently drops** `RigidBody`/`Collider`
/// (the `Locked`/`GroupedChildren`/`VecPathRef` bug). Registered here, they
/// round-trip through undo + save with zero snapshot-side code.
pub fn register_physics_components(reg: &mut ComponentRegistry) {
    reg.register_default::<RigidBody>("ph2d::physics::RigidBody");
    reg.register_default::<Collider>("ph2d::physics::Collider");
    reg.register_default::<PhysicsJoint>("ph2d::physics::PhysicsJoint");
    reg.register_default::<GravityScale>("ph2d::physics::GravityScale");
    reg.register_default::<PlayerSignals>("ph2d::physics::PlayerSignals");
    reg.register_default::<SignalOnHit>("ph2d::physics::SignalOnHit");
    reg.register_default::<SignalOnLeave>("ph2d::physics::SignalOnLeave");
    reg.register_default::<InitialVelocity>("ph2d::physics::InitialVelocity");
    reg.register_default::<Ccd>("ph2d::physics::Ccd");
    reg.register_default::<LockRotation>("ph2d::physics::LockRotation");
    reg.register_default::<LockPositionX>("ph2d::physics::LockPositionX");
    reg.register_default::<LockPositionY>("ph2d::physics::LockPositionY");
    reg.register::<MassOverride>("ph2d::physics::MassOverride");
    reg.register::<Dominance>("ph2d::physics::Dominance");
    reg.register_default::<MaterialCombine>("ph2d::physics::MaterialCombine");
    reg.register_default::<PulleyWheel>("ph2d::physics::PulleyWheel");
    reg.register_default::<DampingOverride>("ph2d::physics::DampingOverride");
    reg.register_default::<OneWayPlatform>("ph2d::physics::OneWayPlatform");
    reg.register_default::<AreaEffector>("ph2d::physics::AreaEffector");
    reg.register_default::<AreaDrag>("ph2d::physics::AreaDrag");
    reg.register_default::<AreaBuoyancy>("ph2d::physics::AreaBuoyancy");
    reg.register_default::<AreaFormDrag>("ph2d::physics::AreaFormDrag");
    reg.register_default::<AreaTorque>("ph2d::physics::AreaTorque");
    reg.register_default::<AreaForceWorldAxes>("ph2d::physics::AreaForceWorldAxes");
    reg.register_default::<AreaFalloff>("ph2d::physics::AreaFalloff");
    reg.register_default::<WestonAxle>("ph2d::physics::WestonAxle");
    reg.register_default::<RopeStops>("ph2d::physics::RopeStops");
    reg.register_default::<JointWorldAnchor>("ph2d::physics::JointWorldAnchor");
    reg.register_default::<PlatformPlayer>("ph2d::physics::PlatformPlayer");
    reg.register_default::<PlayerMode>("ph2d::physics::PlayerMode");
    reg.register_default::<WalkSurface>("ph2d::physics::WalkSurface");
    reg.register_default::<NoWallCling>("ph2d::physics::NoWallCling");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This count exists to hurt (mirrors `register_ecs_components_populates_registry`):
    /// a physics component that skips `register_physics_components` is
    /// dropped from every snapshot in silence. If you add one, bump this.
    #[test]
    fn registers_every_physics_component() {
        let mut reg = ComponentRegistry::new();
        register_physics_components(&mut reg);
        assert_eq!(reg.len(), 32);
        assert!(reg.get_by_name("ph2d::physics::RigidBody").is_some());
        assert!(reg.get_by_name("ph2d::physics::Collider").is_some());
        assert!(reg.get_by_name("ph2d::physics::PhysicsJoint").is_some());
        assert!(reg.get_by_name("ph2d::physics::GravityScale").is_some());
        assert!(reg.get_by_name("ph2d::physics::InitialVelocity").is_some());
        assert!(reg.get_by_name("ph2d::physics::Ccd").is_some());
        assert!(reg.get_by_name("ph2d::physics::LockRotation").is_some());
        assert!(reg.get_by_name("ph2d::physics::LockPositionX").is_some());
        assert!(reg.get_by_name("ph2d::physics::LockPositionY").is_some());
        assert!(reg.get_by_name("ph2d::physics::MassOverride").is_some());
        assert!(reg.get_by_name("ph2d::physics::Dominance").is_some());
        assert!(reg.get_by_name("ph2d::physics::WalkSurface").is_some());
        assert!(reg.get_by_name("ph2d::physics::NoWallCling").is_some());
        assert!(reg.get_by_name("ph2d::physics::MaterialCombine").is_some());
        assert!(reg.get_by_name("ph2d::physics::DampingOverride").is_some());
        assert!(reg.get_by_name("ph2d::physics::OneWayPlatform").is_some());
        assert!(reg.get_by_name("ph2d::physics::PlayerSignals").is_some());
        assert!(reg.get_by_name("ph2d::physics::SignalOnHit").is_some());
        assert!(reg.get_by_name("ph2d::physics::SignalOnLeave").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaEffector").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaDrag").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaBuoyancy").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaFormDrag").is_some());
        assert!(reg.get_by_name("ph2d::physics::PulleyWheel").is_some());
        assert!(reg.get_by_name("ph2d::physics::AreaTorque").is_some());
        assert!(
            reg.get_by_name("ph2d::physics::AreaForceWorldAxes")
                .is_some()
        );
        assert!(reg.get_by_name("ph2d::physics::AreaFalloff").is_some());
        assert!(reg.get_by_name("ph2d::physics::WestonAxle").is_some());
        assert!(reg.get_by_name("ph2d::physics::RopeStops").is_some());
        assert!(reg.get_by_name("ph2d::physics::PlayerMode").is_some());
    }
}
