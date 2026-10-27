//! **A CRISTA da lâmina** ([`Grid::crista`], Óleo Real W3 K3a): o rebordo que a faca deixa nas bordas do que
//! varre ([`crate::lamina`]), num campo PRÓPRIO — o mesmo valor que ela penteia em [`Grid::altura`] (a normal), mas
//! só dela: a marcha da SOMBRA soma-o à altura sem levar os fios do Oil Bristle, que moram na `altura`
//! (`docs/Painter/Pinceis/W3_K3a_faca.md` §3). Unidade da `altura` (px de relevo ÷ px por célula). Preguiçoso:
//! vazio até a primeira faca com rebordo pintar — quem não o usa não paga nem muda.

use super::Grid;

/// O campo, alocado a zeros se ainda não existia.
pub fn garante(g: &mut Grid) -> &mut [f32] {
    if g.crista.is_empty() {
        g.crista = vec![0.0; g.cells];
    }
    &mut g.crista
}

/// A crista da célula `i`; `0` sem o campo.
#[must_use]
pub fn em(g: &Grid, i: usize) -> f32 {
    g.crista.get(i).copied().unwrap_or(0.0)
}

/// Devolve a janela de um recorte (`linhas`); vazio = a grade não tinha o campo ⇒ a janela volta a zero.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[f32]) {
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(&mut g.crista, n, s, janela, linhas, 0.0);
}
