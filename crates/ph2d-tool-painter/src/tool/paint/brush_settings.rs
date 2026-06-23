//! Brush + Stroke-section parameter snapshot & setters, split out of `paint.rs` to keep both
//! files under the workspace LOC cap (HR-18). A submodule of `paint` so it shares its private
//! `PaintState`/`BrushSpec` access; the setters are the single clamp source for every UI edit.

use super::{
    BRUSH_AIRBRUSH_RATE_MAX_S, BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_COUNT_SLIDER_MAX,
    BRUSH_JITTER_ABS_MAX_PX, BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX, BRUSH_SPACING_MAX,
};
use crate::tool::PainterTool;
use ph2d_painter_brush::{
    BrushBlend, Dab, Falloff, FalloffPoint, HandleType, ImageMask, JitterUnit, MAX_FALLOFF_POINTS,
    StrokeMethod, TEX_ANGLE_MAX_DEG, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN,
    TextureKind, TextureMapping, eval_falloff_curve,
};

/// An imported brush-texture image: owned grayscale luminance + dims, held in
/// [`super::PaintState`] (the heavy pixels can't live in the `Copy` `BrushSpec`). [`stamp_dabs`]
/// borrows it as an [`ImageMask`] for the engine.
pub(super) struct BrushTextureImage {
    lum: Vec<u8>,
    width: u32,
    height: u32,
}

impl BrushTextureImage {
    /// Borrow as the engine's [`ImageMask`].
    pub(super) fn as_mask(&self) -> ImageMask<'_> {
        ImageMask {
            lum: &self.lum,
            width: self.width,
            height: self.height,
        }
    }
}

/// Snap `cursor` to the nearest 45° ray from `anchor` (Blender Line Alt-constrain), projecting the
/// cursor onto that ray. Transcendental-free (only abs/signum/compare/mul — `tan(22.5°)≈0.4142`,
/// `tan(67.5°)≈2.4142` are the sector boundaries, `√½` the diagonal unit), so it stays
/// bit-deterministic across platforms (HR-5) instead of going through `atan2`/`sin`/`cos`. Lives
/// here (with the other stroke helpers) to keep `paint.rs` under the workspace LOC cap.
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

/// The stroke direction at dab `i`, as a raw (un-normalised) vector — the texture's **Rake**
/// rotation aligns to it ([`ph2d_painter_brush::texture::dab_basis`] normalises). Uses the chord to
/// the next dab when there is one, else from the previous; a lone dab returns `[0, 0]` (Rake then
/// falls back to the Angle). Derived here rather than carried on `Dab` so the engine's dab stays
/// pressure-only.
pub(super) fn dab_tangent(dabs: &[Dab], i: usize) -> [f32; 2] {
    let (a, b) = if i + 1 < dabs.len() {
        (dabs[i].center, dabs[i + 1].center)
    } else if i > 0 {
        (dabs[i - 1].center, dabs[i].center)
    } else {
        return [0.0, 0.0];
    };
    [b[0] - a[0], b[1] - a[1]]
}

