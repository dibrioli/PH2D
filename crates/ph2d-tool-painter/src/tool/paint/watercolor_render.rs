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

use super::*;
use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
use ph2d_painter_brush::blend::ryb_mix;
use std::sync::OnceLock;

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

/// Fixed strength of the **Paper** substrate tooth in the density (how much an active Paper always
/// textures the wash, before the separate Granulation amount): `±PAPER_TOOTH/2` around neutral. A paper
/// property, not a slider — the artist controls its look via the Paper preset + Contrast + Size.
const PAPER_TOOTH: f32 = 0.5; // LITERAL-OK: watercolor paper-substrate tooth strength

// ── Coverage / colour feather (wet_edges `stampCoverage` radial-gradient stops) ─────────────────────
/// The soft-disc weight at normalised radius `dn ∈ [0, 1]`: `1.0` at the centre, `0.92` at `0.62`,
/// `0` at the rim (the two-segment linear gradient of `stampCoverage` / `stampColor`).
fn feather(dn: f32) -> f32 {
    if dn <= 0.62 {
        1.0 - dn / 0.62 * 0.08 // 1.00 → 0.92 across the plateau
    } else {
        0.92 * (1.0 - (dn - 0.62) / 0.38) // 0.92 → 0 across the rim
    }
}

// ── LUTs (built once; the ln/exp/pow run only here, never per pixel — HR-5) ──────────────────────────
const L2S_N: usize = 4096;
const EXP_N: usize = 2048;
/// Largest `|exponent|` the Beer–Lambert `exp` LUT spans; `exp(-32) ≈ 1.3e-14 ≈ 0`, so anything past
/// this is transmittance 0 (opaque pigment) — a safe clamp for even a very dense wash.
const EXP_MAX: f32 = 32.0;

struct Luts {
    /// sRGB byte → linear-light intensity.
    s2l: [f32; 256],
    /// `ln(max(linear, 1e-4))` per sRGB byte — the pigment's log-transmittance (clamp avoids `-∞` on black).
    lnl: [f32; 256],
    /// linear-light intensity (`[0, 1]`, quantised to `L2S_N + 1`) → sRGB byte.
    l2s: [u8; L2S_N + 1],
    /// `exp(-mag)` for `mag ∈ [0, EXP_MAX]` (quantised to `EXP_N + 1`) — Beer–Lambert transmittance.
    exp_neg: [f32; EXP_N + 1],
}

fn luts() -> &'static Luts {
    static LUTS: OnceLock<Luts> = OnceLock::new();
    LUTS.get_or_init(|| {
        let mut s2l = [0.0f32; 256];
        let mut lnl = [0.0f32; 256];
        for i in 0..256 {
            let lin = srgb_to_linear_byte(i as u8);
            s2l[i] = lin;
            lnl[i] = lin.max(1e-4).ln();
        }
        let mut l2s = [0u8; L2S_N + 1];
        for (i, slot) in l2s.iter_mut().enumerate() {
            *slot = linear_to_srgb_byte(i as f32 / L2S_N as f32);
        }
        let mut exp_neg = [0.0f32; EXP_N + 1];
        for (i, slot) in exp_neg.iter_mut().enumerate() {
            *slot = (-(EXP_MAX * i as f32 / EXP_N as f32)).exp();
        }
        Luts {
            s2l,
            lnl,
            l2s,
            exp_neg,
        }
    })
}

impl Luts {
    /// linear intensity → sRGB byte via the LUT (clamped index).
    #[inline]
    fn l2s_byte(&self, v: f32) -> u8 {
        let idx = (v.clamp(0.0, 1.0) * L2S_N as f32) as usize;
        self.l2s[idx.min(L2S_N)]
    }
    /// Beer–Lambert transmittance `pigment^(od)` for a pigment byte `c` and optical depth `od ≥ 0`,
    /// via `exp(lnl[c]·od)` looked up in the `exp` LUT (`lnl[c] ≤ 0` ⇒ the exponent is `≤ 0`).
    #[inline]
    fn transmittance(&self, c: u8, od: f32) -> f32 {
        let mag = -self.lnl[c as usize] * od; // = |exponent|, ≥ 0
        let idx = (mag / EXP_MAX * EXP_N as f32) as usize;
        self.exp_neg[idx.min(EXP_N)]
    }
}

