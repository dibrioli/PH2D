//! **Watercolor** section setters + panel router (the wet-media look: edge darkening + granulation +
//! pigment build-up; no fluid sim — `docs/Painter/08_plano_aquarela_edge_grain_pigment.md`). The single
//! clamp source for those UI edits, mirroring [`super::jitter_settings`]. A submodule of `paint` so it
//! shares `PainterTool`'s private `paint.brush` access. The stored values are consumed at stamp time
//! (granulation / pigment, `ph2d_painter_brush::dab`) and at stroke end (edge darkening).

use super::brush_settings::BrushTextureImage;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::{
    BrushSpec, TEX_OFFSET_MAX, TEX_OFFSET_MIN, TEX_SIZE_MAX, TEX_SIZE_MIN, TextureKind,
    TextureMapping, TextureSettings,
};

/// Drying-Time slider bounds in SECONDS (doc 13 #11) — the wet-session fusion window. Canvas-level
/// (not a brush param); mapped to `PaintState::dry_rate_per_s` as `255 / seconds`.
pub const DRY_TIME_MIN_S: f32 = 2.0;
pub const DRY_TIME_MAX_S: f32 = 60.0;

/// Default **Paper** Size for a PROCEDURAL kind (`set_brush_paper_kind`). The paper is canvas-Tiled
/// (`rel = px·size/256`), so Size `1` = 256-px features = "giant blobs"; this gives ~21-px features (a
/// fine, legible paper tooth) instead. Not finer, because the panel preview shows `3·size` tiles — a much
/// larger Size would mush the preview into an unreadable field (the user tunes finer from here if wanted).
pub const PAPER_PROCEDURAL_DEFAULT_SIZE: f32 = 12.0;

/// True when a Paper kind is a **procedural** pattern (canvas-Tiled, needs a fine default Size), as
/// opposed to a baked 256² **preset** tile, a loaded **Image**, or **None** (all one full tile per 256 px
/// ⇒ Size `1`). Used by [`PainterTool::set_brush_paper_kind`] to default the Size per scale class.
fn is_procedural_paper(kind: TextureKind) -> bool {
    !matches!(
        kind,
        TextureKind::None
            | TextureKind::Image
            | TextureKind::PaperCold
            | TextureKind::PaperRough
            | TextureKind::PaperHot
    )
}

