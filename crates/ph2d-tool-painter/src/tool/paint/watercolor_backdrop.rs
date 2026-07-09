//! Watercolor **ground** (real backdrop) + water **soak** (dwell) — the per-stroke state the optical
//! composite reads its Beer–Lambert base / rewet reference / lift target from, and the "the longer
//! the water sits, the more it dissolves" field. Split from `watercolor_render.rs` for the LOC cap.
//!
//! The ground REPLACES the old virtual-cream constant (`PAPER`): where the active layer is
//! transparent, the optics now see the **composite of the layers below it** over the document
//! [`paper colour`](super::PaintState::paper_color) — painting over a white layer no longer pulls
//! toward beige, and the rewet's "what is paint" question is answered against the true local
//! ground (Enio 2026-07-06, doc `11_aquarela_avaliacao_padrao_ouro.md` §3).

use super::*;
use crate::compositor::composite_below;

/// How fast the parked brush pours dwell into [`PaintState::wet_soak`]: a held nib saturates its
/// disc in ~2 s (`255 / 2 s`). Moving strokes soak too (each tick pours at the current dab), but the
/// dt spreads across positions, so a sweep stays light — only LINGERING builds a deep soak.
const SOAK_RATE_PER_S: f32 = 127.5;

impl PainterTool {
    /// Freeze the watercolor ground for a beginning stroke: the pre-stroke `canvas_rgba` as the
    /// optical base (shared `Arc`, O(1)) + the real **backdrop** under the active layer. Also resets
    /// the per-stroke soak. No-op (clears both) when the watercolor render-path is off.
    pub(super) fn freeze_watercolor_ground(&mut self) {
        if !self.watercolor_render_active() {
            self.paint.watercolor_base = None;
            self.paint.wet_backdrop = None;
            return;
        }
        self.paint.watercolor_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_backdrop = Some(Arc::new(self.build_wet_backdrop()));
        self.paint.wet_soak.iter_mut().for_each(|s| *s = 0);
        self.paint.wet_soak_pos = None;
        self.paint.wet_soak_active = false;
        // Manual Shape tip normaliser (see `PaintState::wet_shape_norm`): 1 / max luminance of the
        // Shape image, computed once per stroke — the watercolor coverage must SATURATE in the tip's
        // core (wetness geometry), so grey tips scale up to full while keeping their relative texture.
        self.paint.wet_shape_norm = 1.0;
        if !self.paint.brush.watercolor_shape_auto
            && let Some(img) = self.paint.shape_image.as_ref()
        {
            let (lum, _, _) = img.parts();
            let max = lum.iter().copied().max().unwrap_or(0);
            if max > 0 {
                self.paint.wet_shape_norm = 255.0 / f32::from(max);
            }
        }
        // Substrate cache: invalidate for the new stroke (the paper is constant WITHIN a stroke, so a
        // full reset at pen-down is the only invalidation needed — no in-stroke staleness). Filled
        // lazily by the composite ([`Self::apply_watercolor`]); `NaN` marks an untouched pixel.
        let n = (self.source_size.0 as usize) * (self.source_size.1 as usize);
        if self.paint.wet_substrate.len() == n {
            self.paint
                .wet_substrate
                .iter_mut()
                .for_each(|v| *v = f32::NAN);
        } else {
            self.paint.wet_substrate = vec![f32::NAN; n];
        }
    }

    /// Composite the layers strictly BELOW the anchor over the document paper colour → the opaque
    /// RGBA8 ground. The anchor is the active layer; painting a MASK anchors at the mask's PARENT
    /// (the mask shows through its parent, so the ground under the parent is what the eye sees).
    /// Runs once per stroke (pen-down); `composite_below` handles an unknown anchor (plain paper).
    fn build_wet_backdrop(&self) -> Vec<u8> {
        let (w, h) = self.source_size;
        let paper = self.paper_color_rgb8();
        let anchor = self.layers.active().and_then(|id| {
            if self.layers.is_mask(id) {
                // The mask isn't in the z-order — resolve the raster that owns it.
                self.layers
                    .all_ids()
                    .find(|&pid| self.layers.get(pid).is_some_and(|l| l.mask == Some(id)))
            } else {
                Some(id)
            }
        });
        let Some(anchor) = anchor else {
            let mut ground = vec![0u8; (w as usize) * (h as usize) * 4];
            for px in ground.chunks_exact_mut(4) {
                px.copy_from_slice(&[paper[0], paper[1], paper[2], 255]);
            }
            return ground;
        };
        let src = ToolPixelSource {
            active_id: anchor,
            active_rgba: &self.canvas_rgba,
            images: &self.images,
        };
        composite_below(&self.layers, &src, w, h, anchor, paper)
    }

