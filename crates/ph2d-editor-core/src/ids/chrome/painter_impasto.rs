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
/// **Adjust Last Stroke** — whether moving a slider re-derives the stroke already on the canvas
/// (ON = the section's historical behaviour) or speaks only to the strokes still to come.
/// `Click` → `toggle_impasto_live_edit`. An editing preference, not a property of the paint: it is
/// never baked into a pixel.
pub const PAINTER_IMPASTO_LIVE_EDIT: NodeId = hash_node_id("painter_brush.impasto_live_edit");
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
/// **Push** (`0..1`) — **volume conservation**: how much of the paint ALREADY on the canvas this brush
/// shoves aside as it passes. `0` (default) = a stroke piles onto what is under it, as if two bodies of
/// paint could occupy the same space; up = the brush ploughs a channel and the displaced material stands
/// up as a ridge along the stroke's edges. Nothing is created, nothing destroyed — it only moves.
/// Live on the last stroke (a pure function of ground × footprint). `SetValue` → `set_brush_impasto_push`.
pub const PAINTER_IMPASTO_PUSH: NodeId = hash_node_id("painter_brush.impasto_push");
/// **Plow** (`0..1`) — how strongly the **Smear** drags EXISTING relief along with the colour: the
/// palette knife. Painted only in the Smear (the one mode that deposits no paint and still has
/// something to say about the body), where it replaces the whole Body card — there is no Depth to set
/// when nothing is being laid down. `SetValue` → `set_brush_impasto_plow`.
pub const PAINTER_IMPASTO_PLOW: NodeId = hash_node_id("painter_brush.impasto_plow");

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
// (There deliberately is NO "Amount" here: it was a second gain over the same percept as the brush's
// Depth — the two scaled one another and the panel read as "hard to adjust". The slope is now pure
// geometry, `DEPTH_UNIT_PX` in the light pass; `docs/Painter/17_impasto_deposito_pesquisa2.md` §5–6.)
/// **Body** (`0..1`) — the relief's cross-section: `1` = level film with a wall (the body of paint),
/// `0` = the relief obeys the falloff exactly (the perfectly rounded ridge). A SHAPE percept, not a
/// gain — orthogonal to Depth. `SetValue` → `set_brush_impasto_body`.
pub const PAINTER_IMPASTO_BODY: NodeId = hash_node_id("painter_brush.impasto_body");
/// **Shine** (`0..1`) — the specular highlight riding the crests. `0` = matte, high = wet oil.
/// `SetValue` → `set_impasto_shine`. A property of the PAINT, so it is GLOBAL, not per-lamp. (Krita
/// gives every light its own `ks`; that is four knobs for one material, and it is a good part of why its
/// Phong Bumpmap ends up with 24 controls.)
pub const PAINTER_IMPASTO_SHINE: NodeId = hash_node_id("painter_brush.impasto_shine");
/// **Roughness** (`0..1`) — how BROAD the highlight is (`0` a tight glint … `1` a wide soft sheen).
/// The exponent used to be the hard-coded `SHININESS = 24`; it is the PAINT's now.
/// `SetValue` → `set_impasto_roughness`.
pub const PAINTER_IMPASTO_ROUGHNESS: NodeId = hash_node_id("painter_brush.impasto_roughness");
/// **Metallic** (`0..1`) — whose colour the highlight takes: the lamp's (`0`) or the paint's own (`1`).
/// `SetValue` → `set_impasto_metallic`.
pub const PAINTER_IMPASTO_METALLIC: NodeId = hash_node_id("painter_brush.impasto_metallic");
/// **Wax** (`0..1`) — the soft terminator of paint the light enters and leaves nearby. Wrap lighting,
/// not a subsurface simulation. `SetValue` → `set_impasto_wax`.
pub const PAINTER_IMPASTO_WAX: NodeId = hash_node_id("painter_brush.impasto_wax");