/// A compact snapshot of the active brush for the layers panel's Brush section.
/// Published each frame by the shell bridge (mirror of the `LayerStack`
/// snapshot) — the panel reads it to position the size/colour sliders and the
/// blend chip; it never owns brush state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSettings {
    /// Radius in image pixels (UI label "Size").
    pub size_px: f32,
    /// [`Self::size_px`] mapped onto the size slider's `0..1` track (squared, so
    /// small brushes get more of the track).
    pub size_norm: f32,
    /// Overall opacity, `0..1` (UI "Strength").
    pub strength: f32,
    /// Distance-falloff preset wire discriminant ([`Falloff::to_u8`]) — Blender's
    /// "Falloff Curve Preset". Defines the dab profile (replaces a Hardness slider).
    /// [`Falloff::Custom`] (`9`) reads [`Self::falloff_points`].
    pub falloff: u8,
    /// The `Custom` falloff curve's control points (id + `[distance, strength]` +
    /// handle), the first [`Self::falloff_len`] valid, ascending by distance. The
    /// panel plots these + places draggable handles (keyed by the stable id) when
    /// `falloff == Custom`.
    pub falloff_points: [FalloffPoint; MAX_FALLOFF_POINTS],
    /// Count of valid entries in [`Self::falloff_points`] (`2..=MAX_FALLOFF_POINTS`).
    pub falloff_len: u8,
    /// Straight-RGB paint colour in `[0, 1]`.
    pub color: [f32; 3],
    /// Blend-mode wire discriminant ([`BrushBlend::to_u8`]).
    pub blend: u8,
    /// Eraser mode — paints with Erase Alpha regardless of [`Self::blend`].
    pub eraser: bool,

    // ── Stroke section (raw values; the panel maps to slider tracks via the BRUSH_*_MAX consts) ──
    /// Stroke-method wire discriminant ([`StrokeMethod::to_u8`]).
    pub stroke_method: u8,
    /// Spacing as a fraction of diameter (`0.10` = 10%); the slider track is this value.
    pub spacing: f32,
    /// "Adjust Strength for Spacing" on/off.
    pub space_attenuation: bool,
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
    /// Airbrush emission period in seconds (the "Rate" slider; only meaningful for the Airbrush
    /// method). The panel maps it onto the slider track via `BRUSH_AIRBRUSH_RATE_{MIN,MAX}_S`.
    pub airbrush_rate_s: f32,
    /// "Edge to Edge" toggle — Anchored only (the stamp spans anchor→cursor instead of growing
    /// from the anchor).
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
    /// Per-pattern parameter slots, normalized `[0, 1]`; meaning per kind (see
    /// `ph2d_painter_brush::param_specs`). The panel paints a slider per exposed slot.
    pub texture_params: [f32; ph2d_painter_brush::MAX_TEX_PARAMS],

    // ── Texture Color Ramp (maps the texture's scalar to a colour when enabled) ──
    /// Whether the Color Ramp drives the paint colour.
    pub texture_ramp_enabled: bool,
    /// Ramp colour-interpolation space (`RampColorMode::to_u8`).
    pub texture_ramp_mode: u8,
    /// Ramp interpolation mode (`RampInterp::to_u8`).
    pub texture_ramp_interp: u8,
    /// Ramp stops as `(pos, r, g, b, a)` (LINEAR), the first [`Self::texture_ramp_stop_count`] valid,
    /// sorted by `pos`. The panel plots the bar + a draggable handle per stop.
    pub texture_ramp_stops: [[f32; 5]; PANEL_RAMP_STOPS],
    /// Count of valid entries in [`Self::texture_ramp_stops`].
    pub texture_ramp_stop_count: u8,
}

/// Max ramp stops the panel snapshot carries (a ramp may hold up to `MAX_RAMP_STOPS = 32`; the editor
/// shows the first this-many — more than enough for hand-authored gradients).
pub const PANEL_RAMP_STOPS: usize = 16;

/// Strength of the brush's active falloff at normalized distance `t` (`0` =
/// centre, `1` = rim), for the panel's live curve preview. Reads the editable
/// [`BrushSettings::falloff_points`] when the `Custom` preset is selected, else
/// the matching [`Falloff`] formula — so the graph the panel draws matches the
/// dab the engine stamps.
#[must_use]
pub fn brush_falloff_weight_at(s: &BrushSettings, t: f32) -> f32 {
    if s.falloff == Falloff::Custom.to_u8() {
        eval_falloff_curve(&s.falloff_points[..s.falloff_len as usize], t)
    } else {
        Falloff::from_u8(s.falloff).weight(t)
    }
}

