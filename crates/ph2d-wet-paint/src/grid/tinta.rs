//! O campo PREGUIÇOSO [`Grid::tinta`] (F4-2 E3, ADR-0197): o estado da lei ÚNICA da tinta por célula —
//! as somas do filme e do aberto ([`Soma`]), o fundo curado `R_g`, o meio `m`, a abertura `o` e a altura
//! que já curou. VAZIO até [`crate::sim::Sim::tinta`] `== Pigmento` correr um passo na grade ([`garante`]):
//! com a lei velha a grade não paga um byte. Nada o escreve ainda (E4–E8); o histórico, os recortes, o
//! clone e a retoma já o levam. Doc: `docs/Painter/Arquitetura/F4_2_plano_wet_paint.md` §2.1.

use super::Grid;
use ph2d_paint_tinta::Soma;

/// A tinta de uma célula na lei nova (64 B).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelulaTinta {
    /// A fatia MOLHADA (o filme): volume `vᶠ` e as somas pesadas por ele.
    pub filme: Soma,
    /// A fatia assentada mas ainda ABERTA (`vᵃ`), que o pincel ainda levanta pela abertura.
    pub aberto: Soma,
    /// A reflectância do fundo já CURADO por banda (`1` = o branco de referência).
    pub r_g: [f32; 3],
    /// O meio `m ∈ [0,1]`, médio pelo volume.
    pub m: f32,
    /// A abertura `o ∈ [0,1]`: `1` = fresco; `o → 0` fecha o aberto sobre `r_g`.
    pub o: f32,
    /// O volume que já curou (o relevo não some no fecho).
    pub altura_fechada: f32,
}

impl CelulaTinta {
    /// A célula sem tinta: sem volume, sobre o branco, aberta.
    pub const NEUTRA: Self = Self {
        filme: Soma {
            v: 0.0,
            lab: [0.0; 3],
            t: 0.0,
        },
        aberto: Soma {
            v: 0.0,
            lab: [0.0; 3],
            t: 0.0,
        },
        r_g: [1.0; 3],
        m: 0.0,
        o: 1.0,
        altura_fechada: 0.0,
    };
}

/// O campo, alocado [`CelulaTinta::NEUTRA`] se ainda não existia.
pub fn garante(g: &mut Grid) -> &mut [CelulaTinta] {
    if g.tinta.is_empty() {
        g.tinta = vec![CelulaTinta::NEUTRA; g.cells];
    }
    &mut g.tinta
}

/// A tinta da célula `i`; [`CelulaTinta::NEUTRA`] sem o campo.
#[must_use]
pub fn em(g: &Grid, i: usize) -> CelulaTinta {
    g.tinta.get(i).copied().unwrap_or(CelulaTinta::NEUTRA)
}

/// Devolve a janela de um recorte (`linhas` vazio = a grade não tinha o campo ⇒ a janela volta a NEUTRA).
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[CelulaTinta]) {
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(&mut g.tinta, n, s, janela, linhas, CelulaTinta::NEUTRA);
}

#[cfg(test)]
#[path = "tinta_tests.rs"]
mod tests;