impl PainterTool {
    /// Route the Watercolor section controls (master enable + Pigment toggle + section reset, and the
    /// Edge / Spread / Granulation / Mix sliders) from the layers panel's generic channel to the
    /// setters below. Returns `true` when it consumed the event. Mirrors
    /// [`Self::route_brush_jitter_event`]; called from `handle_panel_event` before the main match.
    pub(crate) fn route_brush_watercolor_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        match event {
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_ENABLE => {
                self.toggle_brush_watercolor();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SHAPE_WATERCOLOR_AUTO => {
                self.toggle_brush_watercolor_shape_auto();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_PIGMENT => {
                self.toggle_brush_pigment();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_RESET => {
                self.reset_brush_watercolor();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_GRAN_SAME => {
                self.toggle_granulation_use_paper();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_PAPER_RESET => {
                self.reset_brush_paper();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_DRY_NOW => {
                self.dry_session_now();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_WATERCOLOR_WET_NOW => {
                self.wet_canvas_now();
                true
            }
            PanelEvent::SetValue(id, v) => {
                let v = *v as f32;
                match *id {
                    x if x == core_ids::PAINTER_WATERCOLOR_EDGE => {
                        self.set_brush_edge_gain(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_SPREAD => {
                        self.set_brush_edge_spread(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_GRANULATION => {
                        self.set_brush_granulation(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_MIX => {
                        self.set_brush_pigment_mixing(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_FILL => {
                        self.set_brush_fill(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_DEPTH => {
                        self.set_brush_depth(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_OPACITY => {
                        self.set_brush_opacity(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_WARP => {
                        self.set_brush_warp(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_SMUDGE => {
                        self.set_brush_wet_smudge(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_WET => {
                        self.set_brush_wet_rewet(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_CHARGE => {
                        self.set_brush_wet_charge(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_DILUTION => {
                        self.set_brush_wet_dilution(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_DRY_TIME => {
                        self.set_dry_time_s(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_WET_PREVIEW => {
                        self.set_wet_preview_intensity(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PULL => {
                        self.set_brush_wet_pull(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_X => {
                        self.set_brush_paper_size(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_SIZE_Y => {
                        self.set_brush_paper_size(1, v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_ANGLE => {
                        self.set_brush_paper_angle(v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_OFFSET_X => {
                        self.set_brush_paper_offset(0, v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_OFFSET_Y => {
                        self.set_brush_paper_offset(1, v);
                        true
                    }
                    x if x == core_ids::PAINTER_WATERCOLOR_PAPER_DEPTH => {
                        self.set_brush_paper_depth(v);
                        true
                    }
                    x if core_ids::PAINTER_WATERCOLOR_PAPER_PARAMS.contains(&x) => {
                        let slot = core_ids::PAINTER_WATERCOLOR_PAPER_PARAMS
                            .iter()
                            .position(|&p| p == x)
                            .unwrap_or(0);
                        self.set_brush_paper_param(slot, v);
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Toggle the **Wet edges** master enable — gates the whole section (edge / granulation / pigment).
    /// Off (default) makes a stroke byte-identical to a plain brush.
    pub fn toggle_brush_watercolor(&mut self) {
        self.paint.brush.watercolor = !self.paint.brush.watercolor;
    }

    /// Toggle the Shape section's **Automatic** (watercolor silhouette, doc 13 #1). Turning it OFF
    /// auto-selects the [`ph2d_painter_brush::Falloff::Watercolor`] preset (the built-in feather as a
    /// curve) so the transition is visually CONTINUOUS — unchecking without touching anything else
    /// paints the same stamp; the user tunes from there. Turning it back ON just restores the
    /// built-in stamp (the falloff choice is kept for the plain brush).
    pub fn toggle_brush_watercolor_shape_auto(&mut self) {
        let b = &mut self.paint.brush;
        b.watercolor_shape_auto = !b.watercolor_shape_auto;
        if !b.watercolor_shape_auto {
            b.falloff = ph2d_painter_brush::Falloff::Watercolor;
        }
    }

    /// Toggle **Pigment** (subtractive Kubelka–Munk wet-on-wet colour mixing).
    pub fn toggle_brush_pigment(&mut self) {
        self.paint.brush.pigment = !self.paint.brush.pigment;
    }

    /// Set the **Edge** darkening gain (the wet-edge "fringe"), clamped to `0..=8`.
    pub fn set_brush_edge_gain(&mut self, v: f32) {
        self.paint.brush.edge_gain = v.clamp(0.0, 8.0);
    }

    /// Set the **Spread** (edge-darkening blur radius in canvas px), clamped to `1..=48`.
    pub fn set_brush_edge_spread(&mut self, v: f32) {
        self.paint.brush.edge_spread = v.clamp(1.0, 48.0);
    }

    /// Set the **Granulation** amount (paper-tooth deposit gate), clamped to `0..=1`.
    pub fn set_brush_granulation(&mut self, v: f32) {
        self.paint.brush.granulation = v.clamp(0.0, 1.0);
    }

    /// Set the **Mix** (subtractive pigment amount), clamped to `0..=1`. Does NOT touch the `pigment`
    /// gate — the panel drives that via [`Self::set_brush_pigment_mixing`]; kept for the tool API.
    pub fn set_brush_pigment_mix(&mut self, v: f32) {
        self.paint.brush.pigment_mix = v.clamp(0.0, 1.0);
    }

    /// Set the merged **Pigment** slider (the panel's single control replacing the old Pigment toggle +
    /// Mix pair, redesign 2026-07-07): `> 0` turns the subtractive-mixing gate ON at that amount; `0`
    /// turns it OFF while KEEPING the last non-zero amount (the `pigment_mix` field is left untouched at
    /// `0`), so sliding back up restores it — zero-loss, the point of the merge. Clamped to `0..=1`.
    pub fn set_brush_pigment_mixing(&mut self, v: f32) {
        let v = v.clamp(0.0, 1.0);
        if v <= 0.0 {
            self.paint.brush.pigment = false;
        } else {
            self.paint.brush.pigment = true;
            self.paint.brush.pigment_mix = v;
        }
    }

    /// Set the render-path **Fill** (wash interior density), clamped to `0..=1`.
    pub fn set_brush_fill(&mut self, v: f32) {
        self.paint.brush.fill = v.clamp(0.0, 1.0);
    }

    /// Set the render-path **Depth** (Beer–Lambert optical-depth scale), clamped to `0.1..=8` (must be
    /// `> 0` so `Tᵢ = pigmentᵢ^(D·depth)` is a real attenuation; the floor keeps a visible wash).
    pub fn set_brush_depth(&mut self, v: f32) {
        self.paint.brush.depth = v.clamp(0.1, 8.0);
    }

    /// Set the render-path **Opacity** (pigment body / hiding power), clamped to `0..=1`. `0` = pure
    /// transparent Beer–Lambert (byte-identical); higher lets light pigments deposit at their hue (#17).
    pub fn set_brush_opacity(&mut self, v: f32) {
        self.paint.brush.opacity = v.clamp(0.0, 1.0);
    }

    /// Set the render-path **Warp** (organic-boundary displacement, canvas px), clamped to `0..=48`.
    pub fn set_brush_warp(&mut self, v: f32) {
        self.paint.brush.warp = v.clamp(0.0, 48.0);
    }

    /// Set the **Wet Mix / smudge** amount (mixer-brush lift+carry vs fresh pigment), clamped to `0..=1`.
    pub fn set_brush_wet_smudge(&mut self, v: f32) {
        self.paint.brush.wet_smudge = v.clamp(0.0, 1.0);
    }

    /// Set the **Wet** amount (per-pixel wet-on-wet lift + dissolve + pool), clamped to `0..=1`.
    pub fn set_brush_wet_rewet(&mut self, v: f32) {
        self.paint.brush.wet_rewet = v.clamp(0.0, 1.0);
    }

    /// Set the Wet Mix **Charge** (fresh-paint reserve; `1` = mixer off), clamped to `0..=1`.
    pub fn set_brush_wet_charge(&mut self, v: f32) {
        self.paint.brush.wet_charge = v.clamp(0.0, 1.0);
    }

    /// Set the Wet Mix **Dilution** (water thinning the deposit), clamped to `0..=1`.
    pub fn set_brush_wet_dilution(&mut self, v: f32) {
        self.paint.brush.wet_dilution = v.clamp(0.0, 1.0);
    }

    /// Set the Wet Mix **Pull** (colour-carry / smudge length), clamped to `0..=1`.
    pub fn set_brush_wet_pull(&mut self, v: f32) {
        self.paint.brush.wet_pull = v.clamp(0.0, 1.0);
    }

    /// Set the paper **Drying Time** in SECONDS (canvas-level, `2..60 s`) → the wetness-bytes/second
    /// rate (`255 / seconds`); the wet-session fusion window (doc 13 #11). Read back via
    /// [`Self::dry_time_s`].
    pub fn set_dry_time_s(&mut self, seconds: f32) {
        let s = seconds.clamp(DRY_TIME_MIN_S, DRY_TIME_MAX_S);
        self.paint.dry_rate_per_s = 255.0 / s;
    }

    /// The current Drying Time in seconds (the panel slider's value) — inverse of the stored rate.
    #[must_use]
    pub fn dry_time_s(&self) -> f32 {
        (255.0 / self.paint.dry_rate_per_s.max(0.001)).clamp(DRY_TIME_MIN_S, DRY_TIME_MAX_S)
    }

    /// Set the on-canvas **Wetness Preview** strength (`0..=1`, `0` = no preview) — the max veil alpha
    /// the shell paints over the wet paper (canvas-level display setting, doc 14 #12a). Read back via
    /// [`Self::wet_preview_intensity`].
    pub fn set_wet_preview_intensity(&mut self, v: f32) {
        self.paint.wet_preview_intensity = v.clamp(0.0, 1.0);
    }

    /// Reset the **Paper** slot to empty (kind `None` → the render-path falls back to the built-in paper
    /// noise), dropping any tagged-layer image. Plain state edit (no undo / pixel touch).
    pub fn reset_brush_paper(&mut self) {
        self.paint.brush.paper = TextureSettings::default();
        self.paint.paper_image = None;
        self.paint.paper_image_version = self.paint.paper_image_version.wrapping_add(1);
    }

    /// Set the **Paper** slot kind (`TextureKind` wire u8) + force canvas-anchored mapping, reset the
    /// params to the kind's `param_specs` defaults (mirroring the Grain slot's `reset_texture_params` —
    /// without it Voronoi rendered with the neutral Randomness `0.5` + Metric `0.5` = Chebyshev square
    /// cells instead of its own defaults, Enio 2026-07-11), and give a scale-appropriate default **Size**.
    ///
    /// The paper is canvas-**Tiled** (`rel = px·size/256`), so a PROCEDURAL at Size `1` shows 256-px
    /// features — "giant blobs", nothing like paper tooth — while a baked 256² **preset** / a loaded
    /// **Image** IS one full tile per 256 px (Size `1`). So a procedural defaults to a fine tooth
    /// ([`PAPER_PROCEDURAL_DEFAULT_SIZE`]); a preset/image to `1`. The default only re-applies when the
    /// SCALE CLASS changes (procedural ↔ bitmap), so switching between two procedural kinds preserves a
    /// Size the user tuned (Enio 2026-07-11).
    pub fn set_brush_paper_kind(&mut self, k: u8) {
        let old = self.paint.brush.paper.kind;
        let kind = TextureKind::from_u8(k);
        self.paint.brush.paper.kind = kind;
        self.paint.brush.paper.mapping = TextureMapping::Tiled;
        self.reset_paper_params();
        if is_procedural_paper(kind) != is_procedural_paper(old) {
            let s = if is_procedural_paper(kind) {
                PAPER_PROCEDURAL_DEFAULT_SIZE
            } else {
                1.0 // baked preset tile / centred Image = one full 256-px tile
            };
            self.paint.brush.paper.size = [s, s];
        }
    }

    /// Reset the Paper slot params to the active kind's `param_specs` defaults (unused slots stay at the
    /// neutral `0.5`). The Paper twin of [`PainterTool::reset_texture_params`], called on a kind change so
    /// each pattern starts from its own sensible knobs — not the generic neutral that mis-renders Voronoi.
    fn reset_paper_params(&mut self) {
        let mut params = [0.5; ph2d_painter_brush::MAX_TEX_PARAMS];
        for (i, s) in ph2d_painter_brush::param_specs(self.paint.brush.paper.kind)
            .iter()
            .enumerate()
        {
            params[i] = s.default;
        }
        self.paint.brush.paper.params = params;
    }

    /// Set the **Paper** slot Size on `axis` (0 = x, 1 = y), clamped to `[0.1, 100]`.
    pub fn set_brush_paper_size(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.paper.size[axis] = v.clamp(TEX_SIZE_MIN, TEX_SIZE_MAX);
        }
    }

    /// Set the **Paper** slot Angle (whole degrees, wrapped to `0..360`).
    pub fn set_brush_paper_angle(&mut self, deg: f32) {
        self.paint.brush.paper.angle_deg = deg.rem_euclid(360.0) as u16;
    }

    /// Set the **Paper** slot Mapping (`TextureMapping` wire u8).
    pub fn set_brush_paper_mapping(&mut self, m: u8) {
        self.paint.brush.paper.mapping = TextureMapping::from_u8(m);
    }

    /// Set the **Paper** slot Offset on `axis` (0 = x, 1 = y), clamped to `[-1, 1]`.
    pub fn set_brush_paper_offset(&mut self, axis: usize, v: f32) {
        if axis < 2 {
            self.paint.brush.paper.offset[axis] = v.clamp(TEX_OFFSET_MIN, TEX_OFFSET_MAX);
        }
    }

    /// Set the **Paper Depth** (how strongly the paper tooth textures the wash), clamped to `[0, 1]`.
    pub fn set_brush_paper_depth(&mut self, v: f32) {
        self.paint.brush.paper_depth = v.clamp(0.0, 1.0);
    }

    /// Set the **Paper** per-pattern param on `slot` (`0..6`; slots 0/1 = Contrast/Brightness), `[0, 1]`.
    pub fn set_brush_paper_param(&mut self, slot: usize, v: f32) {
        if slot < self.paint.brush.paper.params.len() {
            self.paint.brush.paper.params[slot] = v.clamp(0.0, 1.0);
        }
    }

    /// Toggle **Granulation "Same as Paper"** — on = the granulation settles into the paper's own tooth
    /// (the Grain slot texture is ignored); off = the **Grain** slot IS the granulation map.
    pub fn toggle_granulation_use_paper(&mut self) {
        self.paint.brush.granulation_use_paper = !self.paint.brush.granulation_use_paper;
    }

    /// Reset the **Watercolor** section to defaults (section off; all params neutral). Plain paint
    /// state — no undo / pixel touch, like the other section resets.
    pub fn reset_brush_watercolor(&mut self) {
        let d = BrushSpec::default();
        let b = &mut self.paint.brush;
        b.watercolor = d.watercolor;
        b.edge_gain = d.edge_gain;
        b.edge_spread = d.edge_spread;
        b.granulation = d.granulation;
        b.pigment = d.pigment;
        b.pigment_mix = d.pigment_mix;
        b.fill = d.fill;
        b.depth = d.depth;
        b.opacity = d.opacity;
        b.warp = d.warp;
        b.wet_smudge = d.wet_smudge;
        b.wet_rewet = d.wet_rewet;
        b.wet_charge = d.wet_charge;
        b.wet_dilution = d.wet_dilution;
        b.wet_pull = d.wet_pull;
        b.watercolor_shape_auto = d.watercolor_shape_auto;
        // Drying Time + wetness-preview are canvas-level (not on `b`): reset them to their defaults too.
        self.paint.dry_rate_per_s = super::watercolor_backdrop::CANVAS_WET_DRY_DEFAULT;
        self.paint.wet_preview_intensity = super::watercolor_backdrop::WET_PREVIEW_DEFAULT;
    }

    /// Install a tagged Hierarchy layer/group (its luminance `lum`, `width × height`) into the watercolor
    /// **Paper** slot: the substrate tooth the wash sits on (Fase D — "Use as Paper"). Canvas-anchored; the
    /// render-path is turned on. A Group tag passes the composited group pixels.
    pub fn use_layers_as_watercolor_paper(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.paint.paper_image = Some(BrushTextureImage::new(lum, width, height));
        self.paint.paper_image_version = self.paint.paper_image_version.wrapping_add(1);
        self.paint.brush.paper.kind = TextureKind::Image;
        self.paint.brush.paper.mapping = TextureMapping::Tiled;
        self.paint.brush.watercolor = true;
    }

    /// The Paper slot's imported image `(luminance, w, h)` for the panel's Paper preview. `None` if unset.
    #[must_use]
    pub fn brush_paper_image(&self) -> Option<(&[u8], u32, u32)> {
        self.paint
            .paper_image
            .as_ref()
            .map(BrushTextureImage::parts)
    }

    /// Version of [`Self::brush_paper_image`] — the shell re-publishes only when it changes.
    #[must_use]
    pub fn brush_paper_image_version(&self) -> u64 {
        self.paint.paper_image_version
    }

    /// Install a tagged layer/group into the **Granulation** map — the **Grain** slot (`brush.texture`),
    /// DISTINCT from the paper (Fase D — "Use as Granulation"). Turns off "Same as Paper" so this map is
    /// used, and gives a pronounced granulation amount so the pigment pools in the layer's valleys.
    pub fn use_layers_as_granulation(&mut self, lum: Vec<u8>, width: u32, height: u32) {
        self.set_brush_texture_image(lum, width, height); // → the Grain slot (kind = Image)
        self.paint.brush.texture.mapping = TextureMapping::Tiled;
        self.paint.brush.granulation_use_paper = false;
        self.paint.brush.watercolor = true;
        self.paint.brush.granulation = 0.65; // LITERAL-OK: pronounced mineral-settling granulation
    }

    /// Apply a one-click **brush preset** (the top-of-panel dropdown): `0` = **Digital Basic** (the plain
    /// brush), `1` = **Watercolor Basic** (the optical wash configured to reproduce
    /// `docs/Painter/wet_edges_paint.html`). Both PRESERVE the current colour + radius (a preset is a
    /// look, not a reset of what/where you paint); everything else is set from scratch so switching is
    /// deterministic. Plain state edit (no undo / pixel touch), like the section resets.
    pub fn apply_brush_preset(&mut self, idx: u8) {
        let cur = self.paint.brush;
        self.paint.brush = match idx {
            // Watercolor Basic — wet_edges defaults: soft round dab (`Falloff::Smooth`, `hardness 0`),
            // Mix blend, dense spacing, the optical render-path on (Fill/Depth/Edge/Spread/Warp/Granulation
            // set to the wet_edges constants). Pigment stays off (wet_edges default `realistic = false`).
            1 => BrushSpec {
                radius_px: cur.radius_px,
                color: cur.color,
                spacing: 0.05, // LITERAL-OK: dense wash dabs (wet_edges segments ≈ r·0.22)
                watercolor: true,
                fill: 0.12,        // LITERAL-OK: wet_edges fillDensity
                depth: 1.2,        // LITERAL-OK: wet_edges DEPTH
                edge_gain: 3.0,    // LITERAL-OK: wet_edges edgeGain
                edge_spread: 7.0,  // LITERAL-OK: wet_edges spread
                warp: 6.0,         // LITERAL-OK: wet_edges warpAmp
                granulation: 0.30, // LITERAL-OK: wet_edges granAmt
                pigment: false,
                // Paper slot = a canvas-anchored cold-press paper (the substrate the wash sits on);
                // Granulation follows the paper's tooth (Same as Paper, the default).
                paper: TextureSettings {
                    kind: TextureKind::PaperCold,
                    mapping: TextureMapping::Tiled,
                    ..TextureSettings::default()
                },
                ..BrushSpec::default()
            },
            // Digital Basic (0 or any unknown) — the plain default brush, keeping the user's colour + size.
            _ => BrushSpec {
                radius_px: cur.radius_px,
                color: cur.color,
                ..BrushSpec::default()
            },
        };
    }
}

#[cfg(test)]
mod tests;
