//! Brush/Stroke parameter snapshot & setters (the single UI-edit clamp source); a submodule of
//! `paint`, split from `paint.rs` for the workspace LOC cap. Per-dab-jitter setters: `jitter_settings`.

use super::shape_layers::MAX_SHAPE_LAYERS;
use super::{
    BRUSH_AIRBRUSH_RATE_MAX_S, BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_COUNT_SLIDER_MAX,
    BRUSH_JITTER_ABS_MAX_PX, BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX, BRUSH_SPACING_MAX,
};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushBlend, Falloff, FalloffPoint, HandleType, JitterUnit, MAX_FALLOFF_POINTS, StrokeMethod,
    TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
    TextureMapping,
};

// `BrushTextureImage` lives in the sibling `brush_image` module (LOC cap); re-exported so the existing
// `super::brush_settings::BrushTextureImage` import paths stay stable.
pub(super) use super::brush_image::BrushTextureImage;

/// Snap `cursor` to the nearest 45° ray from `anchor` (Blender Line Alt-constrain), projecting onto it.
/// Transcendental-free (abs/signum/mul + the `tan(22.5°)`/`tan(67.5°)`/`√½` constants) for HR-5.
pub(super) fn snap_to_45(anchor: [f32; 2], cursor: [f32; 2]) -> [f32; 2] {
    const TAN_22_5: f32 = 0.414_213_56; // tan(22.5°)
    const TAN_67_5: f32 = 2.414_213_5; // tan(67.5°)
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2; // √½
    let dx = cursor[0] - anchor[0];
    let dy = cursor[1] - anchor[1];
    let (adx, ady) = (dx.abs(), dy.abs());
    // The snapped unit direction (one of the 8 rays).
    let (ux, uy) = if ady <= adx * TAN_22_5 {
        (dx.signum(), 0.0) // horizontal
    } else if ady >= adx * TAN_67_5 {
        (0.0, dy.signum()) // vertical
    } else {
        (dx.signum() * DIAG, dy.signum() * DIAG) // diagonal
    };
    // Project the cursor onto the ray (dot product = signed distance along the unit direction).
    let proj = dx * ux + dy * uy;
    [anchor[0] + ux * proj, anchor[1] + uy * proj]
}

