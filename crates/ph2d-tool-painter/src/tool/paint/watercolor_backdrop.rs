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
    /// optical base (shared `Arc`, O(1)) + the real **backdrop** under the active layer. The soak
    /// (dwell) resets only on a FRESH session — a CONTINUING wet session keeps the paper's dwell,
    /// so the union re-renders reproduce the bakes byte-exact (the soak boosts rim/settle per
    /// pixel; zeroing it per stroke re-rendered the neighbour wash weaker, Enio 2026-07-09).
    /// No-op (clears both) when the watercolor render-path is off.
    pub(super) fn freeze_watercolor_ground(&mut self, wet_session: bool) {
        if !self.watercolor_render_active() {
            self.paint.watercolor_base = None;
            self.paint.wet_backdrop = None;
            return;
        }
        self.paint.watercolor_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_backdrop = Some(Arc::new(self.build_wet_backdrop()));
        // A new wet render supersedes any live-editable wash — the previous one is now permanent (it was
        // baked into `canvas_rgba` at its pen-up, which THIS base freeze captured). Drop its snapshot.
        self.clear_wet_editable();
        if !wet_session {
            self.paint.wet_soak.iter_mut().for_each(|s| *s = 0);
            self.paint.wet_soak_active = false;
        }
        self.paint.wet_soak_pos = None;
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
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
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
                        alock.as_deref().map(Vec::as_slice),
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

/// EDGE-1 — DEFAULT paper drying rate in wetness-bytes per second (~10 s window, Enio 2026-07-10:
/// 60 s ficou "extremamente alto"; histórico: 30 ≈ 8,5 s DiVerdi/Adobe TVCG 2013 → 4.25 ≈ 60 s). The
/// live rate is now `PaintState::dry_rate_per_s` (the Drying-Time slider, doc 13 #11); this is only
/// its default. `255 / seconds`: 10 s → 25.5.
pub(super) const CANVAS_WET_DRY_DEFAULT: f32 = 25.5;

/// #12a (doc 14) — DEFAULT wetness-preview strength: the max veil alpha the shell paints over the wet
/// paper (a discreet damp darkening). `0` = no preview. The live value is `PaintState::wet_preview_intensity`
/// (the Wetness card's slider); this is only its default.
pub(super) const WET_PREVIEW_DEFAULT: f32 = 0.3;

/// #3 (doc 14) — the **Wet the layer** forced rewet level: how strongly strokes lift/blend the EXISTING
/// paint after the Wet button re-wets the canvas (Rebelle-style), independent of the brush's own Rewet.
/// Strong so the reactivation is clearly visible; the artist still tempers it with the brush's Rewet up.
pub(super) const WET_CANVAS_REWET: f32 = 0.8;

/// #12b (doc 14) — edges-to-centre drying: a wet pool recedes from its PERIMETER inward, not uniformly.
/// On each decay step a pixel loses the flat `step` PLUS this gain × the gap to its driest 4-neighbour
/// (`0` in a flat interior ⇒ uniform there; large at the wet front ⇒ the boundary recedes first). `2`
/// makes a sharp front dry ~3× the interior rate.
pub(super) const WET_ERODE_GAIN: u32 = 2;

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
    /// (max-blend over THIS STROKE's OWN footprint). Called at the bake, AFTER the commit composite —
    /// the stroke's own rim renders against dry paper; only strokes that follow within the drying
    /// window merge into it. Restricted to the pixels THIS stroke OWNS (`wet_styles.owner == cur_o`,
    /// recency): `stroke_coverage` is the whole session UNION, so pouring it over any RECT re-wet the
    /// earlier washes' pixels that fell inside the rect — a rectangular re-wet patch over the neighbour
    /// (Enio 2026-07-11 "retângulo gigante na união"; the per-stroke RECT of #4 only shrank it). The owner
    /// map is exactly this stroke's footprint (its overlap with a prior wash included, by recency).
    pub(super) fn pour_canvas_wet(&mut self) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let n = fw * fh;
        if n == 0 || self.paint.stroke_coverage.len() != n {
            return;
        }
        let Some(r) = self.paint.wet_stroke_dirty else {
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
        // Only THIS stroke's OWN footprint re-wets (not the union coverage over the rect) — else a stroke
        // whose bbox merely CONTAINS an earlier wash re-wet that wash into a rectangle. `has_owner` false
        // (no style map) ⇒ pour the whole rect, the pre-owner-map behaviour.
        let cur_o = self.paint.wet_styles.current_owner();
        let has_owner = self.paint.wet_styles.owner.len() == n;
        for y in y0..y1 {
            let row = y * fw;
            for x in x0..x1 {
                if has_owner && self.paint.wet_styles.owner[row + x] != cur_o {
                    continue;
                }
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

    /// EDGE-1: the paper DRIES on the heartbeat — [`CANVAS_WET_DRY_PER_S`] with a fractional carry (slow
    /// frames still dry at the same wall-clock rate). #12b: the drying is EDGES-TO-CENTRE — the boundary
    /// recedes faster than the interior ([`WET_ERODE_GAIN`]), a wet pool shrinking from its perimeter like
    /// real paper. Drops the buffer once fully dry, restoring the moisture-free fast path.
    pub(super) fn dry_canvas_wet(&mut self, dt_s: f32) {
        let Some((x0, y0, x1, y1)) = self.paint.canvas_wet_rect else {
            return;
        };
        let (fw, fh) = self.source_size;
        let fw = fw as usize;
        // Belt-and-braces (sweep 2026-07-12): the moisture map OUTLIVES the gesture, so a document rebind
        // can leave it shaped for the PREVIOUS sprite — and the loop below indexes it with the CURRENT
        // sprite's stride and a rect in the OLD sprite's coordinates (a bigger sprite ⇒ slice past the end
        // ⇒ SIGSEGV). `reset_transient_edit_state` now drops it at the rebind, so this is unreachable; it
        // stays because the buffer is canvas-sized and the guard asked "does it exist?" instead of "does
        // its SHAPE match?" — the exact hole that produced Bug #12. Physics untouched: when the shape
        // matches (always, in every real path) this is byte-identical to the old `is_empty()` check.
        if self.paint.canvas_wet.len() != fw * (fh as usize) {
            self.paint.canvas_wet = Vec::new();
            self.paint.canvas_wet_rect = None;
            return;
        }
        self.paint.canvas_wet_carry += dt_s.max(0.0) * self.paint.dry_rate_per_s.max(0.0);
        let step = self.paint.canvas_wet_carry.min(255.0) as u8;
        if step == 0 {
            return;
        }
        self.paint.canvas_wet_carry -= f32::from(step);
        // Snapshot the rect so the 4-neighbour gap reads the PRE-step moisture (order-independent);
        // outside the rect counts as dry (0), so the wet front recedes at the rect edge too.
        let (rw, rh) = (x1 - x0, y1 - y0);
        let mut old = vec![0u8; rw * rh];
        for oy in 0..rh {
            let src = (y0 + oy) * fw + x0;
            old[oy * rw..oy * rw + rw].copy_from_slice(&self.paint.canvas_wet[src..src + rw]);
        }
        let mut wettest = 0u8;
        for oy in 0..rh {
            for ox in 0..rw {
                let o = old[oy * rw + ox];
                if o == 0 {
                    continue;
                }
                let up = if oy > 0 { old[(oy - 1) * rw + ox] } else { 0 };
                let down = if oy + 1 < rh {
                    old[(oy + 1) * rw + ox]
                } else {
                    0
                };
                let left = if ox > 0 { old[oy * rw + ox - 1] } else { 0 };
                let right = if ox + 1 < rw {
                    old[oy * rw + ox + 1]
                } else {
                    0
                };
                let gap = o.saturating_sub(up.min(down).min(left).min(right));
                let erode = ((u32::from(gap) * u32::from(step) * WET_ERODE_GAIN) / 255) as u8;
                let nv = o.saturating_sub(step.saturating_add(erode));
                self.paint.canvas_wet[(y0 + oy) * fw + x0 + ox] = nv;
                wettest = wettest.max(nv);
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
            self.dry_session_now();
        }
    }

    /// EDGE-1 (doc 13 #9): END the wet session NOW — the atomic teardown (drop the moisture map +
    /// every session/union buffer + the frozen base + soak). The baked washes STAY in `canvas_rgba`
    /// (this doesn't touch pixels); only the FUSION with future strokes ends — Rebelle "Dry the
    /// layer". Shared by the drying-deadline path above, the **Dry** button, and undo/redo (the canvas
    /// identity changed, so the moisture map is stale — clear it, Enio 2026-07-11). Idempotent.
    pub(crate) fn dry_session_now(&mut self) {
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
        self.paint.wet_soak = Vec::new();
        self.paint.wet_soak_active = false;
        self.paint.wet_cum_dirty = None;
        self.paint.wet_stroke_dirty = None;
        self.paint.wet_session_wetness = 0.0; // the forced "Wet the layer" rewet ends with the session (#3)
        self.clear_wet_editable(); // the wash is now permanent — no more live texture edits
    }

    /// Drop the live-editable wash snapshot (Enio 2026-07-11): the last wash stops re-rendering on a
    /// texture-param change. Called when the wash becomes permanent — the session dries or a NEW wet render
    /// starts (a fresh ground is frozen). The baked pixels in `canvas_rgba` are untouched.
    pub(super) fn clear_wet_editable(&mut self) {
        self.paint.wet_editable_base = None;
        self.paint.wet_editable_backdrop = None;
        self.paint.wet_editable_region = None;
        self.paint.wet_editable_tex = None;
    }

    /// Re-render the **live-editable wash** (Enio 2026-07-11): reconstruct the LAST committed wash from its
    /// kept pre-wash base + frozen ground with the CURRENT brush texture, over the committed footprint
    /// (already full-axis on a tiled axis ⇒ central AND every Tiling copy). Driven by the paint tick when a
    /// Grain/Paper param moved while the paper is still wet, so the wash reflects the new texture live — not
    /// only the next stroke. No editable wash / mis-sized buffers ⇒ no-op (byte-identical).
    pub(super) fn rerender_editable_wash(&mut self) {
        let (Some(base), Some(region)) = (
            self.paint.wet_editable_base.clone(),
            self.paint.wet_editable_region,
        ) else {
            return;
        };
        let (fw, fh) = self.source_size;
        let n = (fw as usize) * (fh as usize);
        if n == 0
            || base.len() != n * 4
            || self.paint.stroke_coverage.len() != n
            || self.paint.stroke_color.len() != n * 4
        {
            return;
        }
        // The paper-tooth memo is keyed by canvas pixel and NOTHING else — its only invalidation is the
        // NaN-reset at pen-down, on the premise (written into the field's doc) that "the paper cannot
        // change mid-stroke". THIS function is what made that premise false: it exists precisely to
        // re-render when a Paper param moved. `fill_substrate_cache` fills only the NaN misses, so without
        // this reset every already-memoised pixel keeps the tooth of the OLD paper — the pool re-renders
        // and the paper does not change, which defeats this feature's whole purpose for the Paper slot.
        // (The Grain is sampled live, which is why the Grain half worked and the smoke passed.)
        // Sweep 2026-07-12, `BUGS_painter.md` #14. Same root as Bug #12: a reuse guard that never asks
        // whether the cached value still belongs to the inputs that produced it.
        self.paint
            .wet_substrate
            .iter_mut()
            .for_each(|v| *v = f32::NAN);
        // `close_stroke` dropped the live base + ground; point the render at the kept snapshot and force the
        // committed footprint dirty, then reconstruct the wash with the CURRENT texture (`apply_watercolor`
        // reads `brush.texture`/`paper`). `commit=false` keeps the base for the next edit.
        self.paint.watercolor_base = Some(Arc::clone(&base));
        self.paint.wet_backdrop = self.paint.wet_editable_backdrop.clone();
        self.paint.wet_frame_dirty = Some(region);
        self.apply_watercolor(false);
        // Keep ONLY the snapshot live (mirror the post-`close_stroke` state) and re-arm the session guard —
        // `apply_watercolor`'s `Arc::make_mut` forked `canvas_rgba`, so the old guard Arc no longer matches.
        self.paint.watercolor_base = None;
        self.paint.wet_backdrop = None;
        self.paint.wet_session_canvas = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_editable_tex = Some((self.paint.brush.texture, self.paint.brush.paper));
    }

    /// EDGE-1 (doc 13 #10 + doc 14 #3): **Wet the layer** (Rebelle). Re-moisten the whole canvas WITHOUT
    /// depositing pigment AND re-open a wet session bound to the CURRENT canvas so strokes made now FUSE
    /// with the EXISTING paint — dropping water on a dry wash reactivates it. Fills `canvas_wet` = 255
    /// (visible damp #12a + the drying/fusion window), freezes the current pixels as the session base (what
    /// the rewet lifts) over the ground below (the lift target), starts the union buffers EMPTY (untouched
    /// paint stays exactly as-is until a stroke reaches it), and FORCES the rewet ([`WET_CANVAS_REWET`]) so
    /// the reactivation shows even with the brush's own Rewet at `0`. No pixel touch (the lift happens when
    /// you paint). Off the watercolor render-path it only arms the moisture (nothing to fuse).
    pub(super) fn wet_canvas_now(&mut self) {
        let (fw, fh) = self.source_size;
        let n = (fw as usize) * (fh as usize);
        if n == 0 {
            return;
        }
        self.paint.canvas_wet = vec![255u8; n];
        self.paint.canvas_wet_rect = Some((0, 0, fw as usize, fh as usize));
        self.paint.canvas_wet_carry = 0.0;
        if !self.watercolor_render_active() {
            return;
        }
        // The existing pixels ARE the wet-session base (what the rewet reactivates); the ground below is
        // the lift target. Empty union buffers ⇒ `wet_session_continues` passes and the next stroke joins
        // this session (lifting the base) instead of starting fresh over the dry paint.
        self.paint.watercolor_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_backdrop = Some(Arc::new(self.build_wet_backdrop()));
        self.paint.wet_session_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_session_canvas = Some(Arc::clone(&self.canvas_rgba));
        self.paint.stroke_coverage = vec![0u8; n];
        self.paint.stroke_color = vec![0u8; n * 4];
        self.paint.stroke_density = Vec::new();
        self.paint.stroke_deplete = Vec::new();
        self.paint.stroke_water = Vec::new();
        self.paint.wet_styles.clear();
        self.paint.wet_cum_dirty = None;
        self.paint.wet_stroke_dirty = None;
        self.paint.wet_session_wetness = WET_CANVAS_REWET;
    }

    /// #12a (doc 14): the canvas MOISTURE map + its live rect for the on-canvas wetness overlay —
    /// `(bytes, w, h, [x0, y0, x1, y1])`; each byte is the local wetness (`0` = dry, `255` = fully
    /// wet). `None` when the paper is dry. Read-only; the shell tints the wet region as a sheen.
    #[must_use]
    pub fn canvas_wet_view(&self) -> Option<CanvasWetView<'_>> {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        let (x0, y0, x1, y1) = self.paint.canvas_wet_rect?;
        (n > 0 && self.paint.canvas_wet.len() == n).then_some((
            &self.paint.canvas_wet[..],
            w,
            h,
            [x0 as u32, y0 as u32, x1 as u32, y1 as u32],
        ))
    }

    /// #12a (doc 14): the on-canvas wetness PREVIEW strength (`0..1`) — the max veil alpha the shell
    /// paints over the wet paper. `0` = no preview. The Wetness card's slider drives it.
    #[must_use]
    pub fn wet_preview_intensity(&self) -> f32 {
        self.paint.wet_preview_intensity.clamp(0.0, 1.0)
    }
}

/// The on-canvas wetness overlay's read (#12a): `(moisture bytes, w, h, rect [x0, y0, x1, y1])`.
pub type CanvasWetView<'a> = (&'a [u8], u32, u32, [u32; 4]);
