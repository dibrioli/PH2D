//! Direct tools (port of `tools.js`, SPEC §11): erase, wet, dry, blow, smear.
//! Each dispatches per dab (no trail), with the fixed tool hardness 4.8 and
//! stamp = min(1, falloff * texture * min(3, intensity)). All respect the
//! brushable area (the stamp iterator enforces it) and expand the active
//! bbox.

use crate::brush::{for_each_stamp_pixel, for_each_stamp_pixel_shaped};
use crate::drying::{lift_settled, staining_multiplier};
use crate::grid::{Grid, wet_byte_from_paper};
use crate::jsmath::{clamp01, js_round};
use crate::opacity::alpha_of_mass;
use crate::sim::Params;
use crate::trail::{Dab, TouchedRect};
use crate::tuning::Knob;

/// Doc 23: the shared active-lift gain — `wetLift · stainingMultiplier`.
/// Every active rewetter (Wet dissolve, wet Smear, wet Blend) derives its
/// per-pixel `b` from this ONE product, so "how re-soluble is dried paint"
/// has a single answer across tools. `wetLift = 0` or `staining = 1` turn
/// every active lift off exactly.
#[inline]
pub(crate) fn active_lift_gain(p: &Params) -> f64 {
    p.k(Knob::WetLift) * staining_multiplier(p.k(Knob::ExtStaining))
}

pub const TOOL_HARDNESS: f64 = 4.8;

fn rect_around(g: &Grid, x: f64, y: f64, r: f64) -> TouchedRect {
    TouchedRect {
        x0: ((x - r).floor() as i32 - 1).max(2),
        y0: ((y - r).floor() as i32 - 1).max(2),
        x1: ((x + r).ceil() as i32 + 1).min(g.w as i32 - 1),
        y1: ((y + r).ceil() as i32 + 1).min(g.h as i32 - 1),
    }
}

/// O [`rect_around`] de quem o usa ANTES de pintar (a fotografia da Blow e da Smear): `None` quando o
/// carimbo cai inteiro fora do interior. Os dois cantos prendem-se por lados opostos, e aí o
/// rectângulo sai INVERTIDO — a largura negativa feita `usize` era um pânico em debug e um pedido de
/// memória astronómico em release (o rato fora da tela; gate `a_dab_outside_the_grid_touches_nothing_…`).
fn rect_around_nao_vazio(g: &Grid, x: f64, y: f64, r: f64) -> Option<TouchedRect> {
    let rr = rect_around(g, x, y, r);
    (rr.x1 >= rr.x0 && rr.y1 >= rr.y0).then_some(rr)
}

fn expand_to(g: &mut Grid, r: &TouchedRect) {
    g.expand_bbox(r.x0, r.y0, r.x1, r.y1);
}

/// erase: paper-gated multiplicative removal of both layers + water.
pub fn apply_erase(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    erase_slider: f64,
) -> Option<TouchedRect> {
    apply_erase_impl(g, p, tex, dab, erase_slider, None, None)
}

/// [`apply_erase`] with the HOST's silhouette + optional grain (the shaped
/// product door, twin of `Trail::accumulate_paint_shaped`): `sil` replaces
/// the internal falloff + footprint, `grain` replaces the bristle. ONE pixel
/// body serves both faces — `apply_erase` delegates with `None`s.
pub fn apply_erase_shaped(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    erase_slider: f64,
    sil: crate::brush::CellFn<'_>,
    grain: Option<crate::brush::CellFn<'_>>,
) -> Option<TouchedRect> {
    apply_erase_impl(g, p, tex, dab, erase_slider, Some(sil), grain)
}

