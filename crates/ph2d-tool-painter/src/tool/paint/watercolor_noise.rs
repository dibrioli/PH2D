//! Watercolor **canvas-anchored noise** — the deterministic integer-hash value noise behind the
//! RaggedEdge warp, the built-in paper tooth and the backrun jag, plus the [`NoiseTile`] sprite-wrap
//! that makes every one of them SEAMLESS across the tile seam (doc 13 #2). Split from
//! `watercolor_field.rs` for the workspace LOC cap; re-exported there so callers keep using
//! `watercolor_field::{value_noise_tiled, warp_axis, NoiseTile, …}`. HR-5: transcendental-free.

use ph2d_painter_brush::TextureKind;
use ph2d_painter_brush::TextureMapping;
use ph2d_painter_brush::TextureSettings;
use ph2d_painter_brush::texture::{TEX_TILE_BASE_PX, analytic_tile_period, lattice_tileable};

// ── Deterministic value noise (integer hash; HR-5 transcendental-free) ───────────────────────────────
// Distinct seeds keep the two warp axes + the paper grain decorrelated (else the boundary would ripple
// along the diagonal and the granulation would track the warp).
pub(super) const SEED_WARP_X_A: u32 = 0x1111_1111;
pub(super) const SEED_WARP_X_B: u32 = 0x2222_2222;
pub(super) const SEED_WARP_Y_A: u32 = 0x3333_3333;
pub(super) const SEED_WARP_Y_B: u32 = 0x4444_4444;
pub(super) const SEED_GRAIN: u32 = 0x5555_5555;
pub(super) const SEED_GRAIN_FINE: u32 = 0x6666_6666;

