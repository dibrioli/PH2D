//! **As BORDAS da água** (filho de [`super`], LOC cap): a frente da água que EMPURRA o pigmento ([`apply_wet_push`],
//! `Knob::WaterPush`) e a orla que o SECADOR deixa ([`apply_dry_edge`], `Knob::DryerEdge`) — as duas pelos lados do
//! traço, com o `prev` a dar a normal.

use super::*;

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