/// A compact snapshot of the active brush for the layers panel's Brush section, published each frame
/// by the shell bridge — the panel reads it to position the size/colour sliders + blend chip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSettings {
    /// Radius in image pixels (UI label "Size").
    pub size_px: f32,
    /// [`Self::size_px`] mapped onto the size slider's `0..1` track (squared, so small brushes get more).
    pub size_norm: f32,
    /// Overall opacity, `0..1` (UI "Strength").
    pub strength: f32,
    /// Distance-falloff preset ([`Falloff::to_u8`]) — the dab profile (replaces Hardness); [`Falloff::Custom`] (`9`) reads [`Self::falloff_points`].
    pub falloff: u8,
    /// The `Custom` falloff curve's control points (id + `[distance, strength]` + handle), first [`Self::falloff_len`] valid, ascending by distance; panel plots + drags by stable id.
    pub falloff_points: [FalloffPoint; MAX_FALLOFF_POINTS],
    /// Count of valid entries in [`Self::falloff_points`] (`2..=MAX_FALLOFF_POINTS`).
    pub falloff_len: u8,
    /// Straight-RGB paint colour in `[0, 1]`.
    pub color: [f32; 3],
    /// Blend-mode wire discriminant ([`BrushBlend::to_u8`]).
    pub blend: u8,
    /// Eraser mode — paints with Erase Alpha regardless of [`Self::blend`].
    pub eraser: bool,
    /// Active-op flags: **Smear**/**Blur**/**Clone** process pixels (no colour → hide colour controls +
    /// incremental-only methods; [`Self::paints_no_color`]); **Mask** paints a grayscale value (keeps all methods).
    pub is_smear: bool,
    pub is_blur: bool,
    pub is_clone: bool,
    pub is_mask: bool,
    /// Clone flags: source sampled? · **Aligned** (offset persists across strokes)? · "Set Source" armed?
    pub clone_has_source: bool,
    pub clone_aligned: bool,
    pub clone_sample_armed: bool,
    /// **Composite Brush** on: the Strength slider hides + the 3-layer stack card shows (panel).
    pub composite_enabled: bool,
    /// Composite stack op per position `[layer1, layer2, layer3]` (`CompositeOp::to_u8`: 0 Brush/1 Smear/2 Blur).
    pub composite_ops: [u8; 3],
    /// Composite stack Strength per position `[layer1, layer2, layer3]` (`0..1`).
    pub composite_strength: [f32; 3],
    /// Seamless **Tiling** (wrap-around painting) flags `[x, y]`.
    pub tiling: [bool; 2],
    /// **Repeat Image** tile-preview toggle (the on-canvas 3×3 grid).
    pub repeat_image: bool,

    // ── Symmetry (drawing mirror / radial; section above Tiling). `symmetry_axis` = `MirrorAxis::to_u8`
    //    (0 X / 1 Y / 2 Custom); segments 3..12; the two `*_pick_*` flags = a canvas pick mode is armed. ──
    pub symmetry_enabled: bool,
    pub symmetry_circular: bool,
    pub symmetry_axis: u8,
    pub symmetry_segments: u32,
    pub symmetry_pick_line: bool,
    pub symmetry_pick_center: bool,

    // ── Stroke section (raw values; the panel maps to slider tracks via the BRUSH_*_MAX consts) ──
    /// Stroke-method wire discriminant ([`StrokeMethod::to_u8`]).
    pub stroke_method: u8,
    /// Spacing as a fraction of diameter (`0.10` = 10%); the slider track is this value.
    pub spacing: f32,
    /// **Offset** slider track (`0..1`, `0.5` = no offset) — perpendicular path offset for the shape editors.
    pub offset: f32,
    /// **Trim** the offset's self-intersections (Offset card checkbox).
    pub offset_trim: bool,
    /// Whether the Simplify button shows: a curve is editing AND (Free Hand, or the user has added a point).
    pub can_simplify: bool,
    /// A curve with points is being edited (Curve / Free Hand / converted shape) — gates the Save-As-Object button.
    pub has_drawn_curve: bool,
    /// "Adjust Strength for Spacing" on/off.
    pub space_attenuation: bool,
    /// **Accumulate** on/off: off (default) caps a stroke at Strength.
    pub accumulate: bool,
    /// Relative jitter (`0..1`, fraction of diameter) — the Jitter slider under the Brush unit.
    pub jitter: f32,
    /// Absolute jitter in pixels — the Jitter slider under the View unit.
    pub jitter_absolute_px: f32,
    /// Jitter-unit wire discriminant ([`JitterUnit::to_u8`]; `0` = Brush, `1` = View).
    pub jitter_unit: u8,
    /// Dash on-fraction (`0..1`).
    pub dash_ratio: f32,
    /// Dash period in dab-slots.
    pub dash_samples: u32,
    /// Input-samples averaging window (`>= 1`).
    pub input_samples: u32,
    /// Stroke stabilizer intensity, `0..1` (the "how regular" knob).
    pub stabilizer: f32,
    /// Airbrush "Rate" — emission period in seconds (Airbrush only; track via `BRUSH_AIRBRUSH_RATE_*_S`).
    pub airbrush_rate_s: f32,
    /// "Edge to Edge" toggle — Anchored only (the stamp spans anchor→cursor, not grows from the anchor).
    pub edge_to_edge: bool,

    // ── Texture section (the brush texture mask; raw values — the panel maps to slider tracks) ──
    /// Texture kind wire discriminant ([`TextureKind::to_u8`]; `0` = None = no texture assigned).
    pub texture_kind: u8,
    /// Texture mapping wire discriminant ([`TextureMapping::to_u8`]).
    pub texture_mapping: u8,
    /// Texture rotation in whole degrees (`0..=360`).
    pub texture_angle_deg: u16,
    /// "Rake" — the texture rotation follows the stroke direction.
    pub texture_rake: bool,
    /// "Random" — the texture rotation is randomised per dab.
    pub texture_random: bool,
    /// Texture offset in tile fractions, per axis (`−1..1`).
    pub texture_offset: [f32; 2],
    /// Texture per-axis scale (`0.1..10`; `1.0` = one tile).
    pub texture_size: [f32; 2],
    /// **Stencil** rect centre, per axis (`−1..1`) — the gizmo placement, independent of the texture tiling.
    pub stencil_offset: [f32; 2],
    /// **Stencil** rect half-extent as a canvas fraction, per axis (`0.1..10`, default `0.5`); Stencil mapping only.
    pub stencil_size: [f32; 2],
    /// **Stencil** rect rotation in whole degrees (`0..=360`). Independent of [`Self::texture_angle_deg`].
    pub stencil_angle_deg: u16,
    /// Per-pattern parameter slots, normalized `[0, 1]`; meaning per kind (`param_specs`).
    pub texture_params: [f32; ph2d_painter_brush::MAX_TEX_PARAMS],
    /// **Grain Depth** (`0..1`; `1` = full bite, the default). How strongly the Grain modulates.
    pub grain_depth: f32,

    // ── Shape section (the silhouette tip; the falloff is its procedural default) ──
    /// Shape **source** kind (`TextureKind::to_u8`): `None` falloff · `Image` replaces it · procedural is masked. Drives the panel "Texture" picker.
    pub shape_kind: u8,
    /// Whether a Shape **image** is assigned (meaningful only when [`Self::shape_kind`] is `Image`).
    pub shape_has_image: bool,
    /// Shape rotation in whole degrees (`0..=360`).
    pub shape_angle_deg: u16,
    /// "Rake" — the Shape rotation follows the stroke direction.
    pub shape_rake: bool,
    /// "Random" — the Shape rotation is randomised per dab.
    pub shape_random: bool,
    /// Shape offset in tile fractions, per axis (`−1..1`).
    pub shape_offset: [f32; 2],
    /// Shape per-axis scale (`0.1..10`; `1.0` = the image fills the footprint).
    pub shape_size: [f32; 2],
    /// Procedural Shape per-pattern params (Contrast / Brightness + the kind's knob, `[0,1]`) — the Grain's `texture_params` twin for the Shape slot.
    pub shape_params: [f32; ph2d_painter_brush::MAX_TEX_PARAMS],
    /// Number of captured Shape layers (`0` = single-image / falloff; `> 1` shows the Per-Layer Color UI).
    pub shape_layer_count: u8,
    /// Capturable layers in the active document (visible top-level rasters); "Use Document Layers" shows when `> 1`.
    pub document_layer_count: u8,
    /// "Per-Layer Color" mode — each Shape layer paints its own colour, higher above lower; hides the ramp.
    pub shape_per_layer_color: bool,
    /// Per-layer "use a custom colour" toggle (entries `0..shape_layer_count` valid).
    pub shape_layer_color_on: [bool; MAX_SHAPE_LAYERS],
    /// Per-layer custom colour (straight RGB), used when [`Self::shape_layer_color_on`]`[i]`. With the
    /// checkbox OFF (default) the layer paints its own captured texture colour instead.
    pub shape_layer_color: [[f32; 3]; MAX_SHAPE_LAYERS],
    /// Per-layer blend mode ([`ph2d_painter_effects::BlendMode`] discriminant; the "B" chip).
    pub shape_layer_blend: [u8; MAX_SHAPE_LAYERS],
    /// Per-layer **opacity** `0..1` — a BRUSH-only scale on that layer's tip contribution (the numeric
    /// box), seeded from the captured document layer's opacity. Does NOT edit the painted document.
    pub shape_layer_opacity: [f32; MAX_SHAPE_LAYERS],
    /// **Dab Flatten** (`0..1`; `0` = round) — the Shape gizmo squishes the dab footprint into an ellipse.
    pub dab_flatten: f32,
    /// **Dab rotation** of the flatten/rotate gizmo, whole degrees (`0..=360`).
    pub dab_angle_deg: u16,

    // ── Grain Colour Ramp (maps the Grain scalar to a colour when enabled) ──
    /// Whether the Color Ramp drives the paint colour.
    pub texture_ramp_enabled: bool,
    /// **B&W** filter: desaturate the ramp to luminance (paint + display).
    pub texture_ramp_bw: bool,
    /// Ramp colour-interpolation space (`RampColorMode::to_u8`).
    pub texture_ramp_mode: u8,
    /// Ramp interpolation mode (`RampInterp::to_u8`).
    pub texture_ramp_interp: u8,
    /// Ramp stops `(pos, r, g, b, a, id)` display sRGB, first [`Self::texture_ramp_stop_count`] valid, sorted by `pos`.
    pub texture_ramp_stops: [[f32; 6]; PANEL_RAMP_STOPS],
    pub texture_ramp_stop_count: u8,
    /// Ramp alpha action (`RampAlphaMode::to_u8`): `0` off · `1` scales Strength · `2` drives sprite alpha.
    pub texture_ramp_alpha_mode: u8,

    // ── Shape Colour Ramp — colour twin of the Grain ramp (same fields). B&W off = owns colour, on = tone. ──
    pub shape_color_ramp_enabled: bool,
    pub shape_color_ramp_bw: bool,
    pub shape_color_ramp_mode: u8,
    pub shape_color_ramp_interp: u8,
    pub shape_color_ramp_stops: [[f32; 6]; PANEL_RAMP_STOPS],
    pub shape_color_ramp_stop_count: u8,
    pub shape_color_ramp_alpha_mode: u8,

    /// Per-dab randomize: Randomize-Color enable + HSV amounts + Jitter Scale/Rotate/Spacing (`0..1`).
    pub color_jitter_enabled: bool,
    pub color_jitter: [f32; 3],
    pub jitter_scale: f32,
    pub jitter_rotate: f32,
    pub jitter_spacing: f32,
}