fn apply_erase_impl(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    erase_slider: f64,
    sil: Option<crate::brush::CellFn<'_>>,
    grain: Option<crate::brush::CellFn<'_>>,
) -> Option<TouchedRect> {
    let gate = p.k(Knob::PaperGate);
    let pig_cap = p.k(Knob::GateSaturation);
    let force = p.k(Knob::Eraser) * erase_slider;
    // Doc 23 P4 — dried, staining paint resists the eraser: the settled
    // layer takes a weaker force (`force · (1 − 0.5·staining)`; staining 0
    // erases like the wet layers, staining 1 at half force). Suspended
    // pigment and water keep the full force — they are not bound to the
    // fibre.
    let force_sett = force * (1.0 - 0.5 * p.k(Knob::ExtStaining));
    let clamp_i = dab.intensity.min(3.0);
    let mut touched = false;
    let Grid {
        s,
        w,
        h,
        susp,
        sett,
        film,
        paper,
        ..
    } = g;
    let _ = s;
    let body = |i: usize, _x: i32, _y: i32, fall: f64, texv: f64| {
        let mut stamp = fall * texv * clamp_i;
        if stamp > 1.0 {
            stamp = 1.0;
        }
        let tooth = if (susp[i] as f64 + sett[i] as f64) < pig_cap {
            paper[i] as f64
        } else {
            0.45
        };
        let mut gg = stamp - (1.0 - tooth) * gate;
        if gg <= 0.0 {
            return;
        }
        if gg > 1.0 {
            gg = 1.0;
        }
        let k = 1.0 - force * gg;
        if k <= 0.0 {
            return; // never wipe a cell outright
        }
        susp[i] = (susp[i] as f64 * k) as f32;
        film[i] = (film[i] as f64 * k) as f32;
        if (susp[i] as f64) < 1.0 {
            susp[i] = 0.0; // stale color bytes may remain; mass 0 hides them
        }
        let k_sett = 1.0 - force_sett * gg;
        sett[i] = (sett[i] as f64 * k_sett) as f32;
        if (sett[i] as f64) < 1.0 {
            sett[i] = 0.0;
        }
        touched = true;
    };
    match sil {
        Some(sil) => {
            let r = dab.r * dab.reach;
            for_each_stamp_pixel_shaped(*s, *w, *h, tex, dab.x, dab.y, r, sil, grain, body);
        }
        None => {
            debug_assert!(grain.is_none(), "grain override requires the shaped path");
            for_each_stamp_pixel(*s, *w, *h, tex, dab.x, dab.y, dab.r, TOOL_HARDNESS, body);
        }
    }
    if touched {
        let r = rect_around(g, dab.x, dab.y, dab.r * dab.reach);
        expand_to(g, &r);
        Some(r)
    } else {
        None
    }
}

/// wet: drip-feed water; gentle, accumulates over repeated passes — and
/// (doc 23 P1) an ACTIVE lift: the stamp dissolves settled pigment back
/// into suspension on the spot. Watercolor is re-soluble (the gum-arabic
/// binder re-dissolves; "lifting" is a standard technique) and Rebelle's
/// Re-wet is stroke-driven — the passive re-wet in the drying pass is
/// ~1%/s ambience and cannot serve a gesture. `wetLift = 0` restores the
/// water-only model to the byte.
pub fn apply_wet(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    water_unit: f64,
) -> Option<TouchedRect> {
    let add = 2.0 * (water_unit * 0.2) * (water_unit * 0.2) * p.k(Knob::WaterPerDab);
    let cap = p.k(Knob::WaterCap);
    let lift_gain = active_lift_gain(p);
    let mix = p.mix;
    let mut out = [0.0f64; 3];
    let clamp_i = dab.intensity.min(3.0);
    let mut touched = false;
    let Grid {
        s,
        w,
        h,
        film,
        paper,
        wet,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        vela,
        ..
    } = g;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        |i, _x, _y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            if stamp <= 0.0 {
                return;
            }
            let mut f = film[i] as f64 + stamp * add;
            if f > cap {
                f = cap;
            }
            film[i] = f as f32;
            wet[i] = wet_byte_from_paper(paper[i] as f64);
            // The active lift, through the SAME door as the passive re-wet
            // (drying.rs) — the freshly suspended pigment then bleeds,
            // follows gravity and answers the Blow/Smear tools.
            if lift_gain > 0.0 && sett[i] > 0.0 {
                let b = clamp01(stamp * lift_gain);
                if b > 0.0 {
                    lift_settled(
                        b,
                        &mut susp[i],
                        &mut sett[i],
                        &mut susp_rgb[i],
                        sett_rgb[i],
                        mix,
                        p.glaze,
                        p.ecra.com_vela(vela[i]),
                        &mut out,
                    );
                }
            }
            touched = true;
        },
    );
    if touched {
        let r = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &r);
        Some(r)
    } else {
        None
    }
}

/// O alcance da frente da água de [`apply_wet_push`], em raios do dab: o Water Soft do Rebelle (38 px de largura) deixa
/// a orla a ~120–130 px do eixo do traço sobre a faixa molhada (`docs/Painter/Pinceis/fixtures/rebelle_mistura/WATER`:
/// as linhas a x 380 · 630 com os cruzamentos a 250 · 500 · 750) — 125 ÷ 19.
const WATER_FRONT_RADII: f64 = 6.6;
/// O filme da célula que a água ainda atravessa: a frente pára na última.
const WATER_WET_FILM: f32 = 0.01;
/// A água que transborda da pegada pela tinta molhada: a parte empurrada cai como `e^{−(|s|−r)/(2 r)}` fora dela —
/// o Water Soft do Rebelle deixa 0,13 no eixo, 0,50 a ±40 px e 0,84 a ±60 (a `1 r` o nosso dava 0,5 a ±30).
const WATER_SPILL_RADII: f64 = 2.0;
/// O pigmento empurrado de cada célula reparte-se por quatro pontos sorteados pela posição dela: 15 % numa rampa de
/// [`ORLA_RAMPA_RADII`] raios antes da frente (a linha onde a água parou) e 85 % pelo caminho da água, de `2 r` à
/// frente — com tudo na frente a linha lia 2,4 contra 1,54 do Rebelle (a faixa nossa é mais clara: a tinta dobrada
/// pesa mais no verde); assim 2,0 e o caminho 1,0–1,3 como o dele.
const ORLA_PESOS: [f64; 4] = [0.075, 0.075, 0.425, 0.425];
const ORLA_RAMPA_RADII: f64 = 1.0;
/// Sobre a tinta SECA (nenhuma frente de lado) a água arrasta pelo traço `0,5 · push` do que dissolveu por dab: o
/// escorrido de ~20–30 px abaixo da faixa seca do Rebelle (a `1 ·` arrastava tudo até ao fim do traço).
const WATER_DRAG: f64 = 0.5;

