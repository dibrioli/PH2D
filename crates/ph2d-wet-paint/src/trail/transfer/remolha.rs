//! ⭐ **A água que RE-MOLHA** (dono 30/10, J10; prancha `docs/Painter/Pinceis/pranchas/J10_remolha_*.png`, escolhida
//! «re-molha médio»): durante o traço, a água deste pincel dissolve a fracção `Re-wet × Solubility` da tinta ASSENTADA
//! debaixo dela ([`Knob::Dissolve`] do pincel × [`OleoCelula::soluvel`] da célula). O dissolvido passa ao suspenso pela
//! porta do lift ([`crate::drying::lift_settled`]) pela lei de DUAS TINTAS NA MESMA ÁGUA (`Ecra::Porto`: a mistura e a
//! velatura — os pigmentos somam a absorção, o azul com o amarelo dá verde; a lei do ecrã que «não muda o que se vê»
//! deixava a água azul-ardósia ao sair do óleo, medido 30/10): mistura-se com a tinta nova e viaja com a água — o arrasto durante o
//! traço (a ponta suja apanha-o) e a mancha que se espalha ao simular. O relevo do óleo cai com o assentado.
//!
//! [`Knob::Dissolve`]: crate::tuning::Knob::Dissolve
//! [`OleoCelula::soluvel`]: crate::grid::oleo_celula::OleoCelula::soluvel

use crate::grid::Grid;

/// A fracção do alvo de uma célula que um toque de cobertura plena dissolve: o alvo chega em ~4 toques, ao longo do
/// traço (A: a tinta de baixo já se mexe a meio dele), nunca de uma vez no primeiro dab.
const PASSO: f64 = 0.25;

/// O assentado de cada célula no PRIMEIRO toque deste traço (`< 0` = ainda não tocada): o tecto do que ele dissolve é
/// medido contra ele, para o traço não dissolver tudo por voltar à mesma célula. Vazio fora de um traço que re-molha.
#[derive(Clone, Default)]
pub(crate) struct Inicio(Vec<f32>);

impl Inicio {
    /// Um traço novo: esquece os toques do anterior (sem largar a memória).
    pub(crate) fn traco_novo(&mut self) {
        self.0.clear();
    }
}

/// Re-molha a célula `i` com a força `forca` (o `Re-wet` do pincel) e a cobertura `in_a` do toque.
#[allow(clippy::too_many_arguments)]
pub(super) fn remolha(
    g: &mut Grid,
    i: usize,
    forca: f64,
    in_a: f64,
    inicio: &mut Inicio,
    mix: crate::colorops::ColorMix,
    glaze: f64,
    out: &mut [f64; 3],
) {
    let f = (forca * f64::from(crate::grid::oleo_celula::da_celula(&g.oleo, i).soluvel)).min(1.0);
    if f <= 0.0 || g.sett[i] <= 0.0 || in_a <= 0.0 {
        return;
    }
    if inicio.0.len() != g.cells {
        inicio.0 = vec![-1.0; g.cells];
    }
    if inicio.0[i] < 0.0 {
        inicio.0[i] = g.sett[i];
    }
    let s0 = f64::from(inicio.0[i]);
    let sett = f64::from(g.sett[i]);
    let sobra = sett - s0 * (1.0 - f);
    if sobra <= 0.0 {
        return;
    }
    let dm = (s0 * f * PASSO).min(sobra);
    let sett_rgb = g.sett_rgb[i];
    crate::drying::lift_settled(
        dm / sett,
        &mut g.susp[i],
        &mut g.sett[i],
        &mut g.susp_rgb[i],
        sett_rgb,
        mix,
        glaze,
        crate::assenta::Ecra::Porto,
        out,
    );
}
