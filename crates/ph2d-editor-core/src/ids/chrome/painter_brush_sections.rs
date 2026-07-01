//! Brush-panel **collapsible-section** + **slider numeric-chip** ids (split from
//! [`super::painter`], which is at its file-LOC cap). Same `PAINTER_BRUSH_*`
//! prefix + public path (`ph2d_editor_core::ids::<NAME>`) via the chrome re-export.
//!
//! - `*_CHIP` ids back the editable [`crate::widget::NumberInput`] paired with each
//!   brush slider (canonical slider-with-chip; the chip is `link_slider_number`-linked
//!   to the slider so an edit propagates back as the slider's `ValueChanged`).
//! - `PAINTER_BRUSH_RANDOMIZE_SECTION` is the collapsible "Randomize Color" header;
//!   `*_SECTION_COLOR` is its assignable color-dot (a picker swatch, like the
//!   Inspector's section circles).

use super::{NodeId, hash_node_id};

// Numeric-chip ids for the canonical slider-with-chip brush rows.
pub const PAINTER_BRUSH_SIZE_CHIP: NodeId = hash_node_id("painter_brush.size_chip");
pub const PAINTER_BRUSH_STRENGTH_CHIP: NodeId = hash_node_id("painter_brush.strength_chip");
pub const PAINTER_BRUSH_COLOR_JITTER_HUE_CHIP: NodeId =
    hash_node_id("painter_brush.color_jitter_hue_chip");
pub const PAINTER_BRUSH_COLOR_JITTER_SAT_CHIP: NodeId =
    hash_node_id("painter_brush.color_jitter_sat_chip");
pub const PAINTER_BRUSH_COLOR_JITTER_VAL_CHIP: NodeId =
    hash_node_id("painter_brush.color_jitter_val_chip");

/// Chip ids paired 1:1 with [`super::PAINTER_BRUSH_RANDOMIZE_SLIDERS`]'s first three
/// (Hue / Saturation / Value) — the "Randomize Color" section sliders.
pub const PAINTER_BRUSH_RANDOMIZE_CHIPS: [NodeId; 3] = [
    PAINTER_BRUSH_COLOR_JITTER_HUE_CHIP,
    PAINTER_BRUSH_COLOR_JITTER_SAT_CHIP,
    PAINTER_BRUSH_COLOR_JITTER_VAL_CHIP,
];

// Numeric-chip ids for the Stroke + Jitter rows — converted from the bare slider+text-readout form
// to the canonical slider-with-chip (Enio 2026-06-26) so every brush slider matches Size/Strength.
// Each is `link_slider_number`-linked + `set_number_range(0,1,step)`-normalised in `crate::populate`.
pub const PAINTER_BRUSH_RATE_CHIP: NodeId = hash_node_id("painter_brush.rate_chip");
pub const PAINTER_BRUSH_SPACING_CHIP: NodeId = hash_node_id("painter_brush.spacing_chip");
// Perpendicular path **Offset** slider + chip (`0..1` track, `0.5` = none). `SetValue` → `set_brush_offset`.
pub const PAINTER_BRUSH_OFFSET: NodeId = hash_node_id("painter_brush.offset");
pub const PAINTER_BRUSH_OFFSET_CHIP: NodeId = hash_node_id("painter_brush.offset_chip");
/// "Simplify": re-fit the editable curve to a clean minimal control polygon. `Click` → `curve_simplify`.
pub const PAINTER_BRUSH_STROKE_SIMPLIFY: NodeId = hash_node_id("painter_brush.stroke_simplify");
/// "Save As Object" (floppy icon) beside the Stroke Method dropdown — shown only while a curve with points
/// is drawn on the canvas. FUTURE (not wired yet): persist the drawn curve + the current Brush/Layer panel
/// settings as a dynamic object that can be reloaded and even animated later.
pub const PAINTER_BRUSH_STROKE_SAVE_OBJECT: NodeId =
    hash_node_id("painter_brush.stroke_save_object");
/// "Trim" checkbox: cut the offset spine's self-intersections. `Click` → `toggle_offset_trim`.
pub const PAINTER_BRUSH_OFFSET_TRIM: NodeId = hash_node_id("painter_brush.offset_trim");
pub const PAINTER_BRUSH_DASH_RATIO_CHIP: NodeId = hash_node_id("painter_brush.dash_ratio_chip");
pub const PAINTER_BRUSH_DASH_LENGTH_CHIP: NodeId = hash_node_id("painter_brush.dash_length_chip");
pub const PAINTER_BRUSH_INPUT_SAMPLES_CHIP: NodeId =
    hash_node_id("painter_brush.input_samples_chip");