/// Max ramp stops the panel snapshot carries (a ramp may hold up to `MAX_RAMP_STOPS = 32`; the editor
/// shows the first this-many — more than enough for hand-authored gradients).
pub const PANEL_RAMP_STOPS: usize = 16;

/// Map a radius in pixels onto the size slider's `0..1` track (inverse of [`size_norm_to_px`]).
/// Squared track → finer control at small sizes.
pub(super) fn size_px_to_norm(px: f32) -> f32 {
    let span = BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX;
    ((px - BRUSH_SIZE_MIN_PX) / span).clamp(0.0, 1.0).sqrt()
}

/// Map the size slider's `0..1` track onto a radius in pixels.
fn size_norm_to_px(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    BRUSH_SIZE_MIN_PX + t * t * (BRUSH_SIZE_MAX_PX - BRUSH_SIZE_MIN_PX)
}

/// Map a slider's `0..1` track onto a count in `1..=BRUSH_COUNT_SLIDER_MAX` (Input Samples /
/// Dash Length). Inverse of `count_to_norm` in the panel.
fn count_from_norm(t: f32) -> u32 {
    let span = (BRUSH_COUNT_SLIDER_MAX - 1) as f32;
    1 + (t.clamp(0.0, 1.0) * span).round() as u32
}

impl PainterTool {
    /// Set the brush distance-falloff preset from a wire discriminant (out-of-range → Smooth). `9` =
    /// the editable `Custom` curve ([`Self::set_brush_falloff_point`]).
    pub fn set_brush_falloff(&mut self, preset: u8) {
        self.paint.brush.falloff = Falloff::from_u8(preset);
    }

