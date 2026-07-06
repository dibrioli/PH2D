//! Brush **Watercolor** section NodeIds (the wet-media look: edge darkening + granulation + pigment
//! build-up; no fluid sim — see `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). Fixed-id,
//! tool-global widgets forwarding over the frozen `PanelEvent` channel to `PainterTool` setters
//! (`set_brush_edge_gain` / `set_brush_granulation` / `toggle_brush_pigment` / …). The two toggles
//! forward as `PanelEvent::Click`; the four sliders are drag-scrub `NumberInput`s forwarding the real
//! value as `SetValue` (routed via [`super::PAINTER_WATERCOLOR_FIELDS`] +
//! `is_param_field`). Split into its own file like `painter_shape.rs` to keep the ids tidy.

use super::{NodeId, fnv_node_id_runtime, hash_node_id};

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

/// **Fill** — wash interior density (`0..1`, render-path). `SetValue` → `set_brush_fill`.
pub const PAINTER_WATERCOLOR_FILL: NodeId = hash_node_id("painter_brush.watercolor_fill");
/// **Depth** — Beer–Lambert optical-depth scale (render-path). `SetValue` → `set_brush_depth`.
pub const PAINTER_WATERCOLOR_DEPTH: NodeId = hash_node_id("painter_brush.watercolor_depth");
/// **Warp** — organic-boundary displacement in canvas px (render-path). `SetValue` → `set_brush_warp`.
pub const PAINTER_WATERCOLOR_WARP: NodeId = hash_node_id("painter_brush.watercolor_warp");

// ── Paper section (canvas-anchored substrate; its own section above Grain; `docs/Painter/10…` §5) ──
/// Collapsible **Paper** section header (ALL-CAPS + chevron + colour dot).
pub const PAINTER_WATERCOLOR_PAPER_SECTION: NodeId =
    hash_node_id("painter_brush.watercolor_paper_section");
/// The Paper header's colour dot (assignable swatch).
pub const PAINTER_WATERCOLOR_PAPER_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.watercolor_paper_section_color");
/// Paper section **reset** icon button.
pub const PAINTER_WATERCOLOR_PAPER_RESET: NodeId =
    hash_node_id("painter_brush.watercolor_paper_reset");
/// **Paper** slot kind dropdown chip (None / Cold-Rough-Hot Press / other procedural / Image). `SelectOption`
/// → `set_brush_paper_kind`. Options via [`painter_paper_kind_option_id`].
pub const PAINTER_WATERCOLOR_PAPER_KIND: NodeId = hash_node_id("painter_brush.watercolor_paper_kind");
/// **Paper** Size X (`0.1..100`). `SetValue` → `set_brush_paper_size`(0).
pub const PAINTER_WATERCOLOR_PAPER_SIZE_X: NodeId =
    hash_node_id("painter_brush.watercolor_paper_size_x");
/// **Paper** Size Y. `SetValue` → `set_brush_paper_size`(1).
pub const PAINTER_WATERCOLOR_PAPER_SIZE_Y: NodeId =
    hash_node_id("painter_brush.watercolor_paper_size_y");
/// **Paper** Angle (degrees, fibre orientation). `SetValue` → `set_brush_paper_angle`.
pub const PAINTER_WATERCOLOR_PAPER_ANGLE: NodeId =
    hash_node_id("painter_brush.watercolor_paper_angle");
/// **Paper** Mapping dropdown chip. `SelectOption` → `set_brush_paper_mapping`. Options via
/// [`painter_paper_mapping_option_id`].
pub const PAINTER_WATERCOLOR_PAPER_MAPPING: NodeId =
    hash_node_id("painter_brush.watercolor_paper_mapping");
/// **Paper** Rake toggle. `Click` → `toggle_brush_paper_rake`.
pub const PAINTER_WATERCOLOR_PAPER_RAKE: NodeId =
    hash_node_id("painter_brush.watercolor_paper_rake");
/// **Paper** Random-Angle toggle. `Click` → `toggle_brush_paper_random`.
pub const PAINTER_WATERCOLOR_PAPER_RANDOM: NodeId =
    hash_node_id("painter_brush.watercolor_paper_random");
