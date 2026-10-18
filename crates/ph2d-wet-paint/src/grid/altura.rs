//! O campo PREGUIÇOSO [`Grid::altura`] (W3): vazio até o primeiro pincel com relevo de fios pintar na grade —
//! a 4096² seriam 67 MB por camada (e por instantâneo) para quem nunca o usa.

use super::Grid;

/// O campo, alocado a zeros se ainda não existia.
pub fn garante(g: &mut Grid) -> &mut [f32] {
    if g.altura.is_empty() {
        g.altura = vec![0.0; g.cells];
    }
    &mut g.altura
}

/// A altura (px) da célula `i`; `0` sem o campo.
#[must_use]
pub fn em(g: &Grid, i: usize) -> f32 {
    g.altura.get(i).copied().unwrap_or(0.0)
}

/// Devolve a janela `[x0..=x1] × [y0..=y1]` de um recorte (`linhas`, linha-a-linha): vazio = a grade não tinha
/// o campo quando o recorte foi tirado ⇒ a janela volta a zero (sem alocar, se o campo também não existe).
pub fn restaura_janela(g: &mut Grid, (x0, y0, x1, y1): (i32, i32, i32, i32), linhas: &[f32]) {
    if linhas.is_empty() && g.altura.is_empty() {
        return;
    }
    let (s, w) = (g.s, (x1 - x0 + 1) as usize);
    let campo = garante(g);
    for (r, y) in (y0..=y1).enumerate() {
        let o = y as usize * s + x0 as usize;
        match linhas.get(r * w..(r + 1) * w) {
            Some(l) => campo[o..o + w].copy_from_slice(l),
            None => campo[o..o + w].fill(0.0),
        }
    }
}

#[cfg(test)]
#[path = "altura_tests.rs"]
mod tests;
