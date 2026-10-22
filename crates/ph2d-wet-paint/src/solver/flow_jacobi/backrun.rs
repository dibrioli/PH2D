//! **O BACKRUN** do fluxo de Jacobi — os predicados dos vizinhos materializados e o levante do pigmento assentado
//! (filho de [`super`], LOC cap). A prova de que o gather iguala o serial está no cabeçalho do pai.

use super::*;

/// Os dois predicados que o backrun pergunta aos VIZINHOS, materializados
/// antes de qualquer escrita — ver o cabeçalho do módulo para a prova de que
/// os dois são invariantes sob o passe.
fn bflag_rows(g: &mut Grid, mode: Rows) {
    let s = g.s;
    // A janela é a bbox FINA dilatada de um bloco: o levante alcança vizinhos
    // fora da região ativa.
    let rf = g.flow.rf as i32;
    let (y0, y1) = (
        (g.by0 - rf).max(0) as usize,
        (g.by1 + rf).min(g.h as i32 + 1) as usize,
    );
    if y0 > y1 {
        return;
    }
    let Grid {
        scratch,
        sett,
        bloom,
        ..
    } = g;
    let sett_full: &[f32] = sett;
    let bloom_full: &[u8] = bloom;
    let band = y0 * s..(y1 + 1) * s;
    par::walk_rows(mode, &mut scratch.bflags[band], s, |ri, row| {
        let base = (y0 + ri) * s;
        for (x, cell) in row.iter_mut().enumerate() {
            let i = base + x;
            *cell = u8::from(sett_full[i] > 0.0) * BF_HAS_SETT
                + u8::from(bloom_full[i] < 6) * BF_CAN_BLOOM;
        }
    });
}

