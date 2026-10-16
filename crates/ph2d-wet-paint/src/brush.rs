//! Brush stamp (port of `brush.js`, SPEC §7): radial falloff x bristle texture.
//!
//! Porosity is load-bearing. The bristle texture is ~94% near-zero felt with
//! ~950 tiny bright tips, so a painted blob is a SIEVE of wet channels, not a
//! uniform film: the solver can nucleate discrete rivulets at its edge and the
//! reservoir keeps its pigment. The texture is anchored to the dab center in
//! canvas pixels (1 texel = 1 px at every brush size), so grain has constant
//! physical size and the tips streak with the stroke.
//!
//! Two tip populations, BOTH placed uniformly at random over the whole tile
//! (the SPEC §7 uniformity clause is binding: any deliberate region-based
//! placement makes the stroke profile a function of the brush radius):
//! FINE tips are the sub-gate sieve (visual felt between lanes); CORE tips
//! (a fraction of the count) are the gate-clearing bristle lanes.

use crate::jsmath::{js_round, wrap_mask};
use crate::rng::Rng;

mod unidade;
pub use unidade::{BRISTLE_UNIT_KNOBS, knob_from_unit, unit_from_knob};

pub const BRISTLE_SIZE: usize = 128;
const BMASK: i32 = BRISTLE_SIZE as i32 - 1;
const BRISTLE_SEED: u32 = 0x7a5b_0010;

// Tip-distribution calibration (SPEC §7, deposit-integral bullet). Tuned so
// the defaults hit: tile mean ~0.02, ~6-7% of texels > 0.04, origin deposit
// integral ~4.0 with 13-23 gate-clearing texels, 3x3-grid integral mean
// 4.0 +-25% with min >= 1.5, and the §18.11 budgets at both r=12 and r=25.
const FINE_RADIUS_MIN: f64 = 0.4;
const FINE_RADIUS_SPAN: f64 = 0.4;
const FINE_STRENGTH_MIN: f64 = 0.03;
const FINE_STRENGTH_SPAN: f64 = 0.05; // sub-gate grain: visual tooth, no deposit
const CORE_FRACTION: f64 = 0.26; // share of the bristle count drawn as core tips
const CORE_RADIUS_MIN: f64 = 0.9;
const CORE_RADIUS_SPAN: f64 = 0.6;
const CORE_STRENGTH_MIN: f64 = 0.16;
const CORE_STRENGTH_SPAN: f64 = 0.16;
const TIP_PLATEAU: f64 = 0.55; // fraction of the radius at full strength

/// The paint-group knobs the texture build reads.
#[derive(Clone, Copy)]
pub struct BristleKnobs {
    pub felt: f64,
    pub bristle_count: f64,
    pub bristle_strength: f64,
    pub bristle_size: f64,
}

impl Default for BristleKnobs {
    fn default() -> Self {
        BristleKnobs {
            felt: 0.01,
            bristle_count: 950.0,
            bristle_strength: 1.0,
            bristle_size: 1.0,
        }
    }
}

/// Build the 128x128 bristle tile. Deterministic: same knobs -> same texture.
/// - felt: near-zero noise floor between the hairs, mean = felt knob;
/// - tips: plateau-topped round dots (fine sieve + core blobs), max-blended.
pub fn build_bristle_texture(knobs: BristleKnobs) -> Vec<f32> {
    let felt = knobs.felt;
    let count = js_round(knobs.bristle_count).max(0.0) as i64;
    let tip_strength = knobs.bristle_strength;
    let tip_size = knobs.bristle_size;
    let mut rng = Rng::new(BRISTLE_SEED);
    let mut t = vec![0.0f32; BRISTLE_SIZE * BRISTLE_SIZE];
    for v in t.iter_mut() {
        *v = (rng.next() * 2.0 * felt) as f32; // mean = felt
    }
    let inv_fall = 1.0 / (1.0 - TIP_PLATEAU);
    let core_count = js_round(count as f64 * CORE_FRACTION) as i64;
    for b in 0..count {
        let rr = rng.next();
        let rs = rng.next();
        let is_core = b < core_count;
        let radius = if is_core {
            CORE_RADIUS_MIN + rr * CORE_RADIUS_SPAN
        } else {
            FINE_RADIUS_MIN + rr * FINE_RADIUS_SPAN
        } * tip_size;
        let strength = if is_core {
            CORE_STRENGTH_MIN + rs * CORE_STRENGTH_SPAN
        } else {
            FINE_STRENGTH_MIN + rs * FINE_STRENGTH_SPAN
        } * tip_strength;
        // Uniform random placement for EVERY tip — no reserved regions, no
        // re-draws (SPEC §7 uniformity clause).
        let cx = rng.next() * BRISTLE_SIZE as f64;
        let cy = rng.next() * BRISTLE_SIZE as f64;
        let reach = radius.ceil() as i32;
        let inv = 1.0 / (radius + 0.0001);
        let x0 = cx.floor() as i32 - reach;
        let x1 = cx.floor() as i32 + reach;
        let y0 = cy.floor() as i32 - reach;
        let y1 = cy.floor() as i32 + reach;
        for yy in y0..=y1 {
            for xx in x0..=x1 {
                let dx = xx as f64 - cx;
                let dy = yy as f64 - cy;
                let d = (dx * dx + dy * dy).sqrt() * inv;
                if d >= 1.0 {
                    continue;
                }
                // Plateau top with a smoothstep shoulder: round, not skirt-heavy.
                let mut w = 1.0;
                if d > TIP_PLATEAU {
                    let u = (d - TIP_PLATEAU) * inv_fall;
                    w = 1.0 - u * u * (3.0 - 2.0 * u);
                }
                let v = (strength * w) as f32;
                let idx = wrap_mask(yy, BMASK) * BRISTLE_SIZE + wrap_mask(xx, BMASK);
                if v > t[idx] {
                    t[idx] = v; // max-blend keeps tips crisp
                }
            }
        }
    }
    t
}