/// ⭐ **A água EMPURRA o pigmento** ([`Knob::WaterPush`], J4 mistura L-6): o dab da ferramenta Wet leva a fracção
/// `push · w` do pigmento SUSPENSO de cada célula molhada da sua faixa para a FRENTE da água desse lado — a última
/// célula molhada a `≤ 6,6 r` do eixo, linha a linha da faixa, pela normal do traço (a água corre para os lados, não
/// ao longo do próprio traço, onde o nível é o mesmo). `w = 1` na pegada e `e^{−(|s|−r)/2r}` fora dela (a água que
/// transborda); a parte de cada dab é a de UM diâmetro de caminho repartida pelo passo — o traço rápido abre o mesmo
/// que o lento. Sobre a tinta SECA (a água não passa da pegada de nenhum lado) arrasta pelo traço o que o `WetLift`
/// dissolveu ([`WATER_DRAG`]). O 1.º dab (sem `prev`) não empurra.
pub fn apply_wet_push(g: &mut Grid, p: &Params, dab: &Dab, prev: [f64; 2]) -> Option<TouchedRect> {
    let push = p.k(Knob::WaterPush);
    let (dx, dy) = (dab.x - prev[0], dab.y - prev[1]);
    let passo = dx.hypot(dy);
    if push <= 0.0 || passo < 1e-6 || dab.r <= 0.0 {
        return None;
    }
    let (tx, ty) = (dx / passo, dy / passo);
    let (nx, ny) = (-ty, tx);
    let r = dab.r;
    let alcance = r * WATER_FRONT_RADII;
    let expoente = (passo / (2.0 * r)).min(1.0);
    let molhada = |g: &Grid, x: f64, y: f64| -> Option<usize> {
        let (cx, cy) = (x.round() as i64, y.round() as i64);
        if cx < 1 || cy < 1 || cx > g.w as i64 || cy > g.h as i64 {
            return None;
        }
        let i = cx as usize + cy as usize * g.s;
        (g.film[i] > WATER_WET_FILM).then_some(i)
    };
    // A frente de cada LINHA da faixa (o passo `t` ao longo do traço), de cada lado: a última célula molhada pela
    // normal. Só há para onde correr quando a água passa da pegada (sobre a tinta seca a frente é a própria pegada).
    let meia = r.ceil() as i64;
    let frentes: Vec<[f64; 2]> = (-meia..=meia)
        .map(|k| {
            let (ax, ay) = (dab.x + k as f64 * tx, dab.y + k as f64 * ty);
            [-1.0, 1.0].map(|sinal| {
                let mut d = 0.0;
                let mut q = 1.0;
                while q <= alcance && molhada(g, ax + sinal * q * nx, ay + sinal * q * ny).is_some()
                {
                    d = q;
                    q += 1.0;
                }
                if d > r + 1.0 { d } else { 0.0 }
            })
        })
        .collect();
    let rr = rect_around_nao_vazio(g, dab.x, dab.y, alcance + 1.0)?;
    let mut levar: Vec<(usize, f64, [f32; 3])> = Vec::new();
    for cy in rr.y0..=rr.y1 {
        for cx in rr.x0..=rr.x1 {
            let (ox, oy) = (f64::from(cx) - dab.x, f64::from(cy) - dab.y);
            let t = ox * tx + oy * ty;
            if t.abs() > r {
                continue;
            }
            let s = ox * nx + oy * ny;
            let lado = usize::from(s >= 0.0);
            let fila = frentes[(t.round() as i64 + meia).clamp(0, 2 * meia) as usize];
            let f = fila[lado];
            let i = cx as usize + cy as usize * g.s;
            if fila == [0.0, 0.0] && s.abs() <= r {
                // Sobre a tinta SECA a água não tem para onde correr de lado: arrasta o que dissolveu ao longo do
                // próprio traço — `push` do caminho de cada dab, ~`push · 2r` ao todo (o escorrido de ~30 px abaixo da
                // faixa seca do Rebelle).
                if g.film[i] <= WATER_WET_FILM || g.susp[i] <= 0.0 {
                    continue;
                }
                let m = f64::from(g.susp[i]) * (push * WATER_DRAG).min(1.0);
                let (fx, fy) = (f64::from(cx) + passo * tx, f64::from(cy) + passo * ty);
                let (x0, y0) = (fx.floor(), fy.floor());
                let (ax, ay) = (fx - x0, fy - y0);
                let mut levou = 0.0;
                for (qx, qy, peso) in [
                    (x0, y0, (1.0 - ax) * (1.0 - ay)),
                    (x0 + 1.0, y0, ax * (1.0 - ay)),
                    (x0, y0 + 1.0, (1.0 - ax) * ay),
                    (x0 + 1.0, y0 + 1.0, ax * ay),
                ] {
                    if let Some(j) = molhada(g, qx, qy).filter(|&j| j != i && peso > 0.0) {
                        levar.push((j, m * peso, g.susp_rgb[i]));
                        levou += m * peso;
                    }
                }
                g.susp[i] = (f64::from(g.susp[i]) - levou) as f32;
                continue;
            }
            if s.abs() >= f {
                continue;
            }
            if g.film[i] <= WATER_WET_FILM || g.susp[i] <= 0.0 {
                continue;
            }
            let w = if s.abs() <= r {
                1.0
            } else {
                (-(s.abs() - r) / (WATER_SPILL_RADII * r)).exp()
            };
            let parte = 1.0 - (1.0 - (push * w).min(1.0)).powf(expoente);
            let sinal = if lado == 1 { 1.0 } else { -1.0 };
            let m = f64::from(g.susp[i]) * parte;
            let mut levou = 0.0;
            // A orla e o caminho ([`ORLA_PESOS`]): cada célula nos seus pontos, sorteados pela posição dela (sem riscas).
            let sorteio = |k: u64| {
                let mut z = (i as u64 ^ k.wrapping_mul(0xD1B5_4A32_D192_ED03))
                    .wrapping_add(0x9E37_79B9_7F4A_7C15);
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
            };
            for (k, peso) in ORLA_PESOS.iter().enumerate() {
                let d0 = if k < 2 {
                    let u = sorteio(2 * k as u64).max(sorteio(2 * k as u64 + 1));
                    f - ORLA_RAMPA_RADII * r * (1.0 - u)
                } else {
                    let a0 = s.abs().max(2.0 * r).min(f);
                    a0 + (f - a0) * sorteio(2 * k as u64)
                };
                let mut d = d0.max(s.abs() + 1.0);
                let destino = loop {
                    if let Some(j) = molhada(
                        g,
                        dab.x + t * tx + sinal * d * nx,
                        dab.y + t * ty + sinal * d * ny,
                    ) {
                        break Some(j);
                    }
                    d -= 1.0;
                    if d <= s.abs() {
                        break None;
                    }
                };
                if let Some(j) = destino.filter(|&j| j != i) {
                    levar.push((j, m * peso, g.susp_rgb[i]));
                    levou += m * peso;
                }
            }
            g.susp[i] = (f64::from(g.susp[i]) - levou) as f32;
        }
    }
    if levar.is_empty() {
        return None;
    }
    let mix = p.mix;
    let mut out = [0.0f64; 3];
    for (j, m, c) in levar {
        let ja = f64::from(g.susp[j]);
        if ja > 0.0 {
            let u = g.susp_rgb[j];
            let [cr, cg, cb] = c.map(f64::from);
            mix.mix(
                f64::from(u[0]),
                f64::from(u[1]),
                f64::from(u[2]),
                cr,
                cg,
                cb,
                m / (m + ja),
                &mut out,
            );
            g.susp_rgb[j] = out.map(|v| v as f32);
        } else {
            g.susp_rgb[j] = c;
        }
        g.susp[j] = (ja + m) as f32;
    }
    expand_to(g, &rr);
    Some(rr)
}