/// Map a radius in pixels onto the size slider's `0..1` track (inverse of [`size_norm_to_px`]).
/// Squared track → finer control at small sizes.
fn size_px_to_norm(px: f32) -> f32 {
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
    /// Snapshot the active brush for the panel's Brush section.
    #[must_use]
    pub fn brush_settings(&self) -> BrushSettings {
        let b = &self.paint.brush;
        // Snapshot the Custom curve's control points into the Copy array (the
        // panel reads these to plot + place handles when the Custom preset is on).
        let mut falloff_points = [FalloffPoint::default(); MAX_FALLOFF_POINTS];
        let pts = b.custom_falloff.points();
        falloff_points[..pts.len()].copy_from_slice(pts);
        // Snapshot the texture Color Ramp's stops into the Copy array (the panel plots the bar +
        // draggable handles).
        let ramp = self.texture_ramp();
        let mut texture_ramp_stops = [[0.0f32; 5]; PANEL_RAMP_STOPS];
        let ramp_count = ramp.stops().len().min(PANEL_RAMP_STOPS);
        // Stops are stored LINEAR; the panel paints in display sRGB, so convert here (alpha straight).
        let srgb = |x: f32| f32::from(ph2d_color::srgb::linear_to_srgb_byte(x)) / 255.0;
        for (slot, s) in texture_ramp_stops.iter_mut().zip(ramp.stops()) {
            *slot = [
                s.pos,
                srgb(s.color[0]),
                srgb(s.color[1]),
                srgb(s.color[2]),
                s.color[3],
            ];
        }
        BrushSettings {
            size_px: b.radius_px,
            size_norm: size_px_to_norm(b.radius_px),
            strength: b.strength,
            falloff: b.falloff.to_u8(),
            falloff_points,
            falloff_len: b.custom_falloff.len() as u8,
            color: b.color,
            blend: b.blend.to_u8(),
            eraser: self.paint.eraser,
            stroke_method: b.stroke_method.to_u8(),
            spacing: b.spacing,
            space_attenuation: b.space_attenuation,
            jitter: b.jitter,
            jitter_absolute_px: b.jitter_absolute_px,
            jitter_unit: b.jitter_unit.to_u8(),
            dash_ratio: b.dash_ratio,
            dash_samples: b.dash_samples,
            input_samples: b.input_samples,
            stabilizer: b.stabilizer,
            airbrush_rate_s: b.airbrush_rate_s,
            edge_to_edge: b.edge_to_edge,
            texture_kind: b.texture.kind.to_u8(),
            texture_mapping: b.texture.mapping.to_u8(),
            texture_angle_deg: b.texture.angle_deg,
            texture_rake: b.texture.rake,
            texture_random: b.texture.random_angle,
            texture_offset: b.texture.offset,
            texture_size: b.texture.size,
            texture_params: b.texture.params,
            texture_ramp_enabled: self.paint.texture_ramp_enabled,
            texture_ramp_mode: ramp.color_mode.to_u8(),
            texture_ramp_interp: ramp.interp.to_u8(),
            texture_ramp_stops,
            texture_ramp_stop_count: ramp_count as u8,
        }
    }

    /// Set the brush distance-falloff preset from a wire discriminant
    /// (Blender's "Falloff Curve Preset"; out-of-range → Smooth). `9` = the
    /// editable `Custom` curve ([`Self::set_brush_falloff_point`]).
    pub fn set_brush_falloff(&mut self, preset: u8) {
        self.paint.brush.falloff = Falloff::from_u8(preset);
    }

    /// Move `Custom` falloff control point `id` to `(distance, strength)` in
    /// `[0, 1]²`. The point may pass its neighbours (the curve re-sorts and
    /// adapts); the stable `id` keeps the dragged handle grabbed. Pure brush
    /// state — no undo/preview (a brush param change only affects future dabs).
    pub fn set_brush_falloff_point(&mut self, id: u8, distance: f32, strength: f32) {
        self.paint
            .brush
            .custom_falloff
            .set_point(id, distance, strength);
    }

    /// Insert a `Custom` falloff control point at the widest gap (its strength
    /// sampled on the current curve). Returns the new stable id, or `None` at the
    /// point cap. Drives the panel's "+" button.
    pub fn add_brush_falloff_point(&mut self) -> Option<u8> {
        self.paint.brush.custom_falloff.add_point()
    }

    /// Insert a `Custom` falloff control point at `(distance, strength)` — where
    /// the artist clicked on the curve canvas. Returns the new stable id, or
    /// `None` at the point cap.
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

    /// Set the brush radius in pixels, clamped to the interactive size range.
    pub fn set_brush_size_px(&mut self, px: f32) {
        self.paint.brush.radius_px = px.clamp(BRUSH_SIZE_MIN_PX, BRUSH_SIZE_MAX_PX);
    }

    /// Set the brush radius from the size slider's `0..1` track.
    pub fn set_brush_size_norm(&mut self, t: f32) {
        self.set_brush_size_px(size_norm_to_px(t));
    }

    /// Nudge the brush radius by one step — `[` (`dir < 0`) / `]` (`dir >= 0`).
    /// Multiplicative for a constant *perceptual* step, with a ±1 px floor so the
    /// smallest brushes still change. Returns the new radius in pixels.
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

    // ── Texture section setters (the single clamp source; the panel forwards raw UI values) ──

    /// Set the brush texture kind from a wire discriminant (out-of-range → None = no texture).
    /// Picking [`TextureKind::Image`] requests a file pick from the shell (the engine has no I/O);
    /// until one is supplied via [`Self::set_brush_texture_image`] the Image texture is inert.
    pub fn set_brush_texture_kind(&mut self, k: u8) {
        let kind = TextureKind::from_u8(k);
        self.paint.brush.texture.kind = kind;
        self.reset_texture_params();
        if kind == TextureKind::Image {
            self.paint.texture_image_pending = true;
        }
    }

    /// Store an imported grayscale `lum` image (`width × height`, row-major) as the brush texture and
    /// switch the kind to Image. Called by the shell after a file pick + decode.
    pub fn set_brush_texture_image(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.paint.texture_image = Some(BrushTextureImage { lum, width, height });
        self.paint.brush.texture.kind = TextureKind::Image;
        self.paint.texture_image_pending = false;
        // Invalidate the cached stamp's baked Image mask.
        self.paint.texture_image_version = self.paint.texture_image_version.wrapping_add(1);
    }

    /// Take (and clear) the "the user picked Image — open a file picker" request. The shell polls
    /// this each frame; on a successful pick it calls [`Self::set_brush_texture_image`].
    pub fn take_brush_texture_image_request(&mut self) -> bool {
        std::mem::take(&mut self.paint.texture_image_pending)
    }

    /// Assign the default procedural texture (Noise) — the Texture section's "New" button.
    pub fn new_brush_texture(&mut self) {
        self.paint.brush.texture.kind = TextureKind::Noise;
        self.reset_texture_params();
    }

    /// Set the texture mapping from a wire discriminant (out-of-range → View Plane).
    pub fn set_brush_texture_mapping(&mut self, m: u8) {
        self.paint.brush.texture.mapping = TextureMapping::from_u8(m);
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

    /// Set the texture offset for `axis` (`0` = X, `1` = Y) from the slider's `0..1` track, mapped
    /// linearly onto `[TEX_OFFSET_MIN, TEX_OFFSET_MAX]` (tile fractions).
    pub fn set_brush_texture_offset_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_OFFSET_MAX - TEX_OFFSET_MIN;
            self.paint.brush.texture.offset[axis] = TEX_OFFSET_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    /// Set the texture scale for `axis` (`0` = X, `1` = Y) from the slider's `0..1` track, mapped
    /// linearly onto `[TEX_SIZE_MIN, TEX_SIZE_MAX]`.
    pub fn set_brush_texture_size_norm(&mut self, axis: usize, t: f32) {
        if axis < 2 {
            let span = TEX_SIZE_MAX - TEX_SIZE_MIN;
            self.paint.brush.texture.size[axis] = TEX_SIZE_MIN + t.clamp(0.0, 1.0) * span;
        }
    }

    /// Set per-pattern parameter `slot` from the slider's `0..1` track (stored normalized; each
    /// pattern maps its own range — see `ph2d_painter_brush::param_specs`).
    pub fn set_brush_texture_param_norm(&mut self, slot: usize, t: f32) {
        if slot < ph2d_painter_brush::MAX_TEX_PARAMS {
            self.paint.brush.texture.params[slot] = t.clamp(0.0, 1.0);
        }
    }

    /// Enable / disable the texture **Color Ramp**: when on, the texture's scalar drives the per-texel
    /// paint colour (via the ramp) instead of only attenuating the brush's single colour.
    pub fn set_texture_ramp_enabled(&mut self, on: bool) {
        self.paint.texture_ramp_enabled = on;
    }

    /// Replace the texture Color Ramp (the reusable `ph2d_color::ColorRamp`); re-bakes the LUT before
    /// the next ramped stamp.
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
    }

    /// Set the absolute texture scale for `axis` (clamped) — used by the Stencil drag gesture.
    pub fn set_brush_texture_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.texture.size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
    }
}