/// Nearest-texel wrap sample, offsets in canvas px from the dab center.
#[inline]
pub fn sample_bristle(tex: &[f32], dx_px: f64, dy_px: f64) -> f64 {
    tex[bristle_index(bristle_round(dx_px), bristle_round(dy_px))] as f64
}

/// The texel [`sample_bristle`] reads for the rounded offset `(x, y)` (wrapped).
#[inline]
#[must_use]
pub fn bristle_index(x: i32, y: i32) -> usize {
    wrap_mask(y, BMASK) * BRISTLE_SIZE + wrap_mask(x, BMASK)
}

/// The rounding [`sample_bristle`] applies to an offset (JS `Math.round`).
#[inline]
#[must_use]
pub fn bristle_round(v: f64) -> i32 {
    js_round(v) as i32
}

/// Radial falloff: a = (1-d)*hardness; 1 when a>=1, else smoothstep(a).
#[inline]
pub fn radial_falloff(d: f64, hardness: f64) -> f64 {
    let a = (1.0 - d) * hardness;
    if a >= 1.0 {
        return 1.0;
    }
    if a <= 0.0 {
        return 0.0;
    }
    a * a * (3.0 - 2.0 * a)
}

/// A silhueta ou o grão do HOSPEDEIRO, avaliados numa célula (coordenadas absolutas).
///
/// ⚠️ **`Fn + Sync` e não `FnMut`, de propósito** (ADR-0175): o depósito de um dab do produto caminha
/// as linhas em paralelo, logo a silhueta é lida por várias threads ao mesmo tempo. Ela SEMPRE foi
/// uma função pura da célula (a forma do pincel não muda enquanto o dab pousa) — o `FnMut` antigo
/// só não o dizia, e um hospedeiro que guardasse estado nela passaria a ser erro de compilação em
/// vez de uma corrida.
pub type CellFn<'a> = &'a (dyn Fn(i32, i32) -> f64 + Sync);

/// A COR do hospedeiro numa célula (a Shape Color do pincel, prompt 41): `(x, y, cobertura)` → sRGB
/// `0..255` e o alfa da tinta `0..1`, que pesa o depósito. `cobertura` = a silhueta × o padrão da
/// célula — o índice da Falloff Color, como no traço digital e na prévia do Shape. Pura e `Sync` pela
/// razão do [`CellFn`].
pub type ColorFn<'a> = &'a (dyn Fn(i32, i32, f64) -> [f32; 4] + Sync);

/// A caixa de células `[x0, x1] × [y0, y1]` de um dab do caminho com forma, recortada à área
/// pincelável `[2..W-1] × [2..H-1]`; `None` quando o dab cai inteiro fora dela.
///
/// Uma porta só para os dois caminhantes ([`for_each_stamp_pixel_shaped`] em série e o depósito
/// por linhas do [`crate::trail::Trail`]) — a caixa escrita duas vezes seria o sítio onde as duas
/// rotas deixariam de pousar a mesma tinta.
pub(crate) fn shaped_bounds(
    grid_w: usize,
    grid_h: usize,
    cx: f64,
    cy: f64,
    r: f64,
) -> Option<(i32, i32, i32, i32)> {
    let x0 = ((cx - r).ceil() as i32).max(2);
    let x1 = ((cx + r).floor() as i32).min(grid_w as i32 - 1);
    let y0 = ((cy - r).ceil() as i32).max(2);
    let y1 = ((cy + r).floor() as i32).min(grid_h as i32 - 1);
    (x0 <= x1 && y0 <= y1).then_some((x0, y0, x1, y1))
}

