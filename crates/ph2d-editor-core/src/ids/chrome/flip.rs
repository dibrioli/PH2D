//! Flip module chrome NodeIds (ADR-0114 W2 — docked `ph2d-panel-flip`).
//!
//! The `flip` tool's Brush / Color / Layers controls live in a right-docked
//! `Panel<State>` (the tool `FloatingPanel` is unpainted, mirror of the Vector
//! Style panel). Fixed chrome ids below (`FLIP_*`); the per-layer row widgets
//! use a runtime-hashed id family ([`flip_layer_widget_id`], mirror of the
//! Painter layers panel) since the layer count is only known at runtime.
use super::{NodeId, hash_node_id};

// ── Flip Style panel (docked `ph2d-panel-flip`) ──────────────────────────────
/// Flip panel outer rect id (for `z_order` + hit-barrier).
pub const FLIP_PANEL: NodeId = hash_node_id("flip.panel");
/// Flip panel close (X) button.
pub const FLIP_CLOSE: NodeId = hash_node_id("flip.close");

// ── Canvas mode (Select / Draw / Erase — ADR-0112 arbitration) ───────────────
/// Select: the sprite gizmo moves the object (no drawing).
pub const FLIP_MODE_SELECT: NodeId = hash_node_id("flip.mode.select");
/// Draw: each canvas drag creates a stroke on the active drawing.
pub const FLIP_MODE_DRAW: NodeId = hash_node_id("flip.mode.draw");
/// Erase: removes coverage / strokes (see the Erase sub-mode row).
pub const FLIP_MODE_ERASE: NodeId = hash_node_id("flip.mode.erase");

// ── Brush section (size / hardness / opacity / smoothing) ────────────────────
/// Stroke width slider (track `0..1` → `1..64` px) + its px chip.
pub const FLIP_SIZE: NodeId = hash_node_id("flip.size");
pub const FLIP_SIZE_NUM: NodeId = hash_node_id("flip.size_num");
/// Edge hardness slider (`0..1`) + its chip.
pub const FLIP_HARDNESS: NodeId = hash_node_id("flip.hardness");
pub const FLIP_HARDNESS_NUM: NodeId = hash_node_id("flip.hardness_num");
/// Stroke opacity slider (`0..1` → `0..100 %`) + its chip.
pub const FLIP_OPACITY: NodeId = hash_node_id("flip.opacity");
pub const FLIP_OPACITY_NUM: NodeId = hash_node_id("flip.opacity_num");
/// Active-smoothing slider (`0..1`, the "settle") + its chip.
pub const FLIP_SMOOTHING: NodeId = hash_node_id("flip.smoothing");
pub const FLIP_SMOOTHING_NUM: NodeId = hash_node_id("flip.smoothing_num");

// ── Color section ────────────────────────────────────────────────────────────
/// Stroke-colour swatch — a picker swatch (opens the shared OKLCH picker on
/// Down); the shell `flip_bridge` reads the pick back into the tool.
pub const FLIP_STROKE_SWATCH: NodeId = hash_node_id("flip.stroke_swatch");

// ── Erase sub-mode (shown only in Erase mode) ────────────────────────────────
/// Soft (reduce opacity — default, most paint-like), Hard (cut), Stroke (erase
/// whole touched stroke). Mirror of GP `erase.cc`.
pub const FLIP_ERASE_SOFT: NodeId = hash_node_id("flip.erase.soft");
pub const FLIP_ERASE_HARD: NodeId = hash_node_id("flip.erase.hard");
pub const FLIP_ERASE_STROKE: NodeId = hash_node_id("flip.erase.stroke");

// ── Layers toolbar (fixed) ───────────────────────────────────────────────────
/// Add a new layer on top (of the active object).
pub const FLIP_LAYER_ADD: NodeId = hash_node_id("flip.layer.add");
/// Delete the active layer.
pub const FLIP_LAYER_DELETE: NodeId = hash_node_id("flip.layer.delete");