/// **Paper** Offset X. `SetValue` → `set_brush_paper_offset`(0).
pub const PAINTER_WATERCOLOR_PAPER_OFFSET_X: NodeId =
    hash_node_id("painter_brush.watercolor_paper_offset_x");
/// **Paper** Offset Y. `SetValue` → `set_brush_paper_offset`(1).
pub const PAINTER_WATERCOLOR_PAPER_OFFSET_Y: NodeId =
    hash_node_id("painter_brush.watercolor_paper_offset_y");
/// **Paper** Depth (how strongly the paper tooth bites). `SetValue` → `set_brush_paper_depth`.
pub const PAINTER_WATERCOLOR_PAPER_DEPTH: NodeId =
    hash_node_id("painter_brush.watercolor_paper_depth");
/// **Paper** per-pattern params (Contrast / Brightness / kind knobs). `SetValue` → `set_brush_paper_param`.
pub const PAINTER_WATERCOLOR_PAPER_PARAMS: [NodeId; 6] = [
    hash_node_id("painter_brush.watercolor_paper_param0"),
    hash_node_id("painter_brush.watercolor_paper_param1"),
    hash_node_id("painter_brush.watercolor_paper_param2"),
    hash_node_id("painter_brush.watercolor_paper_param3"),
    hash_node_id("painter_brush.watercolor_paper_param4"),
    hash_node_id("painter_brush.watercolor_paper_param5"),
];

/// Derive the stable [`NodeId`] for **Paper** mapping option `m` in the open Paper mapping popover.
#[must_use]
pub fn painter_paper_mapping_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.papermapopt.{m}"))
}

// ── Paper Colors ramp (tints the substrate; mirrors the Shape ramp id set) ──────────────────────────
pub const PAINTER_PAPER_RAMP_SECTION: NodeId = hash_node_id("painter_brush.paper_ramp_section");
pub const PAINTER_PAPER_RAMP_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.paper_ramp_section_color");
pub const PAINTER_PAPER_RAMP_RESET: NodeId = hash_node_id("painter_brush.paper_ramp_reset");
pub const PAINTER_PAPER_RAMP_ENABLE: NodeId = hash_node_id("painter_brush.paper_ramp_enable");
pub const PAINTER_PAPER_RAMP_MODE: NodeId = hash_node_id("painter_brush.paper_ramp_mode");
pub const PAINTER_PAPER_RAMP_INTERP: NodeId = hash_node_id("painter_brush.paper_ramp_interp");
pub const PAINTER_PAPER_RAMP_BW: NodeId = hash_node_id("painter_brush.paper_ramp_bw");
pub const PAINTER_PAPER_RAMP_ALPHA_MODE: NodeId =
    hash_node_id("painter_brush.paper_ramp_alpha_mode");
pub const PAINTER_PAPER_RAMP_EDIT: NodeId = hash_node_id("painter_brush.paper_ramp_edit");
pub const PAINTER_PAPER_RAMP_ADD: NodeId = hash_node_id("painter_brush.paper_ramp_add");
pub const PAINTER_PAPER_RAMP_REMOVE: NodeId = hash_node_id("painter_brush.paper_ramp_remove");
pub const PAINTER_PAPER_RAMP_INVERT: NodeId = hash_node_id("painter_brush.paper_ramp_invert");
pub const PAINTER_PAPER_RAMP_SWATCH: NodeId = hash_node_id("painter_brush.paper_ramp_swatch");
pub const PAINTER_PAPER_RAMP_STOP_INDEX: NodeId =
    hash_node_id("painter_brush.paper_ramp_stop_index");
pub const PAINTER_PAPER_RAMP_STOP_POS: NodeId = hash_node_id("painter_brush.paper_ramp_stop_pos");

/// The Paper ramp **Click** buttons (enable / add / remove / invert / B&W / reset).
pub const PAINTER_PAPER_RAMP_BUTTONS: [NodeId; 6] = [
    PAINTER_PAPER_RAMP_ENABLE,
    PAINTER_PAPER_RAMP_ADD,
    PAINTER_PAPER_RAMP_REMOVE,
    PAINTER_PAPER_RAMP_INVERT,
    PAINTER_PAPER_RAMP_BW,
    PAINTER_PAPER_RAMP_RESET,
];

