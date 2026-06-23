//! Brush **Texture** section NodeIds (the per-dab texture mask; Blender `MTex`,
//! 2D-adapted). Fixed-id, tool-global widgets forwarding over the frozen
//! `PanelEvent` channel to `PainterTool::set_brush_texture_*` (the single clamp
//! source). Split from `painter.rs` to keep that file under the workspace LOC cap;
//! the option-id factories reuse the FNV runtime-hash twin in `painter.rs`.

use super::painter::fnv_node_id_runtime;
use super::{NodeId, hash_node_id};

/// Brush texture **kind** picker chip (None/Noise/Checker/Voronoi/Stripes — the
/// thumbnail picker). `SelectOption` → `set_brush_texture_kind`. Options via
/// [`painter_brush_texture_kind_option_id`].
pub const PAINTER_BRUSH_TEXTURE_KIND: NodeId = hash_node_id("painter_brush.texture_kind");
/// Brush texture **"New"** button — assigns the default procedural (Noise). `Click` → tool.
pub const PAINTER_BRUSH_TEXTURE_NEW: NodeId = hash_node_id("painter_brush.texture_new");
/// Brush texture **Mapping** dropdown chip (View Plane/Tiled/Random). `SelectOption` →
/// `set_brush_texture_mapping`. Options via [`painter_brush_texture_mapping_option_id`].
pub const PAINTER_BRUSH_TEXTURE_MAPPING: NodeId = hash_node_id("painter_brush.texture_mapping");
/// Brush texture **Angle** slider (`0..1` track → `0..=360°`). `SetValue` → `set_brush_texture_angle_norm`.
pub const PAINTER_BRUSH_TEXTURE_ANGLE: NodeId = hash_node_id("painter_brush.texture_angle");
/// Brush texture **Rake** toggle (angle follows the stroke). `Click` → `toggle_brush_texture_rake`.
pub const PAINTER_BRUSH_TEXTURE_RAKE: NodeId = hash_node_id("painter_brush.texture_rake");
/// Brush texture **Random** toggle (random angle per dab). `Click` → `toggle_brush_texture_random`.
pub const PAINTER_BRUSH_TEXTURE_RANDOM: NodeId = hash_node_id("painter_brush.texture_random");
/// Brush texture **Offset X** slider (`0..1` track → `−1..1` tile). `SetValue` →
/// `set_brush_texture_offset_norm(0, ..)`.
pub const PAINTER_BRUSH_TEXTURE_OFFSET_X: NodeId = hash_node_id("painter_brush.texture_offset_x");
/// Brush texture **Offset Y** slider (axis 1). See [`PAINTER_BRUSH_TEXTURE_OFFSET_X`].
pub const PAINTER_BRUSH_TEXTURE_OFFSET_Y: NodeId = hash_node_id("painter_brush.texture_offset_y");
/// Brush texture **Size X** slider (`0..1` track → `0.1..10` scale). `SetValue` →
/// `set_brush_texture_size_norm(0, ..)`.
pub const PAINTER_BRUSH_TEXTURE_SIZE_X: NodeId = hash_node_id("painter_brush.texture_size_x");
/// Brush texture **Size Y** slider (axis 1). See [`PAINTER_BRUSH_TEXTURE_SIZE_X`].
pub const PAINTER_BRUSH_TEXTURE_SIZE_Y: NodeId = hash_node_id("painter_brush.texture_size_y");
/// Per-pattern parameter slider **0** (`0..1` track). The kind's [`ParamSpec`] at this slot gives the
/// label + default; `SetValue` → `set_brush_texture_param_norm(0, ..)`. Painted only when the active
/// kind exposes this slot (see `param_specs`). [`ParamSpec`]: ph2d_painter_brush::ParamSpec
pub const PAINTER_BRUSH_TEXTURE_PARAM_0: NodeId = hash_node_id("painter_brush.texture_param_0");
/// Per-pattern parameter slider **1**. See [`PAINTER_BRUSH_TEXTURE_PARAM_0`].
pub const PAINTER_BRUSH_TEXTURE_PARAM_1: NodeId = hash_node_id("painter_brush.texture_param_1");
/// Per-pattern parameter slider **2**. See [`PAINTER_BRUSH_TEXTURE_PARAM_0`].
pub const PAINTER_BRUSH_TEXTURE_PARAM_2: NodeId = hash_node_id("painter_brush.texture_param_2");
/// Per-pattern parameter slider **3**. See [`PAINTER_BRUSH_TEXTURE_PARAM_0`].
pub const PAINTER_BRUSH_TEXTURE_PARAM_3: NodeId = hash_node_id("painter_brush.texture_param_3");

