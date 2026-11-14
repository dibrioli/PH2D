//! **B13.7 — a difusão (Diffusion) em JACOBI** (ADR-0147): cada célula escreve só o seu `susp`/`susp_rgb`, lendo
//! a fotografia de antes do passe — a mesma resposta em série e em paralelo. O Gauss-Seidel de antes
//! ([`super::diffusion_pass`] com a lei do pigmento) visitava as arestas pela ordem das linhas e cada uma lia o que a
//! anterior escreveu. As arestas são as mesmas: da célula DONA `i` (viva, `film > 0,1`) para `i + 1` e `i + s`, com
//! a taxa da dona. Cada aresta leva no máximo ¼ do suspenso de quem dá (≤ 4 arestas por célula ⇒ a massa nunca fica
//! negativa); com o knob no seu tecto (0,25) o corte nunca morde. A cor de quem recebe mistura-se aresta a aresta
//! numa ordem FIXA (esquerda, cima, direita, baixo), como a do Gauss-Seidel mistura cada chegada.
use crate::grid::{Grid, span_x_of};
use crate::par::{self, Rows};
use crate::sim::Params;
use crate::tuning::Knob;

/// `0` = Jacobi (o produto) · `1` = o Gauss-Seidel de antes também na lei do produto — SÓ a régua de velocidade
/// (`examples/sonda_difusao_4k.rs`): dá outra resposta, nunca é ligada pelo produto.
#[doc(hidden)]
pub static DIFUSAO_GS: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// A difusão Jacobi com a rota forçada (os gates de identidade); o produto chama [`super::diffusion_pass`].
pub fn diffusion_pass_jacobi_rows(g: &mut Grid, p: &Params, mode: Rows) {
    let (s, h) = (g.s, g.h as i32);
    if g.by1 < g.by0 {
        return;
    }
    let rate_knob = p.k(Knob::ExtDiffusion);
    let mix = p.mix;
    // Linhas escritas: as donas (by0..=by1) e a de baixo delas (recebe por `i + s`).
    let (y0, y1) = (g.by0, (g.by1 + 1).min(h - 1));
    // A fotografia lê uma linha acima e uma abaixo das escritas.
    let (f0, f1) = ((y0 - 1).max(0) as usize, ((y1 + 1).min(h - 1)) as usize);
    let foto = f0 * s..(f1 + 1) * s;
    let susp0: Vec<f32> = g.susp[foto.clone()].to_vec();
    let rgb0: Vec<[f32; 3]> = g.susp_rgb[foto].to_vec();
    let Grid {
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        by0,
        by1,
        film,
        active,
        susp,
        susp_rgb,
        ..
    } = g;
    let (row_lo, row_hi, spans_on): (&[i32], &[i32], bool) = (row_lo, row_hi, *spans_enabled);
    let (bx0, bx1, by0, by1) = (*bx0, *bx1, *by0, *by1);
    let film: &[f32] = film;
    let active: &[u8] = active;
    let base = f0 * s;
    let sv = |i: usize| f64::from(susp0[i - base]);
    // A célula `i` é dona das arestas para `i + 1` e `i + s` (a varredura do Gauss-Seidel: só a faixa viva).
    let dona = |i: usize| -> Option<f64> {
        let y = (i / s) as i32;
        let x = (i % s) as i32;
        if y < by0 || y > by1 {
            return None;
        }
        let (a, b) = span_x_of(row_lo, row_hi, spans_on, bx0, bx1, y);
        if x < a || x > b || active[i] == 0 || film[i] as f64 <= 0.1 {
            return None;
        }
        Some(rate_knob * (film[i] as f64 / 1.5).min(1.0))
    };
    // A massa que a aresta (i → nb) leva, com sinal: > 0 = de `i` para `nb`.
    let aresta = |i: usize, nb: usize| -> f64 {
        let Some(rate) = dona(i) else { return 0.0 };
        let flux = rate * (sv(i) - sv(nb));
        let from = if flux > 0.0 { i } else { nb };
        flux.signum() * flux.abs().min(0.25 * sv(from))
    };
    let b = y0 as usize * s..(y1 as usize + 1) * s;
    par::walk_rows2(
        mode,
        &mut susp[b.clone()],
        &mut susp_rgb[b],
        s,
        |r, sr, cr| {
            let y = y0 as usize + r;
            // Só as colunas que tocam uma dona: a faixa desta linha e a da de cima, ±1.
            let faixa = |yy: i32| {
                if yy < by0 || yy > by1 {
                    return (i32::MAX, i32::MIN);
                }
                span_x_of(row_lo, row_hi, spans_on, bx0, bx1, yy)
            };
            let (a0, a1) = faixa(y as i32);
            let (c0, c1) = faixa(y as i32 - 1);
            let (lo, hi) = (a0.min(c0), a1.max(c1));
            if lo > hi {
                return;
            }
            let (lo, hi) = ((lo - 1).max(0) as usize, ((hi + 1) as usize).min(s - 1));
            for x in lo..=hi {
                let j = x + y * s;
                // (vizinho, massa que CHEGA a `j`), na ordem fixa esquerda, cima, direita, baixo.
                let mut e = [(0usize, 0.0f64); 4];
                if x > 0 {
                    e[0] = (j - 1, aresta(j - 1, j));
                }
                if y > 0 {
                    e[1] = (j - s, aresta(j - s, j));
                }
                if x + 1 < s {
                    e[2] = (j + 1, -aresta(j, j + 1));
                }
                if (y as i32) < h - 1 {
                    e[3] = (j + s, -aresta(j, j + s));
                }
                if e.iter().all(|&(_, d)| d == 0.0) {
                    continue;
                }
                let sai: f64 = e.iter().map(|&(_, d)| (-d).max(0.0)).sum();
                let mut m = sv(j) - sai;
                let c = rgb0[j - base];
                let mut out = [c[0] as f64, c[1] as f64, c[2] as f64];
                for &(nb, d) in &e {
                    if d <= 0.0 {
                        continue;
                    }
                    let w = d / (m + d);
                    let fc = rgb0[nb - base];
                    let tc = out;
                    mix.mix(
                        tc[0],
                        tc[1],
                        tc[2],
                        fc[0] as f64,
                        fc[1] as f64,
                        fc[2] as f64,
                        w,
                        &mut out,
                    );
                    m += d;
                }
                sr[x] = m as f32;
                cr[x] = [out[0] as f32, out[1] as f32, out[2] as f32];
            }
        },
    );
}

/// A rota do produto: paralela acima do piso de células do gather.
pub(super) fn diffusion_pass_jacobi(g: &mut Grid, p: &Params) {
    let rows = (g.by1 - g.by0 + 2).max(0) as usize;
    diffusion_pass_jacobi_rows(g, p, Rows::pick_medido(rows, g.s, par::MIN_CELLS_GATHER));
}

#[cfg(test)]
#[path = "difusao_tests.rs"]
mod tests;
