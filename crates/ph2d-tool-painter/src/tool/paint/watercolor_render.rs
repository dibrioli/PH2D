//! Watercolor **optical render-path** (the wet-media look, `docs/Painter/10_aquarela_render_path_preset_papers.md`).
//!
//! In watercolor mode the normal per-dab deposit is **skipped**; the stroke instead accumulates a
//! coverage mask ([`PaintState::stroke_coverage`](super::PaintState)) and a deposited-colour buffer
//! ([`PaintState::stroke_color`](super::PaintState)), and the whole appearance is **reconstructed
//! optically** each frame over a frozen base — exactly the architecture of
//! `docs/Painter/wet_edges_paint.html`. This kills the per-dab "bubble" + alpha build-up that a
//! bolt-on darkening pass inherits (the wash is a single optical field, not stacked stamps).
//!
//! The model, per pixel (over the frozen base `B`, all in **linear light**):
//! ```text
//!   cover = smoothstep(SS0, SS1, coverage(warp(x,y)))     // hardened, warped silhouette
//!   inner = blur(coverage)                                 // ~1 inside, →0 at the rim
//!   edge  = clamp(cover·(1 − inner)·edge_gain, 0, 1)       // pigment pooled at the receding front
//!   gran  = 1 + (paperHeight − 0.5)·2·granulation          // paper-tooth granulation (value noise)
//!   D     = (cover·fill + edge)·gran                        // optical density
//!   Tᵢ    = pigmentᵢ^(D·depth)                              // Beer–Lambert transmittance per channel
//!   outᵢ  = l2s( s2l(Bᵢ)·Tᵢ + s2l(pigmentᵢ)·(1 − Tᵢ) )     // base attenuated + pigment scattered
//! ```
//! All per-pixel math is table lookups + sums/mults (HR-5): the `s2l`/`lnl`/`l2s`/`exp` LUTs and the
//! integer-hash value noise are built once and deterministic; no transcendental runs in the hot loop.

use super::watercolor_field::*;
use super::watercolor_rewet_px::rewet_px;
use super::*;
use ph2d_painter_brush::blend::ryb_mix;
use rayon::prelude::*;

/// Hardened-coverage smoothstep edges (wet_edges `SS0`/`SS1`): below `SS0` the wash is transparent,
/// above `SS1` fully covered — a crisp-but-soft silhouette from the feathered coverage discs.
pub(super) const SS0: f32 = 0.12; // LITERAL-PX-OK: coverage-hardening smoothstep low edge (wet_edges)
pub(super) const SS1: f32 = 0.60; // LITERAL-PX-OK: coverage-hardening smoothstep high edge (wet_edges)
/// Minimum colour-buffer alpha (0..255) to trust the deposited colour; below it the composite falls
/// back to the live brush colour (wet_edges `COL_EPS`) — a faint rim carries the fresh pigment, not noise.
const COL_EPS: u8 = 20;

impl PainterTool {
    /// Whether the watercolor optical render-path drives this stroke: the Watercolor section is on, we're
    /// the normal **Paint** brush (not Smear/Blur/Clone/Mask/Inpaint/Fill/Selection/Deform), and not
    /// erasing. Off ⇒ a byte-identical plain brush (the whole path is skipped, deposit + composite alike).
    pub(super) fn watercolor_render_active(&self) -> bool {
        self.paint.brush.watercolor
            && matches!(self.paint.paint_mode, PaintMode::Paint)
            && !self.paint.eraser
    }