/// The four per-pattern parameter slider ids, indexed by [`ph2d_painter_brush::TextureSettings::params`]
/// slot. Lets the panel / populate / dispatch iterate without hardcoding each.
pub const PAINTER_BRUSH_TEXTURE_PARAMS: [NodeId; 4] = [
    PAINTER_BRUSH_TEXTURE_PARAM_0,
    PAINTER_BRUSH_TEXTURE_PARAM_1,
    PAINTER_BRUSH_TEXTURE_PARAM_2,
    PAINTER_BRUSH_TEXTURE_PARAM_3,
];

/// Derive the stable [`NodeId`] for texture-mapping option `m` (the `TextureMapping` wire
/// discriminant) in the open Mapping dropdown popover. Only the open popover's options are
/// hit-registered, so the `format!` is bounded.
#[must_use]
pub fn painter_brush_texture_mapping_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.texmapopt.{m}"))
}

/// Derive the stable [`NodeId`] for texture-kind option `k` (the `TextureKind` wire discriminant)
/// in the open kind picker popover.
#[must_use]
pub fn painter_brush_texture_kind_option_id(k: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.texkindopt.{k}"))
}

// ── Texture Color Ramp editor (maps the texture's scalar to a colour) ──
/// Color Ramp **enable** toggle. `Click` → `set_texture_ramp_enabled`.
pub const PAINTER_BRUSH_TEXTURE_RAMP_ENABLE: NodeId =
    hash_node_id("painter_brush.texture_ramp_enable");
/// Ramp **colour mode** dropdown (RGB/HSV/HSL). `SelectOption` → `set_texture_ramp_mode`.
pub const PAINTER_BRUSH_TEXTURE_RAMP_MODE: NodeId = hash_node_id("painter_brush.texture_ramp_mode");
/// Ramp **interpolation** dropdown (Ease/Cardinal/Linear/B-Spline/Constant). `SelectOption` →
/// `set_texture_ramp_interp`.
pub const PAINTER_BRUSH_TEXTURE_RAMP_INTERP: NodeId =
    hash_node_id("painter_brush.texture_ramp_interp");
/// The ramp **bar** — the `CurvePoint` parent canvas the draggable stop handles report against.
pub const PAINTER_BRUSH_TEXTURE_RAMP_EDIT: NodeId = hash_node_id("painter_brush.texture_ramp_edit");
/// Ramp **add stop** button. `Click` → `ramp_add_stop`.
pub const PAINTER_BRUSH_TEXTURE_RAMP_ADD: NodeId = hash_node_id("painter_brush.texture_ramp_add");
/// Ramp **remove stop** button (removes the selected stop). `Click` → `ramp_remove_stop`.
pub const PAINTER_BRUSH_TEXTURE_RAMP_REMOVE: NodeId =
    hash_node_id("painter_brush.texture_ramp_remove");
/// The selected stop's **colour swatch** — opens the colour picker. `Click` → picker target.
pub const PAINTER_BRUSH_TEXTURE_RAMP_SWATCH: NodeId =
    hash_node_id("painter_brush.texture_ramp_swatch");
/// Editable **stop-index selector** chip (`NumberInput`; type an index → select that stop).
/// `ValueChanged` → the panel re-points the selection. Whole numbers, `0..stop_count`.
pub const PAINTER_BRUSH_TEXTURE_RAMP_STOP_INDEX: NodeId =
    hash_node_id("painter_brush.texture_ramp_stop_index");
/// Editable **stop-position** chip (`NumberInput`; type `0..1` → move the selected stop).
/// `ValueChanged` → `ramp_move_stop` (by the selected stop's stable id).
pub const PAINTER_BRUSH_TEXTURE_RAMP_STOP_POS: NodeId =
    hash_node_id("painter_brush.texture_ramp_stop_pos");

/// Stable [`NodeId`] for ramp stop `i`'s **draggable position handle** on the bar (a `CurvePoint`
/// under [`PAINTER_BRUSH_TEXTURE_RAMP_EDIT`]; distinct from its swatch button id).
#[must_use]
pub fn painter_brush_texture_ramp_handle_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.texramphandle.{i}"))
}

/// Stable [`NodeId`] for ramp colour-mode option `m` in the open Mode dropdown popover.
#[must_use]
pub fn painter_brush_texture_ramp_mode_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.texrampmodeopt.{m}"))
}

/// Stable [`NodeId`] for ramp interpolation option `i` in the open Interpolation dropdown popover.
#[must_use]
pub fn painter_brush_texture_ramp_interp_option_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.texrampinterpopt.{i}"))
}

/// Ramp **alpha action** dropdown (Off / → Strength / → Sprite) — what the ramp colour's alpha does
/// when painting. `SelectOption` → `set_texture_ramp_alpha_mode`. Sits just below the Color Ramp.
pub const PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE: NodeId =
    hash_node_id("painter_brush.texture_ramp_alpha_mode");

/// Stable [`NodeId`] for ramp alpha-action option `m` (the `RampAlphaMode` wire discriminant) in the
/// open dropdown popover.
#[must_use]
pub fn painter_brush_texture_ramp_alpha_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.texrampalphaopt.{m}"))
}