    /// Move `Custom` falloff control point `id` to `(distance, strength)` in `[0, 1]²` — may pass its
    /// neighbours (curve re-sorts; the stable `id` keeps the handle grabbed). Pure brush state.
    pub fn set_brush_falloff_point(&mut self, id: u8, distance: f32, strength: f32) {
        self.paint
            .brush
            .custom_falloff
            .set_point(id, distance, strength);
    }

    /// Insert a `Custom` falloff point at the widest gap; returns the new id, or `None` at the cap (panel "+").
    pub fn add_brush_falloff_point(&mut self) -> Option<u8> {
        self.paint.brush.custom_falloff.add_point()
    }

    /// Insert a `Custom` falloff control point at `(distance, strength)` — where the artist clicked.
    /// Returns the new stable id, or `None` at the point cap.
    pub fn add_brush_falloff_point_at(&mut self, distance: f32, strength: f32) -> Option<u8> {
        self.paint
            .brush
            .custom_falloff
            .add_point_at(distance, strength)
    }

    /// Set the handle type of `Custom` falloff control point `id` (`0` = Auto,
    /// `1` = Vector). Drives the right-click handle menu.
    pub fn set_brush_falloff_point_handle(&mut self, id: u8, handle: u8) {
        self.paint
            .brush
            .custom_falloff
            .set_handle(id, HandleType::from_u8(handle));
    }

