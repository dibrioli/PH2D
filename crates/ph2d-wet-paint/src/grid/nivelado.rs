//! **A MARCA do nivelamento** ([`Grid::nivelado`], Óleo Real W3 K7): quanto a lâmina da faca NIVELOU cada célula
//! ([`crate::lamina::nivelar`], knob `knifeLevel`), `0..=1` — o maior `knifeLevel × cobertura` que lá passou. A FORMA
//! do Óleo Real ([`crate::oleo_real::CampoAltura`]) poupa estas células ao desfoque largo: a parede do planalto fica
//! viva (`docs/Painter/49_realshader/oraculo/README.md` §K7). Preguiçoso: vazio até a primeira faca que nivela —
//! quem não o usa não paga nem muda.

use super::Grid;

/// O campo, alocado a zeros se ainda não existia.
pub fn garante(g: &mut Grid) -> &mut [f32] {
    if g.nivelado.is_empty() {
        g.nivelado = vec![0.0; g.cells];
    }
    &mut g.nivelado
}

/// A marca da célula `i`; `0` sem o campo.
#[must_use]
pub fn em(g: &Grid, i: usize) -> f32 {
    g.nivelado.get(i).copied().unwrap_or(0.0)
}

/// Devolve a janela de um recorte (`linhas`); vazio = a grade não tinha o campo ⇒ a janela volta a zero.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[f32]) {
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(&mut g.nivelado, n, s, janela, linhas, 0.0);
}
