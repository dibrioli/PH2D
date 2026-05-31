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
    /// Skew X in radians — shear that mixes the local Y axis into X
    /// (`tan(skew_x)`). Default `0.0` (no skew). Typical authoring
    /// range `[-π/4, +π/4]`; setters clamp to [`Transform::SKEW_LIMIT`]
    /// so `tan()` never blows toward ±∞ (ADR-0025-amendment-1 §2.5).
    ///
    /// `#[serde(default)]` is documentary defense-in-depth only —
    /// under postcard (positional, non-self-describing) it is *dead*
    /// on trailing-missing payloads; the load-bearing back-compat path
    /// is the [`crate::transform_versioned::TransformVersioned`]
    /// wrapper enum + `migrate_v1_to_v2` (ADR-0025-amendment-1 §2.3,
    /// mirrors ADR-0070-amendment-2 for `Sprite`).
    #[serde(default)]
    pub skew_x: f32,
    /// Skew Y in radians — shear that mixes the local X axis into Y
    /// (`tan(skew_y)`). Default `0.0`. See [`Transform::skew_x`].
    #[serde(default)]
    pub skew_y: f32,
}

impl Transform {
    /// Schema version. Bumped (alongside a migration function) when
    /// the on-disk layout of `Transform` changes. v1→v2 added
    /// `skew_x`/`skew_y` (ADR-0025-amendment-1).
    pub const VERSION: u32 = 2;

    /// Clamp bound for `skew_x`/`skew_y`. `tan(θ)` diverges to ±∞ as
    /// θ → ±π/2, which would poison the affine matrix; we cap at
    /// `π/2 − ε` (ε = 0.01 rad ≈ 0.57°) per ADR-0025-amendment-1 §2.5
    /// / §6. Setters route through [`Transform::clamp_skew`].
    pub const SKEW_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

    /// Identity transform: zero translation, zero rotation, unit
    /// scale, zero skew. `const` so it can be used in const
    /// initializers.
    pub const IDENTITY: Self = Self {
        translation: Vec2::new(0.0, 0.0),
        rotation: 0.0,
        scale: Vec2::new(1.0, 1.0),
        skew_x: 0.0,
        skew_y: 0.0,
    };

    /// Pure translation, identity rotation/scale/skew. Most common
    /// spawn helper — equivalent to Unity's `Vector3.position` set.
    pub const fn from_translation(t: Vec2) -> Self {
        Self {
            translation: t,
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            skew_x: 0.0,
            skew_y: 0.0,
        }
    }

    /// Clamp a skew value to `[−SKEW_LIMIT, +SKEW_LIMIT]` so `tan()`
    /// stays finite (ADR-0025-amendment-1 §2.5). Inspector / authoring
    /// setters MUST route skew writes through this.
    #[inline]
    pub fn clamp_skew(v: f32) -> f32 {
        v.clamp(-Self::SKEW_LIMIT, Self::SKEW_LIMIT)
    }