    /// Remove `Custom` falloff control point `id` (no-op when only the two
    /// endpoints remain). Drives the panel's "−" button + the Delete key.
    pub fn remove_brush_falloff_point(&mut self, id: u8) {
        self.paint.brush.custom_falloff.remove_point(id);
    }

    /// Set the brush strength (`0..1`, overall opacity).
    pub fn set_brush_strength(&mut self, t: f32) {
        self.paint.brush.strength = t.clamp(0.0, 1.0);
    }

    /// Toggle eraser mode (overrides the blend with Erase Alpha while on).
    pub fn toggle_brush_eraser(&mut self) {
        self.paint.eraser = !self.paint.eraser;
    }

    // The paint-mode setters (`set_paint_tool_mode` / `is_smear_mode`) live in `stencil.rs`, beside the
    // `route_brush_dab_event` that drives them (workspace LOC cap on this file).

    /// Set the brush radius in pixels, clamped to the interactive size range.
    pub fn set_brush_size_px(&mut self, px: f32) {
        self.paint.brush.radius_px = px.clamp(BRUSH_SIZE_MIN_PX, BRUSH_SIZE_MAX_PX);
    }

    /// Set the brush radius from the size slider's `0..1` track.
    pub fn set_brush_size_norm(&mut self, t: f32) {
        self.set_brush_size_px(size_norm_to_px(t));
    }

    /// Nudge the brush radius by one step — `[` (`dir < 0`) / `]` (`dir >= 0`). Multiplicative (constant
    /// perceptual step) with a ±1 px floor so the smallest brushes still change. Returns the new px.
    pub fn nudge_brush_size(&mut self, dir: i32) -> f32 {
        const STEP: f32 = 1.15;
        let cur = self.paint.brush.radius_px;
        let next = if dir >= 0 {
            (cur * STEP).max(cur + 1.0)
        } else {
            (cur / STEP).min(cur - 1.0)
        };
        self.set_brush_size_px(next);
        self.paint.brush.radius_px
    }

    /// Set one straight-RGB colour channel (`0..3`) of the brush, clamped `0..1`.
    pub fn set_brush_color_channel(&mut self, ch: usize, v: f32) {
        if ch < 3 {
            self.paint.brush.color[ch] = v.clamp(0.0, 1.0);
        }
    }

    /// Set the brush blend mode from a wire discriminant (out-of-range → Mix).
    pub fn set_brush_blend(&mut self, mode: u8) {
        self.paint.brush.blend = BrushBlend::from_u8(mode);
    }