// ── Deterministic value noise (integer hash; HR-5 transcendental-free) ───────────────────────────────
// Distinct seeds keep the two warp axes + the paper grain decorrelated (else the boundary would ripple
// along the diagonal and the granulation would track the warp).
const SEED_WARP_X_A: u32 = 0x1111_1111;
const SEED_WARP_X_B: u32 = 0x2222_2222;
const SEED_WARP_Y_A: u32 = 0x3333_3333;
const SEED_WARP_Y_B: u32 = 0x4444_4444;
const SEED_GRAIN: u32 = 0x5555_5555;

/// Smoothstep-in-`[0,1]` fade (the value-noise interpolation weight; a cubic, so no transcendental).
#[inline]
fn smooth01(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Integer hash of a lattice cell → `[0, 1)` (a `lowbias32`-style avalanche; deterministic + fast).
#[inline]
fn hash2(ix: i32, iy: i32, seed: u32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x8DA6_B343)
        ^ (iy as u32).wrapping_mul(0xD816_3841)
        ^ seed.wrapping_mul(0x1B56_C4E9);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Bilinear value noise with `cell`-px features at `(x, y)` → `[0, 1]` (wet_edges `valueNoise`).
#[inline]
fn value_noise(x: f32, y: f32, cell: f32, seed: u32) -> f32 {
    let fx = x / cell;
    let fy = y / cell;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let sx = smooth01(fx - x0);
    let sy = smooth01(fy - y0);
    let a = hash2(ix, iy, seed);
    let b = hash2(ix + 1, iy, seed);
    let c = hash2(ix, iy + 1, seed);
    let d = hash2(ix + 1, iy + 1, seed);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Fractal displacement field (two octaves, cells 22/8 px) in `[-1, 1]` for a warp axis (wet_edges warp).
#[inline]
fn warp_axis(x: f32, y: f32, sa: u32, sb: u32) -> f32 {
    (value_noise(x, y, 22.0, sa) * 0.65 + value_noise(x, y, 8.0, sb) * 0.35 - 0.5) * 2.0
}

/// Paper-tooth granulation height at `(x, y)` in `[0, 1]` (wet_edges `grain`, 5-px value noise). A
/// built-in field so "Aquarela Básica" granulates before a Paper texture (Fase C) is wired to the Grain slot.
#[inline]
fn paper_height(x: f32, y: f32) -> f32 {
    value_noise(x, y, 5.0, SEED_GRAIN)
}

/// Smoothstep from `e0` to `e1` evaluated at `x` (cubic; clamps outside the edges).
#[inline]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    if e1 <= e0 {
        return if x < e0 { 0.0 } else { 1.0 };
    }
    smooth01(((x - e0) / (e1 - e0)).clamp(0.0, 1.0))
}

