//! Brush **Impasto** section NodeIds — the paint's own thickness, and the light that reveals it
//! (`docs/Painter/16_impasto_plano_implementacao.md`). Fixed-id, tool-global widgets forwarding over
//! the frozen `PanelEvent` channel to `PainterTool` setters. Toggles and cyclers forward as
//! `PanelEvent::Click`; the sliders are drag-scrub `NumberInput`s forwarding the real value as
//! `SetValue` (routed via [`PAINTER_IMPASTO_FIELDS`] + `is_param_field`). Split into its own file like
//! `painter_watercolor.rs` / `painter_shape.rs`.
//!
//! The section splits in two, because the two halves belong to different things:
//! **Impasto** is per-BRUSH (how this brush lays paint down) and **Lighting** is per-CANVAS (how the
//! document is lit — one light for the whole painting, like the paper colour or the drying time).

use super::{NodeId, hash_node_id};

/// Collapsible **Impasto** section header (ALL-CAPS label + collapse chevron + assignable colour dot).
pub const PAINTER_IMPASTO_SECTION: NodeId = hash_node_id("painter_brush.impasto_section");
/// The Impasto header's colour dot — a picker swatch (`register_picker_swatch`).
pub const PAINTER_IMPASTO_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.impasto_section_color");
/// Impasto section **reset** icon button. `Click` → `reset_brush_impasto`.
pub const PAINTER_IMPASTO_RESET: NodeId = hash_node_id("painter_brush.impasto_reset");

// ── The brush half: how this brush deposits body ────────────────────────────────────────────────

/// **Enable** master toggle for the whole section. `Click` → `toggle_brush_impasto`. Off (the default)
/// makes a stroke byte-identical to a build with no Impasto at all — the switch is the only gate.
pub const PAINTER_IMPASTO_ENABLE: NodeId = hash_node_id("painter_brush.impasto_enable");
/// **Depth** (`-1..1`; negative CARVES into the paint). `SetValue` → `set_brush_impasto_depth`.
pub const PAINTER_IMPASTO_DEPTH: NodeId = hash_node_id("painter_brush.impasto_depth");
/// **Depth Source** segmented group + its two options — Uniform (a level body: the Grain textures the
/// pigment, not the paint's body) / Grain (the Grain's striations become bristle marks in the relief).
/// `Click` on an option → `set_brush_impasto_source`.
pub const PAINTER_IMPASTO_SOURCE: NodeId = hash_node_id("painter_brush.impasto_source");
pub const PAINTER_IMPASTO_SOURCE_UNIFORM: NodeId =
    hash_node_id("painter_brush.impasto_source_uniform");
pub const PAINTER_IMPASTO_SOURCE_GRAIN: NodeId = hash_node_id("painter_brush.impasto_source_grain");
/// **Draw To** segmented group + its three options — Color + Depth (an ordinary loaded brush) / Color
/// (pigment, no body) / Depth (a palette knife: body, no pigment — the canvas RGBA is left untouched).
/// `Click` on an option → `set_brush_impasto_draw_to`.
pub const PAINTER_IMPASTO_DRAW_TO: NodeId = hash_node_id("painter_brush.impasto_draw_to");
pub const PAINTER_IMPASTO_DRAW_BOTH: NodeId = hash_node_id("painter_brush.impasto_draw_both");
pub const PAINTER_IMPASTO_DRAW_COLOR: NodeId = hash_node_id("painter_brush.impasto_draw_color");
pub const PAINTER_IMPASTO_DRAW_DEPTH: NodeId = hash_node_id("painter_brush.impasto_draw_depth");
/// **Smoothing** (`0..1`) — how far the deposit settles under its own weight at stroke end.
/// `SetValue` → `set_brush_impasto_smoothing`.
pub const PAINTER_IMPASTO_SMOOTHING: NodeId = hash_node_id("painter_brush.impasto_smoothing");

// ── The canvas half: the light (one per document, like the paper colour) ────────────────────────

/// **Show Impasto** — whether the relief is LIT. `Click` → `toggle_impasto_show`. Off ⇒ the light pass
/// does not run and the composite is byte-identical. Canvas-level, not per-brush.
pub const PAINTER_IMPASTO_SHOW: NodeId = hash_node_id("painter_brush.impasto_show");
/// **Light Angle** in whole degrees (`0..360`) — the azimuth the light comes from.
/// `SetValue` → `set_impasto_light_angle`.
pub const PAINTER_IMPASTO_LIGHT_ANGLE: NodeId = hash_node_id("painter_brush.impasto_light_angle");
/// **Elevation** in whole degrees (`5..90`) above the canvas plane. Low = long raking shadows.
/// `SetValue` → `set_impasto_light_elevation`.
pub const PAINTER_IMPASTO_LIGHT_ELEV: NodeId = hash_node_id("painter_brush.impasto_light_elev");
/// **Amount** (`0..1`) — how strongly the relief bends the normal (height-to-slope).
/// `SetValue` → `set_impasto_light_amount`.
pub const PAINTER_IMPASTO_LIGHT_AMOUNT: NodeId = hash_node_id("painter_brush.impasto_light_amount");
/// **Shine** (`0..1`) — the specular highlight riding the crests. `0` = matte, high = wet oil.
/// `SetValue` → `set_impasto_shine`.
pub const PAINTER_IMPASTO_SHINE: NodeId = hash_node_id("painter_brush.impasto_shine");

/// The Impasto **Click** widgets — one membership check for the panel's click forward, routed by
/// `route_brush_impasto_event`.
pub const PAINTER_IMPASTO_CLICKS: [NodeId; 8] = [
    PAINTER_IMPASTO_ENABLE,
    PAINTER_IMPASTO_RESET,
    PAINTER_IMPASTO_SOURCE_UNIFORM,
    PAINTER_IMPASTO_SOURCE_GRAIN,
    PAINTER_IMPASTO_DRAW_BOTH,
    PAINTER_IMPASTO_DRAW_COLOR,
    PAINTER_IMPASTO_DRAW_DEPTH,
    PAINTER_IMPASTO_SHOW,
];

/// The Impasto **SetValue** number-fields — one membership check for the panel's number-field forward
/// (`is_param_field`) and its register loop.
pub const PAINTER_IMPASTO_FIELDS: [NodeId; 6] = [
    PAINTER_IMPASTO_DEPTH,
    PAINTER_IMPASTO_SMOOTHING,
    PAINTER_IMPASTO_LIGHT_ANGLE,
    PAINTER_IMPASTO_LIGHT_ELEV,
    PAINTER_IMPASTO_LIGHT_AMOUNT,
    PAINTER_IMPASTO_SHINE,
];
