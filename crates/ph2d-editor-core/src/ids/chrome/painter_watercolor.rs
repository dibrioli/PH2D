//! Brush **Watercolor** section NodeIds (the wet-media look: edge darkening + granulation + pigment
//! build-up; no fluid sim — see `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). Fixed-id,
//! tool-global widgets forwarding over the frozen `PanelEvent` channel to `PainterTool` setters
//! (`set_brush_edge_gain` / `set_brush_granulation` / `toggle_brush_pigment` / …). The two toggles
//! forward as `PanelEvent::Click`; the four sliders are drag-scrub `NumberInput`s forwarding the real
//! value as `SetValue` (routed via [`super::PAINTER_WATERCOLOR_FIELDS`] +
//! `is_param_field`). Split into its own file like `painter_shape.rs` to keep the ids tidy.

use super::{NodeId, hash_node_id};

/// Collapsible **Watercolor** section header (ALL-CAPS label + collapse chevron + assignable colour
/// dot). `mark_collapsible_section`-registered in `crate::populate`.
pub const PAINTER_WATERCOLOR_SECTION: NodeId = hash_node_id("painter_brush.watercolor_section");
/// The Watercolor header's colour dot — a picker swatch (`register_picker_swatch`).
pub const PAINTER_WATERCOLOR_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.watercolor_section_color");
/// Watercolor section **reset** icon button. `Click` → `reset_brush_watercolor`.
pub const PAINTER_WATERCOLOR_RESET: NodeId = hash_node_id("painter_brush.watercolor_reset");

/// **Wet edges** master enable toggle for the whole section. `Click` → `toggle_brush_watercolor`.
pub const PAINTER_WATERCOLOR_ENABLE: NodeId = hash_node_id("painter_brush.watercolor_enable");
/// **Pigment** (subtractive Kubelka–Munk wet-on-wet mixing) toggle. `Click` → `toggle_brush_pigment`.
pub const PAINTER_WATERCOLOR_PIGMENT: NodeId = hash_node_id("painter_brush.watercolor_pigment");

/// **Edge** darkening gain (`0..8` track). `SetValue` → `set_brush_edge_gain`.
pub const PAINTER_WATERCOLOR_EDGE: NodeId = hash_node_id("painter_brush.watercolor_edge");
/// **Spread** (blur radius, canvas px, `1..24`). `SetValue` → `set_brush_edge_spread`.
pub const PAINTER_WATERCOLOR_SPREAD: NodeId = hash_node_id("painter_brush.watercolor_spread");
/// **Granulation** (`0..1`). `SetValue` → `set_brush_granulation`.
pub const PAINTER_WATERCOLOR_GRANULATION: NodeId =
    hash_node_id("painter_brush.watercolor_granulation");
/// **Mix** — pigment amount (`0..1`). `SetValue` → `set_brush_pigment_mix`.
pub const PAINTER_WATERCOLOR_MIX: NodeId = hash_node_id("painter_brush.watercolor_mix");

/// The Watercolor **Click** widgets (master enable + Pigment toggle + section reset) — forwarded as a
/// `PanelEvent::Click` by the panel's `event.rs` (a single membership check) and routed by
/// `route_brush_watercolor_event`.
pub const PAINTER_WATERCOLOR_CLICKS: [NodeId; 3] = [
    PAINTER_WATERCOLOR_ENABLE,
    PAINTER_WATERCOLOR_PIGMENT,
    PAINTER_WATERCOLOR_RESET,
];

/// The Watercolor **SetValue** number-fields (Edge / Spread / Granulation / Mix) — one membership
/// check for the panel's number-field forward (`is_param_field`) and register loop (`populate.rs`).
pub const PAINTER_WATERCOLOR_FIELDS: [NodeId; 4] = [
    PAINTER_WATERCOLOR_EDGE,
    PAINTER_WATERCOLOR_SPREAD,
    PAINTER_WATERCOLOR_GRANULATION,
    PAINTER_WATERCOLOR_MIX,
];