// ── The RIG: four lamps, ONE on screen ─────────────────────────────────────────────────────────────
//
// The card edits the SELECTED lamp, so its row count never grows with the rig. Lamps 2-4 start off ⇒ a
// canvas nobody has opened the rig on is byte-identical to the one-lamp build.

/// The lamp selector — four chips (`1 2 3 4`). `Click` → `select_impasto_light`. Editing state: picking
/// a lamp changes no pixel.
pub const PAINTER_IMPASTO_LIGHT_1: NodeId = hash_node_id("painter_brush.impasto_light_1");
/// See [`PAINTER_IMPASTO_LIGHT_1`].
pub const PAINTER_IMPASTO_LIGHT_2: NodeId = hash_node_id("painter_brush.impasto_light_2");
/// See [`PAINTER_IMPASTO_LIGHT_1`].
pub const PAINTER_IMPASTO_LIGHT_3: NodeId = hash_node_id("painter_brush.impasto_light_3");
/// See [`PAINTER_IMPASTO_LIGHT_1`].
pub const PAINTER_IMPASTO_LIGHT_4: NodeId = hash_node_id("painter_brush.impasto_light_4");
/// **Enable** the selected lamp. `Click` → `toggle_impasto_light_on`. Dimmed on lamp 1: the key cannot
/// be switched off (Show Impasto already IS that switch, and a lit canvas with no light is a lie).
pub const PAINTER_IMPASTO_LIGHT_ON: NodeId = hash_node_id("painter_brush.impasto_light_on");
/// **Intensity** of the selected lamp (`0..2`). `SetValue` → `set_impasto_light_intensity`.
pub const PAINTER_IMPASTO_LIGHT_POWER: NodeId = hash_node_id("painter_brush.impasto_light_power");
/// The selected lamp's **Colour** swatch — opens the shared OKLCH picker. `Click` → the shell reads the
/// pick back into `set_impasto_light_color`. A coloured lamp tints the paint where it TILTS and leaves
/// flat paint byte-identical (the shading is relative, per channel).
pub const PAINTER_IMPASTO_LIGHT_COLOR: NodeId = hash_node_id("painter_brush.impasto_light_color");

/// The Impasto **Click** widgets — one membership check for the panel's click forward, routed by
/// `route_brush_impasto_event`.
pub const PAINTER_IMPASTO_CLICKS: [NodeId; 14] = [
    PAINTER_IMPASTO_ENABLE,
    PAINTER_IMPASTO_LIVE_EDIT,
    PAINTER_IMPASTO_RESET,
    PAINTER_IMPASTO_SOURCE_UNIFORM,
    PAINTER_IMPASTO_SOURCE_GRAIN,
    PAINTER_IMPASTO_DRAW_BOTH,
    PAINTER_IMPASTO_DRAW_COLOR,
    PAINTER_IMPASTO_DRAW_DEPTH,
    PAINTER_IMPASTO_SHOW,
    PAINTER_IMPASTO_LIGHT_1,
    PAINTER_IMPASTO_LIGHT_2,
    PAINTER_IMPASTO_LIGHT_3,
    PAINTER_IMPASTO_LIGHT_4,
    PAINTER_IMPASTO_LIGHT_ON,
];

/// The Impasto **SetValue** number-fields — one membership check for the panel's number-field forward
/// (`is_param_field`) and its register loop.
pub const PAINTER_IMPASTO_FIELDS: [NodeId; 12] = [
    PAINTER_IMPASTO_ROUGHNESS,
    PAINTER_IMPASTO_METALLIC,
    PAINTER_IMPASTO_WAX,
    PAINTER_IMPASTO_DEPTH,
    PAINTER_IMPASTO_BODY,
    PAINTER_IMPASTO_PUSH,
    PAINTER_IMPASTO_PLOW,
    PAINTER_IMPASTO_SMOOTHING,
    PAINTER_IMPASTO_LIGHT_ANGLE,
    PAINTER_IMPASTO_LIGHT_ELEV,
    PAINTER_IMPASTO_LIGHT_POWER,
    PAINTER_IMPASTO_SHINE,
];