    /// Composite the watercolor wash over the frozen base ([`PaintState::watercolor_base`](super::PaintState)).
    ///
    /// Reads the coverage + deposited-colour buffers, reconstructs the optical density `D`, and applies
    /// per-channel Beer–Lambert in linear light — see the module docs. The base is a separate `Arc`, so
    /// each frame recomposites cleanly from the pristine pre-stroke pixels (no overlay peel). `commit`
    /// drops the base (pen-up bake, inside the undo transaction); the live passes keep it for the next frame.
    ///
    /// **Dirty-rect (wet_edges `renderFrame`/`endStroke`):** the live passes recomposite ONLY the frame
    /// dirty rect — the dabs landed since the last composite, tracked incrementally by
    /// [`Self::accumulate_wet_coverage`] — padded by the influence radius (a new dab changes `blur(cov)`
    /// only within `spread`, sampled at most `warp` away), so the per-frame cost tracks the new dabs, not
    /// the grown stroke. The pen-up bake makes one cumulative pass, also from an incrementally tracked
    /// bbox — never a full-canvas scan. Returns the recomposited canvas region (`None` = nothing to do).
    pub(super) fn apply_watercolor(&mut self, commit: bool) -> Option<Region> {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let n = fw * fh;
        if n == 0 || self.paint.stroke_coverage.len() != n || self.canvas_rgba.len() != n * 4 {
            if commit {
                self.paint.watercolor_base = None;
            }
            return None;
        }
        // The frozen base (own the Arc handle so the canvas make_mut below doesn't alias the
        // field). EDGE-1 wet session: the composite reads the SESSION base — the canvas frozen at
        // the FIRST stroke of the wet window — so a continuing stroke re-renders the whole UNION
        // from scratch (one wash, one rim) instead of layering over its own previous bake (which
        // would double-count the older strokes). Falls back to the per-stroke base (identical on
        // the session's first stroke).
        let base_arc = match self.paint.wet_session_base.as_ref() {
            Some(b) if b.len() == n * 4 => Arc::clone(b),
            _ => self.paint.watercolor_base.as_ref().map(Arc::clone)?,
        };
        if base_arc.len() != n * 4 {
            if commit {
                self.paint.watercolor_base = None;
            }
            return None;
        }
        // The frozen GROUND — the real backdrop under the active layer ([`super::watercolor_backdrop`]):
        // the Beer–Lambert base where the layer is transparent, the rewet's "what is paint" reference
        // and the lift's target. Frozen together with the base; a missing/mis-sized one (defensive —
        // e.g. a test driving the composite directly) degrades to a uniform paper-colour ground.
        let backdrop_arc = match self.paint.wet_backdrop.as_ref() {
            Some(b) if b.len() == n * 4 => Arc::clone(b),
            _ => {
                let p = self.paper_color_rgb8();
                let mut g = vec![0u8; n * 4];
                for px in g.chunks_exact_mut(4) {
                    px.copy_from_slice(&[p[0], p[1], p[2], 255]);
                }
                Arc::new(g)
            }
        };

        // Pick the recomposite rect and CONSUME the frame one (wet_edges `resetFrame`): live = this
        // frame's dabs; commit = the whole stroke (frame folded in — `paint_end`'s finish dabs land
        // after the last live pass).
        let frame = self.paint.wet_frame_dirty.take();
        let dirty = if commit {
            match (self.paint.wet_cum_dirty, frame) {
                (Some(c), Some(f)) => Some(union_region(c, f)),
                (c, f) => c.or(f),
            }
        } else {
            frame
        };
        let Some(dirty) = dirty else {
            if commit {
                self.paint.watercolor_base = None;
            }
            return None;
        };

        let spread = self.paint.brush.edge_spread.round().clamp(0.0, 48.0) as usize;
        let warp_amp = self.paint.brush.warp.max(0.0);
        let wet = self.paint.brush.wet_rewet.clamp(0.0, 1.0);
        // EDGE-1 per-stroke style (doc 13 topo): geometry/field paths take the SESSION MAXIMA
        // (conservative — any stroke with water builds the rewet fields; the widest warp pads the
        // window); the per-pixel terms resolve the OWNER stroke's own values inside the loop.
        let wet_any = self
            .paint
            .wet_styles
            .table
            .iter()
            .fold(wet, |m, s| m.max(s.wet));
        let warp_any = self
            .paint
            .wet_styles
            .table
            .iter()
            .fold(warp_amp, |m, s| m.max(s.warp));
        // Silhouette-feather radius (`inner`), capped so a pool keeps a saturated core (see below).
        let core_r = spread.min(((self.paint.brush.radius_px * 0.5).round() as usize).max(1));
        // Influence radius of a dab beyond its own disc: the blur reach + the warp displacement +
        // bilinear/rounding slack. Output pixels farther than this from the dirty rect cannot have
        // changed. WITHOUT Wet the only blur is the capped `core_r` feather (a wide Spread does NOT
        // widen the footprint) → a tight window. WITH Wet the dissolve reaches `spread`, and once the
        // stroke poured dwell (`wet_soak_active`) the soak-deepened dissolve reads a 2× blur, so the
        // reach doubles.
        let soaked = wet_any > 0.0 && self.paint.wet_soak_active && self.paint.wet_soak.len() == n;
        // EDGE-2: carried water poured this session (Dilution) — builds the rewet fields (and its
        // own halo) regardless of Rewet, so pure water blooms against the paint beneath.
        let watered = self.paint.stroke_water.len() == n;
        let reach = if soaked || watered {
            spread * 2
        } else if wet_any > 0.0 {
            spread
        } else {
            core_r
        };
        let pad = reach + warp_any.ceil() as usize + 2;
        let x0 = (dirty.x as usize).saturating_sub(pad);
        let y0 = (dirty.y as usize).saturating_sub(pad);
        let x1 = ((dirty.x as usize) + (dirty.w as usize) + pad).min(fw);
        let y1 = ((dirty.y as usize) + (dirty.h as usize) + pad).min(fh);
        if x0 >= x1 || y0 >= y1 {
            if commit {
                self.paint.watercolor_base = None;
            }
            return None;
        }
        let (bw, bh) = (x1 - x0, y1 - y0);
        let region = Region {
            x: x0 as u32,
            y: y0 as u32,
            w: bw as u32,
            h: bh as u32,
        };

        // READ window = the output region padded by the influence radius AGAIN, so the blur under every
        // warped sample position inside the output has full support. (A window clamped at the output's
        // edge would misread the old coverage there — a darkened seam at each frame-region boundary.)
        let rx0 = x0.saturating_sub(pad);
        let ry0 = y0.saturating_sub(pad);
        let rx1 = (x1 + pad).min(fw);
        let ry1 = (y1 + pad).min(fh);
        let (rw, rh) = (rx1 - rx0, ry1 - ry0);

        // Window-local coverage (`[0,1]`) + its blur (reads the true cumulative coverage, so the blur
        // feathers correctly at the real rim, including coverage from earlier frames).
        let mut cov_src = vec![0.0f32; rw * rh];
        for wy in 0..rh {
            let sbase = (ry0 + wy) * fw + rx0;
            let dbase = wy * rw;
            for wx in 0..rw {
                cov_src[dbase + wx] = f32::from(self.paint.stroke_coverage[sbase + wx]) / 255.0;
            }
        }
        // The silhouette-feather blur (`inner` = `core_r`, feeding the edge pool + interior thinning)
        // must SATURATE to ~1 in a pool's core. A Spread wider than the pool otherwise reads the WHOLE
        // pool as "rim": `inner` never reaches 1, the edge term floods the centre, and the loved
        // "Spread clears the centre" dynamic inverts to a flat dark blob (Enio 2026-07-07, after the
        // 24→48 cap raise). `core_r` (computed above) caps the feather at ~half the brush so a pool
        // always keeps a protected core; the WIDE `spread` still drives the dissolve bleed + warp
        // width. At `spread ≤ radius·½` this is a no-op (`core_r == spread`) → byte-identical.
        let blur = box_blur(&cov_src, rw, rh, core_r);

        let lut = luts();
        let brush = &self.paint.brush;
        let base = &**base_arc;
        let ground = &*backdrop_arc;

        // ── Wet-on-wet rewetting (`wet_rewet`, Enio 2026-07-06): where the wash covers already-painted
        // paint, the paint LIFTS (the base under it lightens toward the GROUND), its colour DISSOLVES
        // through the wet region (a one-shot diffusion blur — radius = the water `spread`, growing to
        // 2× where the brush LINGERED, the per-stroke soak), and POOLS back into the wash's density
        // (the edge term concentrates it at the rim: the bloom). Per-pixel — no brush reservoir, no
        // cadence, no physics. `wet = 0` skips it all (byte-identical, zero cost). Fields over the
        // read window: raw paint presence (for the local lift) + blurred presence-weighted base
        // colour at BOTH blur scales (the soak lerps between them) + the soak itself.
        // Rewet reads the PER-STROKE frozen base (refrozen each pen-down, so mid-session it
        // INCLUDES the union baked so far): "old paint" for lift/dissolve must see the neighbour
        // washes, which live in the union buffers and not in the session base.
        let rewet_base_arc = match self.paint.watercolor_base.as_ref() {
            Some(b) if b.len() == n * 4 => Arc::clone(b),
            _ => Arc::clone(&base_arc),
        };
        let rewet = (wet_any > 0.0 || watered).then(|| {
            build_rewet_fields(
                &rewet_base_arc[..],
                ground,
                &self.paint.wet_soak,
                soaked,
                &self.paint.stroke_water,
                watered,
                (fw, fh),
                (rx0, ry0, rx1, ry1),
                spread,
            )
        });

        let fill = brush.fill.clamp(0.0, 1.0);
        let depth = brush.depth.max(0.0);
        let edge_gain = brush.edge_gain.max(0.0);
        // Interior-thinning multiplier: 1.0 at/below the reference Spread (historical look), rising
        // toward `SPREAD_THIN_MAX` as Spread grows — a wider wet front empties the centre more.
        let spread_thin = (1.0 + (spread as f32 - SPREAD_THIN_REF).max(0.0) / SPREAD_THIN_REF)
            .min(SPREAD_THIN_MAX);
        let granulation = brush.granulation.clamp(0.0, 1.0);
        let pigment_mix = brush.effective_pigment_mix();
        // ── Paper (substrate tooth) + Granulation (mineral-settling mottle) — two canvas-anchored slots.
        // The Paper always textures the wash subtly (a physical substrate); the Granulation adds the
        // pronounced heavy-pigment settling by `granulation` amount, into its OWN map or (Same as Paper,
        // the default) the paper's tooth. An inactive Paper falls back to the built-in noise.
        let paper_tex = brush.paper;
        let paper_active = paper_tex.is_active();
        let paper_img = self.paint.paper_image.as_ref().map(|i| i.as_mask());
        // Precompute each slot's Angle rotation basis ONCE (the per-degree walk is not per-pixel-cheap).
        let paper_rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
        // The Granulation map is the **Grain** slot (`brush.texture`) — used only when "Same as Paper" is
        // off; otherwise the granulation settles into the paper's own tooth.
        let gran_tex = brush.texture;
        let gran_own_map = !brush.granulation_use_paper && gran_tex.is_active();
        let gran_img = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let gran_rot = ph2d_painter_brush::texture::angle_basis(gran_tex.angle_deg);
        let paper_depth = brush.paper_depth.clamp(0.0, 1.0);
        // Fallback pigment when the colour buffer is faint (straight brush colour → sRGB bytes).
        let fallback = [
            (brush.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (brush.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (brush.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ];
        let has_color = self.paint.stroke_color.len() == n * 4;
        // Manual textured tip (doc 13 #1 round 3): the per-stroke tip-density buffer scales the
        // interior fill (`cw·fill·dens`) — the tip's texture reads as pigment variation INSIDE a
        // normally-wet wash (water fills the tip's silhouette; texture modulates the deposit).
        // Empty (every non-textured path) ⇒ density ≡ 1 → byte-identical.
        let has_dens = self.paint.stroke_density.len() == n;
        let dens_buf = &self.paint.stroke_density;
        // Wet Mix pigment reserve (MIX-1): scales the whole BRUSH density term (fill + edge) AFTER
        // the rim is derived from the intact coverage — Charge depletion fades the pigment while the
        // water footprint (and so the edge anatomy) stays whole. Empty ⇒ factor ≡ 1 → byte-identical.
        let has_depl = self.paint.stroke_deplete.len() == n;
        let depl_buf = &self.paint.stroke_deplete;
        // EDGE-1 per-stroke style: owner map + table ([`WetSessionStyles`]); `cur_style` mirrors
        // the clamped globals above, so unowned pixels and style-less composites resolve to the
        // EXACT same values as before (bit-identical single-style path).
        let style_table = &self.paint.wet_styles.table;
        let style_owner = &self.paint.wet_styles.owner;
        let has_style = style_owner.len() == n && !style_table.is_empty();
        let cur_style = WetStrokeStyle {
            fill,
            depth,
            edge_gain,
            wet,
            granulation,
            warp: warp_amp,
            pigment_mix,
            color: fallback,
        };
        // Raw per-pixel soak for the granulation settle (GRAN-1) — read-only in the parallel loop.
        let soak_buf = &self.paint.wet_soak;
        let water_buf = &self.paint.stroke_water;

        let color_buf = &self.paint.stroke_color;
        // Substrate memoisation (perf, byte-identical): `paper_h` is canvas-anchored (a pure function of
        // the global `(gx, gy)` + the paper settings, both constant within a stroke), so compute it once
        // per canvas pixel and reuse across frames + the bake. `compute_paper` is the EXACT expression the
        // per-pixel loop used before; the cache stores/returns its `f32` verbatim (index-keyed, no
        // collision) ⇒ identical bytes. Falls back to direct compute if the cache isn't sized (defensive).
        let compute_paper = |gx: usize, gy: usize| -> f32 {
            if paper_active {
                ph2d_painter_brush::texture::sample_tiled_rot(
                    &paper_tex,
                    gx as i64,
                    gy as i64,
                    paper_img.as_ref(),
                    paper_rot,
                )
            } else {
                paper_height(gx as f32, gy as f32)
            }
        };
        let use_substrate_cache = self.paint.wet_substrate.len() == n;
        // Substrate pre-pass (serial): fill the memoised paper height for every OUTPUT pixel not yet
        // cached, so the PARALLEL composite below can read it immutably (no shared mutable state across
        // threads). Fill-on-miss ⇒ across a stroke's frames it's mostly hits.
        if use_substrate_cache {
            let substrate = &mut self.paint.wet_substrate;
            for by in 0..bh {
                let gy = y0 + by;
                for bx in 0..bw {
                    let sidx = gy * fw + (x0 + bx);
                    if substrate[sidx].is_nan() {
                        substrate[sidx] = compute_paper(x0 + bx, gy);
                    }
                }
            }
        }
        let substrate = &self.paint.wet_substrate;
        // Selection + protection gates (final enforcement): the splat gates already stop the wash from
        // FORMING on gated-out texels (so the rim/bleed react at the boundary — the masking-fluid look),
        // but the composite can still REACH them: the warp displaces the coverage sample (a gated-out
        // pixel can read in-bounds coverage up to `warp` px away) and the dissolve/soak fields blur
        // across the boundary. The keep-LERP on the final bytes below (`out = painted·keep +
        // base·(1−keep)`) is the exact restore semantics of the canvas gates
        // (`restore_deselected_region` / `restore_protected_region`) — a hard guarantee independent of
        // any sampling reach. Both `None` (the default) ⇒ `keep ≡ 1`, byte-identical.
        let (gate_sel, gate_prot) = self.wet_splat_gates();
        let gate_on = gate_sel.is_some() || gate_prot.is_some();
        let gsel: Option<&[u8]> = gate_sel.as_deref().map(Vec::as_slice);
        let gprot: Option<&[u8]> = gate_prot.as_deref().map(Vec::as_slice);
        let out = Arc::make_mut(&mut self.canvas_rgba);
        // PARALLEL composite over OUTPUT rows (ADR-0109 exception to the no-rayon default): each output
        // pixel is a pure function of immutable inputs (frozen base/ground, the coverage + blur fields,
        // the substrate cache, the LUTs) — no cross-pixel reduction, no shared mutable state, no RNG — so
        // distributing disjoint rows over the thread pool is BYTE-IDENTICAL to the serial loop (IEEE-754
        // is per-op deterministic). The band `out[y0..y1]` splits into whole canvas rows; each task writes
        // ONLY its own row's pixels (`row[gx*4..]`), reading the full-canvas base/ground/colour by `gi`.
        out[y0 * fw * 4..y1 * fw * 4]
            .par_chunks_mut(fw * 4)
            .enumerate()
            .for_each(|(by, row)| {
                let gy = y0 + by;
                let ly = (gy - ry0) as f32;
                for bx in 0..bw {
                    let gx = x0 + bx;
                    let lx = (gx - rx0) as f32;
                    let gi = (gy * fw + gx) * 4;
                    // Warp the sample position (organic boundary). Window-local coords for the read-window
                    // fields; global for the full-canvas colour buffer (same displacement + window origin).
                    // Per-stroke style: warp AMPLITUDE by the pixel's owner (read PRE-warp —
                    // the displacement needs the amp first); owner 0 = current brush, old path.
                    let st_warp = if has_style {
                        match style_owner[gy * fw + gx] {
                            0 => warp_amp,
                            o => style_table[(o as usize - 1).min(style_table.len() - 1)].warp,
                        }
                    } else {
                        warp_amp
                    };
                    let (sx, sy) = if st_warp > 0.0 {
                        let wx =
                            warp_axis(gx as f32, gy as f32, SEED_WARP_X_A, SEED_WARP_X_B) * st_warp;
                        let wy =
                            warp_axis(gx as f32, gy as f32, SEED_WARP_Y_A, SEED_WARP_Y_B) * st_warp;
                        (lx + wx, ly + wy)
                    } else {
                        (lx, ly)
                    };
                    let cw = smoothstep(SS0, SS1, sample_bilinear(&cov_src, rw, rh, sx, sy));
                    // EDGE-2 (backrun): the WATER channel at a SERRATED coord ([`water_at`]) — a
                    // water pool is live paint-surface even where the PIGMENT coverage is zero
                    // (pure water, Dilution 1), so the early-out only fires where BOTH are dry.
                    let (water, wxg, wyg) = if watered {
                        water_at(water_buf, fw, fh, gx, gy)
                    } else {
                        (0.0, 0.0, 0.0)
                    };
                    if cw <= 0.0 && water <= 0.0 {
                        // Outside the wash: restore the frozen base (peels any previous frame's composite).
                        row[gx * 4] = base[gi];
                        row[gx * 4 + 1] = base[gi + 1];
                        row[gx * 4 + 2] = base[gi + 2];
                        row[gx * 4 + 3] = base[gi + 3];
                        continue;
                    }
                    let inner = sample_bilinear(&blur, rw, rh, sx, sy).min(1.0);
                    // Warped canvas-space indices — shared by the tip-density / pigment-reserve /
                    // style-owner reads (nearest, like the colour buffer).
                    let wgx = (rx0 as f32 + sx).clamp(0.0, (fw - 1) as f32) as usize;
                    let wgy = (ry0 as f32 + sy).clamp(0.0, (fh - 1) as f32) as usize;
                    // Per-stroke style: the OWNER stroke's wash params (recency) — an older
                    // wash keeps ITS Concentration/Edge/water on the re-bake (Enio 2026-07-09).
                    let st = if has_style {
                        match style_owner[wgy * fw + wgx] {
                            0 => cur_style,
                            o => style_table[(o as usize - 1).min(style_table.len() - 1)],
                        }
                    } else {
                        cur_style
                    };
                    let mut edge = (cw * (1.0 - inner) * st.edge_gain).clamp(0.0, 1.0);
                    // Paper tooth (substrate): the active Paper slot, or the built-in noise fallback —
                    // memoised per canvas pixel (`compute_paper` is the identical expression; the cache just
                    // avoids recomputing it every frame for the same pixel).
                    let paper_h = if use_substrate_cache {
                        substrate[gy * fw + gx]
                    } else {
                        compute_paper(gx, gy)
                    };
                    // Granulation height source: its own map, or (Same as Paper) the paper's tooth —
                    // and NOTHING otherwise (no settling substrate ⇒ Amount inert, Enio 2026-07-06;
                    // the built-in noise fallback granulated out of thin air).
                    let gran_h = if gran_own_map {
                        Some(ph2d_painter_brush::texture::sample_tiled_rot(
                            &gran_tex,
                            gx as i64,
                            gy as i64,
                            gran_img.as_ref(),
                            gran_rot,
                        ))
                    } else if brush.granulation_use_paper {
                        Some(paper_h)
                    } else {
                        None
                    };
                    // The paper textures the wash by its Depth (only when a Paper is set) — the
                    // substrate's own subtle symmetric bite, unchanged.
                    let paper_component = if paper_active {
                        (paper_h - 0.5) * paper_depth
                    } else {
                        0.0
                    };
                    // GRAN-1 (Curtis §4.5, Tier-2): valley deposition + the take-3 drying model —
                    // extracted to [`granulation_factor`] (LOC cap); the amount + water follow the
                    // pixel's OWNER stroke (per-stroke style).
                    let soak_v = if soaked {
                        f32::from(soak_buf[gy * fw + gx]) / 255.0
                    } else {
                        0.0
                    };
                    let gran = granulation_factor(
                        gran_h,
                        paper_component,
                        st.granulation,
                        st.wet,
                        soak_v,
                        commit,
                    );
                    // Wet also wets the WASH ITSELF (blank canvas included): more water = the wash's own
                    // pigment redistributes — the interior thins toward the receding front, the edge pools
                    // harder, and the pooling follows the paper tooth (a wetter bloom is ragged, not a
                    // clean ring). This is what makes the Spread read intense + organic under Wet even
                    // before any old paint is involved (Enio 2026-07-06 "mais intenso e menos uniforme").
                    let mut fill_px = st.fill;
                    if st.wet > 0.0 {
                        fill_px =
                            st.fill * (1.0 - (WET_THIN * st.wet * spread_thin * inner).min(0.95));
                        let ragged = (1.0 + (paper_h - 0.5) * 2.0 * WET_RAGGED * st.wet).max(0.0);
                        edge = (edge * (1.0 + WET_EDGE_BOOST * st.wet) * ragged).clamp(0.0, 1.5); // LITERAL-PX-OK: wet edge may overshoot the dry clamp
                    }
                    // Tip density at the warped position (nearest, like the colour buffer).
                    let tip_dens = if has_dens {
                        f32::from(dens_buf[wgy * fw + wgx]) / 255.0
                    } else {
                        1.0
                    };
                    let mut density = ((cw * fill_px * tip_dens + edge) * gran).max(0.0);
                    // MIX-1: the brush's local pigment reserve (fresh + carry) fades fill AND edge
                    // together over the intact water footprint — the depleted tail dries toward
                    // plain water. Applied BEFORE the rewet-pool term: pigment dissolved off the
                    // CANVAS is not the brush's reserve and must not fade with it.
                    if has_depl {
                        density *= f32::from(depl_buf[wgy * fw + wgx]) / 255.0;
                    }
                    // Wet-on-wet lift / dissolve / pool / backrun ring — [`rewet_px`] (sibling,
                    // LOC split), verbatim math. `pool` is the density ADDITION from bloom + ring.
                    let rw_px = match &rewet {
                        Some(f) => rewet_px(
                            f,
                            (rx0, ry0),
                            (sx, sy),
                            (wxg, wyg),
                            water,
                            st.wet,
                            cw,
                            inner,
                        ),
                        None => Default::default(),
                    };
                    let (lift, dissolve, backrun, bleed, wet_paint) = (
                        rw_px.lift,
                        rw_px.dissolve,
                        rw_px.backrun,
                        rw_px.bleed,
                        rw_px.wet_paint,
                    );
                    density += rw_px.pool;
                    let od = density * st.depth;

                    // Pigment colour: the deposited (source-over) colour where present, else the brush colour.
                    let mut pig = if has_color {
                        let wgx = (rx0 as f32 + sx).clamp(0.0, (fw - 1) as f32);
                        let wgy = (ry0 as f32 + sy).clamp(0.0, (fh - 1) as f32);
                        let ci = (wgy as usize * fw + wgx as usize) * 4;
                        if color_buf[ci + 3] > COL_EPS {
                            [color_buf[ci], color_buf[ci + 1], color_buf[ci + 2]]
                        } else {
                            st.color
                        }
                    } else {
                        st.color
                    };
                    // Wet-on-wet DISSOLVE: the lifted paint's colour (diffused through the wet region)
                    // tints the wash's pigment — the old colour bleeds into and beyond its own footprint.
                    // SUBTRACTIVE mix (absorbance-space geometric mean, via the ln/exp LUTs): paints mix
                    // like pigments, not light — the linear sRGB lerp desaturated the blend toward the
                    // paper's cream ("pálida e amarelada sem Pigment", Enio 2026-07-06). Pigment ON still
                    // adds its full RYB pass on top, unchanged.
                    if dissolve > 0.0 {
                        for c in 0..3 {
                            let a = -lut.lnl[pig[c] as usize];
                            let bi = (bleed[c] + 0.5).clamp(0.0, 255.0) as usize;
                            let b = -lut.lnl[bi];
                            let mag = a + (b - a) * dissolve;
                            pig[c] = lut.l2s_byte(lut.exp_mag(mag));
                        }
                    }
                    // Backrun CONCENTRATION (EDGE-2): the ring's pigment is the pushed paint
                    // CONCENTRATED — Beer–Lambert saturates at the pigment colour, so density
                    // alone can never render darker than the wash the pigment came from; the
                    // "severely darkened edge" needs a darker floor (absorbance × ring).
                    if backrun > 0.0 {
                        for p in &mut pig {
                            let a = -lut.lnl[*p as usize] * (1.0 + BACKRUN_CONC * backrun);
                            *p = lut.l2s_byte(lut.exp_mag(a));
                        }
                    }

                    // Effective base in linear light: the layer's own pixels composited over the REAL
                    // ground (the backdrop under the active layer — so a transparent layer attenuates
                    // what is actually beneath it, not a virtual cream; an opaque base uses only itself).
                    let ab = f32::from(base[gi + 3]) / 255.0;
                    let ground_lin = [
                        lut.s2l[ground[gi] as usize],
                        lut.s2l[ground[gi + 1] as usize],
                        lut.s2l[ground[gi + 2] as usize],
                    ];
                    let mut sb = [
                        lut.s2l[base[gi] as usize] * ab + ground_lin[0] * (1.0 - ab),
                        lut.s2l[base[gi + 1] as usize] * ab + ground_lin[1] * (1.0 - ab),
                        lut.s2l[base[gi + 2] as usize] * ab + ground_lin[2] * (1.0 - ab),
                    ];
                    // Wet-on-wet LIFT: rewetting pulls the base's pigment off the ground. Density-
                    // proportional (log-space): remove a FRACTION of the optical density, so the colour
                    // walks its own Beer–Lambert curve toward the LOCAL ground — a lifted red on white
                    // reads PINK, on grey it reads grey-pink. (A linear lerp toward a global cream
                    // desaturated straight to cream — the yellow cast, Enio 2026-07-06.) Paint BRIGHTER
                    // than the ground (light pigment on a dark layer below) walks down the mirrored
                    // curve — both directions converge on the ground, never past it.
                    if lift > 0.0 {
                        for c in 0..3 {
                            let g = ground_lin[c].max(1e-4);
                            let ratio = sb[c] / g;
                            if ratio < 1.0 {
                                let mag = lut.absorbance(ratio) * (1.0 - lift);
                                sb[c] = g * lut.exp_mag(mag);
                            } else if ratio > 1.0 {
                                let mag =
                                    lut.absorbance((g / sb[c]).clamp(0.0, 1.0)) * (1.0 - lift);
                                sb[c] = (g / lut.exp_mag(mag).max(1e-4)).min(1.0);
                            }
                        }
                    }
                    let mut rgb = [0u8; 3];
                    let mut t_lum = 0.0f32;
                    let mut t_min = 1.0f32;
                    const LUM: [f32; 3] = [0.2126, 0.7152, 0.0722];
                    for c in 0..3 {
                        let t = lut.transmittance(pig[c], od);
                        let lin = sb[c] * t + lut.s2l[pig[c] as usize] * (1.0 - t);
                        rgb[c] = lut.l2s_byte(lin);
                        t_lum += LUM[c] * t;
                        t_min = t_min.min(t);
                    }
                    // Perceptual film opacity — how much pigment sits here; drives the deposited alpha and
                    // the subtractive (RYB) paint-mix amount over the base paint.
                    let film_a = (1.0 - t_lum).clamp(0.0, 1.0);
                    // The subtractive wet-on-wet MIX — the wash's pigment blending with the paint beneath
                    // like real paint. It is "o segredo" of the good wet-on-wet look (Enio 2026-07-06), so
                    // **Wet drives it too**: `max(Pigment's Mix, wet × paint-presence)` — a wet wash mixes
                    // with what it rewets even with Pigment unchecked (inert on blank canvas, like the
                    // lift); the checkbox + Mix slider still set the floor and are the only source when
                    // Wet is 0 (byte-identical default preserved). The blend reads the LIFTED base (`sb`)
                    // where the rewet lightened it — mixing against the raw base would paint the lift
                    // right back over.
                    let mix_amt = st.pigment_mix.max(st.wet.max(water) * wet_paint);
                    if mix_amt > 0.0 {
                        // The (possibly lifted) base APPEARANCE over the ground — for an opaque base with
                        // no lift this is the raw base bytes exactly (`l2s(s2l(b)) == b`); for a
                        // transparent one it is the ground showing through (the old raw-bytes read was
                        // black there).
                        let mix_base = [
                            f32::from(lut.l2s_byte(sb[0])) / 255.0,
                            f32::from(lut.l2s_byte(sb[1])) / 255.0,
                            f32::from(lut.l2s_byte(sb[2])) / 255.0,
                        ];
                        let mixed = ryb_mix(
                            mix_base,
                            [
                                f32::from(pig[0]) / 255.0,
                                f32::from(pig[1]) / 255.0,
                                f32::from(pig[2]) / 255.0,
                            ],
                            film_a,
                        );
                        for c in 0..3 {
                            let sub = (mixed[c].clamp(0.0, 1.0) * 255.0 + 0.5).clamp(0.0, 255.0);
                            rgb[c] =
                                (f32::from(rgb[c]) + (sub - f32::from(rgb[c])) * mix_amt) as u8;
                        }
                    }
                    // Coverage alpha = the STRONGEST per-channel absorption (`1 − min_c T_c`), not the
                    // luminance film: the un-premultiply below needs `a ≥ 1 − T_c` on EVERY channel or
                    // the solve leaves gamut and clamps (a red wash's G/B absorb far more than the
                    // luminance says — measured 59-byte flatten error with the luminance alpha).
                    // `film_a` stays the perceptual meter for the paint-mix strength above.
                    let cov_a = (1.0 - t_min).clamp(0.0, 1.0);
                    let out_a = (ab + (1.0 - ab) * cov_a).clamp(0.0, 1.0);
                    // `rgb` is the target APPEARANCE over the ground. The layer stores straight RGBA
                    // that the compositor will blend over that same ground — so solve the un-premultiply
                    // `L = (appearance − ground·(1−a)) / a` in linear light. Baking the appearance
                    // directly (the old path) baked the ground INTO the pixels: over a white backdrop
                    // the wash carried a permanent cream cast ("puxa para o bege", Enio 2026-07-06).
                    // Opaque base ⇒ a = 1 ⇒ L = appearance, byte-identical to the old path.
                    if out_a <= f32::EPSILON {
                        // No film and no base: the layer stays untouched (appearance == ground).
                        row[gx * 4] = base[gi];
                        row[gx * 4 + 1] = base[gi + 1];
                        row[gx * 4 + 2] = base[gi + 2];
                        row[gx * 4 + 3] = base[gi + 3];
                        continue;
                    }
                    let inv_a = 1.0 / out_a;
                    let mut px = [0u8; 4];
                    for c in 0..3 {
                        let app = lut.s2l[rgb[c] as usize];
                        let lin = (app - ground_lin[c] * (1.0 - out_a)) * inv_a;
                        px[c] = lut.l2s_byte(lin);
                    }
                    px[3] = (out_a * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                    // Paint gates (selection / protection): keep-lerp the painted bytes toward the
                    // frozen base — the canvas gates' exact restore semantics, warp/diffusion-proof
                    // (see the gate hoist above the loop). Ungated (default) writes paint verbatim.
                    if gate_on {
                        let keep = watercolor_accum::splat_keep(gsel, gprot, gy * fw + gx);
                        if keep < 1.0 {
                            for (c, p) in px.iter_mut().enumerate() {
                                let painted = f32::from(*p);
                                let orig = f32::from(base[gi + c]);
                                *p = (painted * keep + orig * (1.0 - keep))
                                    .round()
                                    .clamp(0.0, 255.0) as u8;
                            }
                        }
                    }
                    row[gx * 4] = px[0];
                    row[gx * 4 + 1] = px[1];
                    row[gx * 4 + 2] = px[2];
                    row[gx * 4 + 3] = px[3];
                }
            });
        self.mark_dirty(region);
        if commit {
            self.paint.watercolor_base = None;
        }
        Some(region)
    }
}