/// Smoothstep-in-`[0,1]` fade (the value-noise interpolation weight; a cubic, so no transcendental).
#[inline]
pub(super) fn smooth01(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Integer hash of a lattice cell → `[0, 1)` (a `lowbias32`-style avalanche; deterministic + fast).
#[inline]
pub(super) fn hash2(ix: i32, iy: i32, seed: u32) -> f32 {
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

/// Sprite-tiling context for canvas-anchored noise (doc 13 #2): the axis PERIODS (the sprite w/h in
/// px) and which axes wrap. When an axis tiles, [`value_noise_tiled`] wraps its lattice at a WHOLE
/// number of cells spanning the period, so the field is periodic (`noise(x) == noise(x + period)`) and
/// every canvas-anchored texture (RaggedEdge warp, paper granulation, backrun jag) is seamless across
/// the sprite seam. [`NoiseTile::NONE`] (no axis) ⇒ the plain non-periodic noise, byte-identical.
#[derive(Clone, Copy)]
pub(super) struct NoiseTile {
    period: [f32; 2],
    on: [bool; 2],
}

impl NoiseTile {
    /// No axis tiles — the historical non-periodic noise.
    pub(super) const NONE: Self = Self {
        period: [0.0, 0.0],
        on: [false, false],
    };

    /// From the sprite size (px) + the per-axis Tiling flags.
    #[inline]
    pub(super) fn new(size: (usize, usize), on: [bool; 2]) -> Self {
        Self {
            period: [size.0 as f32, size.1 as f32],
            on,
        }
    }

    /// The per-axis sprite PERIOD (px) for a slot-texture snap ([`snap_slot_size`], doc 13 #2b); `0` =
    /// that axis isn't tiled (no snap).
    #[inline]
    pub(super) fn slot_period(self) -> [f32; 2] {
        [
            if self.on[0] { self.period[0] } else { 0.0 },
            if self.on[1] { self.period[1] } else { 0.0 },
        ]
    }

    /// Effective cell + wrap period (in WHOLE cells) for axis `i`: rounds the cell count so an integer
    /// number of cells spans the period exactly — the seam only glues on a whole cell count, and the
    /// effective cell is nudged to `period/cells` so pixel `period` lands on cell `0` again. A non-tiled
    /// axis returns `(cell, 0)` = the input cell verbatim, no wrap (⇒ byte-identical to the old noise).
    #[inline]
    fn axis(self, cell: f32, i: usize) -> (f32, i32) {
        if self.on[i] && self.period[i] > 0.0 {
            let cells = (self.period[i] / cell).round().max(1.0);
            (self.period[i] / cells, cells as i32)
        } else {
            (cell, 0)
        }
    }
}

/// Wrap a lattice index into `[0, p)` when `p > 0` (a tiled axis); `p == 0` (non-tiled) ⇒ verbatim.
#[inline]
fn wrap_cell(i: i32, p: i32) -> i32 {
    if p > 0 { i.rem_euclid(p) } else { i }
}

/// Bilinear value noise with `cell`-px features at `(x, y)` → `[0, 1]` (wet_edges `valueNoise`),
/// optionally PERIODIC per axis (`tile`) so a tiled sprite is seamless (doc 13 #2). `NoiseTile::NONE`
/// ⇒ the plain non-periodic noise (effective cell = `cell`, no wrap — the historical field).
#[inline]
pub(super) fn value_noise_tiled(x: f32, y: f32, cell: f32, seed: u32, tile: NoiseTile) -> f32 {
    let (cx, px) = tile.axis(cell, 0);
    let (cy, py) = tile.axis(cell, 1);
    let fx = x / cx;
    let fy = y / cy;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let sx = smooth01(fx - x0);
    let sy = smooth01(fy - y0);
    let a = hash2(wrap_cell(ix, px), wrap_cell(iy, py), seed);
    let b = hash2(wrap_cell(ix + 1, px), wrap_cell(iy, py), seed);
    let c = hash2(wrap_cell(ix, px), wrap_cell(iy + 1, py), seed);
    let d = hash2(wrap_cell(ix + 1, px), wrap_cell(iy + 1, py), seed);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Fractal displacement field (two octaves, cells 22/8 px) in `[-1, 1]` for a warp axis (wet_edges
/// warp). `tile` makes it seamless across the sprite seam (doc 13 #2); `NONE` ⇒ non-periodic (old).
///
/// ⚠️ **`#[cfg(test)]`: esta é a REFERÊNCIA CONGELADA, não o caminho vivo.** Ela é, verbatim, o que o
/// produto rodava antes de [`warp_offset`] passar a fatorar a grade entre os dois eixos — e é por isso
/// que ela é o oráculo certo de `the_two_axis_warp_factoring_is_byte_exact`: o gate compara a fatoração
/// contra o código que de fato shipava, não contra um espelho escrito para o teste.
///
/// Ela ficou sem chamador de produção no instante da fatoração, e **um `pub(super)` sem chamador não é
/// código morto silencioso: é uma segunda resposta esperando alguém chamá-la** — dois caminhos para
/// *"qual é o deslocamento do warp aqui?"*, livres para divergir. O `cfg(test)` diz o que ela é.
#[cfg(test)]
#[inline]
pub(super) fn warp_axis(x: f32, y: f32, sa: u32, sb: u32, tile: NoiseTile) -> f32 {
    (value_noise_tiled(x, y, 22.0, sa, tile) * 0.65 + value_noise_tiled(x, y, 8.0, sb, tile) * 0.35
        - 0.5)
        * 2.0
}

/// **A MESMA célula, DOIS seeds** — a oitava avaliada uma vez para os dois eixos do warp.
///
/// [`warp_offset`] pede a mesma oitava, na mesma posição, para o eixo X e para o eixo Y: só o `seed`
/// difere. Toda a aritmética de GRADE — as duas divisões, os dois `floor`, os dois `smooth01` e os dois
/// `wrap_cell` — portanto **não depende do seed** e estava sendo computada duas vezes.
///
/// ⚠️ **Byte-exato por construção, e a razão é a ordem:** cada eixo faz as MESMAS operações, na MESMA
/// sequência, sobre os MESMOS valores; o que muda é que o prefixo comum é avaliado uma vez em vez de
/// duas. `fx = x / cx` calculado uma vez e usado duas dá os mesmos bits de calculá-lo duas vezes — não é
/// aproximação, é fatoração. (O que **não** se pode fatorar são os oito `hash2`: eles dependem do seed,
/// e é neles que a decorrelação dos dois eixos mora.)
///
/// Medido em `measure_what_a_watercolor_move_is_made_of`: o warp custa **1,071 ms de um move de 3,082**
/// a 4096² — 56% do que a aquarela cobra sobre o Digital — porque `warp_offset` roda **10× por texel**
/// (o centro + os 9 taps do supersample 3×3, cada um roteado pelo warp de propósito: foi isso que curou
/// a borda serrilhada, e por isso cortar taps está FORA de discussão).
#[inline]
fn value_noise_pair(
    x: f32,
    y: f32,
    cell: f32,
    seed_a: u32,
    seed_b: u32,
    tile: NoiseTile,
) -> (f32, f32) {
    let (cx, px) = tile.axis(cell, 0);
    let (cy, py) = tile.axis(cell, 1);
    let fx = x / cx;
    let fy = y / cy;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let sx = smooth01(fx - x0);
    let sy = smooth01(fy - y0);
    let (wx0, wx1) = (wrap_cell(ix, px), wrap_cell(ix + 1, px));
    let (wy0, wy1) = (wrap_cell(iy, py), wrap_cell(iy + 1, py));
    let lerp4 = |seed: u32| {
        let a = hash2(wx0, wy0, seed);
        let b = hash2(wx1, wy0, seed);
        let c = hash2(wx0, wy1, seed);
        let d = hash2(wx1, wy1, seed);
        let top = a + (b - a) * sx;
        let bot = c + (d - c) * sx;
        top + (bot - top) * sy
    };
    (lerp4(seed_a), lerp4(seed_b))
}

/// Both warp axes at `(x, y)` as a raw displacement `(dx, dy) ∈ [-1, 1]²` (pre-amplitude) — the two
/// decorrelated `warp_axis` reads the composite adds to the sample position; seamless when `tile` wraps.
#[inline]
pub(super) fn warp_offset(x: f32, y: f32, tile: NoiseTile) -> (f32, f32) {
    // Uma oitava, DOIS eixos ([`value_noise_pair`]) — a grade é a mesma e só o seed difere. A conta por
    // eixo é literalmente a de [`warp_axis`], na mesma ordem, então isto é byte-exato e não uma
    // aproximação; `warp_axis` fica como a porta de quem pede UM eixo só.
    let (ax, ay) = value_noise_pair(x, y, 22.0, SEED_WARP_X_A, SEED_WARP_Y_A, tile);
    let (bx, by) = value_noise_pair(x, y, 8.0, SEED_WARP_X_B, SEED_WARP_Y_B, tile);
    (
        (ax * 0.65 + bx * 0.35 - 0.5) * 2.0,
        (ay * 0.65 + by * 0.35 - 0.5) * 2.0,
    )
}

/// Paper-tooth granulation height at `(x, y)` in `[0, 1]` — the built-in fallback when no Paper
/// slot is set. TWO octaves (5 px + 2.5 px, doc 12 GRAN-3): the audit measured the old single
/// octave as mid-band and uniform (good) but mono-scale full-range — the "digital" side of the
/// default look; the fine second octave breaks the single-frequency signature.
#[inline]
pub(super) fn paper_height(x: f32, y: f32, tile: NoiseTile) -> f32 {
    0.65 * value_noise_tiled(x, y, 5.0, SEED_GRAIN, tile)
        + 0.35 * value_noise_tiled(x, y, 2.5, SEED_GRAIN_FINE, tile)
}

/// #2b/#2c/#2-fase2 (doc 13): snap a slot texture's **Size** so a WHOLE number of tiles/cells/periods
/// spans the sprite on each tiled axis, making the slot repeat seamlessly across the sprite seam
/// (matching the procedural noise). Covers the two BITMAP kinds — an **Image** (`fract`-wrapped UV,
/// repeating every 2 units of `rel` because `sample_image` maps `cu = rel·0.5 + 0.5`) and the baked 256²
/// **Paper** tiles (`PaperCold`/`Rough`/`Hot`, every 1 unit of `rel`) — the LATTICE procedurals (#2c:
/// Noise / Clouds / Voronoi / … via [`lattice_tileable`]), whose hash-wrap ([`sample_tiled_rot_wrapped`])
/// is seam-free only when the `rel`-span is a whole number of cells (period `1`) — AND the ANALYTIC
/// patterns (Fase 2: Checker / Stripes / Grid / Bricks / Waves / … via [`analytic_tile_period`]), which
/// are already exactly periodic, so aligning the span to a whole number of their per-axis period tiles
/// them with no sampler change. Non-tiled axis / tiling off / non-tileable kind (Magic / Marble / Wood /
/// Triangles / Hexagons) ⇒ unchanged (byte-identical). The Size quantises to the nearest whole count
/// (inherent to a fixed period — it can only repeat a WHOLE number of times), imperceptible at fine
/// scales. Rotation ≠ 0 still seams (a rotated grid can't align with the axis-aligned seam) — documented.
pub(super) fn snap_slot_size(mut s: TextureSettings, tile: NoiseTile) -> TextureSettings {
    // The period in `rel` units per axis `[u, v]`: a bitmap repeats on its baked period (Image every 2,
    // baked Paper every 1); a LATTICE procedural (doc 13 #2c) snaps to a whole cell count (period 1) so
    // its hash-wrap is seam-free; an ANALYTIC pattern (Fase 2) snaps to its own per-axis period (a `0.0`
    // axis is constant/ignored ⇒ left unchanged). A non-tileable kind returns unchanged.
    let rep: [f32; 2] = match s.kind {
        TextureKind::Image => [2.0, 2.0],
        TextureKind::PaperCold | TextureKind::PaperRough | TextureKind::PaperHot => [1.0, 1.0],
        k if lattice_tileable(k) => [1.0, 1.0],
        k => match analytic_tile_period(k, s.params) {
            Some(p) => p,
            None => return s,
        },
    };
    let snap = |size: f32, period_px: f32, rep: f32| {
        if period_px > 0.0 && rep > 0.0 {
            // Round the texture's span across the sprite to a whole number of periods `rep` so the seam
            // lands on a period boundary (a fixed period can only repeat a WHOLE number of times).
            let reps = (period_px * size / (TEX_TILE_BASE_PX * rep))
                .round()
                .max(1.0)
                * rep;
            reps * TEX_TILE_BASE_PX / period_px
        } else {
            size // untiled axis, or an ignored/constant analytic axis ⇒ leave it
        }
    };
    let p = tile.slot_period();
    s.size = [snap(s.size[0], p[0], rep[0]), snap(s.size[1], p[1], rep[1])];
    s
}

/// Convert a **ViewPlane** (dab-relative) Grain slot to the canvas-anchored Size that reproduces the SAME
/// feature scale the brush shows — so the watercolor granulation MATCHES the brush's grain (Enio
/// 2026-07-11). The wash samples the Grain slot canvas-anchored ([`sample_tiled_rot`]: `rel =
/// px·size/TILE_BASE`), while the brush's default ViewPlane maps `size` per DAB RADIUS (`rel =
/// (px−c)/radius·size`). So the IDENTICAL `size` renders ~`TILE_BASE/radius`× COARSER in the wash — a 40-px
/// brush turns a fine Voronoi (cells ~40 px) into giant 256-px blobs ("muito pior"). This is
/// kind-independent (a mapping/scale gap), but shows worst on cellular kinds (Voronoi / Grid / Checker).
/// Rescaling `size` by `TILE_BASE/radius` makes the canvas sample reproduce the ViewPlane scale EXACTLY
/// (`px·(size·TILE_BASE/radius)/TILE_BASE = px·size/radius`). A **Tiled** / **Stencil** grain is ALREADY
/// canvas-anchored like the wash (matches the brush) ⇒ unchanged; inactive ⇒ unchanged.
pub(super) fn grain_view_to_canvas_size(mut s: TextureSettings, radius_px: f32) -> TextureSettings {
    if s.is_active() && matches!(s.mapping, TextureMapping::ViewPlane) {
        let k = TEX_TILE_BASE_PX / radius_px.max(1.0);
        s.size = [s.size[0] * k, s.size[1] * k];
    }
    s
}

#[cfg(test)]
#[path = "watercolor_noise_tests.rs"]
mod tests;
