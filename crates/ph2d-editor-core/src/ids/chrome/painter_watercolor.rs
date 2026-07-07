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
/// **Smudge** — TRUE-smear strength `0..1` (drags the painted paint). `SetValue` → `set_brush_wet_smudge`.
pub const PAINTER_WATERCOLOR_SMUDGE: NodeId = hash_node_id("painter_brush.watercolor_smudge");
/// **Wet** — wet-on-wet rewetting `0..1` (lift + dissolve + pool). `SetValue` → `set_brush_wet_rewet`.
pub const PAINTER_WATERCOLOR_WET: NodeId = hash_node_id("painter_brush.watercolor_wet");

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
/// **Paper colour** swatch — the document "ground" the watercolor optics see where nothing is painted
/// below the active layer. Click toggles the shared Blender picker onto it (panel-side, like
/// `PAINTER_COLOR_THUMB`); the picker read-back forwards `SelectOption(id, "r,g,b")` →
/// `set_paper_color_rgb8`.
pub const PAINTER_WATERCOLOR_PAPER_COLOR_THUMB: NodeId =
    hash_node_id("painter_brush.watercolor_paper_color_thumb");
/// **Paper** slot kind dropdown chip (None / Cold-Rough-Hot Press / other procedural / Image). `SelectOption`
/// → `set_brush_paper_kind`. Options via [`painter_paper_kind_option_id`].
pub const PAINTER_WATERCOLOR_PAPER_KIND: NodeId =
    hash_node_id("painter_brush.watercolor_paper_kind");
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
/// optics Fill / Depth / Warp + the Wet Mix pair Smudge / Wet + the full Paper slot: Size / Angle /
/// Offset / Depth / params) — one membership check for the panel's number-field forward
/// (`is_param_field`) and register loop.
pub const PAINTER_WATERCOLOR_FIELDS: [NodeId; 21] = [
    PAINTER_WATERCOLOR_EDGE,
    PAINTER_WATERCOLOR_SPREAD,
    PAINTER_WATERCOLOR_GRANULATION,
    PAINTER_WATERCOLOR_MIX,
    PAINTER_WATERCOLOR_FILL,
    PAINTER_WATERCOLOR_DEPTH,
    PAINTER_WATERCOLOR_WARP,
    PAINTER_WATERCOLOR_SMUDGE,
    PAINTER_WATERCOLOR_WET,
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
