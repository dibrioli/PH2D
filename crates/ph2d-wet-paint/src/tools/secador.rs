//! **O secador seca a tinta JÁ NIVELADA** ([`apply_dry_level`], `Knob::DryerLevel`; J6 lacuna 5,
//! `docs/Painter/Pinceis/exames/mistura.md` «Motor J6») — filho de [`super`] por LOC.

use super::*;

/// O filme da célula que ainda tem água.
const DRYER_WET_FILM: f32 = 0.01;
/// «Dentro da tinta» = a célula molhada com `≥ DRYER_IN_PAINT ×` a espessura média da água molhada à volta do dab: os
/// vales entre as cerdas sim, a franja rasa da borda não (sem o piso a faixa do cruzamento alargava 10 px para baixo).
const DRYER_IN_PAINT: f64 = 0.25;

/// ⭐ A tinta fresca tem os sulcos das cerdas (no Wet Round, 43 · 199 · 55 a 12 px) que a água nivela nos segundos
/// seguintes (~100 lisos); o secador, que tira a água de uma vez, congelava-os (as bandas 0,35 · 1,6 · 0,35 do
/// cruzamento, contra a mancha lisa do Rebelle). Cada célula dentro da tinta que o dab do Dry toca pela 1.ª vez neste
/// traço (dentro do carimbo de agora e fora do de `prev`) toma a média do pigmento SUSPENSO das células dentro da tinta
/// da caixa de lado `2·L·r + 1` (`L` = [`Knob::DryerLevel`]; a cor pesada pela massa): a água nivela a tinta sem a
/// levar ao papel seco nem à franja rasa. Somas por tabela integral (O(área) por dab, qualquer `L`). `0` = nada a
/// fazer, ao bit; o 1.º dab (sem `prev`) não nivela.
pub fn apply_dry_level(g: &mut Grid, p: &Params, dab: &Dab, prev: [f64; 2]) -> Option<TouchedRect> {
    let l = p.k(Knob::DryerLevel);
    let r = dab.r;
    if l <= 0.0 || r <= 0.0 || (dab.x - prev[0]).hypot(dab.y - prev[1]) < 1e-6 {
        return None;
    }
    let k = (l * r).round().max(1.0) as usize;
    let rr = rect_around_nao_vazio(g, dab.x, dab.y, r + k as f64 + 1.0)?;
    let (x0, y0) = (rr.x0 as usize, rr.y0 as usize);
    let (w, h) = ((rr.x1 - rr.x0 + 1) as usize, (rr.y1 - rr.y0 + 1) as usize);
    let gs = g.s;
    let idx = |x: usize, y: usize| x0 + x + (y0 + y) * gs;
    // O piso: a espessura média da água molhada do rectângulo.
    let (mut f, mut nf) = (0.0f64, 0.0f64);
    for y in 0..h {
        for x in 0..w {
            let fj = g.film[idx(x, y)];
            if fj > DRYER_WET_FILM {
                f += f64::from(fj);
                nf += 1.0;
            }
        }
    }
    if nf <= 0.0 {
        return None;
    }
    let piso = (DRYER_IN_PAINT * f / nf) as f32;
    let dentro = |g: &Grid, j: usize| g.film[j] > DRYER_WET_FILM && g.film[j] >= piso;
    // A tabela integral de (células dentro, massa, massa × cor) — `(w + 1) × (h + 1)`, a linha e a coluna 0 a zero.
    let sw = w + 1;
    let mut sat = vec![[0.0f64; 5]; sw * (h + 1)];
    for y in 0..h {
        let mut linha = [0.0f64; 5];
        for x in 0..w {
            let j = idx(x, y);
            if dentro(g, j) {
                let m = f64::from(g.susp[j]);
                let c = g.susp_rgb[j];
                linha[0] += 1.0;
                linha[1] += m;
                for q in 0..3 {
                    linha[2 + q] += m * f64::from(c[q]);
                }
            }
            let cima = sat[(x + 1) + y * sw];
            sat[(x + 1) + (y + 1) * sw] = std::array::from_fn(|q| cima[q] + linha[q]);
        }
    }
    // A soma da caixa [ax, bx) × [ay, by), em coordenadas do rectângulo.
    let soma = |ax: usize, ay: usize, bx: usize, by: usize| -> [f64; 5] {
        let (a, b) = (sat[ax + ay * sw], sat[bx + ay * sw]);
        let (c, d) = (sat[ax + by * sw], sat[bx + by * sw]);
        std::array::from_fn(|q| d[q] - b[q] - c[q] + a[q])
    };
    let mut novo: Vec<(usize, f32, [f32; 3])> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let (cx, cy) = ((x0 + x) as f64, (y0 + y) as f64);
            if (cx - dab.x).hypot(cy - dab.y) > r || (cx - prev[0]).hypot(cy - prev[1]) <= r {
                continue;
            }
            let i = idx(x, y);
            if !dentro(g, i) {
                continue;
            }
            let s = soma(
                x.saturating_sub(k),
                y.saturating_sub(k),
                (x + k + 1).min(w),
                (y + k + 1).min(h),
            );
            if s[1] <= 0.0 {
                continue;
            }
            let cor = [s[2], s[3], s[4]].map(|v| (v / s[1]) as f32);
            novo.push((i, (s[1] / s[0]) as f32, cor));
        }
    }
    if novo.is_empty() {
        return None;
    }
    for (i, m, cor) in novo {
        g.susp[i] = m;
        g.susp_rgb[i] = cor;
    }
    expand_to(g, &rr);
    Some(rr)
}