    // ── Stroke section setters (the single clamp source; the panel forwards raw UI values) ──

    /// Set the stroke method from a wire discriminant (out-of-range → Space). Leaving a shape method
    /// (Curve/Circle) with an un-committed session discards it (revert the preview) — the artist
    /// switched away deliberately. Switching INTO the same shape keeps its session.
    pub fn set_brush_stroke_method(&mut self, m: u8) {
        let method = StrokeMethod::from_u8(m);
        if method != StrokeMethod::Curve {
            self.curve_cancel();
        }
        if method != StrokeMethod::Circle {
            self.circle_cancel();
        }
        if method != StrokeMethod::Polygon {
            self.polygon_cancel();
        }
        self.paint.brush.stroke_method = method;
    }

    /// Set spacing as a fraction of diameter (slider track), clamped to the interactive range.
    pub fn set_brush_spacing(&mut self, frac: f32) {
        self.paint.brush.spacing = frac.clamp(0.01, BRUSH_SPACING_MAX);
    }

    /// Toggle "Adjust Strength for Spacing".
    pub fn toggle_brush_space_attenuation(&mut self) {
        self.paint.brush.space_attenuation = !self.paint.brush.space_attenuation;
    }

    /// Toggle **Accumulate** (off caps a stroke at Strength; on lets overlapping dabs build up).
    pub fn toggle_brush_accumulate(&mut self) {
        self.paint.brush.accumulate = !self.paint.brush.accumulate;
    }

    /// Set the Jitter slider (`0..1` track), routed by the current unit: `Brush` → relative jitter
    /// (`0..1`), `View` → absolute pixels (`track × BRUSH_JITTER_ABS_MAX_PX`).
    pub fn set_brush_jitter_norm(&mut self, t: f32) {
        let t = t.clamp(0.0, 1.0);
        match self.paint.brush.jitter_unit {
            JitterUnit::View => self.paint.brush.jitter_absolute_px = t * BRUSH_JITTER_ABS_MAX_PX,
            JitterUnit::Brush => self.paint.brush.jitter = t,
        }
    }

    /// Set the jitter unit from a wire discriminant (out-of-range → Brush).
    pub fn set_brush_jitter_unit(&mut self, u: u8) {
        self.paint.brush.jitter_unit = JitterUnit::from_u8(u);
    }

    /// Set the dash on-fraction (`0..1`).
    pub fn set_brush_dash_ratio(&mut self, t: f32) {
        self.paint.brush.dash_ratio = t.clamp(0.0, 1.0);
    }

    /// Set the dash period from the slider's `0..1` track → `1..=BRUSH_COUNT_SLIDER_MAX` slots.
    pub fn set_brush_dash_length_norm(&mut self, t: f32) {
        self.paint.brush.dash_samples = count_from_norm(t);
    }

    /// Set the input-samples window from the slider's `0..1` track → `1..=BRUSH_COUNT_SLIDER_MAX`.
    pub fn set_brush_input_samples_norm(&mut self, t: f32) {
        self.paint.brush.input_samples = count_from_norm(t);
    }

    /// Set the stroke stabilizer intensity from the slider's `0..1` track (the "how regular" knob).
    pub fn set_brush_stabilizer(&mut self, t: f32) {
        self.paint.brush.stabilizer = t.clamp(0.0, 1.0);
    }

    /// Set the airbrush **Rate** (timer period, seconds) from the slider's `0..1` track, mapped
    /// linearly onto `[BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_AIRBRUSH_RATE_MAX_S]` (default `0.1`).
    pub fn set_brush_airbrush_rate_norm(&mut self, t: f32) {
        let t = t.clamp(0.0, 1.0);
        self.paint.brush.airbrush_rate_s =
            BRUSH_AIRBRUSH_RATE_MIN_S + t * (BRUSH_AIRBRUSH_RATE_MAX_S - BRUSH_AIRBRUSH_RATE_MIN_S);
    }

