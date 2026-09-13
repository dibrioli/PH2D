//! Flip module chrome NodeIds (ADR-0114 W2 — docked `ph2d-panel-flip`).
//!
//! The `flip` tool's Brush / Color / Layers controls live in a right-docked
//! `Panel<State>` (the tool `FloatingPanel` is unpainted, mirror of the Vector
//! Style panel). Fixed chrome ids below (`FLIP_*`); the per-layer row widgets
//! use a runtime-hashed id family ([`flip_layer_widget_id`], mirror of the
//! Painter layers panel) since the layer count is only known at runtime.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/flip.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// Flip panel close (X) button.
pub const FLIP_CLOSE: NodeId = hash_node_id("flip.close");

pub const FLIP_DOT_SPACING_NUM: NodeId = hash_node_id("flip.tip.spacing.num");

pub const FLIP_PRESSURE_MIN_NUM: NodeId = hash_node_id("flip.pressure.min.num");

pub const FLIP_PRESSURE_RESPONSE_NUM: NodeId = hash_node_id("flip.pressure.response.num");

/// Trace section: puts every shifted ghost back where its drawing is (the shifts are
/// shell session state — display scaffolding, never the document).
pub const FLIP_TRACE_RESET: NodeId = hash_node_id("flip.trace.reset");

/// Edit section: deletes the selected strokes.
pub const FLIP_EDIT_DELETE: NodeId = hash_node_id("flip.edit.delete");

/// Edit section: clears the selection (deselect all).
pub const FLIP_EDIT_DESELECT: NodeId = hash_node_id("flip.edit.deselect");

/// Edit section: selects every stroke of the active drawing.
pub const FLIP_EDIT_SELECT_ALL: NodeId = hash_node_id("flip.edit.select_all");

// ── Fill section (shown only in Fill mode, ADR-0114 W4) ─────────────────────
/// Fill-colour swatch — its OWN colour (colouring uses a different palette than
/// drawing; forcing the stroke colour to double as the fill colour would be hostile).
pub const FLIP_FILL_SWATCH: NodeId = hash_node_id("flip.fill.swatch");

pub const FLIP_GAP_NUM: NodeId = hash_node_id("flip.fill.gap_num");

pub const FLIP_GROW_NUM: NodeId = hash_node_id("flip.fill.grow_num");

pub const FLIP_PRECISION_NUM: NodeId = hash_node_id("flip.fill.precision_num");

pub const FLIP_TRAP_NUM: NodeId = hash_node_id("flip.fill.trap_num");

// ── Colorize section (shown only in Colorize mode, C2 — `docs/Flip/09`) ──────
/// Colorize-colour swatch — the colour the NEXT scribble seeds (its own palette).
pub const FLIP_COLORIZE_SWATCH: NodeId = hash_node_id("flip.colorize.swatch");

pub const FLIP_COLORIZE_BLEED_NUM: NodeId = hash_node_id("flip.colorize.bleed_num");

pub const FLIP_SIZE_NUM: NodeId = hash_node_id("flip.size_num");

pub const FLIP_HARDNESS_NUM: NodeId = hash_node_id("flip.hardness_num");

pub const FLIP_OPACITY_NUM: NodeId = hash_node_id("flip.opacity_num");

pub const FLIP_SMOOTHING_NUM: NodeId = hash_node_id("flip.smoothing_num");

pub const FLIP_ERASE_SIZE_NUM: NodeId = hash_node_id("flip.erase.size_num");

pub const FLIP_ERASE_STRENGTH_NUM: NodeId = hash_node_id("flip.erase.strength_num");

// ── Color section ────────────────────────────────────────────────────────────
/// Stroke-colour swatch — a picker swatch (opens the shared OKLCH picker on
/// Down); the shell `flip_bridge` reads the pick back into the tool.
pub const FLIP_STROKE_SWATCH: NodeId = hash_node_id("flip.stroke_swatch");

/// Duplicate the active layer (an independent copy, above the original — ADR-0114 §4.C).
pub const FLIP_LAYER_DUPLICATE: NodeId = hash_node_id("flip.layer.duplicate");

/// Delete the active layer.
pub const FLIP_LAYER_DELETE: NodeId = hash_node_id("flip.layer.delete");

/// Inline layer-rename field (ADR-0114 §4.C): double-clicking a layer's name opens
/// a single-line `TextInput` over the name strip, seeded with the current name.
/// ONE field, reused across rows (only one rename is open at a time) — mirror of the
/// timeline's `TIMELINE_MARKER_RENAME_INPUT`.
pub const FLIP_LAYER_RENAME_INPUT: NodeId = hash_node_id("flip.layer.rename_input");

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
    /// The opacity value CHIP (`NumberInput`, shows `0..100 %`, linked to
    /// [`Self::Opacity`] via `link_slider_number_mapped_integer`). The canonical
    /// label+slider+chip row, mirror of the brush sliders.
    OpacityNum,
    /// The multiplane depth slider (2.5D, ADR-0114 §Decisão 3): stores the parallax
    /// follow-fraction `0..1` (`1` = flat/front, `0` = far/static background).
    Depth,
    /// The depth value CHIP (`NumberInput`, `0..100 %`, linked to [`Self::Depth`]).
    DepthNum,
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
            Self::OpacityNum => "opacity_num",
            Self::Depth => "depth",
            Self::DepthNum => "depth_num",
            Self::Blend => "blend",
            Self::MoveUp => "move_up",
            Self::MoveDown => "move_down",
        }
    }

    /// All kinds, in a fixed order — the decoder iterates this.
    pub const ALL: [FlipLayerWidget; 10] = [
        Self::Row,
        Self::Visibility,
        Self::Lock,
        Self::Opacity,
        Self::OpacityNum,
        Self::Depth,
        Self::DepthNum,
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
    ph2d_tool_registry::hash_node_id_runtime(&format!("flip_layer.{}.{}", kind.tag(), layer_id))
}

/// Derive the stable [`NodeId`] for blend-mode option `mode` (the `BlendMode`
/// wire discriminant, `0..MAX_BLEND_MODES`) in the open blend dropdown popover
/// of the row whose layer has runtime id `layer_id`. Only the single open
/// popover's options are ever hit-registered, so the `format!` cost is bounded.
#[must_use]
pub fn flip_layer_blend_option_id(layer_id: u64, mode: u8) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("flip_layer.blendopt.{layer_id}.{mode}"))
}

// ── Desceu de `ph2d-editor-core/src/ids/chrome/flip.rs` em 2026-09-13 (2.ª passagem: a cerca com a
//    `line/render-loop` prendia-os na fundação até às duas linhas se integrarem).

/// Apply: run the LazyBrush cut over the accumulated scribbles + the line-art, commit
/// each region as a filled stroke, and clear the scribble buffer.
pub const FLIP_COLORIZE_APPLY: NodeId = hash_node_id("flip.colorize.apply");

/// Clear: drop the accumulated scribbles without colouring.
pub const FLIP_COLORIZE_CLEAR: NodeId = hash_node_id("flip.colorize.clear");

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::ids::FLIP_PANEL;
    use ph2d_tool_flip::ids::FLIP_MODE_DRAW;

    #[test]
    fn runtime_hasher_matches_const_hash_node_id() {
        // The runtime twin must agree with the const hasher for a static string,
        // so runtime-derived ids live in the same space as the fixed consts.
        assert_eq!(
            ph2d_tool_registry::hash_node_id_runtime("flip.panel"),
            FLIP_PANEL
        );
        assert_eq!(
            ph2d_tool_registry::hash_node_id_runtime("flip.mode.draw"),
            FLIP_MODE_DRAW
        );
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