    /// Builder: set both skew axes (clamped) on a copy of `self`.
    /// Authoring entry point for the Inspector Skew X/Y sliders.
    #[inline]
    #[must_use]
    pub fn with_skew(mut self, skew_x: f32, skew_y: f32) -> Self {
        self.skew_x = Self::clamp_skew(skew_x);
        self.skew_y = Self::clamp_skew(skew_y);
        self
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
    /// Determinism (HR-5): no FMA, no SIMD reordering; `mul` / `add`
    /// on `f32` are IEEE 754 bit-deterministic. `sin`/`cos`/`tan` are
    /// NOT — Rust std `f32::sin_cos`/`f32::tan` are implementation-
    /// defined and route to platform-specific math libraries (target-
    /// triple dependent on every supported OS), all of which differ
    /// from each other in the last 1-2 ulps for non-trivial inputs.
    /// We therefore route through `libm::sincosf` / `libm::tanf`
    /// (pure-Rust, `default-features = false` so the `arch` feature is
    /// OFF) for ONE implementation across all targets — required by
    /// the `transform_determinism` + `transform_versioned_postcard`
    /// gates' pinned blake3 hashes (T1.3.5 + ADR-0025-amendment-1 §2.4).
    ///
    /// # Skew (T·R·Sk·S ordering, ADR-0025-amendment-1 §2.2)
    ///
    /// `child` translation is mapped into `parent` space as
    /// `parent.R · parent.Sk · parent.S · child.t`. `Sk` is the shear
    /// `[[1, tan(skew_x)], [tan(skew_y), 1]]`. The skew terms are
    /// written as **additive corrections** to the v1 R·S terms so the
    /// `skew == 0` case (`tan(0) == 0.0`) degenerates *bit-identically*
    /// to the v1 math — this is what lets `EXPECTED_GLOBALS_HASH` in
    /// `tests/transform_determinism.rs` survive the v2 bump unchanged
    /// (every existing scene has `skew == 0`).
    ///
    /// `skew_x`/`skew_y` compose **additively** in the output —
    /// `parent.skew + child.skew`. This is a documented APPROXIMATION
    /// (§2.2.1): the exact matrix product of two T·R·Sk·S transforms
    /// does not decompose into orthogonal (skew_x, skew_y) when
    /// rotation/scale are non-trivial. The convention is *skew on leaf
    /// entities, rotation/scale on ancestors* (artists apply skew to
    /// the final image, not the rig), which keeps 99% of real cases
    /// exact. A `MatrixTransform` Component (amendment-2) is the escape
    /// hatch if exact cascade skew is ever needed.
    ///
    /// `debug_assert!(rotation.is_finite())` rejects NaN/Inf inputs
    /// in debug builds — a single corrupted rotation poisons the
    /// entire subtree's `GlobalTransform` via propagation, and
    /// signaling-vs-quiet NaN bit patterns can drift cross-host
    /// (R1 Lens C C-H2). Release builds let the NaN propagate
    /// (graceful degradation; the caller is the bug).
    #[inline]
    pub fn compose(parent: Self, child: Self) -> Self {
        debug_assert!(
            parent.rotation.is_finite() && parent.skew_x.is_finite() && parent.skew_y.is_finite(),
            "Transform::compose: a parent angle is non-finite (rotation={}, skew_x={}, \
             skew_y={}); `libm::tanf(NaN) = NaN` and `base + NaN = NaN` poisons the entire \
             subtree's GlobalTransform and breaks cross-host bit-identical (signaling-vs-quiet \
             NaN drift). The caller fed a corrupted angle — fix at source, not here.",
            parent.rotation,
            parent.skew_x,
            parent.skew_y
        );
        let (sin, cos) = libm::sincosf(parent.rotation);
        let tan_sx = libm::tanf(parent.skew_x);
        let tan_sy = libm::tanf(parent.skew_y);
        // Scale child translation by parent scale (v1 base).
        let s_tx = child.translation.x * parent.scale.x;
        let s_ty = child.translation.y * parent.scale.y;
        // Apply parent skew shear. Additive corrections so `tan == 0`
        // leaves the v1 values bit-identical: `s_ty * 0.0 == 0.0` and
        // `base + 0.0 == base`.
        let sk_tx = s_tx + s_ty * tan_sx; // skew_x mixes Y into X
        let sk_ty = s_ty + s_tx * tan_sy; // skew_y mixes X into Y
        // Rotate (v1 formula, now over the sheared vector).
        let rx = sk_tx * cos - sk_ty * sin;
        let ry = sk_tx * sin + sk_ty * cos;
        Self {
            translation: Vec2::new(parent.translation.x + rx, parent.translation.y + ry),
            rotation: parent.rotation + child.rotation,
            scale: Vec2::new(
                parent.scale.x * child.scale.x,
                parent.scale.y * child.scale.y,
            ),
            // Additive skew cascade (documented approximation §2.2.1).
            skew_x: parent.skew_x + child.skew_x,
            skew_y: parent.skew_y + child.skew_y,
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
    /// local-space [`Transform`]. `sin`/`cos`/`tan` route through
    /// `libm::sincosf` / `libm::tanf` for cross-OS bit-identical output
    /// (see `Transform::compose` rationale + T1.3.5 carry-over).
    ///
    /// The linear part is `R · Sk · S` (T·R·Sk·S ordering,
    /// ADR-0025-amendment-1 §2.2). The skew terms are **additive
    /// corrections** to the v1 `R · S` columns so a `skew == 0`
    /// transform produces a *bit-identical* matrix to v1 — preserving
    /// `EXPECTED_GLOBALS_HASH` across the v2 bump.
    pub fn from_transform(t: Transform) -> Self {
        debug_assert!(
            t.rotation.is_finite() && t.skew_x.is_finite() && t.skew_y.is_finite(),
            "GlobalTransform::from_transform: an angle is non-finite (rotation={}, skew_x={}, \
             skew_y={}); see Transform::compose docstring for HR-5 rationale.",
            t.rotation,
            t.skew_x,
            t.skew_y
        );
        let (sin, cos) = libm::sincosf(t.rotation);
        let tan_sx = libm::tanf(t.skew_x);
        let tan_sy = libm::tanf(t.skew_y);
        let sx = t.scale.x;
        let sy = t.scale.y;
        // v1 base columns (R · S) — kept byte-exact.
        let x_axis_x = cos * sx;
        let x_axis_y = sin * sx;
        let y_axis_x = -sin * sy;
        let y_axis_y = cos * sy;
        // Skew corrections (R · Sk · S minus R · S). Each term carries
        // a `tan_*` factor, so it is `0.0` when the axis is skew-free
        // and the `base ± 0.0` degenerates bit-identically.
        let matrix = Mat3::from_cols(
            Vec3::new(
                x_axis_x - sin * tan_sy * sx,
                x_axis_y + cos * tan_sy * sx,
                0.0,
            ),
            Vec3::new(
                y_axis_x + cos * tan_sx * sy,
                y_axis_y + sin * tan_sx * sy,
                0.0,
            ),
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
/// Root draw-order key: the explicit `RootOrder.0`, or `u32::MAX` when
/// absent (so un-ordered roots fall after explicit ones, matching
/// `build_hierarchy_snapshot`).
fn root_order_key(world: &World, e: Entity) -> u32 {
    world.get::<crate::RootOrder>(e).map_or(u32::MAX, |r| r.0)
}

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

    // Order the root seeds so the canvas DRAW order follows the
    // Hierarchy list (Godot 2D: top of the list draws behind, bottom on
    // top). The Hierarchy panel (`build_hierarchy_snapshot`) sorts roots
    // ASCENDING by `(RootOrder, entity_id)`; this DFS pops the stack
    // LIFO, so we sort DESCENDING by the same key — the lowest-order
    // root ends up on top of the stack, is popped + visited FIRST, and
    // gets the smallest `draw_order` (furthest back). Reordering a root
    // in the Hierarchy rewrites its `RootOrder`, so the stacking follows.
    // (Absent `RootOrder` = `u32::MAX`, matching the snapshot, so
    // un-ordered roots sort after explicit ones by entity id — stable
    // across `bevy_ecs` archetype reordering.)
    worklist.stack.sort_unstable_by(|&(a, _), &(b, _)| {
        let ka = (root_order_key(sim_w, a), a.to_bits());
        let kb = (root_order_key(sim_w, b), b.to_bits());
        kb.cmp(&ka)
    });

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

    /// ADR-0025-amendment-1 §2.5 frozen caps: v2 schema is 5 fields,
    /// 28 bytes (translation 8 + rotation 4 + scale 8 + skew_x 4 +
    /// skew_y 4, 4-align), VERSION = 2. A size drift here means a field
    /// was added/reordered without an amendment-2 + cooker bump.
    #[test]
    fn transform_v2_caps_frozen() {
        assert_eq!(
            std::mem::size_of::<Transform>(),
            28,
            "Transform must be 28 bytes (5 fields); changing the layout requires \
             ADR-0025-amendment-2 + a Transform::VERSION bump + cooker migration"
        );
        assert_eq!(Transform::VERSION, 2);
    }

    #[test]
    fn skew_clamp_bounds_tan_finite() {
        // Over-range skew is clamped so tan() never reaches ±∞.
        let t = Transform::IDENTITY.with_skew(10.0, -10.0);
        assert_eq!(t.skew_x, Transform::SKEW_LIMIT);
        assert_eq!(t.skew_y, -Transform::SKEW_LIMIT);
        assert!(libm::tanf(t.skew_x).is_finite());
        assert!(libm::tanf(t.skew_y).is_finite());
        // In-range values pass through untouched.
        let u = Transform::IDENTITY.with_skew(0.2, -0.3);
        assert_eq!(u.skew_x, 0.2);
        assert_eq!(u.skew_y, -0.3);
    }

    #[test]
    fn zero_skew_compose_matches_v1_math_bit_identical() {
        // The whole golden-hash-survives-the-bump guarantee: skew=0
        // compose must produce bit-identical translation to the pre-skew
        // formula. Exercise a non-trivial rotation+scale parent.
        let parent = Transform {
            translation: Vec2::new(3.0, -2.0),
            rotation: 0.9,
            scale: Vec2::new(2.0, 0.5),
            ..Transform::IDENTITY
        };
        let child = Transform::from_translation(Vec2::new(1.5, -0.75));
        let got = Transform::compose(parent, child);
        // Recompute with the explicit v1 formula (no skew terms).
        let (sin, cos) = libm::sincosf(parent.rotation);
        let sx = child.translation.x * parent.scale.x;
        let sy = child.translation.y * parent.scale.y;
        let rx = sx * cos - sy * sin;
        let ry = sx * sin + sy * cos;
        let v1_x = parent.translation.x + rx;
        let v1_y = parent.translation.y + ry;
        assert_eq!(got.translation.x.to_bits(), v1_x.to_bits());
        assert_eq!(got.translation.y.to_bits(), v1_y.to_bits());
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
            ..Transform::IDENTITY
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
            ..Transform::IDENTITY
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
            ..Transform::IDENTITY
        };
        let gt = GlobalTransform::from_transform(t);
        assert_eq!(gt.translation(), t.translation);
    }

    #[test]
    fn root_draw_order_follows_root_order_not_entity_id() {
        // Hierarchy order must drive canvas stacking (Godot-style): the
        // DFS visit order (→ draw_order) follows `RootOrder`, not spawn
        // order. Spawn so entity-id order (a<b<c) disagrees with RootOrder
        // (c=0 < a=1 < b=2); the visit order must be c, a, b.
        let mut world = World::new();
        let a = world.spawn((Transform::IDENTITY, crate::RootOrder(1))).id();
        let b = world.spawn((Transform::IDENTITY, crate::RootOrder(2))).id();
        let c = world.spawn((Transform::IDENTITY, crate::RootOrder(0))).id();
        let mut state = TransformPropagationState::new(&mut world);
        let mut present = World::new();
        let mut buf = WorklistBuf::with_capacity(8);
        let mut order = Vec::new();
        propagate_transforms(&world, &mut state, &mut present, &mut buf, |_s, _p, e, _gt| {
            order.push(e);
        });
        // First-visited = smallest draw_order = furthest back, matching
        // the Hierarchy list (top of list = behind).
        assert_eq!(order, vec![c, a, b]);
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