/// O lado da caixa com que o secador NIVELA a tinta suspensa da pegada antes de a assentar, em raios.
const DRYER_LEVEL_RADII: f64 = 0.2;
/// A largura da faixa de tinta molhada, de cada lado da pegada, que o secador puxa para a borda dele, em raios.
const DRYER_RING_RADII: f64 = 0.15;

/// ⭐ **O secador ASSENTA a tinta e deixa ORLA** ([`Knob::DryerEdge`], J4 mistura L-7 Dry/Lift): antes de selar, o dab
/// do secador (1) NIVELA o pigmento suspenso da pegada (uma caixa de `0,2 r`: a tinta que seca depressa não guarda as
/// riscas que a molhada ia alisar) e (2) puxa para DENTRO da borda dela o pigmento da faixa molhada de `0,15 r` de cada
/// lado (a água corre para onde evapora: a mancha seca do Rebelle tem a linha escura por dentro, 1,37–1,39 em 3 px, e
/// por fora um claro de 0,84). As duas em `e` por diâmetro de caminho, pelos lados do traço (o `prev` dá a normal).
/// `0` = o secador de sempre, ao bit. O 1.º dab (sem `prev`) não faz nada.
pub fn apply_dry_edge(g: &mut Grid, p: &Params, dab: &Dab, prev: [f64; 2]) -> Option<TouchedRect> {
    let e = p.k(Knob::DryerEdge);
    let (dx, dy) = (dab.x - prev[0], dab.y - prev[1]);
    let passo = dx.hypot(dy);
    if e <= 0.0 || passo < 1e-6 || dab.r <= 0.0 {
        return None;
    }
    let (tx, ty) = (dx / passo, dy / passo);
    let (nx, ny) = (-ty, tx);
    let r = dab.r;
    let parte = 1.0 - (1.0 - e.min(1.0)).powf((passo / (2.0 * r)).min(1.0));
    let delta = (DRYER_RING_RADII * r).max(2.0);
    let k = (DRYER_LEVEL_RADII * r).round().max(1.0) as i32;
    let rr = rect_around_nao_vazio(g, dab.x, dab.y, r + delta + f64::from(k) + 1.0)?;
    let molhada = |g: &Grid, i: usize| g.film[i] > WATER_WET_FILM;
    // (1) O nível: a média da caixa (a cor pesada pela massa), lida de uma fotografia.
    let mut novo: Vec<(usize, f32, [f32; 3])> = Vec::new();
    for cy in rr.y0..=rr.y1 {
        for cx in rr.x0..=rr.x1 {
            let (ox, oy) = (f64::from(cx) - dab.x, f64::from(cy) - dab.y);
            if ox.hypot(oy) > r {
                continue;
            }
            let i = cx as usize + cy as usize * g.s;
            let (mut m, mut c, mut n, mut com) = (0.0f64, [0.0f64; 3], 0.0f64, 0.0f64);
            for qy in (cy - k).max(1)..=(cy + k).min(g.h as i32) {
                for qx in (cx - k).max(1)..=(cx + k).min(g.w as i32) {
                    let j = qx as usize + qy as usize * g.s;
                    let mj = f64::from(g.susp[j]);
                    m += mj;
                    for (a, b) in c.iter_mut().zip(g.susp_rgb[j]) {
                        *a += f64::from(b) * mj;
                    }
                    n += 1.0;
                    com += f64::from(u8::from(mj > 0.0));
                }
            }
            // Só DENTRO da tinta (a maioria da caixa com pigmento): os sulcos entre as cerdas enchem-se, a borda da
            // faixa não se espalha pelo papel.
            if m <= 0.0 || 2.0 * com < n {
                continue;
            }
            let media = m / n;
            let cor = c.map(|v| (v / m) as f32);
            let s0 = f64::from(g.susp[i]);
            novo.push((i, (s0 + (media - s0) * parte) as f32, cor));
        }
    }
    let mix = p.mix;
    let mut out = [0.0f64; 3];
    for (i, m, cor) in novo {
        let s0 = g.susp[i];
        if s0 > 0.0 {
            let u = g.susp_rgb[i];
            mix.mix(
                f64::from(u[0]),
                f64::from(u[1]),
                f64::from(u[2]),
                f64::from(cor[0]),
                f64::from(cor[1]),
                f64::from(cor[2]),
                parte,
                &mut out,
            );
            g.susp_rgb[i] = out.map(|v| v as f32);
        } else {
            g.susp_rgb[i] = cor;
        }
        g.susp[i] = m;
    }
    // (2) A orla: a faixa molhada de cada lado (r < |s| ≤ r + δ, |t| ≤ r) para dentro da borda, em espelho.
    let mut levar: Vec<(usize, f64, [f32; 3])> = Vec::new();
    for cy in rr.y0..=rr.y1 {
        for cx in rr.x0..=rr.x1 {
            let (ox, oy) = (f64::from(cx) - dab.x, f64::from(cy) - dab.y);
            let t = ox * tx + oy * ty;
            let s = ox * nx + oy * ny;
            if t.abs() > r || s.abs() <= r || s.abs() > r + delta {
                continue;
            }
            let i = cx as usize + cy as usize * g.s;
            if !molhada(g, i) || g.susp[i] <= 0.0 {
                continue;
            }
            let d = 2.0 * r - s.abs();
            let sinal = s.signum();
            let (fx, fy) = (
                dab.x + t * tx + sinal * d * nx,
                dab.y + t * ty + sinal * d * ny,
            );
            let (qx, qy) = (fx.round() as i64, fy.round() as i64);
            if qx < 1 || qy < 1 || qx > g.w as i64 || qy > g.h as i64 {
                continue;
            }
            let j = qx as usize + qy as usize * g.s;
            let m = f64::from(g.susp[i]) * parte;
            g.susp[i] = (f64::from(g.susp[i]) - m) as f32;
            levar.push((j, m, g.susp_rgb[i]));
        }
    }
    for (j, m, c) in levar {
        let ja = f64::from(g.susp[j]);
        if ja > 0.0 {
            let u = g.susp_rgb[j];
            let [cr, cg, cb] = c.map(f64::from);
            mix.mix(
                f64::from(u[0]),
                f64::from(u[1]),
                f64::from(u[2]),
                cr,
                cg,
                cb,
                m / (m + ja),
                &mut out,
            );
            g.susp_rgb[j] = out.map(|v| v as f32);
        } else {
            g.susp_rgb[j] = c;
        }
        g.susp[j] = (ja + m) as f32;
    }
    expand_to(g, &rr);
    Some(rr)
}

