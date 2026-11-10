//! **O meio anda com o LÍQUIDO** (janela 9, J9 §1): o `OleoCelula::meio` é propriedade do volume molhado
//! (`susp + K_W·film`, a régua de [`crate::meio::mistura`]); o gather do [`super::advect_jacobi`] move esse volume
//! com os mesmos coeficientes, e aqui o meio vai na mesma combinação. A célula seca não tem voto: a água que chega a
//! óleo seco é água. Sem meio na faixa o passe não corre (a água ao bit). Doc: `docs/Painter/Arquitetura/J9_meio_no_liquido.md`.

use crate::grid::Grid;
use crate::painter::receita::K_W;
use crate::par::{self, Rows};

/// Ferramenta de MEDIÇÃO (sondas), nunca o produto: `true` = o meio fica na célula (a lei de antes da J9).
#[doc(hidden)]
pub static MEIO_PARADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Há meio na faixa viva (o campo existe e alguma célula tem `m > 0`).
pub(super) fn precisa(g: &Grid) -> bool {
    if MEIO_PARADO.load(std::sync::atomic::Ordering::Relaxed) || g.oleo.is_empty() || g.by0 > g.by1
    {
        return false;
    }
    let banda = g.by0 as usize * g.s..(g.by1 as usize + 1) * g.s;
    banda.into_iter().any(|i| g.oleo[i].meio > 0.0)
}

#[inline]
fn molhado(g: &Grid, i: usize) -> f64 {
    f64::from(g.susp[i]) + K_W * f64::from(g.film[i])
}

/// O gather do meio com o `uv`/`outflow` que o passe deixou no rascunho, lido do estado do INÍCIO (antes do
/// `commit`): `m' = Σ coef·molhado·m ÷ Σ coef·molhado`; sem volume molhado (ou com o mesmo meio em todas as fontes) a
/// célula guarda o seu. Por linhas, na rota `mode` do passe.
pub(super) fn advect(g: &mut Grid, mode: Rows) {
    let s = g.s;
    let banda = g.by0 as usize * s..(g.by1 as usize + 1) * s;
    let mut dst: Vec<f32> = banda.clone().map(|i| g.oleo[i].meio).collect();
    let g0: &Grid = g;
    par::walk_rows(mode, &mut dst, s, |r, row| {
        let y = g0.by0 + r as i32;
        let (lo, hi) = g0.span_x(y);
        for x in lo.max(0)..=hi {
            if let Some(m) = gather(g0, x, y) {
                row[x as usize] = m;
            }
        }
    });
    // Só o que mudou ao bit se escreve: o pedaço do chão que o passe não move não acorda (B12.3).
    for (i, m) in banda.zip(dst) {
        if g.oleo[i].meio.to_bits() != m.to_bits() {
            g.oleo[i].meio = m;
        }
    }
}

/// O meio novo da célula `(x, y)`, `None` = fica o seu.
#[inline]
fn gather(g: &Grid, x: i32, y: i32) -> Option<f32> {
    let (wf, hf, s) = (g.w as f64, g.h as f64, g.s);
    let i = x as usize + y as usize * s;
    let m0 = g.oleo[i].meio;
    if g.active[i] == 0 {
        return None;
    }
    let uv = g.scratch.uv[i];
    let (x0, fx, y0, fy) = super::backtrace(x, y, f64::from(uv[0]), f64::from(uv[1]), wf, hf)?;
    let i00 = x0 as usize + y0 as usize * s;
    let cantos = [i00, i00 + 1, i00 + s, i00 + s + 1];
    if cantos.iter().all(|&c| g.oleo[c].meio == m0) {
        return None;
    }
    let leave = f64::from(g.scratch.outflow[i]).min(1.0);
    let fica = molhado(g, i) * (1.0 - leave);
    let (mut peso, mut soma) = (fica, fica * f64::from(m0));
    let w = [
        (1.0 - fx) * (1.0 - fy),
        fx * (1.0 - fy),
        (1.0 - fx) * fy,
        fx * fy,
    ];
    for (c, wk) in cantos.into_iter().zip(w) {
        let v = wk * super::take_scale(f64::from(g.scratch.outflow[c])) * molhado(g, c);
        peso += v;
        soma += v * f64::from(g.oleo[c].meio);
    }
    (peso > 0.0).then(|| (soma / peso).clamp(0.0, 1.0) as f32)
}

#[cfg(test)]
#[path = "meio_transporte_tests.rs"]
mod tests;
