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