/// Runtime FNV-1a 64-bit over `s`, byte-identical to the `const fn`
/// [`hash_node_id`] (which only accepts `&'static str`). Needed because the
/// per-layer row ids are derived from a runtime `format!` (the `LayerId` is only
/// known at runtime). Kept flip-local (isolation, ADR-0114) — its agreement with
/// `hash_node_id` is pinned by the test at the bottom of this module.
fn flip_fnv_node_id(s: &str) -> NodeId {
    const FNV_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;
    let mut hash: u64 = FNV_OFFSET_BASIS_64;
    for &b in s.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    if hash == 0 {
        hash = 1; // reserve NodeId(0) = a11y root, mirror of hash_node_id
    }
    NodeId(hash)
}

/// Which control on a Flip layers-panel row a runtime id addresses. The layer id
/// is only known at runtime, so per-row widgets hash `(layer_u64, kind)` into a
/// [`NodeId`] via [`flip_layer_widget_id`] (mirror of `PainterLayerWidget`). A
/// new control kind needs no new fixed const — add a variant here.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FlipLayerWidget {
    /// The row body — click selects (activates) the layer.
    Row,
    /// The eye toggle — click flips the layer's visibility.
    Visibility,
    /// The padlock toggle — click flips the layer's lock.
    Lock,
    /// The opacity slider (stores `0..1`).
    Opacity,
    /// The blend-mode dropdown chip (opens the blend popover).
    Blend,
    /// The move-up (↑) reorder button — moves the layer toward the top.
    MoveUp,
    /// The move-down (↓) reorder button — moves the layer toward the back.
    MoveDown,
}

impl FlipLayerWidget {
    /// Stable tag woven into the hashed id string. Changing a tag changes every
    /// derived id for that kind — keep stable.
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Visibility => "vis",
            Self::Lock => "lock",
            Self::Opacity => "opacity",
            Self::Blend => "blend",
            Self::MoveUp => "move_up",
            Self::MoveDown => "move_down",
        }
    }

    /// All kinds, in a fixed order — the decoder iterates this.
    pub const ALL: [FlipLayerWidget; 7] = [
        Self::Row,
        Self::Visibility,
        Self::Lock,
        Self::Opacity,
        Self::Blend,
        Self::MoveUp,
        Self::MoveDown,
    ];
}

/// Derive the stable [`NodeId`] for the `kind` control on the Flip layers-panel
/// row whose layer has runtime id `layer_id`. FNV-hashed from
/// `"flip_layer.<kind>.<layer_id>"`. Runtime `format!` is acceptable here: the
/// layers panel is not a hot path (≤ a handful of layers, repainted per frame
/// like the sidebar formats "NN px"). See [`FlipLayerWidget`].
#[must_use]
pub fn flip_layer_widget_id(layer_id: u64, kind: FlipLayerWidget) -> NodeId {
    flip_fnv_node_id(&format!("flip_layer.{}.{}", kind.tag(), layer_id))
}

/// Derive the stable [`NodeId`] for blend-mode option `mode` (the `BlendMode`
/// wire discriminant, `0..MAX_BLEND_MODES`) in the open blend dropdown popover
/// of the row whose layer has runtime id `layer_id`. Only the single open
/// popover's options are ever hit-registered, so the `format!` cost is bounded.
#[must_use]
pub fn flip_layer_blend_option_id(layer_id: u64, mode: u8) -> NodeId {
    flip_fnv_node_id(&format!("flip_layer.blendopt.{layer_id}.{mode}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_hasher_matches_const_hash_node_id() {
        // The runtime twin must agree with the const hasher for a static string,
        // so runtime-derived ids live in the same space as the fixed consts.
        assert_eq!(flip_fnv_node_id("flip.panel"), FLIP_PANEL);
        assert_eq!(flip_fnv_node_id("flip.mode.draw"), FLIP_MODE_DRAW);
    }

    #[test]
    fn per_layer_ids_are_distinct_by_layer_and_kind() {
        let a = flip_layer_widget_id(0, FlipLayerWidget::Visibility);
        let b = flip_layer_widget_id(1, FlipLayerWidget::Visibility);
        let c = flip_layer_widget_id(0, FlipLayerWidget::Lock);
        assert_ne!(a, b, "different layer -> different id");
        assert_ne!(a, c, "different kind -> different id");
    }
}
