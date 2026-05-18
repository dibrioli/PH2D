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
}