    /// The document paper colour as straight sRGB8 bytes.
    pub(super) fn paper_color_rgb8(&self) -> [u8; 3] {
        let c = self.paint.paper_color;
        [
            (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    /// Set one paper-colour channel (`0..1` straight sRGB); the shared-picker read-back drives all
    /// three per frame via `SelectOption(PAINTER_WATERCOLOR_PAPER_COLOR_THUMB, "r,g,b")`.
    pub(crate) fn set_paper_color_rgb8(&mut self, r: u8, g: u8, b: u8) {
        self.paint.paper_color = [
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        ];
    }

    /// Pour `dt_s` of water dwell into the soak disc at the last dab position (the tick heartbeat,
    /// moving or parked). Returns the poured disc's region when the soak actually grew AND the
    /// rewet reads it (`wet_rewet > 0`), so the caller can fold it into the frame dirty rect and
    /// recomposite — a parked wet brush visibly deepens its bleed while held.
    pub(super) fn grow_wet_soak(&mut self, dt_s: f32) -> Option<Region> {
        let wet = self.paint.brush.wet_rewet;
        let (center, radius) = self.paint.wet_soak_pos?;
        if wet <= 0.0 || dt_s <= 0.0 || radius <= 0.0 {
            return None;
        }
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return None;
        }
        if self.paint.wet_soak.len() != fw * fh {
            self.paint.wet_soak = vec![0u8; fw * fh];
        }
        // Byte units per tick; at least 1 step so slow frames still soak.
        let add = (SOAK_RATE_PER_S * dt_s).clamp(1.0, 255.0) as u16;
        let (cx, cy) = (center[0], center[1]);
        // A sitting puddle SPREADS: the pour disc grows with the centre's own saturation (up to
        // 2× the nib) — holding the brush pushes the dwell past the disc edge, so the widened
        // dissolve visibly creeps outward instead of stopping at the footprint.
        let ci = (cy.clamp(0.0, (fh - 1) as f32) as usize) * fw
            + (cx.clamp(0.0, (fw - 1) as f32) as usize);
        let center_soak = if self.paint.wet_soak.len() == fw * fh {
            f32::from(self.paint.wet_soak[ci]) / 255.0
        } else {
            0.0
        };
        let r = radius * (1.0 + center_soak);
        let inv_r = 1.0 / r;
        let x0 = (cx - r).floor().max(0.0) as usize;
        let y0 = (cy - r).floor().max(0.0) as usize;
        let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
        let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
        if x0 >= x1 || y0 >= y1 {
            return None;
        }
        // Selection + protection gates: water doesn't pool on gated-out texels either (the soak
        // deepens lift/dissolve — letting it accumulate outside the selection would deepen the
        // dissolve just inside the boundary from paint the user masked off). See `splat_keep`.
        let (sel, prot) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some();
        let soak = &mut self.paint.wet_soak;
        let mut grew = false;
        for y in y0..y1 {
            let dy = (y as f32 + 0.5) - cy;
            let base = y * fw;
            for x in x0..x1 {
                let dx = (x as f32 + 0.5) - cx;
                let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                if dn >= 1.0 {
                    continue;
                }
                let idx = base + x;
                let keep = if gated {
                    super::watercolor_accum::splat_keep(
                        sel.as_deref().map(Vec::as_slice),
                        prot.as_deref().map(Vec::as_slice),
                        idx,
                    )
                } else {
                    1.0
                };
                if keep <= 0.0 {
                    continue;
                }
                // Full pour inside the core, fading to the rim (the water pools under the nib).
                let w = (1.0 - dn).min(0.6) / 0.6;
                let cur = soak[idx];
                let next = (u16::from(cur) + (f32::from(add) * w * keep) as u16).min(255) as u8;
                if next != cur {
                    soak[idx] = next;
                    grew = true;
                }
            }
        }
        if grew {
            self.paint.wet_soak_active = true;
        }
        grew.then(|| Region {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }
}

/// EDGE-1 — paper drying rate in wetness-bytes per second: a fully-wet cell (255) dries in
/// ~8.5 s, the DiVerdi/Adobe TVCG 2013 wet-map constant (*"wet map cells are set to 255 when
/// wetted... they take 8.5 seconds to dry"*). Calibration knob: lower = washes stay mergeable
/// longer.
const CANVAS_WET_DRY_PER_S: f32 = 30.0;

impl PainterTool {
    /// EDGE-1: whether the stroke beginning NOW continues the live **wet session** (one wash):
    /// watercolor mode, some paper still wet, the session base is sized, the union buffers exist,
    /// and — the load-bearing guard — `canvas_rgba` is EXACTLY the Arc our last bake produced
    /// (`Arc::ptr_eq`). Every foreign mutation path (undo restore, layer switch/bind, fill,
    /// resize, document switch) assigns a fresh Arc, so no per-site invalidation is needed.
    pub(super) fn wet_session_continues(&self) -> bool {
        let (fw, fh) = self.source_size;
        let n = (fw as usize) * (fh as usize);
        self.watercolor_render_active()
            && n > 0
            && self.paint.canvas_wet_rect.is_some()
            && self.paint.stroke_coverage.len() == n
            && self.paint.stroke_color.len() == n * 4
            && self
                .paint
                .wet_session_base
                .as_ref()
                .is_some_and(|b| b.len() == n * 4)
            && self
                .paint
                .wet_session_canvas
                .as_ref()
                .is_some_and(|c| Arc::ptr_eq(c, &self.canvas_rgba))
    }

    /// EDGE-1: pour the finished stroke's coverage into the persistent canvas MOISTURE map
    /// (max-blend over the stroke's cumulative rect). Called at the bake, AFTER the commit
    /// composite — the stroke's own rim renders against dry paper; only strokes that follow
    /// within the drying window merge into it.
    pub(super) fn pour_canvas_wet(&mut self) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let n = fw * fh;
        if n == 0 || self.paint.stroke_coverage.len() != n {
            return;
        }
        let Some(r) = self.paint.wet_cum_dirty else {
            return;
        };
        let (x0, y0) = ((r.x as usize).min(fw), (r.y as usize).min(fh));
        let (x1, y1) = (
            ((r.x + r.w) as usize).min(fw),
            ((r.y + r.h) as usize).min(fh),
        );
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        if self.paint.canvas_wet.len() != n {
            self.paint.canvas_wet = vec![0u8; n];
            self.paint.canvas_wet_rect = None;
        }
        for y in y0..y1 {
            let row = y * fw;
            for x in x0..x1 {
                // Pour the HARDENED coverage (the composite's own `cw` smoothstep): moisture is
                // "the wash is here", so the wash INTERIOR is fully wet even at low dab flow —
                // pouring the raw coverage left the rim only half-suppressed at the junction.
                let cov = f32::from(self.paint.stroke_coverage[row + x]) / 255.0;
                let cw = super::watercolor_field::smoothstep(
                    super::watercolor_render::SS0,
                    super::watercolor_render::SS1,
                    cov,
                );
                let c = (cw * 255.0) as u8;
                if c > self.paint.canvas_wet[row + x] {
                    self.paint.canvas_wet[row + x] = c;
                }
            }
        }
        self.paint.canvas_wet_rect = Some(match self.paint.canvas_wet_rect {
            Some((rx0, ry0, rx1, ry1)) => (rx0.min(x0), ry0.min(y0), rx1.max(x1), ry1.max(y1)),
            None => (x0, y0, x1, y1),
        });
    }

    /// EDGE-1: the paper DRIES on the heartbeat — [`CANVAS_WET_DRY_PER_S`] with a fractional
    /// carry (slow frames still dry at the same wall-clock rate). Drops the buffer once fully
    /// dry, restoring the composite's moisture-free fast path and zero idle cost.
    pub(super) fn dry_canvas_wet(&mut self, dt_s: f32) {
        let Some((x0, y0, x1, y1)) = self.paint.canvas_wet_rect else {
            return;
        };
        let (fw, _fh) = self.source_size;
        let fw = fw as usize;
        if self.paint.canvas_wet.is_empty() {
            self.paint.canvas_wet_rect = None;
            return;
        }
        self.paint.canvas_wet_carry += dt_s.max(0.0) * CANVAS_WET_DRY_PER_S;
        let step = self.paint.canvas_wet_carry.min(255.0) as u8;
        if step == 0 {
            return;
        }
        self.paint.canvas_wet_carry -= f32::from(step);
        let mut wettest = 0u8;
        for y in y0..y1 {
            let row = y * fw;
            for w in &mut self.paint.canvas_wet[row + x0..row + x1] {
                *w = w.saturating_sub(step);
                wettest = wettest.max(*w);
            }
        }
        // Fully dry = the wet SESSION is over — but the teardown is ATOMIC and deferred past any
        // OPEN stroke: the drying deadline can land mid-stroke (a stroke started near the end of
        // the window), and dropping the session base while the union buffers live on made the
        // pen-up bake fall back to the per-stroke base — which already CONTAINS the union baked —
        // re-rendering it over itself (double-count: the whole wash suddenly darkened hard, Enio
        // smoke 2026-07-09). With a stroke open we leave the zeroed map in place; its own bake
        // re-pours (session extends), or a later idle tick tears everything down together.
        if wettest == 0 && self.paint.stroke.is_none() {
            self.paint.canvas_wet = Vec::new();
            self.paint.canvas_wet_rect = None;
            self.paint.canvas_wet_carry = 0.0;
            self.paint.wet_session_base = None;
            self.paint.wet_session_canvas = None;
            self.paint.stroke_coverage = Vec::new();
            self.paint.stroke_color = Vec::new();
            self.paint.stroke_density = Vec::new();
            self.paint.stroke_deplete = Vec::new();
            self.paint.wet_styles.clear();
            self.paint.stroke_water = Vec::new();
            self.paint.wet_cum_dirty = None;
        }
    }
}
