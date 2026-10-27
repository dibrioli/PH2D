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

/// **A marca apaga-se pela MASSA** (W3 K8): a tinta que um dab que NÃO nivela ([`nivela`] falso) pousa na célula `i`
/// — que tinha `m_old` — chega por nivelar, marca `0`, e a marca vira a média pela massa (a lei do
/// [`super::Grid::relevo`] e do [`super::material::pousa`]): `marca × m_old ÷ (m_old + v)`. Sem o campo, nada.
pub fn dilui(g: &mut Grid, i: usize, m_old: f64, v: f64) {
    if let Some(m) = g.nivelado.get_mut(i)
        && *m > 0.0
    {
        let total = m_old + v;
        *m = if total > 0.0 {
            (f64::from(*m) * m_old / total) as f32
        } else {
            0.0
        };
    }
}

/// O arrasto leva a marca com a tinta (o `t` do material: a célula vazia toma a de lá; a outra, a fracção).
pub fn arrasta(g: &mut Grid, i: usize, si: usize, t: f32) {
    if g.nivelado.is_empty() {
        return;
    }
    let (a, b) = (g.nivelado[i], g.nivelado[si]);
    g.nivelado[i] = a + (b - a) * t;
}

/// O dab de `p` NIVELA (`knifeLevel` > 0): a sua tinta é a da lâmina — a marca é dela ([`crate::lamina::nivelar`]).
#[must_use]
pub fn nivela(p: &crate::sim::Params) -> bool {
    p.k(crate::tuning::Knob::KnifeLevel) > 0.0
}
