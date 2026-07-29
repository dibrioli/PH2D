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

pub mod blend;
pub mod flip_object_ref;
pub mod masking;
pub mod name;
pub mod painted_doc;
pub mod present;
pub mod root_order;
pub mod sampling;
pub mod scene;
pub mod sim;
pub mod sort_key;
pub mod sorting;
pub mod transform;
pub mod transform_inverse;
pub mod transform_versioned;

pub use crate::transform_inverse::{
    parent_world_transform, parent_world_transform_into, world_transform, world_transform_into,
};
pub mod vec_path_ref;
pub mod vec_shape;
pub mod visibility;
pub mod visibility_layer;

pub use blend::BlendMode;
pub use flip_object_ref::FlipObjectRef;
pub use masking::{ClipChildren, ClipMode, Mask2D, MaskInteraction, MaskMode};
pub use name::{Name, stable_name_id};
pub use painted_doc::PaintedDoc;
pub use present::{PresentComponent, PresentWorld};
pub use root_order::{RootOrder, assign_missing_root_order};
pub use sampling::{
    FilterMode, RepeatMode, TextureFilter, TextureRepeat, UvTransform, resolve_texture_filter,
    resolve_texture_repeat,
};
pub use sim::{SimComponent, SimWorld};
pub use sort_key::{SortInput, SortKey, compute_sort_ranks};
pub use sorting::{
    LayerId, OrderInLayer, ShowBehindParent, SortPoint, SortingGroup, SortingLayer, SortingLayers,
    TopLevel, YSort, ZAsRelative, ZIndexOverride,
};
pub use transform::{
    GlobalTransform, GroupedChildren, Locked, SimRef, Transform, TransformPropagationState,
    WorklistBuf, is_locked_for_edit, propagate_transforms, propagate_transforms_into_present,
};
pub use transform_versioned::{
    TransformV1, TransformVersioned, load_transform, migrate_v1_to_v2, save_transform,
};
pub use vec_path_ref::VecPathRef;
pub use vec_shape::{MAX_SHAPE_VALUES, VecShape, VecTextParams};

/// **O CONECTOR** — a linha que gruda em duas formas e as segue. Espelha o padrão da Live
/// Shape: o componente guarda a RELAÇÃO, a geometria é uma função pura dela. O alvo é um
/// `VecPathId` (o id do documento), nunca bits de entidade — o undo respawna tudo com bits
/// novos, e um conector guardado por bits se soltaria a cada Ctrl+Z.
mod vec_connector;
pub use vec_connector::{Anchor, ConnectorEnd, DEFAULT_CURVE_ARM, RouteKind, VecConnector};

/// **O Blend Object vivo** (ADR-0128) — o objeto único que interpola 2..=5 formas. Mesma família
/// do conector, e pela mesma razão: o componente guarda a RELAÇÃO (quais formas, na ordem, e
/// quantos passos), e a aparência é uma função pura dela. As fontes são `VecPathId`, nunca bits
/// de entidade — o undo respawna tudo, e um blend guardado por bits se soltaria a cada Ctrl+Z.
mod vec_blend;
pub use vec_blend::VecBlend;
mod vec_contour;
mod vec_filter;
mod vec_filter_kinds;
mod vec_filter_new;
mod vec_offset;
mod vec_pattern_path;
mod vec_pattern_rotation;
mod vec_text_path;
pub use vec_contour::{MAX_CONTOUR_STEPS, VecContour};
pub use vec_filter::{FxKindSpec, FxOp, VecFilter};
pub use vec_offset::VecOffset;
pub use vec_pattern_path::VecPatternPath;
pub use vec_pattern_rotation::VecPatternRotation;
pub use vec_text_path::VecTextPath;

mod vec_morph;
pub use vec_morph::VecMorph;

mod vec_envelope;
pub use vec_envelope::{
    ENVELOPE_DEFAULT_BEND, EnvelopeKind, EnvelopeWarp, VecEnvelope, VecEnvelopeChild,
};

/// **O RÓTULO** — o texto que pertence a uma forma (ou a um conector) e a segue. Mesma família
/// do conector, e pela mesma razão: o componente guarda a RELAÇÃO (de quem, e onde em relação a
/// ele), e a pose é uma função pura dela. O alvo é um `VecPathId`, nunca bits de entidade.
mod vec_label;
pub use vec_label::VecLabel;
pub use visibility::Visibility;
pub use visibility_layer::{EnableMode, OnScreenEnabler, VisibilityLayer};

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
