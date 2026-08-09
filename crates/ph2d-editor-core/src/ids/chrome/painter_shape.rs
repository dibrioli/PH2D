//! Brush **Shape** section NodeIds (the silhouette tip; Procreate "Shape"). The Shape section hosts
//! the Falloff (the procedural silhouette, default) plus — once a Shape **image** is assigned — the
//! image preview and its rotation controls (the falloff then goes inactive). Also the **Grain Depth**
//! slider (which lives in the renamed "Grain" section). Fixed-id, tool-global widgets forwarding over
//! the frozen `PanelEvent` channel to `PainterTool::set_brush_shape_*` / `set_brush_grain_depth`.
//! Split from `painter.rs` / `painter_texture.rs` to keep those under the workspace LOC cap.

use super::painter::fnv_node_id_runtime;
use super::{NodeId, hash_node_id};

/// **Grain Depth** slider (`0..1` track; `1` = full bite, default). `SetValue` → `set_brush_grain_depth`.
/// Lives in the Grain section (the renamed Texture section).
pub const PAINTER_BRUSH_GRAIN_DEPTH: NodeId = hash_node_id("painter_brush.grain_depth");

/// Shape **source** picker chip — `None` (the procedural Falloff) vs `Image` (an assigned silhouette).
/// Mirrors the Grain Kind picker: selecting `Image` opens a file pick (or use the Hierarchy "Use as
/// Brush Shape"); `None` clears the image. `SelectOption` → `set_brush_shape_kind`. Options via
/// [`painter_shape_kind_option_id`].
pub const PAINTER_SHAPE_KIND: NodeId = hash_node_id("painter_brush.shape_kind");
/// **Automatic** checkbox (watercolor mode only, doc 13 #1): checked (default) = the watercolor's
/// built-in feather silhouette (byte-identical historical stamp; the Shape items hide); unchecked =
/// the Shape section (Falloff incl. the "Watercolor" preset + image + rotation) drives the watercolor
/// coverage stamp. `Click` → `toggle_brush_watercolor_shape_auto`.
pub const PAINTER_SHAPE_WATERCOLOR_AUTO: NodeId =
    hash_node_id("painter_brush.shape_watercolor_auto");

/// Derive the stable [`NodeId`] for Shape-source option `k` (the `TextureKind` wire discriminant —
/// only `None`/`Image` are offered) in the open Shape picker popover. Only the open popover's options
/// are hit-registered, so the `format!` is bounded.
#[must_use]
pub fn painter_shape_kind_option_id(k: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapekindopt.{k}"))
}

/// Collapsible **Shape** section header (Inspector pattern: ALL-CAPS label + collapse chevron +
/// assignable color dot). `mark_collapsible_section`-registered in `crate::populate`.
pub const PAINTER_SHAPE_SECTION: NodeId = hash_node_id("painter_brush.shape_section");
/// The Shape header's color dot — a picker swatch (`register_picker_swatch`).
pub const PAINTER_SHAPE_SECTION_COLOR: NodeId = hash_node_id("painter_brush.shape_section_color");
/// Shape section **reset** icon button — clears the Shape image (→ falloff) and resets the rotation
/// controls. `Click` → tool reset. Part of [`super::PAINTER_BRUSH_SECTION_RESETS`].
pub const PAINTER_SHAPE_RESET: NodeId = hash_node_id("painter_brush.shape_reset");

/// Shape **Angle** slider (`0..1` track → `0..=360°`). `SetValue` → `set_brush_shape_angle_norm`.
pub const PAINTER_SHAPE_ANGLE: NodeId = hash_node_id("painter_brush.shape_angle");
/// Shape **Follow** dropdown — how the silhouette relates to the stroke direction: `0` = Off (the fixed
/// [`PAINTER_SHAPE_ANGLE`]), `1` = Rake (rotate each stamp to the tangent), `2` = Flow (lay the pattern in
/// the stroke's arc-length frame so its lines stay parallel through curves). Supersedes the old Rake
/// checkbox (Enio 2026-07-19). `SelectOption` → `set_brush_shape_follow`. Options via
/// [`painter_shape_follow_option_id`]. (A random per-dab spin is the Stroke Jitter Rotate; the per-slot
/// "Random Angle" was retired 2026-07-19.)
pub const PAINTER_SHAPE_FOLLOW: NodeId = hash_node_id("painter_brush.shape_follow");