pub const PAINTER_BRUSH_STABILIZE_CHIP: NodeId = hash_node_id("painter_brush.stabilize_chip");
pub const PAINTER_BRUSH_JITTER_CHIP: NodeId = hash_node_id("painter_brush.jitter_chip");
pub const PAINTER_BRUSH_JITTER_SCALE_CHIP: NodeId = hash_node_id("painter_brush.jitter_scale_chip");
pub const PAINTER_BRUSH_JITTER_ROTATE_CHIP: NodeId =
    hash_node_id("painter_brush.jitter_rotate_chip");
pub const PAINTER_BRUSH_JITTER_SPACING_CHIP: NodeId =
    hash_node_id("painter_brush.jitter_spacing_chip");

// ── Composite Brush card (below Strength, above Accumulate) ─────────────────────────────────────
// A checkbox that turns the Brush into a stack of 3 reorderable operation layers (Brush / Smear /
// Blur), each with its own Strength. The stroke runs the layers bottom→top per dab, so each op
// processes the canvas as modified by the one below it. The position numbers (1/2/3) are FIXED; the
// tool sitting at each position moves via the up/down buttons. The per-position slider drives that
// position's layer Strength (routed in the tool's `route_composite_event`).
/// The "Composite Brush" enable checkbox — forwards a plain `Click` → `toggle_composite`.
pub const PAINTER_BRUSH_COMPOSITE_ENABLE: NodeId = hash_node_id("painter_brush.composite_enable");
/// Per-position layer **Strength** sliders (`0..1`, bare — a plain readout, no chip; the Layers-row
/// pattern), position 0 = layer 1 (top) … 2 = layer 3 (bottom).
pub const PAINTER_BRUSH_COMPOSITE_STRENGTH: [NodeId; 3] = [
    hash_node_id("painter_brush.composite_strength_0"),
    hash_node_id("painter_brush.composite_strength_1"),
    hash_node_id("painter_brush.composite_strength_2"),
];
/// Per-position "move layer up" buttons (toward layer 1 / top) — `Click` → `move_composite_layer_up`.
pub const PAINTER_BRUSH_COMPOSITE_UP: [NodeId; 3] = [
    hash_node_id("painter_brush.composite_up_0"),
    hash_node_id("painter_brush.composite_up_1"),
    hash_node_id("painter_brush.composite_up_2"),
];
/// Per-position "move layer down" buttons (toward layer 3 / bottom) — `Click` → `move_composite_layer_down`.
pub const PAINTER_BRUSH_COMPOSITE_DOWN: [NodeId; 3] = [
    hash_node_id("painter_brush.composite_down_0"),
    hash_node_id("painter_brush.composite_down_1"),
    hash_node_id("painter_brush.composite_down_2"),
];
// ── Mask section (collapsible, TOP of the Brush panel in Mask mode) ───────────────────────────────
// A dedicated Mask-editing header hosting three width-adaptive button groups: the mask sub-brush
// (Paint/Erase/Blur/Smear), the whole-canvas ops (Expand/Contract/Blur/Sharpen/Invert/Clear), and the
// on-canvas overlay-tint colour (neutral gray + 4 fluorescent). All are registered in `crate::populate`.
/// Collapsible "Mask" section header (top of the Brush panel in Mask mode). `mark_collapsible_section`.
pub const PAINTER_MASK_SECTION: NodeId = hash_node_id("painter_brush.mask_section");
/// Mask **sub-brush** toggle-group segments — `0` Paint · `1` Erase · `2` Blur · `3` Smear. `Click` →
/// `set_mask_brush(idx)`. A width-adaptive segmented control (reflows when the panel is narrow).
pub const PAINTER_MASK_BRUSH: [NodeId; 4] = [
    hash_node_id("painter_brush.mask_brush_0"),
    hash_node_id("painter_brush.mask_brush_1"),
    hash_node_id("painter_brush.mask_brush_2"),
    hash_node_id("painter_brush.mask_brush_3"),
];
/// Whole-canvas mask-op one-click buttons — `0` Expand · `1` Contract · `2` Blur · `3` Sharpen · `4`
/// Invert · `5` Clear. `Click` → `mask_canvas_op(idx)`. Width-adaptive flow row.
pub const PAINTER_MASK_OP: [NodeId; 6] = [
    hash_node_id("painter_brush.mask_op_0"),
    hash_node_id("painter_brush.mask_op_1"),
    hash_node_id("painter_brush.mask_op_2"),
    hash_node_id("painter_brush.mask_op_3"),
    hash_node_id("painter_brush.mask_op_4"),
    hash_node_id("painter_brush.mask_op_5"),
];
/// Mask overlay-tint colour swatches — `0` neutral gray + `1..=4` fluorescent-marker hues. `Click` →
/// `set_mask_overlay_color(idx)`. Width-adaptive flow row of square swatches.
pub const PAINTER_MASK_COLOR: [NodeId; 5] = [
    hash_node_id("painter_brush.mask_color_0"),
    hash_node_id("painter_brush.mask_color_1"),
    hash_node_id("painter_brush.mask_color_2"),
    hash_node_id("painter_brush.mask_color_3"),
    hash_node_id("painter_brush.mask_color_4"),
];

