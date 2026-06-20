//! Background-Removal panel chrome NodeIds (BGR_*).
use super::{NodeId, hash_node_id};

/// Background-Removal panel container — the typed `ph2d-panel-bgremoval`
/// outer rect. Right-docked (same geometry slot as the Inspector) and
/// only visible while the `bgremoval` tool is active.
pub const BGR_PANEL: NodeId = hash_node_id("bgr_panel");
/// Mode segmented control — "Chroma" half.
pub const BGR_MODE_CHROMA: NodeId = hash_node_id("bgr_mode_chroma");
/// Mode segmented control — "Smart Cut" half.
pub const BGR_MODE_GRABCUT: NodeId = hash_node_id("bgr_mode_grabcut");
/// Tolerance slider (0..1 → ΔE 0..0.30 Oklab).
pub const BGR_TOLERANCE: NodeId = hash_node_id("bgr_tolerance");
/// Feather slider (0..1 → soft-band 0..0.20 Oklab).
pub const BGR_FEATHER: NodeId = hash_node_id("bgr_feather");
/// Refine slider (0..1 → guided-filter radius 0..100 px).
pub const BGR_REFINE: NodeId = hash_node_id("bgr_refine");
/// Grow/Shrink slider (bipolar; 0.5 = neutral, <0.5 erodes the matte to
/// eat residual background outline, >0.5 dilates it).
pub const BGR_GROW: NodeId = hash_node_id("bgr_grow");
/// Editable numeric chips (NumberInput) paired with the sliders above —
/// keyboard + drag-scrub edit the normalized 0..1 value.
pub const BGR_TOLERANCE_NUM: NodeId = hash_node_id("bgr_tolerance_num");
pub const BGR_FEATHER_NUM: NodeId = hash_node_id("bgr_feather_num");
pub const BGR_REFINE_NUM: NodeId = hash_node_id("bgr_refine_num");
pub const BGR_GROW_NUM: NodeId = hash_node_id("bgr_grow_num");
/// Apply button — commits the removal at full resolution.
pub const BGR_APPLY: NodeId = hash_node_id("bgr_apply");
/// Reset-all button — returns every param to its default in one click.
/// The tool also runs this from `on_activate` so reopening the panel
/// never inherits a previous session's slider/toggle state.
pub const BGR_RESET: NodeId = hash_node_id("bgr_reset");
/// Cancel button — abandons the preview and deactivates the tool
/// (returns to the Inspector).
pub const BGR_CANCEL: NodeId = hash_node_id("bgr_cancel");
/// Eyedropper toggle — when armed, click-drag over the sprite on the
/// canvas samples extra background colours into the swatch row below
/// the sliders. Right-click a swatch to delete it.
pub const BGR_EYEDROPPER: NodeId = hash_node_id("bgr_eyedropper");
/// Protection-brush toggle — when armed, click-drag over the sprite on
/// the canvas paints a freehand "keep" mask: every painted pixel is
/// forced foreground (never removed) in BOTH modes (Chroma force-keep /
/// Smart Cut `FgHard` trimap lock).
pub const BGR_PROTECT: NodeId = hash_node_id("bgr_protect");
/// Clear-protection button — wipes the painted protection mask.
pub const BGR_PROTECT_CLEAR: NodeId = hash_node_id("bgr_protect_clear");
/// "Add area" toggle — when armed, a single click on the canvas runs a
/// flood-fill from the clicked source pixel, expanding to every
/// 4-connected neighbour whose RGB is within a threshold of the seed,
/// and marks the connected region in the FORCE-REMOVE mask: those
/// pixels are forced to alpha=0 in the final compose, overriding both
/// the silhouette auto-protect AND the user's protect-brush mask.
/// Shown in the eyedropper row's slot ONLY when `auto_protect_subject`
/// is on (Pick Colors doesn't apply to the silhouette path, so the slot
/// is repurposed for this automatic destructive selector — Enio
/// 2026-05-26). Symmetric to the eyedropper: arm → single click → done.
pub const BGR_ADD_AREA: NodeId = hash_node_id("bgr_add_area");
/// Clear button for the force-remove mask — mirror of `BGR_PROTECT_CLEAR`.
pub const BGR_ADD_AREA_CLEAR: NodeId = hash_node_id("bgr_add_area_clear");
/// Show-mask toggle — shows/hides the on-canvas protection-mask overlay
/// tint (so the user can preview the clean result without the tint, or
/// turn it back on to keep painting).
pub const BGR_SHOW_MASK: NodeId = hash_node_id("bgr_show_mask");
/// Protection-brush size slider (0..1 → brush radius in source px) +
/// its editable numeric chip. Drives the canvas brush-size gizmo ring.
pub const BGR_BRUSH_SIZE: NodeId = hash_node_id("bgr_brush_size");
pub const BGR_BRUSH_SIZE_NUM: NodeId = hash_node_id("bgr_brush_size_num");
/// Protection-brush falloff profile — 4-option segmented control
/// (mirrors the Mode segmented group). Shapes the painted dab's
/// strength from centre (255) to edge (0): Constant = hard disc,
/// Smooth = smoothstep, Sphere = sqrt(1−d²), Sharp = concentrated peak.
pub const BGR_FALLOFF_SMOOTH: NodeId = hash_node_id("bgr_falloff_smooth");
pub const BGR_FALLOFF_SPHERE: NodeId = hash_node_id("bgr_falloff_sphere");
pub const BGR_FALLOFF_SHARP: NodeId = hash_node_id("bgr_falloff_sharp");
pub const BGR_FALLOFF_CONSTANT: NodeId = hash_node_id("bgr_falloff_constant");
/// Extra-colour swatch hit slots 0..11. Painted only when the
/// corresponding extra colour exists (a fixed pool, like the Blender
/// palette's `BLENDER_SWATCH_*`). Capacity matches
/// `ph2d_tool_bgremoval::params::MAX_EXTRA_BG_COLORS`. Right-clicking a
/// painted slot removes that colour.
pub const BGR_SWATCH_0: NodeId = hash_node_id("bgr_swatch_0");
pub const BGR_SWATCH_1: NodeId = hash_node_id("bgr_swatch_1");
pub const BGR_SWATCH_2: NodeId = hash_node_id("bgr_swatch_2");
pub const BGR_SWATCH_3: NodeId = hash_node_id("bgr_swatch_3");
pub const BGR_SWATCH_4: NodeId = hash_node_id("bgr_swatch_4");
pub const BGR_SWATCH_5: NodeId = hash_node_id("bgr_swatch_5");
pub const BGR_SWATCH_6: NodeId = hash_node_id("bgr_swatch_6");
pub const BGR_SWATCH_7: NodeId = hash_node_id("bgr_swatch_7");
pub const BGR_SWATCH_8: NodeId = hash_node_id("bgr_swatch_8");
pub const BGR_SWATCH_9: NodeId = hash_node_id("bgr_swatch_9");
pub const BGR_SWATCH_10: NodeId = hash_node_id("bgr_swatch_10");
pub const BGR_SWATCH_11: NodeId = hash_node_id("bgr_swatch_11");

/// Fixed-pool extra-colour swatch ids, indexed 0..11.
pub const BGR_SWATCHES: [NodeId; 12] = [
    BGR_SWATCH_0,
    BGR_SWATCH_1,
    BGR_SWATCH_2,
    BGR_SWATCH_3,
    BGR_SWATCH_4,
    BGR_SWATCH_5,
    BGR_SWATCH_6,
    BGR_SWATCH_7,
    BGR_SWATCH_8,
    BGR_SWATCH_9,
    BGR_SWATCH_10,
    BGR_SWATCH_11,
];

/// Recover the extra-colour swatch index `0..12` from a `NodeId` when
/// it matches one of the [`BGR_SWATCHES`] pool consts. Used by the
/// shell's right-click-delete dispatch to map a hit id → list index.
pub fn bgr_swatch_index(id: NodeId) -> Option<usize> {
    BGR_SWATCHES.iter().position(|&s| s == id)
}