/// The three Shape **Follow** modes as `(wire value, label)` — the single source for the dropdown options
/// and the snapshot's `shape_follow`. `0` = Off, `1` = Rake, `2` = Flow.
pub const PAINTER_SHAPE_FOLLOW_MODES: [(u8, &str); 3] = [(0, "Off"), (1, "Rake"), (2, "Flow")];

/// Derive the stable [`NodeId`] for Shape-follow option `v` (`0`/`1`/`2`) in the open dropdown popover.
/// Only the open popover's options are hit-registered, so the `format!` is bounded.
#[must_use]
pub fn painter_shape_follow_option_id(v: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapefollowopt.{v}"))
}
/// Shape **Offset X** slider (`0..1` track → `−1..1` tile). `SetValue` → `set_brush_shape_offset_norm(0, ..)`.
pub const PAINTER_SHAPE_OFFSET_X: NodeId = hash_node_id("painter_brush.shape_offset_x");
/// Shape **Offset Y** slider (axis 1). See [`PAINTER_SHAPE_OFFSET_X`].
pub const PAINTER_SHAPE_OFFSET_Y: NodeId = hash_node_id("painter_brush.shape_offset_y");
/// Shape **Size X** slider (`0..1` track → `0.1..10` scale). `SetValue` → `set_brush_shape_size_norm(0, ..)`.
pub const PAINTER_SHAPE_SIZE_X: NodeId = hash_node_id("painter_brush.shape_size_x");
/// Shape **Size Y** slider (axis 1). See [`PAINTER_SHAPE_SIZE_X`].
pub const PAINTER_SHAPE_SIZE_Y: NodeId = hash_node_id("painter_brush.shape_size_y");

/// The Shape + Grain-Depth **SetValue** sliders, as one array — a single membership check for the
/// panel's slider-drag forward (`event.rs`) and a single register loop (`populate.rs`).
pub const PAINTER_SHAPE_SLIDERS: [NodeId; 6] = [
    PAINTER_BRUSH_GRAIN_DEPTH,
    PAINTER_SHAPE_ANGLE,
    PAINTER_SHAPE_OFFSET_X,
    PAINTER_SHAPE_OFFSET_Y,
    PAINTER_SHAPE_SIZE_X,
    PAINTER_SHAPE_SIZE_Y,
];

// ── Per-Layer Color (multi-layer Shape; Enio 2026-06-26) ────────────────────────────────────────
/// "Use Document Layers" button — captures the active Painter document's visible raster layers
/// (z-ordered, bottom-to-top) as a multi-layer Shape, each becoming a colourable layer. `Click` →
/// `capture_layers_as_brush_shape`. Sits below the Shape **Texture** dropdown.
pub const PAINTER_SHAPE_USE_LAYERS: NodeId = hash_node_id("painter_brush.shape_use_layers");

/// "Per-Layer Color" mode toggle — shown below the Shape **Texture** dropdown only when a multi-layer
/// (`> 1`) Shape image is assigned. `Click` → `toggle_brush_shape_per_layer_color`. While on, each
/// layer paints its own colour (higher above lower) and the Shape Color ramp section is hidden +
/// nullified (the route uses the cached coloured stamp instead).
pub const PAINTER_SHAPE_PER_LAYER_COLOR: NodeId =
    hash_node_id("painter_brush.shape_per_layer_color");

/// "Alpha From Image" — the Shape silhouette comes from the image's own ALPHA instead of from its
/// light/dark differences. `Click` -> `toggle_brush_shape_alpha_from_image`.
///
/// Painted above the texture-colour checkbox, and only when there IS an alternative (captured RGB,
/// so a luminance exists to switch to) — a checkbox with one reachable value is a dead control. The
/// DEFAULT is measured at capture, not chosen: alpha when it carries any transparency, luminance
/// when the image is opaque edge to edge, because an opaque image's alpha is not a silhouette (it
/// would make every photo a square stamp).
pub const PAINTER_SHAPE_ALPHA_FROM_IMAGE: NodeId =
    hash_node_id("painter_brush.shape_alpha_from_image");

/// Stable [`NodeId`] for Shape layer `i`'s **colour checkbox** (the "Layer i+1 Color" row, shown when
/// Per-Layer Color is on). A FACTORY id (paint-time registration; only `0..layer_count` rows are
/// painted, so the `format!` is bounded). `Click` → `toggle_brush_shape_layer_color(i)`.
#[must_use]
pub fn painter_shape_layer_color_check_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapelayercolorck.{i}"))
}