/// UMA linha `y` do caminho com forma, de `x0` a `x1`: chama `cb(x, falloff, textura, padrão)` em
/// cada célula que a silhueta, o padrão e a textura deixam pousar.
///
/// O terceiro canal, `padrao`, é o padrão procedural do Shape do hospedeiro (plano 27 W3, a lei B):
/// ele NÃO entra no `stamp` que o dente do papel corta — o depósito multiplica-o DEPOIS do portão
/// ([`crate::trail`]`::deposit`). `None` = `1` (o redondo, ao bit).
///
/// ⭐ **É a lei da célula, escrita UMA vez** — [`for_each_stamp_pixel_shaped`] é esta função num
/// laço de linhas, e o depósito paralelo é esta função em várias threads. Não existe «versão
/// paralela» da amostragem para divergir da serial (a garantia do [`crate::par`]).
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) fn stamp_row_shaped(
    y: i32,
    x0: i32,
    x1: i32,
    tex: &[f32],
    cx: f64,
    cy: f64,
    sil: CellFn<'_>,
    grain: Option<CellFn<'_>>,
    padrao: Option<CellFn<'_>>,
    mut cb: impl FnMut(i32, f64, f64, f64),
) {
    for x in x0..=x1 {
        let fall = sil(x, y);
        if fall > 0.0 {
            let sv = padrao.map_or(1.0, |p| p(x, y));
            if sv <= 0.0 {
                continue;
            }
            let texv = match grain {
                Some(gr) => gr(x, y),
                None => sample_bristle(tex, x as f64 - cx, y as f64 - cy),
            };
            if texv > 0.0 {
                cb(x, fall, texv, sv);
            }
        }
    }
}

/// The PRODUCT's shaped variant of [`for_each_stamp_pixel`]: the host owns
/// the dab's whole silhouette — falloff curve, Shape image, flatten/rotate
/// footprint — so `sil(x, y)` (absolute cell coords) REPLACES the engine's
/// radial falloff and disc test (`0` = outside).
/// The bristle `tex` is the engine's DEFAULT texture factor, sampled in RAW
/// offsets exactly as the round stamp samples it; when the host has a grain
/// system of its own, `grain(x, y)` REPLACES the bristle sample outright —
/// two textures multiplying would double-darken, and the artist's Grain is
/// THE texture everywhere else in the app. The engine's own paths never call
/// this — the port's behaviour is pinned by the session fingerprint.
///
/// Serial by construction: [`stamp_row_shaped`] walked over [`shaped_bounds`]. The paint deposit
/// walks the SAME row function in parallel (ADR-0175); the eraser keeps this serial walker.
#[allow(clippy::too_many_arguments)]
pub fn for_each_stamp_pixel_shaped(
    s: usize,
    grid_w: usize,
    grid_h: usize,
    tex: &[f32],
    cx: f64,
    cy: f64,
    r: f64,
    sil: CellFn<'_>,
    grain: Option<CellFn<'_>>,
    mut cb: impl FnMut(usize, i32, i32, f64, f64),
) {
    let Some((x0, y0, x1, y1)) = shaped_bounds(grid_w, grid_h, cx, cy, r) else {
        return;
    };
    for y in y0..=y1 {
        let row = y as usize * s;
        stamp_row_shaped(
            y,
            x0,
            x1,
            tex,
            cx,
            cy,
            sil,
            grain,
            None,
            |x, fall, texv, _| {
                cb(x as usize + row, x, y, fall, texv);
            },
        );
    }
}

/// Iterate every stamp pixel of one ROUND dab, restricted to the brushable area
/// [2..W-1] x [2..H-1]. Calls cb(i, x, y, falloff, texture) — the caller
/// multiplies them (plus its own intensity/clamps); they stay separate
/// because the wet-edge-softening extension keys on the falloff alone.
///
/// Takes the grid DIMENSIONS, not the grid: the callback is where deposits
/// mutate cell arrays, so the iterator must not hold a borrow of them.
///
/// The engine's only footprint is the disc: the JS port's Flat/Fan ellipses
/// left with plan 27 W5 (no product caller — the host's Shape owns the
/// footprint through [`for_each_stamp_pixel_shaped`]).
#[allow(clippy::too_many_arguments)]
pub fn for_each_stamp_pixel(
    s: usize,
    grid_w: usize,
    grid_h: usize,
    tex: &[f32],
    cx: f64,
    cy: f64,
    r: f64,
    hardness: f64,
    mut cb: impl FnMut(usize, i32, i32, f64, f64),
) {
    let x0 = ((cx - r).ceil() as i32).max(2);
    let x1 = ((cx + r).floor() as i32).min(grid_w as i32 - 1);
    let y0 = ((cy - r).ceil() as i32).max(2);
    let y1 = ((cy + r).floor() as i32).min(grid_h as i32 - 1);
    if x1 < x0 || y1 < y0 {
        return;
    }
    let inv_r = 1.0 / r;
    for y in y0..=y1 {
        let mut i = x0 as usize + y as usize * s;
        for x in x0..=x1 {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let nx = dx * inv_r;
            let ny = dy * inv_r;
            let d2 = nx * nx + ny * ny;
            if d2 < 1.0 {
                // reject on squared distance before any sqrt
                let fall = radial_falloff(d2.sqrt(), hardness);
                if fall > 0.0 {
                    let texv = sample_bristle(tex, dx, dy);
                    if texv > 0.0 {
                        cb(i, x, y, fall, texv);
                    }
                }
            }
            i += 1;
        }
    }
}