// ── Clone card (shown in Clone mode) ────────────────────────────────────────────────────────────
/// "Set Source" — arms the on-canvas pick mode; the next canvas Down samples the clone source anchor.
/// `Click` → `arm_clone_sample`.
pub const PAINTER_BRUSH_CLONE_SET_SOURCE: NodeId = hash_node_id("painter_brush.clone_set_source");
/// "Aligned" checkbox — keep the source→dest offset fixed across strokes. `Click` → `toggle_clone_aligned`.
pub const PAINTER_BRUSH_CLONE_ALIGNED: NodeId = hash_node_id("painter_brush.clone_aligned");

/// Every Composite-card **Click** target (enable + the 6 reorder buttons) — one membership check for
/// the panel's brush-click forward whitelist.
pub const PAINTER_BRUSH_COMPOSITE_BUTTONS: [NodeId; 7] = [
    PAINTER_BRUSH_COMPOSITE_ENABLE,
    PAINTER_BRUSH_COMPOSITE_UP[0],
    PAINTER_BRUSH_COMPOSITE_UP[1],
    PAINTER_BRUSH_COMPOSITE_UP[2],
    PAINTER_BRUSH_COMPOSITE_DOWN[0],
    PAINTER_BRUSH_COMPOSITE_DOWN[1],
    PAINTER_BRUSH_COMPOSITE_DOWN[2],
];

/// Collapsible "Randomize Color" section header (Inspector pattern: ALL-CAPS label +
/// collapse chevron + assignable color dot). Click toggles collapse; right-click opens
/// the section-outline color menu.
pub const PAINTER_BRUSH_RANDOMIZE_SECTION: NodeId = hash_node_id("painter_brush.randomize_section");
/// The "Randomize Color" header's color dot — a picker swatch; clicking it opens the
/// shared Blender picker to assign the section's accent color (stored in `widget_color`).
pub const PAINTER_BRUSH_RANDOMIZE_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.randomize_section_color");

// Collapsible-section headers + their assignable color dots (Inspector pattern, same as Randomize
// Color above). Texture + Stroke default expanded; Color Ramp + Tiling default collapsed (Enio
// 2026-06-24). Each `*_SECTION` is `mark_collapsible_section`-registered + the `*_SECTION_COLOR` is a
// `register_picker_swatch` dot, both in `crate::populate`.
pub const PAINTER_BRUSH_TEXTURE_SECTION: NodeId = hash_node_id("painter_brush.texture_section");
pub const PAINTER_BRUSH_TEXTURE_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.texture_section_color");
pub const PAINTER_BRUSH_COLOR_RAMP_SECTION: NodeId =
    hash_node_id("painter_brush.color_ramp_section");
pub const PAINTER_BRUSH_COLOR_RAMP_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.color_ramp_section_color");
pub const PAINTER_BRUSH_STROKE_SECTION: NodeId = hash_node_id("painter_brush.stroke_section");
pub const PAINTER_BRUSH_STROKE_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.stroke_section_color");
pub const PAINTER_BRUSH_TILING_SECTION: NodeId = hash_node_id("painter_brush.tiling_section");
pub const PAINTER_BRUSH_TILING_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.tiling_section_color");

// Per-section **reset** icon buttons (Inspector-Transform pattern: an `IconId::Reset` button in the
// header, just left of the colour dot). Clicking restores that section's brush fields to defaults
// (Enio 2026-06-24). Registered as Buttons in `crate::populate`; the tool resets on the forwarded Click.
pub const PAINTER_BRUSH_RANDOMIZE_RESET: NodeId = hash_node_id("painter_brush.randomize_reset");
pub const PAINTER_BRUSH_TEXTURE_RESET: NodeId = hash_node_id("painter_brush.texture_reset");
pub const PAINTER_BRUSH_COLOR_RAMP_RESET: NodeId = hash_node_id("painter_brush.color_ramp_reset");
pub const PAINTER_BRUSH_STROKE_RESET: NodeId = hash_node_id("painter_brush.stroke_reset");
pub const PAINTER_BRUSH_TILING_RESET: NodeId = hash_node_id("painter_brush.tiling_reset");

/// The 5 per-section reset icon buttons as one array — a single membership check for the panel's
/// Click-forward (and reusable wherever the whole set is needed).
pub const PAINTER_BRUSH_SECTION_RESETS: [NodeId; 5] = [
    PAINTER_BRUSH_RANDOMIZE_RESET,
    PAINTER_BRUSH_TEXTURE_RESET,
    PAINTER_BRUSH_COLOR_RAMP_RESET,
    PAINTER_BRUSH_STROKE_RESET,
    PAINTER_BRUSH_TILING_RESET,
];