/// Stable [`NodeId`] for Shape layer `i`'s **colour swatch** (shown when its checkbox is on). Opens the
/// shared colour picker; the per-frame readback forwards `set_brush_shape_layer_color(i, rgb)`.
#[must_use]
pub fn painter_shape_layer_color_swatch_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapelayercolorsw.{i}"))
}

/// Stable [`NodeId`] for Shape layer `i`'s **blend** dropdown chip (the "B" button on the layer row) —
/// picks the layer's blend mode. Registered as an `InteractiveState::Dropdown`; option clicks forward
/// `set_brush_shape_layer_blend(i, mode)`.
#[must_use]
pub fn painter_shape_layer_blend_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapelayerblend.{i}"))
}

/// Stable [`NodeId`] for blend option `mode` of Shape layer `i` in the open blend popover (only the open
/// popover's options are hit-registered, so the `format!` is bounded).
#[must_use]
pub fn painter_shape_layer_blend_option_id(i: u8, mode: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapelayerblendopt.{i}.{mode}"))
}

/// Stable [`NodeId`] for Shape layer `i`'s **opacity** numeric box (right of the colour box) — mirrors the
/// source document layer's opacity (`0..100`). `SetValue` → `set_brush_shape_layer_opacity(i, v/100)`.
#[must_use]
pub fn painter_shape_layer_opacity_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shapelayeropacity.{i}"))
}

// ── Dab **flatten + rotate** gizmo (Procreate Shape panel; Enio 2026-06-26) ─────────────────────
/// The flatten/rotate gizmo **canvas** — the `CurvePoint` parent the two handles report against. A
/// handle drag emits `ValueChanged(PAINTER_BRUSH_DAB_GIZMO)`; the panel decodes the pointer (radial
/// distance → flatten, angle → rotation) and forwards [`PAINTER_BRUSH_DAB_FLATTEN`] / [`PAINTER_BRUSH_DAB_ANGLE`].
pub const PAINTER_BRUSH_DAB_GIZMO: NodeId = hash_node_id("painter_brush.dab_gizmo");
/// Dab **Flatten** value sink (`0..1`). `SetValue` → `set_brush_dab_flatten`.
pub const PAINTER_BRUSH_DAB_FLATTEN: NodeId = hash_node_id("painter_brush.dab_flatten");
/// Dab **rotation** value sink (whole degrees). `SetValue` → `set_brush_dab_angle`.
pub const PAINTER_BRUSH_DAB_ANGLE: NodeId = hash_node_id("painter_brush.dab_angle");

/// Stable [`NodeId`] for the gizmo handle on `channel` (`0` = flatten, on the minor axis; `1` =
/// rotation, on the rim). A FACTORY id (paint-time `CurvePoint` registration) so it stays off the
/// static hit↔populate wiring scan.
#[must_use]
pub fn painter_brush_dab_handle_id(channel: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.dabhandle.{channel}"))
}

// ── **Taper** (Procreate *Touch Taper*; Enio 2026-08-08) ─────────────────────────────────────────
// The stroke thins toward its ends because it is near an end, not because a pen pressed lightly — the
// only way a mouse draws a calligraphic line. Sits directly below the Falloff, because it shapes the
// same thing the falloff does (how wide the mark is) along the OTHER axis: the falloff across the dab,
// the taper along the stroke.

/// The taper **widget**: a stroke-shaped preview with a draggable handle at each end. Both handles are
/// `CurvePoint`s parented here (channel `0` = the START length, `1` = the END length); the panel decodes
/// the drag and forwards [`PAINTER_TAPER_START`] / [`PAINTER_TAPER_END`], exactly like the dab gizmo.
pub const PAINTER_TAPER_GIZMO: NodeId = hash_node_id("painter_brush.taper_gizmo");
/// Taper length at the stroke's **start**, in brush DIAMETERS. `SetValue` → `set_brush_taper_start`.
pub const PAINTER_TAPER_START: NodeId = hash_node_id("painter_brush.taper_start");
/// Taper length at the stroke's **end**, in brush DIAMETERS. `SetValue` → `set_brush_taper_end`.
pub const PAINTER_TAPER_END: NodeId = hash_node_id("painter_brush.taper_end");
/// **Tip size** at the start, `0..1` (`0` = a sharp point, `1` = blunt). Procreate's "Tip".
pub const PAINTER_TAPER_TIP_START: NodeId = hash_node_id("painter_brush.taper_tip_start");
/// **Tip size** at the end, `0..1`. Only painted while [`PAINTER_TAPER_LINK`] is unchecked.
pub const PAINTER_TAPER_TIP_END: NodeId = hash_node_id("painter_brush.taper_tip_end");
/// **Link tip sizes**: one tip size drives both ends. Click toggle (`Click` → `toggle_brush_taper_link`).
pub const PAINTER_TAPER_LINK: NodeId = hash_node_id("painter_brush.taper_link");
/// Taper **Opacity**, `0..1`: how much the taper fades as well as narrows.
pub const PAINTER_TAPER_OPACITY: NodeId = hash_node_id("painter_brush.taper_opacity");

