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
use super::*;
use ph2d_painter_brush::blend::ryb_mix;

/// Hardened-coverage smoothstep edges (wet_edges `SS0`/`SS1`): below `SS0` the wash is transparent,
/// above `SS1` fully covered — a crisp-but-soft silhouette from the feathered coverage discs.
const SS0: f32 = 0.12; // LITERAL-PX-OK: coverage-hardening smoothstep low edge (wet_edges)
const SS1: f32 = 0.60; // LITERAL-PX-OK: coverage-hardening smoothstep high edge (wet_edges)
/// Virtual **paper** colour (wet_edges `PAPER`): where the painted layer is transparent, the optical
/// model composites the wash over this so the Beer–Lambert base is a real paper, not black. On an
/// already-opaque base (a paper fill / prior paint) the layer's own pixels are used instead.
const PAPER: [u8; 3] = [239, 233, 220];
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
        // The frozen base (own the Arc handle so the canvas make_mut below doesn't alias the field).
        let base_arc = self.paint.watercolor_base.as_ref().map(Arc::clone)?;
        if base_arc.len() != n * 4 {
            if commit {
                self.paint.watercolor_base = None;
            }
            return None;
        }

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

        let spread = self.paint.brush.edge_spread.round().clamp(0.0, 24.0) as usize;
        let warp_amp = self.paint.brush.warp.max(0.0);
        // Influence radius of a dab beyond its own disc: the blur spread + the warp displacement +
        // bilinear/rounding slack. Output pixels farther than this from the dirty rect cannot have changed.
        let pad = spread + warp_amp.ceil() as usize + 2;
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
        let blur = box_blur(&cov_src, rw, rh, spread);

        let lut = luts();
        let brush = &self.paint.brush;
        let base = &**base_arc;

        // ── Wet-on-wet rewetting (`wet_rewet`, Enio 2026-07-06): where the wash covers already-painted
        // paint, the paint LIFTS (the base under it lightens toward the paper), its colour DISSOLVES
        // through the wet region (a one-shot diffusion blur — radius = the same water `spread`), and
        // POOLS back into the wash's density (the edge term concentrates it at the rim: the bloom).
        // Per-pixel and stateless — no brush reservoir, no cadence, no physics. `wet = 0` skips it all
        // (byte-identical, zero cost). Fields over the read window: raw paint presence (for the local
        // lift) + blurred presence-weighted base colour (the bleed field the dissolve tints from).
        let wet = brush.wet_rewet.clamp(0.0, 1.0);
        let rewet = (wet > 0.0).then(|| {
            let mut pres = vec![0.0f32; rw * rh];
            let mut wr = vec![0.0f32; rw * rh];
            let mut wg = vec![0.0f32; rw * rh];
            let mut wb = vec![0.0f32; rw * rh];
            for wy in 0..rh {
                let sbase = ((ry0 + wy) * fw + rx0) * 4;
                let dbase = wy * rw;
                for wx in 0..rw {
                    let bi = sbase + wx * 4;
                    let ab = f32::from(base[bi + 3]) / 255.0;
                    // The base over the virtual paper, straight sRGB bytes (the paint as seen).
                    let r = f32::from(base[bi]) * ab + f32::from(PAPER[0]) * (1.0 - ab);
                    let g = f32::from(base[bi + 1]) * ab + f32::from(PAPER[1]) * (1.0 - ab);
                    let b = f32::from(base[bi + 2]) * ab + f32::from(PAPER[2]) * (1.0 - ab);
                    // Presence = how much this pixel DARKENS the paper (pigment only absorbs —
                    // Beer-Lambert). Anything at/above the paper's own brightness is NOT liftable
                    // paint: a plain WHITE canvas is brighter than the cream virtual paper, and a
                    // symmetric colour-distance read it as ~0.8 presence everywhere — the pool then
                    // flooded the wash interior and drowned the receding-edge (Spread) dynamic
                    // (Enio 2026-07-06, "matou o efeito dinâmico do spread de novo"). Dead-zoned so
                    // paper grain doesn't count as paint (wet_edges `PAINT_LO`/`PAINT_HI`).
                    let d = (f32::from(PAPER[0]) - r)
                        .max(f32::from(PAPER[1]) - g)
                        .max(f32::from(PAPER[2]) - b)
                        .max(0.0);
                    let p = smoothstep(14.0, 50.0, d); // LITERAL-PX-OK: wet_edges PAINT_LO/PAINT_HI
                    let di = dbase + wx;
                    pres[di] = p;
                    wr[di] = r * p; // presence-premultiplied, so the blur averages PAINT colour only
                    wg[di] = g * p;
                    wb[di] = b * p;
                }
            }
            let bpres = box_blur(&pres, rw, rh, spread);
            let br = box_blur(&wr, rw, rh, spread);
            let bg = box_blur(&wg, rw, rh, spread);
            let bb = box_blur(&wb, rw, rh, spread);
            (pres, bpres, br, bg, bb)
        });
        /// Max fraction of the base's pigment the rewet lifts at `wet = 1` under full water (never a
        /// full erase — dried pigment doesn't fully redissolve).
        const REWET_LIFT: f32 = 0.85;
        /// How much of the dissolved pigment re-enters the wash's optical density (the bloom's body).
        const REWET_POOL: f32 = 0.35;
        /// How much a fully-wet wash thins its own interior fill (deepest where `inner` ≈ 1 — the
        /// pigment migrated out to the receding front; the rim keeps full body).
        const WET_THIN: f32 = 0.35;
        /// Edge-pool gain of a fully-wet wash (`wet = 1` doubles the receding-front pooling).
        const WET_EDGE_BOOST: f32 = 1.0;
        /// How strongly the paper tooth modulates the wet edge (a wet bloom is ragged, not a clean
        /// ring): ±75% of the pool at `wet = 1`.
        const WET_RAGGED: f32 = 0.75;

        let fill = brush.fill.clamp(0.0, 1.0);
        let depth = brush.depth.max(0.0);
        let edge_gain = brush.edge_gain.max(0.0);
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
        let paper_lin = [
            lut.s2l[PAPER[0] as usize],
            lut.s2l[PAPER[1] as usize],
            lut.s2l[PAPER[2] as usize],
        ];

        let color_buf = &self.paint.stroke_color;
        let out = Arc::make_mut(&mut self.canvas_rgba);
        for by in 0..bh {
            let gy = y0 + by;
            let ly = (gy - ry0) as f32;
            for bx in 0..bw {
                let gx = x0 + bx;
                let lx = (gx - rx0) as f32;
                let gi = (gy * fw + gx) * 4;
                // Warp the sample position (organic boundary). Window-local coords for the read-window
                // fields; global for the full-canvas colour buffer (same displacement + window origin).
                let (sx, sy) = if warp_amp > 0.0 {
                    let wx =
                        warp_axis(gx as f32, gy as f32, SEED_WARP_X_A, SEED_WARP_X_B) * warp_amp;
                    let wy =
                        warp_axis(gx as f32, gy as f32, SEED_WARP_Y_A, SEED_WARP_Y_B) * warp_amp;
                    (lx + wx, ly + wy)
                } else {
                    (lx, ly)
                };
                let cw = smoothstep(SS0, SS1, sample_bilinear(&cov_src, rw, rh, sx, sy));
                if cw <= 0.0 {
                    // Outside the wash: restore the frozen base (peels any previous frame's composite).
                    out[gi] = base[gi];
                    out[gi + 1] = base[gi + 1];
                    out[gi + 2] = base[gi + 2];
                    out[gi + 3] = base[gi + 3];
                    continue;
                }
                let inner = sample_bilinear(&blur, rw, rh, sx, sy).min(1.0);
                let mut edge = (cw * (1.0 - inner) * edge_gain).clamp(0.0, 1.0);
                // Paper tooth (substrate): the active Paper slot, or the built-in noise fallback.
                let paper_h = if paper_active {
                    ph2d_painter_brush::texture::sample_tiled_rot(
                        &paper_tex,
                        gx as i64,
                        gy as i64,
                        paper_img.as_ref(),
                        paper_rot,
                    )
                } else {
                    paper_height(gx as f32, gy as f32)
                };
                // Granulation field: its own map, or (Same as Paper) the paper's tooth — and NOTHING
                // otherwise: with no Grain image and Same-as-Paper off there is no settling substrate,
                // so Amount must be inert (falling through to the built-in noise granulated out of thin
                // air — Enio 2026-07-06).
                let gran_component = if gran_own_map {
                    let g = ph2d_painter_brush::texture::sample_tiled_rot(
                        &gran_tex,
                        gx as i64,
                        gy as i64,
                        gran_img.as_ref(),
                        gran_rot,
                    );
                    (g - 0.5) * 2.0 * granulation
                } else if brush.granulation_use_paper {
                    (paper_h - 0.5) * 2.0 * granulation
                } else {
                    0.0
                };
                // Additive: the paper textures the wash by its Depth (only when a Paper is set); the
                // granulation adds the pronounced mineral mottle by amount.
                let paper_component = if paper_active {
                    (paper_h - 0.5) * paper_depth
                } else {
                    0.0
                };
                let gran = (1.0 + paper_component + gran_component).max(0.0);
                // Wet also wets the WASH ITSELF (blank canvas included): more water = the wash's own
                // pigment redistributes — the interior thins toward the receding front, the edge pools
                // harder, and the pooling follows the paper tooth (a wetter bloom is ragged, not a
                // clean ring). This is what makes the Spread read intense + organic under Wet even
                // before any old paint is involved (Enio 2026-07-06 "mais intenso e menos uniforme").
                let mut fill_px = fill;
                if wet > 0.0 {
                    fill_px = fill * (1.0 - WET_THIN * wet * inner);
                    let ragged = (1.0 + (paper_h - 0.5) * 2.0 * WET_RAGGED * wet).max(0.0);
                    edge = (edge * (1.0 + WET_EDGE_BOOST * wet) * ragged).clamp(0.0, 1.5); // LITERAL-PX-OK: wet edge may overshoot the dry clamp
                }
                let mut density = ((cw * fill_px + edge) * gran).max(0.0);
                // Wet-on-wet: sample the bleed field (blurred presence + presence-weighted paint colour)
                // at the warped position; `lp` = raw local presence (only real paint lifts), `bp` = how
                // much dissolved pigment reaches this pixel, `bleed` = its (presence-normalised) colour.
                let mut lift = 0.0f32;
                let mut dissolve = 0.0f32;
                let mut bleed = [0.0f32; 3];
                let mut wet_paint = 0.0f32; // local paint presence — gates the wet-driven paint-mix
                if let Some((pres, bpres, br, bg, bb)) = &rewet {
                    let lp = sample_bilinear(pres, rw, rh, sx, sy);
                    wet_paint = lp.clamp(0.0, 1.0);
                    let bp = sample_bilinear(bpres, rw, rh, sx, sy).clamp(0.0, 1.0);
                    lift = REWET_LIFT * wet * cw * lp;
                    if bp > 1e-4 {
                        let inv = 1.0 / bp;
                        bleed = [
                            sample_bilinear(br, rw, rh, sx, sy) * inv,
                            sample_bilinear(bg, rw, rh, sx, sy) * inv,
                            sample_bilinear(bb, rw, rh, sx, sy) * inv,
                        ];
                        dissolve = wet * bp;
                        // The dissolved pigment re-enters the wash as optical density AT THE RECEDING
                        // FRONT (the same rim shape as the edge term, gain-independent): pigment in
                        // suspension migrates to the wet boundary — the bloom. A UNIFORM pool flooded
                        // the interior and flattened the Spread dynamic (interior must CLEAR as the
                        // frontier advances; the lift even enhances that over old paint).
                        density += REWET_POOL * wet * bp * (cw * (1.0 - inner)).clamp(0.0, 1.0);
                    }
                }
                let od = density * depth;

                // Pigment colour: the deposited (source-over) colour where present, else the brush colour.
                let mut pig = if has_color {
                    let wgx = (rx0 as f32 + sx).clamp(0.0, (fw - 1) as f32);
                    let wgy = (ry0 as f32 + sy).clamp(0.0, (fh - 1) as f32);
                    let ci = (wgy as usize * fw + wgx as usize) * 4;
                    if color_buf[ci + 3] > COL_EPS {
                        [color_buf[ci], color_buf[ci + 1], color_buf[ci + 2]]
                    } else {
                        fallback
                    }
                } else {
                    fallback
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

                // Effective base in linear light: the layer's own pixels composited over the virtual paper
                // (so a transparent layer still has a paper to attenuate; an opaque base uses only itself).
                let ab = f32::from(base[gi + 3]) / 255.0;
                let mut sb = [
                    lut.s2l[base[gi] as usize] * ab + paper_lin[0] * (1.0 - ab),
                    lut.s2l[base[gi + 1] as usize] * ab + paper_lin[1] * (1.0 - ab),
                    lut.s2l[base[gi + 2] as usize] * ab + paper_lin[2] * (1.0 - ab),
                ];
                // Wet-on-wet LIFT: rewetting pulls the base's pigment off the paper. Density-
                // proportional (log-space): remove a FRACTION of the optical density, so the colour
                // walks its own Beer–Lambert curve toward the paper — a lifted red reads PINK. (The
                // linear lerp toward the cream paper desaturated straight to cream — the yellow cast,
                // Enio 2026-07-06.) Channels at/above the paper's brightness are left alone (nothing
                // to lift there; pulling them DOWN to the paper would darken, not lift).
                if lift > 0.0 {
                    for c in 0..3 {
                        let ratio = sb[c] / paper_lin[c];
                        if ratio < 1.0 {
                            let mag = lut.absorbance(ratio) * (1.0 - lift);
                            sb[c] = paper_lin[c] * lut.exp_mag(mag);
                        }
                    }
                }
                let mut rgb = [0u8; 3];
                let mut t_lum = 0.0f32;
                const LUM: [f32; 3] = [0.2126, 0.7152, 0.0722];
                for c in 0..3 {
                    let t = lut.transmittance(pig[c], od);
                    let lin = sb[c] * t + lut.s2l[pig[c] as usize] * (1.0 - t);
                    rgb[c] = lut.l2s_byte(lin);
                    t_lum += LUM[c] * t;
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
                let mix_amt = pigment_mix.max(wet * wet_paint);
                if mix_amt > 0.0 {
                    let mix_base = if lift > 0.0 {
                        [
                            f32::from(lut.l2s_byte(sb[0])) / 255.0,
                            f32::from(lut.l2s_byte(sb[1])) / 255.0,
                            f32::from(lut.l2s_byte(sb[2])) / 255.0,
                        ]
                    } else {
                        [
                            f32::from(base[gi]) / 255.0,
                            f32::from(base[gi + 1]) / 255.0,
                            f32::from(base[gi + 2]) / 255.0,
                        ]
                    };
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
                        rgb[c] = (f32::from(rgb[c]) + (sub - f32::from(rgb[c])) * mix_amt) as u8;
                    }
                }
                let out_a = (ab + (1.0 - ab) * film_a).clamp(0.0, 1.0);
                out[gi] = rgb[0];
                out[gi + 1] = rgb[1];
                out[gi + 2] = rgb[2];
                out[gi + 3] = (out_a * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            }
        }
        self.mark_dirty(region);
        if commit {
            self.paint.watercolor_base = None;
        }
        Some(region)
    }
}