/// **O BACKRUN** — o levante do pigmento assentado e o orçamento de floração.
///
/// ⚠️ **Gather:** onde o serial fazia a célula-gatilho ESCREVER nos quatro
/// vizinhos, aqui cada célula CONTA quantos vizinhos a disparam e aplica o
/// levante esse número de vezes. É o mesmo número — ver o cabeçalho.
pub(super) fn backrun_rows(g: &mut Grid, c: &FlowConst, mode: Rows) {
    let fine_s = g.s;
    let (gw, gh) = (g.w as i32, g.h as i32);
    let geom = g.flow;
    let (gbx0, by0, gbx1, by1) = crate::flow::flow_bbox((g.bx0, g.by0, g.bx1, g.by1), geom.rf);
    if by0 > by1 || gbx0 > gbx1 {
        return;
    }
    g.scratch.ensure_backrun(g.cells);
    bflag_rows(g, mode);

    let spans_on = g.spans_enabled;
    // ⚠️ O EMPURRÃO (`ext_backrun`) é do núcleo — ele escreve `flow`, e escrever
    // `flow` aqui seria a segunda porta do mesmo número. Aqui só o levante e o
    // orçamento de floração, que são as duas escritas cross-linha.
    let mix = c.mix;
    let Grid {
        scratch,
        frow_lo: row_lo,
        frow_hi: row_hi,
        film,
        susp,
        sett,
        susp_rgb,
        sett_rgb,
        active,
        bloom,
        ..
    } = g;
    let film_full: &[f32] = film;
    let active_full: &[u8] = active;
    let sett_rgb_full: &[[f32; 3]] = sett_rgb;
    let bflags: &[u8] = &scratch.bflags;

    // ⚠️ **Tudo aqui caminha em espaço de FLUXO**, e não por conveniência: os
    // probes de duas células de fluxo vizinhas distam exatamente `rf` células
    // finas, então *o vizinho fino do backrun É a célula de fluxo ao lado*. É
    // isso que deixa o limiar do gatilho sair das coordenadas dele sem
    // nenhuma conversão de volta.
    let probe_row = |y: i32| crate::flow::flow_probe(y, gh, geom.rf) as usize;

    // (a) O ORÇAMENTO DE FLORAÇÃO — cada célula decide o próprio `bloom` a
    //     partir dos SEUS quatro vizinhos, exatamente como o serial fazia.
    let (bl_lo, bl_hi) = (probe_row(by0), probe_row(by1));
    par::walk_rows(
        mode,
        &mut bloom[bl_lo * fine_s..(bl_hi + 1) * fine_s],
        fine_s,
        |ri, br| {
            let fy = bl_lo + ri;
            let y = crate::flow::fine_to_flow(fy as i32, geom.rf);
            if probe_row(y) != fy || y < by0 || y > by1 {
                return;
            }
            let (bx0, bx1) = crate::grid::span_x_of(row_lo, row_hi, spans_on, gbx0, gbx1, y);
            if bx0 > bx1 {
                return;
            }
            for x in bx0..=bx1 {
                let px = crate::flow::flow_probe(x, gw, geom.rf) as usize;
                let fi = fy * fine_s + px;
                if active_full[fi] == 0 {
                    continue;
                }
                let f = film_full[fi] as f64;
                let thr = 0.8 + 0.2 * hash2_signed(x, y, BLOOM_SEED);
                let can_bloom = bflags[fi] & BF_CAN_BLOOM != 0;
                let mut met = false;
                for (dxn, dyn_) in NB4 {
                    let nb = probe_row(y + dyn_ as i32) * fine_s
                        + crate::flow::flow_probe(x + dxn as i32, gw, geom.rf) as usize;
                    if f - film_full[nb] as f64 > thr && bflags[nb] & BF_HAS_SETT != 0 {
                        met = true;
                    }
                }
                if met {
                    if can_bloom {
                        br[px] += 1;
                    }
                } else {
                    br[px] = 0;
                }
            }
        },
    );

    // (b) O LEVANTE — cada célula CONTA quantos vizinhos a disparam e aplica
    //     `F` esse número de vezes. A janela é a das células de fluxo
    //     dilatada de UMA, porque o alvo de um levante não precisa estar vivo.
    let (lo, hi) = (
        probe_row((by0 - 1).max(0)),
        probe_row((by1 + 1).min(gh + 1)),
    );
    let band = lo * fine_s..(hi + 1) * fine_s;
    par::walk_rows3(
        mode,
        &mut susp[band.clone()],
        &mut sett[band.clone()],
        &mut susp_rgb[band],
        fine_s,
        |ri, ur, tr, cr| {
            let fy = lo + ri;
            let y = crate::flow::fine_to_flow(fy as i32, geom.rf);
            if probe_row(y) != fy {
                return;
            }
            let mut out = [0.0f64; 3];
            for x in (gbx0 - 1).max(0)..=(gbx1 + 1).min(geom.w as i32 + 1) {
                let px = crate::flow::flow_probe(x, gw, geom.rf) as usize;
                let fi = fy * fine_s + px;
                if bflags[fi] & BF_HAS_SETT == 0 {
                    continue;
                }
                let mut n = 0u32;
                for (dxn, dyn_) in NB4 {
                    // O GATILHO é o VIZINHO: ele é quem tem de estar vivo, ter
                    // orçamento, e ver o degrau de film contra ESTA célula.
                    let (dx, dy) = (x + dxn as i32, y + dyn_ as i32);
                    if dx < 0 || dy < 0 || dx > geom.w as i32 + 1 || dy > geom.h as i32 + 1 {
                        continue;
                    }
                    let du =
                        probe_row(dy) * fine_s + crate::flow::flow_probe(dx, gw, geom.rf) as usize;
                    if active_full[du] == 0 || bflags[du] & BF_CAN_BLOOM == 0 {
                        continue;
                    }
                    let thr = 0.8 + 0.2 * hash2_signed(dx, dy, BLOOM_SEED);
                    if film_full[du] as f64 - film_full[fi] as f64 > thr {
                        n += 1;
                    }
                }
                // `F^n` — e `F` não conhece o gatilho, então a ordem em que os
                // `n` foram descobertos não entra em nenhum termo.
                let sc = sett_rgb_full[fi];
                for _ in 0..n {
                    let lift = tr[px] as f64 * 0.1;
                    let w = lift / (ur[px] as f64 + lift);
                    let uc = cr[px];
                    mix.mix(
                        uc[0] as f64,
                        uc[1] as f64,
                        uc[2] as f64,
                        sc[0] as f64,
                        sc[1] as f64,
                        sc[2] as f64,
                        w,
                        &mut out,
                    );
                    cr[px] = [out[0] as f32, out[1] as f32, out[2] as f32];
                    ur[px] = (ur[px] as f64 + lift) as f32;
                    tr[px] = (tr[px] as f64 - lift) as f32;
                }
            }
        },
    );
}

/// Os quatro vizinhos, na ORDEM do serial (esquerda, direita, cima, baixo).
const NB4: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