/// Bilinear sample of a region-local scalar field (clamped to the field edges).
#[inline]
fn sample_bilinear(src: &[f32], w: usize, h: usize, fx: f32, fy: f32) -> f32 {
    let fx = fx.clamp(0.0, (w - 1) as f32);
    let fy = fy.clamp(0.0, (h - 1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let a = src[y0 * w + x0];
    let b = src[y0 * w + x1];
    let c = src[y1 * w + x0];
    let d = src[y1 * w + x1];
    let top = a + (b - a) * tx;
    let bot = c + (d - c) * tx;
    top + (bot - top) * ty
}

impl PainterTool {
    /// Whether the watercolor optical render-path drives this stroke: the Watercolor section is on, we're
    /// the normal **Paint** brush (not Smear/Blur/Clone/Mask/Inpaint/Fill/Selection/Deform), and not
    /// erasing. Off ⇒ a byte-identical plain brush (the whole path is skipped, deposit + composite alike).
    pub(super) fn watercolor_render_active(&self) -> bool {
        self.paint.brush.watercolor
            && matches!(self.paint.paint_mode, PaintMode::Paint)
            && !self.paint.eraser
    }

    /// Zero the per-stroke coverage (retain capacity). Shape-preview methods (Drag Dot / Anchored / Line)
    /// rebuild coverage each frame from the current batch, so the moving preview leaves no trail.
    pub(super) fn clear_wet_coverage(&mut self) {
        self.paint.stroke_coverage.iter_mut().for_each(|c| *c = 0);
    }

    /// Zero the per-stroke deposited-colour buffer (retain capacity); twin of [`Self::clear_wet_coverage`].
    pub(super) fn clear_wet_color(&mut self) {
        self.paint.stroke_color.iter_mut().for_each(|c| *c = 0);
    }

    /// Splat each dab's soft disc into the per-stroke coverage (max-blend = the wet "one pass" union,
    /// independent of colour, like wet_edges `stampCoverage`). Coverage is geometric but scaled by the
    /// dab's own opacity, so a faint wash pools a fainter rim.
    pub(super) fn accumulate_wet_coverage(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_coverage.len() != fw * fh {
            self.paint.stroke_coverage = vec![0u8; fw * fh];
        }
        let cov = &mut self.paint.stroke_coverage;
        for d in dabs {
            let r = d.radius_px;
            let peak = d.coverage.clamp(0.0, 1.0);
            if r <= 0.0 || peak <= 0.0 {
                continue;
            }
            let inv_r = 1.0 / r;
            let (cx, cy) = (d.center[0], d.center[1]);
            let x0 = (cx - r).floor().max(0.0) as usize;
            let y0 = (cy - r).floor().max(0.0) as usize;
            let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
            let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
            for y in y0..y1 {
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let v = (peak * feather(dn) * 255.0) as u8;
                    let idx = base + x;
                    if v > cov[idx] {
                        cov[idx] = v;
                    }
                }
            }
        }
    }

    /// Splat each dab's colour into the per-stroke colour buffer, **source-over** (recent dab wins on
    /// overlap, carrying its colour — wet_edges `stampColor`), straight-alpha RGBA. Same soft-disc
    /// feather as the coverage, so the deposited colour and the silhouette agree.
    pub(super) fn accumulate_wet_color(&mut self, dabs: &[Dab]) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return;
        }
        if self.paint.stroke_color.len() != fw * fh * 4 {
            self.paint.stroke_color = vec![0u8; fw * fh * 4];
        }
        let buf = &mut self.paint.stroke_color;
        for d in dabs {
            let r = d.radius_px;
            let peak = d.coverage.clamp(0.0, 1.0);
            if r <= 0.0 || peak <= 0.0 {
                continue;
            }
            let col = [
                (d.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (d.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (d.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ];
            let inv_r = 1.0 / r;
            let (cx, cy) = (d.center[0], d.center[1]);
            let x0 = (cx - r).floor().max(0.0) as usize;
            let y0 = (cy - r).floor().max(0.0) as usize;
            let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
            let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
            for y in y0..y1 {
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                for x in x0..x1 {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let a = peak * feather(dn); // source alpha
                    if a <= 0.0 {
                        continue;
                    }
                    let idx = (base + x) * 4;
                    let da = f32::from(buf[idx + 3]) / 255.0;
                    let na = a + da * (1.0 - a); // straight-alpha over
                    if na <= 0.0 {
                        continue;
                    }
                    for c in 0..3 {
                        let dc = f32::from(buf[idx + c]) / 255.0;
                        let sc = f32::from(col[c]) / 255.0;
                        let out = (sc * a + dc * da * (1.0 - a)) / na;
                        buf[idx + c] = (out * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                    }
                    buf[idx + 3] = (na * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    /// Composite the watercolor wash over the frozen base ([`PaintState::watercolor_base`](super::PaintState)).
    ///
    /// Reads the coverage + deposited-colour buffers, reconstructs the optical density `D`, and applies
    /// per-channel Beer–Lambert in linear light — see the module docs. The base is a separate `Arc`, so
    /// each frame recomposites cleanly from the pristine pre-stroke pixels (no overlay peel). `commit`
    /// drops the base (pen-up bake, inside the undo transaction); the live passes keep it for the next frame.
    pub(super) fn apply_watercolor(&mut self, commit: bool) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let n = fw * fh;
        if n == 0 || self.paint.stroke_coverage.len() != n || self.canvas_rgba.len() != n * 4 {
            if commit {
                self.paint.watercolor_base = None;
            }
            return;
        }
        // The frozen base (own the Arc handle so the canvas make_mut below doesn't alias the field).
        let Some(base_arc) = self.paint.watercolor_base.as_ref().map(Arc::clone) else {
            return;
        };
        if base_arc.len() != n * 4 {
            if commit {
                self.paint.watercolor_base = None;
            }
            return;
        }

        // Cumulative coverage bbox, padded by the blur radius + the warp displacement so the feathered
        // rim + the warped samples are all read (wet_edges pads the read window the same way).
        let (mut minx, mut miny, mut maxx, mut maxy) = (fw, fh, 0usize, 0usize);
        let mut any = false;
        for y in 0..fh {
            let row = y * fw;
            for x in 0..fw {
                if self.paint.stroke_coverage[row + x] > 0 {
                    any = true;
                    minx = minx.min(x);
                    maxx = maxx.max(x);
                    miny = miny.min(y);
                    maxy = maxy.max(y);
                }
            }
        }
        if !any {
            if commit {
                self.paint.watercolor_base = None;
            }
            return;
        }
        let spread = self.paint.brush.edge_spread.round().clamp(0.0, 24.0) as usize;
        let warp_amp = self.paint.brush.warp.max(0.0);
        let pad = spread + warp_amp.ceil() as usize + 2;
        let x0 = minx.saturating_sub(pad);
        let y0 = miny.saturating_sub(pad);
        let x1 = (maxx + pad + 1).min(fw);
        let y1 = (maxy + pad + 1).min(fh);
        let (bw, bh) = (x1 - x0, y1 - y0);
        let region = Region {
            x: x0 as u32,
            y: y0 as u32,
            w: bw as u32,
            h: bh as u32,
        };

        // Region-local coverage (`[0,1]`) + its blur (the padding reads the true 0 outside the stroke,
        // so the blur feathers correctly at the real rim).
        let mut cov_src = vec![0.0f32; bw * bh];
        for by in 0..bh {
            let sbase = (y0 + by) * fw + x0;
            let dbase = by * bw;
            for bx in 0..bw {
                cov_src[dbase + bx] = f32::from(self.paint.stroke_coverage[sbase + bx]) / 255.0;
            }
        }
        let blur = box_blur(&cov_src, bw, bh, spread);

        let lut = luts();
        let brush = &self.paint.brush;
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

        let base = &**base_arc;
        let color_buf = &self.paint.stroke_color;
        let out = Arc::make_mut(&mut self.canvas_rgba);
        for by in 0..bh {
            let gy = y0 + by;
            for bx in 0..bw {
                let gx = x0 + bx;
                let gi = (gy * fw + gx) * 4;
                // Warp the sample position (organic boundary). Local coords for the region-local fields;
                // global for the full-canvas colour buffer (same displacement, offset by the region origin).
                let (sx, sy) = if warp_amp > 0.0 {
                    let wx = warp_axis(gx as f32, gy as f32, SEED_WARP_X_A, SEED_WARP_X_B) * warp_amp;
                    let wy = warp_axis(gx as f32, gy as f32, SEED_WARP_Y_A, SEED_WARP_Y_B) * warp_amp;
                    (bx as f32 + wx, by as f32 + wy)
                } else {
                    (bx as f32, by as f32)
                };
                let cw = smoothstep(SS0, SS1, sample_bilinear(&cov_src, bw, bh, sx, sy));
                if cw <= 0.0 {
                    // Outside the wash: restore the frozen base (peels any previous frame's composite).
                    out[gi] = base[gi];
                    out[gi + 1] = base[gi + 1];
                    out[gi + 2] = base[gi + 2];
                    out[gi + 3] = base[gi + 3];
                    continue;
                }
                let inner = sample_bilinear(&blur, bw, bh, sx, sy).min(1.0);
                let edge = (cw * (1.0 - inner) * edge_gain).clamp(0.0, 1.0);
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
                // Granulation field: its own map, or (Same as Paper) the paper's tooth.
                let gran_h = if gran_own_map {
                    ph2d_painter_brush::texture::sample_tiled_rot(
                        &gran_tex,
                        gx as i64,
                        gy as i64,
                        gran_img.as_ref(),
                        gran_rot,
                    )
                } else {
                    paper_h
                };
                // Additive: the paper always textures the wash subtly (only when a Paper is set); the
                // granulation adds the pronounced mineral mottle by amount. `PAPER_TOOTH` is the fixed
                // substrate strength (an inherent property of the paper, not a slider).
                let paper_component = if paper_active {
                    (paper_h - 0.5) * PAPER_TOOTH
                } else {
                    0.0
                };
                let gran = (1.0 + paper_component + (gran_h - 0.5) * 2.0 * granulation).max(0.0);
                let density = ((cw * fill + edge) * gran).max(0.0);
                let od = density * depth;

                // Pigment colour: the deposited (source-over) colour where present, else the brush colour.
                let pig = if has_color {
                    let wgx = (gx as f32 + (sx - bx as f32)).clamp(0.0, (fw - 1) as f32);
                    let wgy = (gy as f32 + (sy - by as f32)).clamp(0.0, (fh - 1) as f32);
                    let ci = (wgy as usize * fw + wgx as usize) * 4;
                    if color_buf[ci + 3] > COL_EPS {
                        [color_buf[ci], color_buf[ci + 1], color_buf[ci + 2]]
                    } else {
                        fallback
                    }
                } else {
                    fallback
                };

                // Effective base in linear light: the layer's own pixels composited over the virtual paper
                // (so a transparent layer still has a paper to attenuate; an opaque base uses only itself).
                let ab = f32::from(base[gi + 3]) / 255.0;
                let sb = [
                    lut.s2l[base[gi] as usize] * ab + paper_lin[0] * (1.0 - ab),
                    lut.s2l[base[gi + 1] as usize] * ab + paper_lin[1] * (1.0 - ab),
                    lut.s2l[base[gi + 2] as usize] * ab + paper_lin[2] * (1.0 - ab),
                ];
                let mut rgb = [0u8; 3];
                let mut t_lum = 0.0f32;
                const LUM: [f32; 3] = [0.2126, 0.7152, 0.0722];
                for c in 0..3 {
                    let t = lut.transmittance(pig[c], od);
                    let lin = sb[c] * t + lut.s2l[pig[c] as usize] * (1.0 - t);
                    rgb[c] = lut.l2s_byte(lin);
                    t_lum += LUM[c] * t;
                }
                // Perceptual film opacity — how much pigment sits here; drives the deposited alpha and,
                // when Pigment is on, the subtractive (RYB) mix amount over the base paint.
                let film_a = (1.0 - t_lum).clamp(0.0, 1.0);
                if pigment_mix > 0.0 {
                    let mixed = ryb_mix(
                        [
                            f32::from(base[gi]) / 255.0,
                            f32::from(base[gi + 1]) / 255.0,
                            f32::from(base[gi + 2]) / 255.0,
                        ],
                        [
                            f32::from(pig[0]) / 255.0,
                            f32::from(pig[1]) / 255.0,
                            f32::from(pig[2]) / 255.0,
                        ],
                        film_a,
                    );
                    for c in 0..3 {
                        let sub = (mixed[c].clamp(0.0, 1.0) * 255.0 + 0.5).clamp(0.0, 255.0);
                        rgb[c] = (f32::from(rgb[c]) + (sub - f32::from(rgb[c])) * pigment_mix) as u8;
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
    }
}

/// Separable box blur of a scalar field, O(n) via per-row/column prefix sums (window count clamped at
/// the borders, so no darkening bias). Deterministic (only sums + one divide). `radius == 0` is a copy.
fn box_blur(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    // Horizontal pass.
    let mut tmp = vec![0.0f32; w * h];
    let mut pref = vec![0.0f32; w + 1];
    for y in 0..h {
        let base = y * w;
        for x in 0..w {
            pref[x + 1] = pref[x] + src[base + x];
        }
        for x in 0..w {
            let lo = x.saturating_sub(radius);
            let hi = (x + radius).min(w - 1);
            tmp[base + x] = (pref[hi + 1] - pref[lo]) / (hi - lo + 1) as f32;
        }
    }
    // Vertical pass.
    let mut out = vec![0.0f32; w * h];
    let mut prefc = vec![0.0f32; h + 1];
    for x in 0..w {
        for y in 0..h {
            prefc[y + 1] = prefc[y] + tmp[y * w + x];
        }
        for y in 0..h {
            let lo = y.saturating_sub(radius);
            let hi = (y + radius).min(h - 1);
            out[y * w + x] = (prefc[hi + 1] - prefc[lo]) / (hi - lo + 1) as f32;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Blur of a constant field is the same constant (window-count clamp handles the borders).
    #[test]
    fn box_blur_preserves_constant() {
        let src = vec![0.5f32; 8 * 6];
        let out = box_blur(&src, 8, 6, 2);
        for v in out {
            assert!((v - 0.5).abs() < 1e-6, "constant field must blur to itself");
        }
    }

    /// A single spike spreads mass to neighbours (peak drops, a neighbour rises) — proves it blurs.
    #[test]
    fn box_blur_spreads_a_spike() {
        let (w, h) = (9, 9);
        let mut src = vec![0.0f32; w * h];
        src[4 * w + 4] = 1.0;
        let out = box_blur(&src, w, h, 1);
        assert!(out[4 * w + 4] < 1.0, "peak dropped");
        assert!(out[4 * w + 5] > 0.0, "mass reached the neighbour");
    }

    /// The Beer–Lambert LUT path is the exact optical model: at full transmittance (`od = 0`) the pigment
    /// is invisible (output = base); as `od` grows the output moves monotonically toward the pigment. This
    /// pins the `s2l`/`exp`/`l2s` LUT composition against the closed-form `base·T + pigment·(1−T)`.
    #[test]
    fn beer_lambert_lut_matches_closed_form() {
        let lut = luts();
        let base: u8 = 200;
        let pig: u8 = 40;
        // od = 0 → T = 1 → output is the base byte exactly.
        assert_eq!(lut.transmittance(pig, 0.0), 1.0, "od=0 ⇒ full transmittance");
        let lin0 = lut.s2l[base as usize] * 1.0 + lut.s2l[pig as usize] * 0.0;
        assert_eq!(lut.l2s_byte(lin0), base, "od=0 composite = base");
        // Growing od darkens toward the pigment (monotone, bounded by the pigment byte).
        let mut prev = base as i32;
        for &od in &[0.5f32, 1.0, 2.0, 4.0, 8.0] {
            let t = lut.transmittance(pig, od);
            let lin = lut.s2l[base as usize] * t + lut.s2l[pig as usize] * (1.0 - t);
            let outv = lut.l2s_byte(lin) as i32;
            assert!(outv <= prev, "od={od}: composite moves toward the (darker) pigment");
            assert!(outv >= pig as i32 - 1, "never past the pigment");
            prev = outv;
        }
    }

    /// Value noise is deterministic, in `[0, 1]`, and varies across cells (not a constant field).
    #[test]
    fn value_noise_is_deterministic_and_bounded() {
        let a = value_noise(12.3, 45.6, 5.0, SEED_GRAIN);
        let b = value_noise(12.3, 45.6, 5.0, SEED_GRAIN);
        assert_eq!(a, b, "same input ⇒ same value (deterministic)");
        assert!((0.0..=1.0).contains(&a), "in range");
        let c = value_noise(112.3, 245.6, 5.0, SEED_GRAIN);
        assert!((a - c).abs() > 1e-4, "distant cells differ (it actually varies)");
    }
}