/// The Paper ramp **value** widgets (edit swatch + selected-stop index/pos).
pub const PAINTER_PAPER_RAMP_VALUE_IDS: [NodeId; 3] = [
    PAINTER_PAPER_RAMP_EDIT,
    PAINTER_PAPER_RAMP_STOP_INDEX,
    PAINTER_PAPER_RAMP_STOP_POS,
];

/// Per-stop drag handle id on the Paper ramp bar.
#[must_use]
pub fn painter_paper_ramp_handle_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.paper_ramp_handle.{i}"))
}
/// Paper ramp colour-Mode dropdown option id.
#[must_use]
pub fn painter_paper_ramp_mode_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.paper_ramp_modeopt.{m}"))
}
/// Paper ramp Interpolation dropdown option id.
#[must_use]
pub fn painter_paper_ramp_interp_option_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.paper_ramp_interpopt.{i}"))
}
/// Paper ramp Alpha-action dropdown option id.
#[must_use]
pub fn painter_paper_ramp_alpha_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.paper_ramp_alphaopt.{m}"))
}

/// **Granulation** "Same as Paper" toggle — shown in the **Grain** section in watercolor mode (the Grain
/// slot IS the granulation map). `Click` → `toggle_granulation_use_paper`.
pub const PAINTER_WATERCOLOR_GRAN_SAME: NodeId = hash_node_id("painter_brush.watercolor_gran_same");

/// Derive the stable [`NodeId`] for **Paper** kind option `k` in the open Paper dropdown popover
/// (its own id namespace so it never collides with the Grain kind picker). Mirror of
/// `painter_brush_texture_kind_option_id`.
#[must_use]
pub fn painter_paper_kind_option_id(k: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.paperkindopt.{k}"))
}

/// The Watercolor **Click** widgets (master enable + Pigment toggle + section reset + Granulation
/// "Same as Paper") — forwarded as a `PanelEvent::Click` by the panel's `event.rs` (a single membership
/// check) and routed by `route_brush_watercolor_event`.
pub const PAINTER_WATERCOLOR_CLICKS: [NodeId; 7] = [
    PAINTER_WATERCOLOR_ENABLE,
    PAINTER_WATERCOLOR_PIGMENT,
    PAINTER_WATERCOLOR_RESET,
    PAINTER_WATERCOLOR_GRAN_SAME,
    PAINTER_WATERCOLOR_PAPER_RESET,
    PAINTER_WATERCOLOR_PAPER_RAKE,
    PAINTER_WATERCOLOR_PAPER_RANDOM,
];

/// The Watercolor **SetValue** number-fields (Edge / Spread / Granulation / Mix + the render-path
/// optics Fill / Depth / Warp + the full Paper slot: Size / Angle / Offset / Depth / params) — one
/// membership check for the panel's number-field forward (`is_param_field`) and register loop.
pub const PAINTER_WATERCOLOR_FIELDS: [NodeId; 19] = [
    PAINTER_WATERCOLOR_EDGE,
    PAINTER_WATERCOLOR_SPREAD,
    PAINTER_WATERCOLOR_GRANULATION,
    PAINTER_WATERCOLOR_MIX,
    PAINTER_WATERCOLOR_FILL,
    PAINTER_WATERCOLOR_DEPTH,
    PAINTER_WATERCOLOR_WARP,
    PAINTER_WATERCOLOR_PAPER_SIZE_X,
    PAINTER_WATERCOLOR_PAPER_SIZE_Y,
    PAINTER_WATERCOLOR_PAPER_ANGLE,
    PAINTER_WATERCOLOR_PAPER_OFFSET_X,
    PAINTER_WATERCOLOR_PAPER_OFFSET_Y,
    PAINTER_WATERCOLOR_PAPER_DEPTH,
    PAINTER_WATERCOLOR_PAPER_PARAMS[0],
    PAINTER_WATERCOLOR_PAPER_PARAMS[1],
    PAINTER_WATERCOLOR_PAPER_PARAMS[2],
    PAINTER_WATERCOLOR_PAPER_PARAMS[3],
    PAINTER_WATERCOLOR_PAPER_PARAMS[4],
    PAINTER_WATERCOLOR_PAPER_PARAMS[5],
];
