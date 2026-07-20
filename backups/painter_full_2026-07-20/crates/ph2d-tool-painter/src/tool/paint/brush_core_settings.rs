//! The brush's **core parameter setters** — falloff (preset + the editable `Custom` curve), size,
//! hardness/strength, spacing, jitter and the dash counts. The single UI-edit clamp point for each.
//!
//! Split out of `brush_settings.rs`, which sat at 694 of the workspace's 700-LOC cap: the Impasto tool
//! list needed two more published fields and there was no room for them. Nothing changed but the file
//! this `impl PainterTool` block lives in; a child module of `paint`, so it keeps the same access.

use super::brush_settings::{count_from_norm, size_norm_to_px};
use super::{
    BRUSH_AIRBRUSH_RATE_MAX_S, BRUSH_AIRBRUSH_RATE_MIN_S, BRUSH_JITTER_ABS_MAX_PX,
    BRUSH_SIZE_MAX_PX, BRUSH_SIZE_MIN_PX, BRUSH_SPACING_MAX,
};
use crate::tool::PainterTool;
use ph2d_painter_brush::{BrushBlend, Falloff, HandleType, JitterUnit};

impl PainterTool {
    /// Set the brush distance-falloff preset from a wire discriminant (out-of-range → Smooth). `9` =
    /// the editable `Custom` curve ([`Self::set_brush_falloff_point`]).
    pub fn set_brush_falloff(&mut self, preset: u8) {
        self.paint.brush.falloff = Falloff::from_u8(preset);
        // The artist spoke: no tool-default arm may override this falloff from here on
        // (`arm_tool_falloff_defaults` — the provenance half of the arming law).
        self.paint.falloff_armed = false;
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

    /// Set the handle type of `Custom` falloff control point `id` (`0` = Auto, `1` = Vector; right-click menu).
    pub fn set_brush_falloff_point_handle(&mut self, id: u8, handle: u8) {
        self.paint
            .brush
            .custom_falloff
            .set_handle(id, HandleType::from_u8(handle));
    }

    /// Remove `Custom` falloff control point `id` (no-op when only the two endpoints remain; "−" / Delete).
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

    // The paint-mode setters (`set_paint_tool_mode` / `is_smear_mode`) live in `stencil.rs` (LOC cap).

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
            self.sync_brush_color_across_modes();
        }
    }

    /// The paint **colour** is shared across EVERY paint mode (unlike the per-mode size / hardness / spacing):
    /// broadcast the live brush colour into every `brush_by_mode` slot so it survives a mode switch. Without
    /// this, a colour picked in one mode was lost when the ColorDrop Fill (or any tool switch) swapped in
    /// another mode's slot — so Fill applied the previous / default (black) colour (Enio 2026-07-04). Mirrors
    /// the Photoshop / Procreate "one foreground colour for all tools" model.
    pub(super) fn sync_brush_color_across_modes(&mut self) {
        let color = self.paint.brush.color;
        for slot in &mut self.paint.brush_by_mode {
            slot.color = color;
        }
    }

    /// The brush paint colour as straight sRGB bytes (`[r, g, b]`). The single source of truth for the paint
    /// colour — used to seed the C&F colour picker + Fill cursor directly, so the picker never falls back to a
    /// stale widget-thumb value (grey / black). Brush = Fill = picker are always this colour (Enio 2026-07-03).
    #[must_use]
    pub fn brush_color_srgb8(&self) -> [u8; 3] {
        let c = self.paint.brush.color;
        [
            (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    /// Set the brush paint colour from straight sRGB bytes — the inverse of [`Self::brush_color_srgb8`].
    /// The C&F picker forwards its live value here every frame (in EVERY mode, and once on the open→close
    /// edge to catch the final pick that closes the picker), so brush = Fill = picker are always the one
    /// colour and Fill never applies the previous colour (Enio 2026-07-03).
    pub fn set_brush_color_srgb8(&mut self, rgb: [u8; 3]) {
        for (ch, &v) in rgb.iter().enumerate() {
            self.paint.brush.color[ch] = f32::from(v) / 255.0;
        }
        self.sync_brush_color_across_modes();
    }

    /// Set the brush blend mode from a wire discriminant (out-of-range → Mix).
    pub fn set_brush_blend(&mut self, mode: u8) {
        self.paint.brush.blend = BrushBlend::from_u8(mode);
    }

    // ── Stroke section setters (the single clamp source; the panel forwards raw UI values) ──

    /// Set spacing as a fraction of diameter (slider track), clamped to the interactive range.
    pub fn set_brush_spacing(&mut self, frac: f32) {
        self.paint.brush.spacing = frac.clamp(0.01, BRUSH_SPACING_MAX);
    }

    // The Inpaint heal-reconstruction setters (`set_inpaint_patch`/`_quality`/`_search`) + their SetValue
    // router live beside the heal itself in `paint::inpaint` (this file is at the workspace LOC cap).

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

    // The Grain-texture / Stencil / Dab setters (set_brush_texture_* / _stencil_* / _dab_*) live in the
    // sibling `brush_texture_settings` module (workspace file-LOC cap).
}
