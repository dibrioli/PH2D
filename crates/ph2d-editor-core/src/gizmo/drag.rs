//! Gizmo drag state machine: kinds + snapshot + active drag struct.
//!
//! Extracted from monolithic `gizmo.rs` in Wave 6+7 Phase 1.B.

// ───────────── M14.7 C: state machine + math helpers ─────────────

/// Which interaction the user opened by mousing down on a gizmo
/// element. Each variant maps to a specific math path in
/// [`compute_gizmo_transform`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GizmoDragKind {
    /// Mouse Down landed on the bbox interior — drag translates the
    /// sprite by the world-space delta between Down and the latest
    /// cursor position.
    Translate,
    /// Mouse Down landed on a corner scale handle. `dx_sign` /
    /// `dy_sign` encode which corner: +1 means "this corner is on
    /// the positive side of the bbox along that axis". The math
    /// derives the new scale factor from the ratio of (cursor →
    /// pivot) vectors at Down vs now.
    ScaleCorner { dx_sign: f32, dy_sign: f32 },
    /// Edge midpoint handle — single-axis scale. `axis` 0 = X, 1 = Y.
    /// `sign` matches the corresponding `dx_sign` / `dy_sign`
    /// convention (+1 = right/top edge, -1 = left/bottom).
    ScaleEdge { axis: u8, sign: f32 },
    /// Rotation around the bbox pivot. The drag tracks the cursor's
    /// angle relative to the pivot.
    Rotate,
    /// Move the PIVOT itself (the TOOL_PIVOT tool). The sprite's quad
    /// stays world-fixed while the pivot point relocates: the host
    /// writes the new `Transform.translation` (= dragged pivot world
    /// pos) AND a compensating `Sprite.anchor` so nothing visually
    /// jumps. Routed through [`super::move_pivot_transform`], NOT
    /// `compute_gizmo_transform` (which can't return an anchor). For
    /// this kind `pivot_world` holds the INVARIANT quad center captured
    /// at drag start.
    MovePivot,
}

/// World-space snapshot of the selected sprite's Transform captured
/// when the drag began. The math runs deltas off this — apply-each-
/// frame mutations would compound otherwise.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TransformSnapshot {
    pub translation: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
}

impl TransformSnapshot {
    /// Identity snapshot: zero translation/rotation + unit scale.
    /// Used as the default `parent_world` for root entities (no
    /// ancestor compose needed) and as a stable test fixture.
    pub const IDENTITY: Self = Self {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [1.0, 1.0],
    };
}

/// Onda 2C: which gizmo the user clicked. Drives `advance_gizmo_drag`'s
/// branch between primary-only, group-with-global-pivot, and group-
/// with-local-pivots transforms.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GizmoTarget {
    /// The primary's gizmo (canonical IDs) — transforms apply to the
    /// primary and, when multi-select is alive, to extras with
    /// pivots LOCAL TO EACH (matches Enio's spec — "transforms via
    /// individual gizmos use each sprite's own pivot").
    PrimaryIndividual,
    /// One of the extra's gizmos (per-entity hashed IDs). Same
    /// semantic as `PrimaryIndividual` — the drag is rooted on this
    /// sprite's pivot, but the transform delta propagates to every
    /// selected sprite locally (each rotates/scales around its own
    /// pivot, translates by the same world delta).
    ExtraIndividual(u64),
    /// The global gizmo (group_offset-XORed IDs). Transforms use a
    /// SINGLE pivot = global bbox center, applied to every selected
    /// sprite (per Enio — "transforms via global gizmo behave as
    /// if the group is one rigid object").
    Global,
}

/// Onda 2C: a single drag-target lookup entry, populated by the
/// painters and consumed by `on_mouse_input` Down. The shell holds a
/// `BTreeMap<NodeId, GizmoHit>` covering every interactive handle on
/// every gizmo painted this frame.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoHit {
    pub target: GizmoTarget,
    pub kind: GizmoDragKind,
}

/// In-progress gizmo drag. Owned by the host (typically the desktop
/// shell) and lives outside `WidgetStore` so the math can stay in
/// `ph2d-editor` without dragging in `ph2d-render` or `ph2d-ecs`.
///
/// The host's MouseInput handler:
/// 1. Down on a gizmo handle id → snapshot the entity's Transform +
///    cursor position → fill this struct.
/// 2. Move → updates `cursor_screen` + calls
///    [`compute_gizmo_transform`] to derive the new Transform.
/// 3. Up → drops the state.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GizmoDragState {
    pub kind: GizmoDragKind,
    /// Sim-entity bits of the selected sprite (same shape
    /// `HeroScreen::gizmo_selection` stores).
    pub entity_bits: u64,
    /// Cursor position in screen pixels at Mouse Down.
    pub start_screen: (f32, f32),
    /// Latest cursor position — updated on every Move.
    pub cursor_screen: (f32, f32),
    /// Entity's Transform at Mouse Down (math operates off this).
    pub start_transform: TransformSnapshot,
    /// World-space pivot — usually the bbox center at Down. The
    /// scale + rotate math references this; translate ignores it.
    pub pivot_world: [f32; 2],
    /// Cursor's world position at Down. Cached so move events don't
    /// have to redo the camera projection of the start point.
    pub start_cursor_world: [f32; 2],
    /// Sprite's INTRINSIC half-size in local frame (i.e. `Sprite::
    /// size * 0.5`, before `Transform::scale`). Captured at Down so
    /// the Scale math can recompute the opposite-corner local offset
    /// under the new scale and derive a translation that keeps
    /// `pivot_world` fixed. `[0.0, 0.0]` falls back to scaling around
    /// the sprite center (no translation update) — same as the
    /// pre-anchor-fix behavior.
    pub sprite_half_intrinsic: [f32; 2],
    /// True iff the pivot is the sprite center (Ctrl / Cmd held at
    /// Down). When set, the Scale branches keep translation
    /// unchanged — center anchor means the sprite scales in place.
    pub anchor_is_center: bool,
    /// Onda 2C: which gizmo opened this drag. Drives the dispatch in
    /// `advance_gizmo_drag` between single-sprite, multi with global
    /// pivot, and multi with local pivots.
    pub target: GizmoTarget,
    /// World-space [`Transform`] of the entity's PARENT chain, captured
    /// at Down. `IDENTITY` for root entities. Used by
    /// [`compute_gizmo_transform`] to convert world-space cursor
    /// deltas back into the entity's LOCAL frame before writing to the
    /// SimWorld — without this, dragging a child of a rotated parent
    /// moves along the local (rotated) axis instead of the visual axis.
    /// Populated via `ph2d_ecs::parent_world_transform` at the Down
    /// handler.
    pub parent_world: TransformSnapshot,
}
