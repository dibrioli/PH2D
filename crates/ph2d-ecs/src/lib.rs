#![forbid(unsafe_code)]
//! ph2d-ecs — wrapper sobre `bevy_ecs` 0.18 implementing ADR-0021
//! (simulation ↔ presentation boundary) and ADR-0025 (GameObject =
//! Entity + Components, hierarchy via `ChildOf`).
//!
//! Two opaque newtype `World`s + an `extract!` macro that's the only
//! ergonomic bridge from sim → present.
//!
//! Why two worlds: per ADR-0021, simulation state (Position, Velocity,
//! Health, FSM) lives in `SimWorld` and is the canonical state used
//! for save/replay/rollback. Presentation state (RenderInstance,
//! AnimationFrame, ParticleBatch, EditorWidget) lives in
//! `PresentWorld` and is rebuilt each frame from sim via `extract!`.
//!
//! Compile-time enforcement of the boundary lives inside the
//! `extract!` macro: the sim handle inside the body is `&World`
//! (immutable), so accidentally mutating it from a presentation
//! system is a `cargo build` error. The marker traits
//! [`SimComponent`] and [`PresentComponent`] document intent; full
//! bundle-level enforcement (preventing `world.spawn(SimComp)` on a
//! `PresentWorld`) lands in M5+ via a custom clippy lint
//! `ph2d-clippy::wrong-world-component`.
//!
//! ## GameObject model (ADR-0025)
//!
//! - [`Transform`] (SimComponent) is local-space pos/rot/scale.
//! - [`GlobalTransform`] (PresentComponent) is world-space, rebuilt
//!   each frame by [`propagate_transforms`].
//! - [`Name`] (SimComponent) is the human-readable label.
//! - [`SimRef`] (PresentComponent) is the back-pointer from a
//!   present entity to its source sim entity (entity ids are
//!   per-`World`; this is the canonical bridge).
//! - Hierarchy is `bevy_ecs::hierarchy::ChildOf` + `Children` (re-
//!   exported below). PH2D does **not** define its own `Node` /
//!   `Parent` types — those would diverge from the upstream
//!   relationship machinery.
//!
//! Re-exports the bevy_ecs prelude essentials. Downstream crates use
//! `ph2d_ecs::*` and rarely need to import `bevy_ecs` directly,
//! preserving the ability to swap or version-bump bevy_ecs without
//! cascading import churn.

pub mod name;
pub mod present;
pub mod root_order;
pub mod scene;
pub mod sim;
pub mod transform;
pub mod transform_versioned;
pub mod visibility;

pub use name::Name;
pub use present::{PresentComponent, PresentWorld};
pub use root_order::RootOrder;
pub use sim::{SimComponent, SimWorld};
pub use transform::{
    GlobalTransform, GroupedChildren, Locked, SimRef, Transform, TransformPropagationState,
    WorklistBuf, is_locked_for_edit, parent_world_transform, propagate_transforms,
    propagate_transforms_into_present,
};
pub use transform_versioned::{
    TransformV1, TransformVersioned, load_transform, migrate_v1_to_v2, save_transform,
};
pub use visibility::Visibility;

// Re-export bevy_ecs essentials. Keep the surface small per LLM1
// audit anti-pattern "Acoplar API pública a tipos wgpu::* ou
// winit::*" (same principle here).
pub use bevy_ecs::component::Component;
pub use bevy_ecs::entity::Entity;
pub use bevy_ecs::hierarchy::{ChildOf, Children};
pub use bevy_ecs::query::{With, Without};
pub use bevy_ecs::resource::Resource;
pub use bevy_ecs::schedule::Schedule;
pub use bevy_ecs::system::{Commands, Query, Res, ResMut};
pub use bevy_ecs::world::World;

/// `extract!` — the canonical bridge from `SimWorld` to `PresentWorld`.
///
/// One-way per ADR-0021: the sim handle is `&World` (immutable), the
/// present handle is `&mut World` (mutable). Trying to spawn /
/// despawn / mutate sim inside the body is a compile-time error.
///
/// # Example
///
/// ```
/// use ph2d_ecs::{Component, PresentComponent, PresentWorld, SimComponent, SimWorld, extract};
///
/// #[derive(Component, Debug, Clone, Copy)]
/// struct Position { x: f32, y: f32 }
/// impl SimComponent for Position {}
///
/// #[derive(Component, Debug, Clone, Copy)]
/// struct RenderInstance { x: f32, y: f32 }
/// impl PresentComponent for RenderInstance {}
///
/// let mut sim = SimWorld::new();
/// let mut present = PresentWorld::new();
/// let e = sim.world_mut().spawn(Position { x: 1.0, y: 2.0 }).id();
///
/// extract!(sim => present, |sim_w, present_w| {
///     let pos = sim_w.get::<Position>(e).unwrap();
///     present_w.spawn(RenderInstance { x: pos.x, y: pos.y });
/// });
///
/// let mut q = present.world_mut().query::<&RenderInstance>();
/// assert_eq!(q.iter(present.world()).count(), 1);
/// ```
///
/// # Compile-fail proof of the read-only enforcement
///
/// Trying to `spawn` (mutate) the sim handle inside the body fails
/// with `cannot borrow `*sim_w` as mutable, as it is behind a `&` reference`:
///
/// ```compile_fail
/// use ph2d_ecs::{Component, PresentWorld, SimComponent, SimWorld, extract};
///
/// #[derive(Component)]
/// struct Bad;
/// impl SimComponent for Bad {}
///
/// let mut sim = SimWorld::new();
/// let mut present = PresentWorld::new();
/// extract!(sim => present, |sim_w, _present_w| {
///     sim_w.spawn(Bad);  // compile error: sim_w is &World, not &mut World
/// });
/// ```
#[macro_export]
macro_rules! extract {
    ($sim:expr => $present:expr, |$sim_w:ident, $present_w:ident| $body:block) => {{
        let __ph2d_sim_handle: &$crate::SimWorld = &$sim;
        let __ph2d_present_handle: &mut $crate::PresentWorld = &mut $present;
        let $sim_w: &$crate::World = __ph2d_sim_handle.world();
        let $present_w: &mut $crate::World = __ph2d_present_handle.world_mut();
        $body
    }};
}
