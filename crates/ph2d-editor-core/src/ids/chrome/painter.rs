//! Painter sidebar + layers chrome NodeIds and per-row id helpers (PAINTER_SIDEBAR_*,
//! PAINTER_LAYERS_*, PAINTER_CURVE_*, PAINTER_GRADIENT_*, PAINTER_MIXER_*, PAINTER_SELCOLOR_*).
//!
//! ⚠️ **O hash de runtime que estes ids derivam é a PORTA** (`ph2d_tool_registry::hash_node_id_runtime`).
//! Até 2026-09-12 este ficheiro guardava uma CÓPIA à mão da lei FNV-1a (`fnv_node_id_runtime`),
//! partilhada por 21 ficheiros irmãos: era a última das três cópias que a porta veio substituir, e
//! prendia na fundação todo id derivado em runtime (um privado só se vê no módulo que o declara).
use super::{NodeId, hash_node_id};

/// Painter sidebar panel container (`ph2d-panel-painter-sidebar` outer rect; right-docked, painter-only).
pub const PAINTER_SIDEBAR_PANEL: NodeId = hash_node_id("painter_sidebar_panel");
/// Floating color thumb painted in the canvas top-right while the
/// Painter tool is active (W2.T2.3). Clicking it opens the shared
/// `INSP_BLENDER_PICKER` seeded with the Painter's active color; the
/// chosen color is applied back via `PainterUiEdit::SetColorSrgb`.
pub const PAINTER_COLOR_THUMB: NodeId = hash_node_id("painter.color_thumb");

/// Painter layers panel container — the typed `ph2d-panel-painter-layers`
/// outer rect. Right-docked (same geometry slot as the Inspector, mirror
/// do `PAINTER_SIDEBAR_PANEL`) and only visible while the `painter` tool
/// is active. W3.T3.4 plan §6 / design 02_layers.md §2.3.
pub const PAINTER_LAYERS_PANEL: NodeId = hash_node_id("painter_layers_panel");
/// Brush Studio panel (W5) — the brush parameter editor. Shares the right-dock
/// geometry with the sidebar (occupies the same slot when opened).
pub const PAINTER_BRUSH_STUDIO_PANEL: NodeId = hash_node_id("painter_brush_studio_panel");

// ── Brush settings (the layers-panel "Brush" section) ──────────────────────
// Fixed ids (the brush is tool-global, not per-layer), so they live as consts
// alongside the chrome buttons and are pre-registered in `populate`. The tool
// owns the brush state; the panel reads a `BrushSettings` snapshot to position
// these and forwards edits over the frozen `PanelEvent` channel.

// ── Stroke section (the layers-panel "Stroke" sub-section; Blender Stroke panel) ──
// Clean-room port of the Blender 2D-paint "Stroke" controls — all forward over the frozen `PanelEvent`.

/// Brush "Stroke Method" dropdown chip. `SelectOption` → `set_brush_stroke_method`.
pub const PAINTER_BRUSH_STROKE_METHOD: NodeId = hash_node_id("painter_brush.stroke_method");

// ── Brush Falloff curve editor (the always-on shape preview that becomes an
// editable curve when the `Custom` preset is selected — Blender's Falloff
// `CurveMapping` graph). The brush is tool-global, so these are fixed ids. The
// 2-D drag reuses the same `CurvePoint` dispatch as the adjustment Curves
// editor; the panel drains the result and forwards `PAINTER_BRUSH_FALLOFF_EDIT`. ─

// The fixed `PAINTER_GRADIENT_*` routing ids live in the sibling `painter_gradient` module (file-LOC cap).
#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_tool_registry::hash_node_id_runtime;

    /// W3 audit-2 B.2: the runtime hash the per-row painter widget ids are derived with
    /// ([`hash_node_id_runtime`], the door — once a hand-copied twin named `fnv_node_id_runtime`)
    /// must agree with the const [`ph2d_tool_registry::hash_node_id`] (same offset basis / prime /
    /// `NodeId(0)` bump). A divergence would silently move the runtime ids out of the const id
    /// space, misrouting clicks. Pin the agreement + the empty-string FNV-1a-64 offset basis (the
    /// value `hash_node_id("")` also returns; `!= 0`, so it never shadows `NodeId(0)` = a11y root).
    #[test]
    fn fnv_node_id_runtime_agrees_with_hash_node_id() {
        assert_eq!(
            hash_node_id_runtime("").0,
            0xcbf2_9ce4_8422_2325,
            "empty-string hash must equal the FNV-1a-64 offset basis",
        );
        assert_eq!(hash_node_id_runtime("").0, hash_node_id("").0);

        for s in [
            "a",
            "painter_layer.row.0",
            "painter_layer.blend.42",
            "painter_layer.blendopt.7.3",
            "painter_layers_panel",
            "the quick brown fox",
            "x",
        ] {
            assert_eq!(
                hash_node_id_runtime(s).0,
                hash_node_id(s).0,
                "hash_node_id_runtime / hash_node_id diverged on {s:?}",
            );
        }
    }
}