/// The taper's three **numeric** rows, as one family.
///
/// ⚠️ This list is what makes them LIVE: a number row is painted and hit-registered by
/// `paint_num_row`, but its `ValueChanged` only reaches the tool if `is_param_field` claims the id —
/// and the panel mirrors the tool's value back into the field every frame, so a row that is not
/// claimed **reverts to the old value the instant the artist lets go**. That is the shape of the defect
/// Enio reported on 2026-08-08 (*"Tip start, Tip End e Opacity não aceitam ajustes (voltam a zero)"*):
/// painted, registered, alive under the mouse, and mute — the fourth of the four independent UI
/// conditions, and the one a paint+populate gate cannot see.
pub const PAINTER_TAPER_FIELDS: [NodeId; 3] = [
    PAINTER_TAPER_TIP_START,
    PAINTER_TAPER_TIP_END,
    PAINTER_TAPER_OPACITY,
];

/// Stable [`NodeId`] for the taper handle on `channel` (`0` = start, `1` = end). A FACTORY id
/// (paint-time `CurvePoint` registration), the twin of [`painter_brush_dab_handle_id`], so it stays off
/// the static hit↔populate wiring scan.
#[must_use]
pub fn painter_taper_handle_id(channel: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.taperhandle.{channel}"))
}

// ── Shape **Colour Ramp** (colourises the silhouette; "Shape Color" section) ─────────────────────
// The Shape ramp is a full `ph2d_color::ColorRamp` (twin of the Grain ramp), with a **B&W** filter:
// off ⇒ the ramp owns the painted colour (indexed by the silhouette coverage); on ⇒ it's the grayscale
// tone remap of the silhouette (the Grain, if any, owns colour). Auto-B&W-on when a Grain is assigned.
/// Collapsible **Shape Color** section header. `mark_collapsible_section`-registered in `crate::populate`.
pub const PAINTER_SHAPE_RAMP_SECTION: NodeId = hash_node_id("painter_brush.shape_ramp_section");
/// The Shape Color header's colour dot (picker swatch).
pub const PAINTER_SHAPE_RAMP_SECTION_COLOR: NodeId =
    hash_node_id("painter_brush.shape_ramp_section_color");
/// Shape Color section **reset** — ramp off + default gradient. `Click` → `reset_shape_ramp`.
pub const PAINTER_SHAPE_RAMP_RESET: NodeId = hash_node_id("painter_brush.shape_ramp_reset");
/// "Use Color Ramp" enable toggle. `Click` → `toggle_shape_ramp_enabled`.
pub const PAINTER_SHAPE_RAMP_ENABLE: NodeId = hash_node_id("painter_brush.shape_ramp_enable");
/// Shape ramp **colour mode** dropdown (RGB/HSV/HSL). `SelectOption` → `set_shape_ramp_mode`.
pub const PAINTER_SHAPE_RAMP_MODE: NodeId = hash_node_id("painter_brush.shape_ramp_mode");
/// Shape ramp **interpolation** dropdown. `SelectOption` → `set_shape_ramp_interp`.
pub const PAINTER_SHAPE_RAMP_INTERP: NodeId = hash_node_id("painter_brush.shape_ramp_interp");
/// Shape ramp **B&W** filter toggle (desaturate the colours to luminance; auto-on with a Grain).
/// `Click` → `toggle_shape_ramp_bw`.
pub const PAINTER_SHAPE_RAMP_BW: NodeId = hash_node_id("painter_brush.shape_ramp_bw");
/// Shape ramp **alpha action** dropdown (Off / → Strength / → Sprite). `SelectOption` →
/// `set_shape_ramp_alpha_mode`.
pub const PAINTER_SHAPE_RAMP_ALPHA_MODE: NodeId =
    hash_node_id("painter_brush.shape_ramp_alpha_mode");
