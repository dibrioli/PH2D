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
