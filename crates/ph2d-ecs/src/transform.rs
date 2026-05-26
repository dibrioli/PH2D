//! 2D `Transform` (local-space) + `GlobalTransform` (world-space) +
//! deterministic hierarchical propagation.
//!
//! # Why two transform types
//!
//! ADR-0025 picks Unity-style ECS-composition; ADR-0021 enforces the
//! Sim ↔ Present split. `Transform` is the canonical local-space
//! representation a game/script author writes — it lives in
//! [`crate::SimWorld`] as a [`SimComponent`]. `GlobalTransform` is
//! the affine-flattened world-space representation the renderer
//! consumes — it lives in [`crate::PresentWorld`] as a
//! [`PresentComponent`], rebuilt every frame by
//! [`propagate_transforms`].
//!
//! # Why a `WorklistBuf`
//!
//! HR-3 forbids dynamic allocation in hot paths. The natural DFS over
//! the hierarchy needs a stack; we keep that stack as a Resource on
//! `PresentWorld` ([`WorklistBuf`]), clear it between frames, and the
//! `Vec` capacity stays put. Zero allocs after warm-up — verified in
//! `tests/propagate_no_alloc.rs` with `dhat-rs`.
//!
//! # Determinism (HR-5)
//!
//! - Roots are sorted by `Entity::to_bits()` before the DFS seeds.
//! - Children are read via [`bevy_ecs::hierarchy::Children`] and
//!   pushed in reverse so the DFS visits them in `Children`-declared
//!   order (which `bevy_ecs` 0.18 preserves as insertion order).
//! - No floating-point reordering inside `Transform::compose`.
//! - Final hash of `GlobalTransform.matrix` over the present world is
//!   bit-identical across Linux/Mac/Windows — verified in
//!   `tests/transform_determinism.rs`.
//!
//! # The cross-world join
//!
//! `propagate_transforms` is generic over an `on_each` closure so the
//! caller (e.g. `shells/desktop` building [`RenderInstance`]s) can
//! join a `SimComponent` read from `sim_w` with the just-computed
//! `GlobalTransform` while still inside the same iteration. This
//! sidesteps the impossible `Query<(&SimComp, &PresentComp)>` cross-
//! world join: the function holds both world handles, the closure
//! receives both, and the spawn decisions stay in caller code.

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::query::{QueryState, With, Without};
use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;
use ph2d_core::{Mat3, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{PresentComponent, SimComponent};

/// Local-space 2D affine: translation (meters), rotation (radians,
/// CCW from +X), and per-axis scale (multiplicative).
///
/// Composes with parent via [`Transform::compose`]. Identity is
/// `translation=0`, `rotation=0`, `scale=(1,1)`.
///
/// `SimComponent` — canonical state in `SimWorld`. HR-14 mitigation:
/// [`Transform::VERSION`] is the stable schema marker until the
/// `Saveable` derive macro lands.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform {
    /// Schema version. Bumped (alongside a migration function) when
    /// the on-disk layout of `Transform` changes.
    pub const VERSION: u32 = 1;

    /// Identity transform: zero translation, zero rotation, unit
    /// scale. `const` so it can be used in const initializers.
    pub const IDENTITY: Self = Self {
        translation: Vec2::new(0.0, 0.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
    };

    /// Pure translation, identity rotation/scale. Most common spawn
    /// helper — equivalent to Unity's `Vector3.position` set.
    pub const fn from_translation(t: Vec2) -> Self {
        Self {
            translation: t,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
        }
    }

    /// Compose `parent` and `child` (both local-space) into a single
    /// local-space transform expressed in `parent`'s parent frame.
    /// Standard TRS composition: scale child, rotate, translate.
    ///
    /// `parent * child` (via [`std::ops::Mul`]) is the operator form
    /// — use whichever is clearer at the call site. Inherent method
    /// kept because the propagation walk needs a `const fn`-amenable
    /// `(Self, Self) -> Self` signature without trait dispatch.
    ///
    /// Determinism (HR-5): no FMA, no SIMD reordering; the explicit
    /// expressions below are stable across `target_arch` because
    /// `f32::sin`/`cos`/`mul`/`add` are bit-deterministic given the
    /// same inputs.
    #[inline]
    pub fn compose(parent: Self, child: Self) -> Self {
        let (sin, cos) = parent.rotation.sin_cos();
        let sx = child.translation.x * parent.scale.x;
        let sy = child.translation.y * parent.scale.y;
        let rx = sx * cos - sy * sin;
        let ry = sx * sin + sy * cos;
        Self {
            translation: Vec2::new(parent.translation.x + rx, parent.translation.y + ry),
            rotation: parent.rotation + child.rotation,
            scale: Vec2::new(
                parent.scale.x * child.scale.x,
                parent.scale.y * child.scale.y,
            ),
        }
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;
    /// Operator form of [`Transform::compose`]: `parent * child`.
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::compose(self, rhs)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl SimComponent for Transform {}

/// Compose the world-space [`Transform`] of the parent chain of `entity`
/// by walking `ChildOf` bottom-up and re-composing top-down via
/// [`Transform::compose`]. Returns [`Transform::IDENTITY`] when `entity`
/// is a root (no `ChildOf`).
///
/// Used by the gizmo drag pipeline (`shells/desktop/src/input_dispatch`)
/// so writes into the entity's LOCAL `Transform` correctly compensate
/// for ancestor rotation/scale — without this helper, a child of a
/// rotated parent translates/scales along the local (rotated) axis
/// instead of the visual (world) axis.
///
/// Allocates a small `Vec<Transform>` for the chain (depth is usually
/// 0–3); fine for one-shot lookup at gesture start.
/// Marker: this entity's `Transform` is locked against gizmo edits.
/// The gizmo Down handler queries `is_locked_for_edit()` and rejects
/// the gesture when the marker is present. Children of a `Locked`
/// entity remain editable (use [`GroupedChildren`] to lock descendants).
/// Enio 2026-05-26: "Cadeado trava apenas o objeto".
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locked;

impl SimComponent for Locked {}

/// Marker: this entity's DESCENDANTS are locked against gizmo edits,
/// but the entity itself remains editable. Enio 2026-05-26: "Agrupar:
/// você pode manipular o objeto pai do grupo mas não os seus filhos".
/// Recursive — every entity whose ancestor chain contains a
/// `GroupedChildren` carrier is considered locked.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupedChildren;

impl SimComponent for GroupedChildren {}

/// True iff `entity` has `Locked`, OR any ancestor (via `ChildOf`)
/// has `GroupedChildren`. Used by the gizmo Down handlers to reject
/// drag gestures on locked entities.
pub fn is_locked_for_edit(world: &World, entity: Entity) -> bool {
    if world.get::<Locked>(entity).is_some() {
        return true;
    }
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    while let Some(p) = cur {
        if world.get::<GroupedChildren>(p).is_some() {
            return true;
        }
        cur = world.get::<ChildOf>(p).map(|c| c.parent());
    }
    false
}

pub fn parent_world_transform(world: &World, entity: Entity) -> Transform {
    let mut chain: Vec<Transform> = Vec::new();
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    while let Some(p) = cur {
        if let Some(t) = world.get::<Transform>(p) {
            chain.push(*t);
        }
        cur = world.get::<ChildOf>(p).map(|c| c.parent());
    }
    let mut acc = Transform::IDENTITY;
    for t in chain.iter().rev() {
        acc = Transform::compose(acc, *t);
    }
    acc
}

/// World-space affine transform as a 3×3 column-major matrix.
///
/// Computed per-frame by [`propagate_transforms`] from a chain of
/// local [`Transform`]s. Lives in `PresentWorld` and is rebuilt
/// from scratch each frame — **never** participates in save/replay
/// (HR-5 says simulated state only).
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct GlobalTransform {
    /// Column-major 2D affine. Columns are:
    /// - col 0 = `(cos·sx,  sin·sx, 0)` — x-basis after rotation+scale
    /// - col 1 = `(-sin·sy, cos·sy, 0)` — y-basis after rotation+scale
    /// - col 2 = `(tx, ty, 1)` — translation in homogeneous form
    pub matrix: Mat3,
}

impl GlobalTransform {
    /// Build a world-space affine from a fully-composed (post-walk)
    /// local-space [`Transform`].
    pub fn from_transform(t: Transform) -> Self {
        let (sin, cos) = t.rotation.sin_cos();
        let matrix = Mat3::from_cols(
            Vec3::new(cos * t.scale.x, sin * t.scale.x, 0.0),
            Vec3::new(-sin * t.scale.y, cos * t.scale.y, 0.0),
            Vec3::new(t.translation.x, t.translation.y, 1.0),
        );
        Self { matrix }
    }

    /// World-space translation extracted from column 2 of the matrix.
    /// Cheaper than reconstructing a full `Transform`.
    pub fn translation(&self) -> Vec2 {
        Vec2::new(self.matrix.z_axis.x, self.matrix.z_axis.y)
    }

    /// Affine coefficients `[a, b, c, d, e, f]` in column-major order
    /// — the layout Kurbo/Vello consume directly via
    /// `kurbo::Affine::new`.
    pub fn affine(&self) -> [f32; 6] {
        [
            self.matrix.x_axis.x,
            self.matrix.x_axis.y,
            self.matrix.y_axis.x,
            self.matrix.y_axis.y,
            self.matrix.z_axis.x,
            self.matrix.z_axis.y,
        ]
    }
}

impl PresentComponent for GlobalTransform {}

/// Back-reference from a `PresentWorld` entity to its source
/// `SimWorld` entity. `bevy_ecs` entity ids are per-`World`, so this
/// is the canonical bridge for anything that needs to find sim-side
/// data from a presentation-side entity (renderer, audio spatial,
/// editor inspector, etc.).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimRef(pub Entity);

impl PresentComponent for SimRef {}

/// Pre-allocated worklist scratch for [`propagate_transforms`].
///
/// One instance lives as a [`bevy_ecs::resource::Resource`] in the
/// `PresentWorld`. `clear()` is called at the start of each
/// propagation pass; the inner `Vec` retains its capacity so the
/// per-frame allocation count is zero (HR-3).
///
/// Default capacity is 8 192 entities — comfortably above
/// `SPRITE_COUNT` in the desktop demo and the editor hero fixture.
/// Resize upward if a scene needs more; resizing is a one-time
/// `Vec::reserve` outside the hot path.
#[derive(Resource)]
pub struct WorklistBuf {
    /// DFS stack: `(sim_entity, accumulated_parent_world_transform)`.
    pub(crate) stack: Vec<(Entity, Transform)>,
    /// Scratch list used by callers that want to materialize a
    /// children iteration without re-querying — exposed via
    /// `children_scratch_mut` for advanced callers (e.g. snapshot
    /// builders in M14.3).
    pub(crate) children_scratch: Vec<Entity>,
}

impl WorklistBuf {
    pub const DEFAULT_CAPACITY: usize = 8192;

    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            stack: Vec::with_capacity(cap),
            children_scratch: Vec::with_capacity(cap),
        }
    }

    /// Reset both inner buffers without releasing capacity.
    pub fn clear(&mut self) {
        self.stack.clear();
        self.children_scratch.clear();
    }

    /// Inspector for tests / advanced callers.
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    /// Capacity of the DFS stack. Useful for HR-3 assertions in tests
    /// that the buffer never reallocates mid-frame.
    pub fn stack_capacity(&self) -> usize {
        self.stack.capacity()
    }

    /// Mutable access to the children scratch — used by snapshot
    /// builders in M14.3 to collect a deterministic children list.
    pub fn children_scratch_mut(&mut self) -> &mut Vec<Entity> {
        &mut self.children_scratch
    }
}

impl Default for WorklistBuf {
    fn default() -> Self {
        Self::new()
    }
}

/// Cached `bevy_ecs` query state for [`propagate_transforms`].
///
/// `QueryState` is the only way to iterate components from `&World`
/// (the read-only handle the `extract!` macro hands inside its body).
/// Constructing one needs `&mut World`, so the caller builds this
/// **once** outside the extract phase (typically at boot) and passes
/// it in by `&mut` each frame.
///
/// ## Why two queries
///
/// `roots` enumerates entities that have `Transform` and **not**
/// `ChildOf` — the seed set for the DFS. `chain` looks up
/// `(&Transform, Option<&Children>)` for an arbitrary entity during
/// the walk; using `get(world, entity)` avoids re-iterating the full
/// archetype set per node.
pub struct TransformPropagationState {
    pub(crate) roots: QueryState<Entity, (With<Transform>, Without<ChildOf>)>,
    pub(crate) chain: QueryState<(&'static Transform, Option<&'static Children>)>,
}

impl TransformPropagationState {
    /// Construct from a `&mut World`. Caller is the simulation world
    /// (typically `sim.world_mut()` once at boot).
    pub fn new(world: &mut World) -> Self {
        Self {
            roots: world.query_filtered::<Entity, (With<Transform>, Without<ChildOf>)>(),
            chain: world.query::<(&Transform, Option<&Children>)>(),
        }
    }
}

/// Run a deterministic hierarchical transform propagation pass.
///
/// Reads `Transform` + `ChildOf` + `Children` from `sim_w` via the
/// pre-built [`TransformPropagationState`] queries; for each
/// reachable entity computes the world-space transform by composing
/// `parent.world * local`. The `on_each` closure is invoked once per
/// entity with the freshly computed `GlobalTransform`, the
/// originating sim entity, and both world handles — so callers can
/// spawn whatever mirror entity they want in `present_w` (typically
/// `(SimRef, GlobalTransform)` plus any subsystem-specific render
/// instance / audio source / debug overlay).
///
/// `present_w` is **not** cleared by this function — callers decide.
/// (Most callers do `present_w.clear_entities()` before calling, but
/// the editor extract phase may want to preserve some entities.)
///
/// **Determinism guarantees** (HR-5):
/// - Root entities are sorted by `Entity::to_bits()` ascending before
///   the DFS seeds them, then popped LIFO so the smallest id is
///   processed first.
/// - Children are read in the order `bevy_ecs::hierarchy::Children`
///   returns them (insertion order in 0.18) and pushed in **reverse**
///   onto the worklist so the DFS visits them in original order.
/// - `Transform::compose` is deterministic (no FMA, no SIMD reordering).
///
/// **Zero-alloc guarantees** (HR-3): all temporary storage lives in
/// `WorklistBuf`; the function performs no `Vec::push` that grows
/// past capacity, no `Box::new`, no `String::from`. Entities without
/// `Transform` are silently skipped.
pub fn propagate_transforms<F>(
    sim_w: &World,
    state: &mut TransformPropagationState,
    present_w: &mut World,
    worklist: &mut WorklistBuf,
    mut on_each: F,
) where
    F: FnMut(&World, &mut World, Entity, GlobalTransform),
{
    worklist.clear();

    // Refresh both QueryStates' archetype caches against the current
    // world. `iter`/`get` would do this automatically per call, but
    // doing it once up-front is cheaper than per-iteration validation
    // and defends against any future bevy_ecs change that drops the
    // auto-update from a specific access path. Critical when new
    // entities of new archetypes spawn between propagations (e.g.
    // M14.4c imports add (Transform, Sprite, Name) without Velocity).
    state.roots.update_archetypes(sim_w);
    state.chain.update_archetypes(sim_w);

    // Phase 1 — seed roots via the pre-built filter.
    for entity in state.roots.iter(sim_w) {
        worklist.stack.push((entity, Transform::IDENTITY));
    }

    // Sort roots ascending so DFS order is platform-stable across
    // archetype reordering by `bevy_ecs` minor-version bumps.
    worklist.stack.sort_unstable_by_key(|&(e, _)| e.to_bits());

    // Phase 2 — DFS. Pop, compute world transform, invoke callback,
    // push children in reverse so the natural insertion order is
    // honored when they're popped.
    while let Some((entity, parent_world)) = worklist.stack.pop() {
        let Ok((local, children)) = state.chain.get(sim_w, entity) else {
            continue;
        };
        let world_local = Transform::compose(parent_world, *local);
        let gt = GlobalTransform::from_transform(world_local);

        on_each(sim_w, present_w, entity, gt);

        if let Some(children) = children {
            // Collect into the worklist scratch first so the iteration
            // is stable across `Children` representation changes,
            // then drain in reverse onto the DFS stack.
            worklist.children_scratch.clear();
            worklist.children_scratch.extend(children.iter());
            // Reverse so DFS visits the first child first.
            for &child in worklist.children_scratch.iter().rev() {
                worklist.stack.push((child, world_local));
            }
        }
    }
}

/// Convenience wrapper around [`propagate_transforms`] for the common
/// "just mirror Transform → GlobalTransform with SimRef" case. The
/// caller that needs a sprite/audio/debug spawn on top should use the
/// generic form with a closure.
pub fn propagate_transforms_into_present(
    sim_w: &World,
    state: &mut TransformPropagationState,
    present_w: &mut World,
    worklist: &mut WorklistBuf,
) {
    propagate_transforms(sim_w, state, present_w, worklist, |_sim, present, e, gt| {
        present.spawn((SimRef(e), gt));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_neutral() {
        let t = Transform::IDENTITY;
        let got = Transform::compose(t, t);
        assert_eq!(got, t);
    }

    #[test]
    fn translation_composes_additively_with_identity_rotation() {
        let parent = Transform::from_translation(Vec2::new(10.0, 5.0));
        let child = Transform::from_translation(Vec2::new(2.0, 3.0));
        let got = Transform::compose(parent, child);
        assert_eq!(got.translation, Vec2::new(12.0, 8.0));
        assert_eq!(got.rotation, 0.0);
        assert_eq!(got.scale, Vec2::new(1.0, 1.0));
    }

    #[test]
    fn scale_multiplies_through_child_translation() {
        let parent = Transform {
            translation: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(2.0, 3.0),
        };
        let child = Transform::from_translation(Vec2::new(1.0, 1.0));
        let got = Transform::compose(parent, child);
        // child.translation (1,1) is scaled by parent.scale (2,3).
        assert_eq!(got.translation, Vec2::new(2.0, 3.0));
        assert_eq!(got.scale, Vec2::new(2.0, 3.0));
    }

    #[test]
    fn rotation_rotates_child_translation() {
        let parent = Transform {
            translation: Vec2::new(0.0, 0.0),
            rotation: std::f32::consts::FRAC_PI_2,
            scale: Vec2::new(1.0, 1.0),
        };
        let child = Transform::from_translation(Vec2::new(1.0, 0.0));
        let got = Transform::compose(parent, child);
        // 90° CCW: (1,0) → (0,1)
        assert!((got.translation.x).abs() < 1e-6);
        assert!((got.translation.y - 1.0).abs() < 1e-6);
        assert_eq!(got.rotation, std::f32::consts::FRAC_PI_2);
    }

    #[test]
    fn global_transform_translation_matches_local() {
        let t = Transform {
            translation: Vec2::new(7.0, -3.0),
            rotation: 1.5,
            scale: Vec2::new(2.0, 2.0),
        };
        let gt = GlobalTransform::from_transform(t);
        assert_eq!(gt.translation(), t.translation);
    }

    #[test]
    fn worklist_buf_clear_preserves_capacity() {
        let mut buf = WorklistBuf::with_capacity(64);
        for i in 0..32 {
            buf.stack
                .push((Entity::from_raw_u32(i).unwrap(), Transform::IDENTITY));
        }
        let cap_before = buf.stack_capacity();
        buf.clear();
        assert_eq!(buf.stack_len(), 0);
        assert_eq!(buf.stack_capacity(), cap_before);
    }
}