/// dry: shrink the film (never to exact zero) and SEAL the paper (wet = 0).
pub fn apply_dry(g: &mut Grid, p: &Params, tex: &[f32], dab: &Dab) -> Option<TouchedRect> {
    let force = p.k(Knob::Dryer);
    let clamp_i = dab.intensity.min(3.0);
    let mut touched = false;
    let Grid {
        s, w, h, film, wet, ..
    } = g;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        |i, _x, _y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            if stamp <= 0.0 {
                return;
            }
            let mut f = film[i] as f64 * (0.999 - stamp * force);
            if f < 0.001 {
                f = 0.001; // floor: the wet map must know the stroke passed
            }
            film[i] = f as f32;
            wet[i] = 0; // sealed: future bleed stops here
            touched = true;
        },
    );
    if touched {
        let r = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &r);
        Some(r)
    } else {
        None
    }
}

/// blow: needs the sim live. Injects the (asymmetrically clamped) pointer
/// displacement into the PERSISTENT velocity of wet cells, and drags the
/// wetness byte from the drag source through a read-before-write snapshot
/// (even across dry cells).
pub fn apply_blow(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    prev_x: f64,
    prev_y: f64,
) -> Option<TouchedRect> {
    let force = p.k(Knob::Blow);
    let clamp_i = dab.intensity.min(3.0);
    let raw_dx = dab.x - prev_x;
    let raw_dy = dab.y - prev_y;
    // Weak, slightly anti-forward: each axis clamps to [-0.6, +0.3].
    let cdx = raw_dx.clamp(-0.6, 0.3);
    let cdy = raw_dy.clamp(-0.6, 0.3);
    let off_x = js_round(raw_dx) as i32; // UNCLAMPED source offset
    let off_y = js_round(raw_dy) as i32;
    // Snapshot the wetness in the affected region so the drag reads
    // pre-write values (sampling live bytes would compound the drag within
    // one dab).
    let r = rect_around_nao_vazio(
        g,
        dab.x,
        dab.y,
        dab.r + off_x.abs().max(off_y.abs()) as f64 + 1.0,
    )?;
    let snap_w = (r.x1 - r.x0 + 1) as usize;
    let snap_h = (r.y1 - r.y0 + 1) as usize;
    let mut snap = vec![0u8; snap_w * snap_h];
    for y in r.y0..=r.y1 {
        for x in r.x0..=r.x1 {
            snap[(x - r.x0) as usize + (y - r.y0) as usize * snap_w] =
                g.wet[x as usize + y as usize * g.s];
        }
    }
    let mut touched = false;
    let w_i32 = g.w as i32;
    let h_i32 = g.h as i32;
    let geom = g.flow;
    let Grid {
        s,
        w,
        h,
        film,
        paper,
        wet,
        vel_x,
        vel_y,
        ..
    } = g;
    let stride = *s;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        |i, x, y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            if stamp <= 0.0 {
                return;
            }
            // Skip the whole pixel when the UNCLAMPED, rounded source leaves
            // the sheet.
            let sx = x - off_x;
            let sy = y - off_y;
            if sx < 1 || sx > w_i32 || sy < 1 || sy > h_i32 {
                return;
            }
            let e = stamp * force;
            if film[i] > 0.0 {
                // ⚠️ O impulso vai para a célula de FLUXO, e só a célula PROBE
                // do bloco o entrega — as `rf²` células finas de um bloco
                // somariam `rf²` vezes o mesmo empurrão, e o sopro sairia `rf²`
                // vezes forte. É a mesma lei da posse do momento no `advect`.
                if crate::flow::is_probe_cell(x, y, w_i32, h_i32, geom.rf) {
                    let vi = if geom.is_identity() {
                        i
                    } else {
                        geom.idx(
                            crate::flow::fine_to_flow(x, geom.rf),
                            crate::flow::fine_to_flow(y, geom.rf),
                        )
                    };
                    vel_x[vi] = (vel_x[vi] as f64 + cdx * e) as f32;
                    vel_y[vi] = (vel_y[vi] as f64 + cdy * e) as f32;
                }
                wet[i] = wet_byte_from_paper(paper[i] as f64); // re-wet under the air jet
            }
            // Wetness dragged from the source wherever the stamp touches,
            // wet or dry — through the read-before-write snapshot.
            let src = if sx < r.x0 || sx > r.x1 || sy < r.y0 || sy > r.y1 {
                wet[sx as usize + sy as usize * stride]
            } else {
                snap[(sx - r.x0) as usize + (sy - r.y0) as usize * snap_w]
            };
            wet[i] = (src as f64 * force + wet[i] as f64 * (1.0 - force)) as u8;
            touched = true;
        },
    );
    if touched {
        let rr = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &rr);
        Some(rr)
    } else {
        None
    }
}