/// The gradient **bar** — the `CurvePoint` parent the draggable stop handles report against.
pub const PAINTER_SHAPE_RAMP_EDIT: NodeId = hash_node_id("painter_brush.shape_ramp_edit");
/// "+" add a stop. `Click` → `shape_ramp_add_stop`.
pub const PAINTER_SHAPE_RAMP_ADD: NodeId = hash_node_id("painter_brush.shape_ramp_add");
/// "−" remove the last stop. `Click` → `shape_ramp_remove_last_stop`.
pub const PAINTER_SHAPE_RAMP_REMOVE: NodeId = hash_node_id("painter_brush.shape_ramp_remove");
/// **Invert** button (flip stop positions L↔R). `Click` → `shape_ramp_invert`.
pub const PAINTER_SHAPE_RAMP_INVERT: NodeId = hash_node_id("painter_brush.shape_ramp_invert");
/// The selected stop's **colour swatch** — opens the colour picker. `Click` → picker target.
pub const PAINTER_SHAPE_RAMP_SWATCH: NodeId = hash_node_id("painter_brush.shape_ramp_swatch");
/// Selected-stop **index** chip (`NumberInput`).
pub const PAINTER_SHAPE_RAMP_STOP_INDEX: NodeId =
    hash_node_id("painter_brush.shape_ramp_stop_index");
/// Selected-stop **position** chip (`NumberInput`, `0..1`). `SelectOption` → `shape_ramp_move_stop`.
pub const PAINTER_SHAPE_RAMP_STOP_POS: NodeId = hash_node_id("painter_brush.shape_ramp_stop_pos");

/// The Shape-ramp **Click** buttons (enable / add / remove / invert / B&W / reset) — forwarded as a
/// `PanelEvent::Click` by the panel's `event.rs` (a single membership check).
pub const PAINTER_SHAPE_RAMP_BUTTONS: [NodeId; 6] = [
    PAINTER_SHAPE_RAMP_ENABLE,
    PAINTER_SHAPE_RAMP_ADD,
    PAINTER_SHAPE_RAMP_REMOVE,
    PAINTER_SHAPE_RAMP_INVERT,
    PAINTER_SHAPE_RAMP_BW,
    PAINTER_SHAPE_RAMP_RESET,
];

/// The Shape-ramp **ValueChanged** ids (bar-stop drag + the editable index / position chips) — routed
/// to `ramp_picker` by the panel's `event.rs` (a single membership check).
pub const PAINTER_SHAPE_RAMP_VALUE_IDS: [NodeId; 3] = [
    PAINTER_SHAPE_RAMP_EDIT,
    PAINTER_SHAPE_RAMP_STOP_INDEX,
    PAINTER_SHAPE_RAMP_STOP_POS,
];

/// Stable [`NodeId`] for the draggable handle of Shape-ramp stop `i` (the open bar's stops are
/// hit-registered, so the `format!` is bounded).
#[must_use]
pub fn painter_shape_ramp_handle_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shaperamphandle.{i}"))
}

/// Stable [`NodeId`] for Shape-ramp colour-mode option `m` in the open Mode dropdown popover.
#[must_use]
pub fn painter_shape_ramp_mode_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shaperampmodeopt.{m}"))
}

/// Stable [`NodeId`] for Shape-ramp interpolation option `i` in the open dropdown popover.
#[must_use]
pub fn painter_shape_ramp_interp_option_id(i: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shaperampinterpopt.{i}"))
}

/// Stable [`NodeId`] for Shape-ramp alpha-action option `m` in the open dropdown popover.
#[must_use]
pub fn painter_shape_ramp_alpha_option_id(m: u8) -> NodeId {
    fnv_node_id_runtime(&format!("painter_brush.shaperampalphaopt.{m}"))
}

/// Per-pattern parameter sliders for a **procedural** Shape (Contrast / Brightness + the kind's shape
/// knob), indexed by [`ph2d_painter_brush::TextureSettings::params`] slot — the Grain's
/// [`PAINTER_BRUSH_TEXTURE_PARAMS`](super::PAINTER_BRUSH_TEXTURE_PARAMS) twin, so the Shape gets every
/// characteristic param. `SetValue` → `set_brush_shape_param_norm`.
pub const PAINTER_SHAPE_PARAMS: [NodeId; 6] = [
    hash_node_id("painter_brush.shape_param_0"),
    hash_node_id("painter_brush.shape_param_1"),
    hash_node_id("painter_brush.shape_param_2"),
    hash_node_id("painter_brush.shape_param_3"),
    hash_node_id("painter_brush.shape_param_4"),
    hash_node_id("painter_brush.shape_param_5"),
];