    /// Toggle "Edge to Edge" (Anchored: the stamp spans anchor→cursor instead of growing from it).
    pub fn toggle_brush_edge_to_edge(&mut self) {
        self.paint.brush.edge_to_edge = !self.paint.brush.edge_to_edge;
    }

    /// Set the brush texture (Grain) kind from a wire discriminant (out-of-range → None). Picking
    /// [`TextureKind::Image`] requests a file pick from the shell (the engine has no I/O).
    pub fn set_brush_texture_kind(&mut self, k: u8) {
        let was_none = self.paint.brush.texture.kind == TextureKind::None;
        let kind = TextureKind::from_u8(k);
        self.paint.brush.texture.kind = kind;
        self.reset_texture_params();
        if kind == TextureKind::Image {
            self.paint.texture_image_pending = true;
        }
        if was_none && kind != TextureKind::None {
            self.on_grain_assigned(); // None→Grain: flip Shape colour→tone + reset the Grain ramp
        }
        self.arm_stencil_preview();
    }

    /// Assign the default procedural texture (Noise) — the Texture section's "New" button.
    pub fn new_brush_texture(&mut self) {
        let was_none = self.paint.brush.texture.kind == TextureKind::None;
        self.paint.brush.texture.kind = TextureKind::Noise;
        self.reset_texture_params();
        if was_none {
            self.on_grain_assigned(); // None→Grain: flip Shape colour→tone + reset the Grain ramp
        }
    }

    /// Set the texture mapping from a wire discriminant (out-of-range → View Plane). Re-fits a loaded
    /// Grain Image's aspect for the new mapping (Stencil → the rect; the rest → the Size), so the image
    /// is never squashed in any mode (Enio 2026-06-28).
    pub fn set_brush_texture_mapping(&mut self, m: u8) {
        let m = TextureMapping::from_u8(m);
        self.paint.brush.texture.mapping = m;
        self.fit_grain_image_aspect(m);
        self.arm_stencil_preview();
    }