/// smear: drag suspended pigment + water from the previous-dab offset (the
/// reach grows with speed). The source is a mass-weighted bilinear sample
/// of a pre-dab snapshot (weighting each corner by its mass keeps edges
/// from dragging toward black).
///
/// Doc 23 P2 — the settled layer is no longer untouchable (the old note
/// said "matching real smudge behavior"; measured against Rebelle's Smudge
/// and real re-soluble watercolor, it wasn't): on a WET cell the pass
/// dissolves settled pigment through the lift door (the next dab's
/// snapshot then drags what this one freed — smudging through wet paint IS
/// dissolve-then-drag); on a DRY cell the settled layer itself is dragged,
/// like every raster smudge. Both are resisted by `1 − staining`, so
/// `staining = 1` restores the suspended-only model to the byte.
pub fn apply_smear(
    g: &mut Grid,
    p: &Params,
    tex: &[f32],
    dab: &Dab,
    prev_x: f64,
    prev_y: f64,
) -> Option<TouchedRect> {
    let force = p.k(Knob::Smear);
    let sett_gain = 1.0 - p.k(Knob::ExtStaining);
    let clamp_i = dab.intensity.min(3.0);
    let dx = dab.x - prev_x;
    let dy = dab.y - prev_y;
    let reach = dx.abs().max(dy.abs()).ceil();
    // Snapshot the affected region (+10 px pad): sampling live arrays would
    // compound the drag within one dab.
    let pad = 10.0 + reach;
    let r = rect_around_nao_vazio(g, dab.x, dab.y, dab.r + pad)?;
    let snap_w = (r.x1 - r.x0 + 1) as usize;
    let snap_h = (r.y1 - r.y0 + 1) as usize;
    let n = snap_w * snap_h;
    let mut s_mass = vec![0.0f32; n];
    let mut s_rgb = vec![[0.0f32; 3]; n];
    let mut s_f = vec![0.0f32; n];
    let mut s_sett = vec![0.0f32; n];
    let mut s_sett_rgb = vec![[0.0f32; 3]; n];
    for y in r.y0..=r.y1 {
        for x in r.x0..=r.x1 {
            let i = x as usize + y as usize * g.s;
            let l = (x - r.x0) as usize + (y - r.y0) as usize * snap_w;
            s_mass[l] = g.susp[i];
            s_rgb[l] = g.susp_rgb[i];
            s_f[l] = g.film[i];
            s_sett[l] = g.sett[i];
            s_sett_rgb[l] = g.sett_rgb[i];
        }
    }
    let mut out = [0.0f64; 3];
    let mix = p.mix;
    let mut touched = false;
    let Grid {
        s,
        w,
        h,
        susp,
        susp_rgb,
        sett,
        sett_rgb,
        film,
        vela,
        ..
    } = g;
    for_each_stamp_pixel(
        *s,
        *w,
        *h,
        tex,
        dab.x,
        dab.y,
        dab.r,
        TOOL_HARDNESS,
        |i, x, y, fall, texv| {
            let mut stamp = fall * texv * clamp_i;
            if stamp > 1.0 {
                stamp = 1.0;
            }
            let u = (stamp * force).min(1.0);
            if u <= 0.0 {
                return;
            }
            // Clamp the sample offset to the snapshot pad.
            let mut fx = x as f64 - dx;
            let mut fy = y as f64 - dy;
            if fx < r.x0 as f64 {
                fx = r.x0 as f64;
            }
            if fx > (r.x1 - 1) as f64 {
                fx = (r.x1 - 1) as f64;
            }
            if fy < r.y0 as f64 {
                fy = r.y0 as f64;
            }
            if fy > (r.y1 - 1) as f64 {
                fy = (r.y1 - 1) as f64;
            }
            let x0 = fx.floor() as i32;
            let y0 = fy.floor() as i32;
            let tx = fx - x0 as f64;
            let ty = fy - y0 as f64;
            let l00 = (x0 - r.x0) as usize + (y0 - r.y0) as usize * snap_w;
            let l10 = l00 + 1;
            let l01 = l00 + snap_w;
            let l11 = l01 + 1;
            let w00 = (1.0 - tx) * (1.0 - ty);
            let w10 = tx * (1.0 - ty);
            let w01 = (1.0 - tx) * ty;
            let w11 = tx * ty;
            // Mass-weighted color sample.
            let m00 = s_mass[l00] as f64 * w00;
            let m10 = s_mass[l10] as f64 * w10;
            let m01 = s_mass[l01] as f64 * w01;
            let m11 = s_mass[l11] as f64 * w11;
            let src_mass = m00 + m10 + m01 + m11;
            if src_mass > 0.0 {
                let inv = 1.0 / src_mass;
                let k00 = s_rgb[l00];
                let k10 = s_rgb[l10];
                let k01 = s_rgb[l01];
                let k11 = s_rgb[l11];
                let cr = (k00[0] as f64 * m00
                    + k10[0] as f64 * m10
                    + k01[0] as f64 * m01
                    + k11[0] as f64 * m11)
                    * inv;
                let cg = (k00[1] as f64 * m00
                    + k10[1] as f64 * m10
                    + k01[1] as f64 * m01
                    + k11[1] as f64 * m11)
                    * inv;
                let cbv = (k00[2] as f64 * m00
                    + k10[2] as f64 * m10
                    + k01[2] as f64 * m01
                    + k11[2] as f64 * m11)
                    * inv;
                let in_c = alpha_of_mass(src_mass) * u;
                let res = alpha_of_mass(susp[i] as f64) * (1.0 - in_c);
                if in_c + res > 0.0 {
                    let wgt = in_c / (in_c + res);
                    let uc = susp_rgb[i];
                    mix.mix(
                        uc[0] as f64,
                        uc[1] as f64,
                        uc[2] as f64,
                        cr,
                        cg,
                        cbv,
                        wgt,
                        &mut out,
                    );
                    susp_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                }
                susp[i] = (susp[i] as f64 + (src_mass - susp[i] as f64) * u) as f32;
            }
            // Water lerps unconditionally (plain bilinear).
            let src_film = s_f[l00] as f64 * w00
                + s_f[l10] as f64 * w10
                + s_f[l01] as f64 * w01
                + s_f[l11] as f64 * w11;
            film[i] = (film[i] as f64 + (src_film - film[i] as f64) * u) as f32;
            // Doc 23 P2 — the settled layer, resisted by staining.
            let u_sett = u * sett_gain;
            if u_sett > 0.0 {
                if film[i] as f64 > 0.001 {
                    // WET cell: dissolve through the lift door; the susp
                    // drag above carries it from the next dab on.
                    if sett[i] > 0.0 {
                        let b = clamp01(u_sett);
                        lift_settled(
                            b,
                            &mut susp[i],
                            &mut sett[i],
                            &mut susp_rgb[i],
                            sett_rgb[i],
                            mix,
                            p.glaze,
                            p.ecra.com_vela(vela[i]),
                            &mut out,
                        );
                    }
                } else {
                    // DRY cell: drag the settled layer itself (same
                    // mass-weighted bilinear sample as the suspended pass).
                    let sm00 = s_sett[l00] as f64 * w00;
                    let sm10 = s_sett[l10] as f64 * w10;
                    let sm01 = s_sett[l01] as f64 * w01;
                    let sm11 = s_sett[l11] as f64 * w11;
                    let src_sett = sm00 + sm10 + sm01 + sm11;
                    if src_sett > 0.0 {
                        let inv = 1.0 / src_sett;
                        let k00 = s_sett_rgb[l00];
                        let k10 = s_sett_rgb[l10];
                        let k01 = s_sett_rgb[l01];
                        let k11 = s_sett_rgb[l11];
                        let cr = (k00[0] as f64 * sm00
                            + k10[0] as f64 * sm10
                            + k01[0] as f64 * sm01
                            + k11[0] as f64 * sm11)
                            * inv;
                        let cg = (k00[1] as f64 * sm00
                            + k10[1] as f64 * sm10
                            + k01[1] as f64 * sm01
                            + k11[1] as f64 * sm11)
                            * inv;
                        let cbv = (k00[2] as f64 * sm00
                            + k10[2] as f64 * sm10
                            + k01[2] as f64 * sm01
                            + k11[2] as f64 * sm11)
                            * inv;
                        let in_c = alpha_of_mass(src_sett) * u_sett;
                        let res = alpha_of_mass(sett[i] as f64) * (1.0 - in_c);
                        if in_c + res > 0.0 {
                            let wgt = in_c / (in_c + res);
                            let sc = sett_rgb[i];
                            mix.mix(
                                sc[0] as f64,
                                sc[1] as f64,
                                sc[2] as f64,
                                cr,
                                cg,
                                cbv,
                                wgt,
                                &mut out,
                            );
                            sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
                        }
                    }
                    if src_sett > 0.0 || sett[i] > 0.0 {
                        sett[i] = (sett[i] as f64 + (src_sett - sett[i] as f64) * u_sett) as f32;
                    }
                }
            }
            touched = true;
        },
    );
    if touched {
        let rr = rect_around(g, dab.x, dab.y, dab.r);
        expand_to(g, &rr);
        Some(rr)
    } else {
        None
    }
}