    /// Set the texture rotation from the slider's `0..1` track → `0..=TEX_ANGLE_MAX_DEG` degrees.
    pub fn set_brush_texture_angle_norm(&mut self, t: f32) {
        self.paint.brush.texture.angle_deg =
            (t.clamp(0.0, 1.0) * f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
    }

    /// Toggle "Rake" (the texture rotation follows the stroke direction).
    pub fn toggle_brush_texture_rake(&mut self) {
        let tex = &mut self.paint.brush.texture;
        tex.rake = !tex.rake;
    }

    /// Toggle "Random" (the texture rotation is randomised per dab).
    pub fn toggle_brush_texture_random(&mut self) {
        let tex = &mut self.paint.brush.texture;
        tex.random_angle = !tex.random_angle;
    }

    /// Set the texture offset for `axis` (`0`=X / `1`=Y) from the `0..1` track → `[TEX_OFFSET_MIN, MAX]` (tiles).
    pub fn set_brush_texture_offset_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_OFFSET_MAX - TEX_OFFSET_MIN;
            self.paint.brush.texture.offset[axis] = TEX_OFFSET_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    /// Set the texture scale for `axis` (`0`=X / `1`=Y) from the `0..1` track → `[TEX_SIZE_MIN, TEX_SIZE_MAX]`.
    pub fn set_brush_texture_size_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_SIZE_MAX - TEX_SIZE_MIN;
            self.paint.brush.texture.size[axis] = TEX_SIZE_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    /// Set per-pattern parameter `slot` from the `0..1` track (normalized; each pattern maps its own range).
    pub fn set_brush_texture_param_norm(&mut self, slot: usize, t: f32) {
        if slot < ph2d_painter_brush::MAX_TEX_PARAMS {
            self.paint.brush.texture.params[slot] = t.clamp(0.0, 1.0);
        }
        self.arm_stencil_preview();
    }

    /// Enable / disable the texture **Color Ramp** (on → the texture's scalar drives the per-texel colour).
    pub fn set_texture_ramp_enabled(&mut self, on: bool) {
        self.paint.texture_ramp_enabled = on;
    }

    /// Replace the texture Color Ramp (`ph2d_color::ColorRamp`); re-bakes the LUT before the next stamp.
    pub fn set_texture_ramp(&mut self, ramp: ph2d_color::ColorRamp) {
        self.paint.texture_ramp = ramp;
        self.paint.texture_ramp_dirty = true;
    }

    /// The current texture Color Ramp (for the panel widget + tests).
    #[must_use]
    pub fn texture_ramp(&self) -> &ph2d_color::ColorRamp {
        &self.paint.texture_ramp
    }

    /// Whether the texture Color Ramp is enabled.
    #[must_use]
    pub fn texture_ramp_enabled(&self) -> bool {
        self.paint.texture_ramp_enabled
    }

    /// Reset the texture params to the active kind's `param_specs` defaults (unused slots stay at the
    /// neutral `0.5`). Called on a kind change so each pattern starts from its own sensible values.
    fn reset_texture_params(&mut self) {
        let mut params = [0.5; ph2d_painter_brush::MAX_TEX_PARAMS];
        for (i, s) in ph2d_painter_brush::param_specs(self.paint.brush.texture.kind)
            .iter()
            .enumerate()
        {
            params[i] = s.default;
        }
        self.paint.brush.texture.params = params;
    }

    /// Set the absolute texture offset for `axis` (tile fractions, clamped) — used by the Stencil
    /// drag gesture, which computes a target value directly rather than a slider track.
    pub fn set_brush_texture_offset(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.offset[axis] = v.clamp(TEX_OFFSET_MIN, TEX_OFFSET_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the absolute texture scale for `axis` (clamped) — used by the Stencil drag gesture.
    pub fn set_brush_texture_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the texture rotation directly in whole **degrees** (the number-field path, Enio 2026-06-25).
    pub fn set_brush_texture_angle(&mut self, deg: f32) {
        self.paint.brush.texture.angle_deg =
            deg.clamp(0.0, f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
        self.arm_stencil_preview();
    }

    /// Set the **Stencil** rect centre for `axis` (`−1..1`, clamped) — the gizmo's own offset, separate
    /// from the texture tiling. Driven by both the Stencil card's number box and the on-canvas drag.
    pub fn set_brush_stencil_offset(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.stencil_offset[axis] = v.clamp(TEX_OFFSET_MIN, TEX_OFFSET_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the **Stencil** rect half-extent fraction for `axis` (`0.1..10`, clamped; `0.5` = 50 % of
    /// the sprite) — the gizmo's own size, separate from the texture tiling.
    pub fn set_brush_stencil_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.stencil_size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
        self.arm_stencil_preview();
    }

    /// Set the **Stencil** rect rotation directly in whole **degrees** — the gizmo's own angle.
    pub fn set_brush_stencil_angle(&mut self, deg: f32) {
        self.paint.brush.texture.stencil_angle_deg =
            deg.clamp(0.0, f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
        self.arm_stencil_preview();
    }

    /// Set the **Dab Flatten** (`0..1`, clamped) — the Shape-panel gizmo squishes the dab footprint
    /// (falloff + Shape + View-Grain) into an ellipse. The engine clamps the effective minor axis.
    pub fn set_brush_dab_flatten(&mut self, v: f32) {
        self.paint.brush.dab_flatten = v.clamp(0.0, 1.0);
    }

    /// Set the **Dab rotation** of the flatten/rotate gizmo in whole **degrees**.
    pub fn set_brush_dab_angle(&mut self, deg: f32) {
        self.paint.brush.dab_angle_deg =
            deg.clamp(0.0, f32::from(TEX_ANGLE_MAX_DEG)).round() as u16;
    }
}
