//! O campo PREGUIÇOSO [`Grid::oleo`] (dono 28/10): a caixa **Real Oil**, o **Depth** e o **Gloss** que o pincel
//! pousou na célula — eram da tela inteira (o cartão Real Oil), são do TRAÇO: mudá-los vale só para os seguintes.
//! VAZIO (= [`OleoCelula::NEUTRA`]: sem óleo) até o primeiro traço com o Real Oil ligado; a tinta sem óleo pousa a
//! NEUTRA (o Depth e o Gloss dela não se vêem), por isso «o campo existe» = «houve óleo nesta grade».
//! Mistura-se pela massa como o [`Grid::relevo`]; só o pouso do rasto o escreve; a advecção não o move (a orla
//! virgem, [`veste_virgens`]). Lido pelo campo de altura e pelo sombreado (`oleo_real::CampoAltura::de_com`).
//! Doc: `docs/Painter/49_realshader/W3b_painel_so_no_futuro.md` §Depth e Gloss.

use super::Grid;
use crate::sim::Params;
use crate::tuning::Knob;

/// O óleo da tinta de uma célula (28 B) — e a SOLUBILIDADE (J10), o MEIO e o CORPO dela (E10), que viajam pela mesma
/// porta (pousam pela massa, desfazem-se com a janela).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OleoCelula {
    /// A fracção da tinta pousada com o **Real Oil** ligado (`1` = toda; mistura-se pela massa).
    pub oleo: f32,
    /// [`Knob::OilDepth`]: o ganho da altura do relevo desta tinta.
    pub profundidade: f32,
    /// [`Knob::OilGloss`]: a força do brilho especular desta tinta.
    pub brilho: f32,
    /// [`Knob::Solubility`] (dono 30/10, J10): quanto a água de um traço seguinte dissolve desta tinta (`0` = nada).
    pub soluvel: f32,
    /// [`Knob::Medium`] (E10): o meio da tinta (`0` água · `1` óleo) — o `m` da lei de Bingham
    /// ([`crate::solver::ladeira_do_oleo::tensao`]). O Real Oil (`oleo`) ficou só a luz e o relevo.
    pub meio: f32,
    /// [`Knob::Body`] (E10, dono Q9): a tensão de cedência do PINCEL, somada à do meio e sem precisar de `φ > φ_c`.
    pub corpo: f32,
    /// O **Real Oil** da ÚLTIMA mão que pousou aqui (E10e1): o óleo sobre o óleo cobre como a opaca
    /// ([`crate::assenta::Ecra::com_oleo`]). Era o pincel da MÃO (a sessão) e ligar a caixa mudava a tinta pousada.
    /// O pouso SOBRESCREVE (não se mistura pela massa).
    pub pincel_de_oleo: bool,
}

impl OleoCelula {
    /// A tinta SEM óleo, com o Depth e o Gloss de fábrica (os do registo; o cartão de antes a `1`).
    pub const NEUTRA: Self = Self {
        oleo: 0.0,
        profundidade: 1.0,
        brilho: 1.0,
        soluvel: 0.0,
        meio: 0.0,
        corpo: 0.0,
        pincel_de_oleo: false,
    };
}

impl Default for OleoCelula {
    fn default() -> Self {
        Self::NEUTRA
    }
}

/// O que o pincel pousa agora: sem o Real Oil, a NEUTRA (o Depth e o Gloss só existem com ele).
#[must_use]
pub fn do_pincel(p: &Params) -> OleoCelula {
    let soluvel = p.k(Knob::Solubility) as f32;
    let (meio, corpo) = (p.k(Knob::Medium) as f32, p.k(Knob::Body) as f32);
    if p.oleo <= 0.0 {
        return OleoCelula {
            soluvel,
            meio,
            corpo,
            ..OleoCelula::NEUTRA
        };
    }
    OleoCelula {
        oleo: p.oleo,
        profundidade: p.k(Knob::OilDepth) as f32,
        brilho: p.k(Knob::OilGloss) as f32,
        soluvel,
        meio,
        corpo,
        pincel_de_oleo: true,
    }
}

/// O ganho da altura de óleo da célula `i` (`Depth × Real Oil`): `0` = a tinta sem óleo é lisa para o campo.
#[inline]
#[must_use]
pub fn ganho(campo: &[OleoCelula], i: usize) -> f64 {
    let c = da_celula(campo, i);
    f64::from(c.profundidade) * f64::from(c.oleo)
}

/// O da célula `i` — [`OleoCelula::NEUTRA`] sem o campo.
#[inline]
#[must_use]
pub fn da_celula(campo: &[OleoCelula], i: usize) -> OleoCelula {
    campo.get(i).copied().unwrap_or(OleoCelula::NEUTRA)
}

/// Pousa `v` de massa com `nova` sobre a célula `i`, que tinha `m_old` (susp + sett): a média pela massa (a lei do
/// relevo); o papel limpo recebe a do pincel. Sem o campo, a NEUTRA não o aloca.
pub fn pousa(g: &mut Grid, i: usize, m_old: f64, nova: OleoCelula, v: f64) {
    if g.oleo.is_empty() {
        if nova == OleoCelula::NEUTRA {
            return;
        }
        g.oleo = vec![OleoCelula::NEUTRA; g.cells];
    }
    let a = g.oleo[i];
    if a != nova {
        g.oleo[i] = if m_old > 0.0 {
            let t = v / (m_old + v);
            let m = |x: f32, y: f32| (f64::from(x) + (f64::from(y) - f64::from(x)) * t) as f32;
            OleoCelula {
                oleo: m(a.oleo, nova.oleo),
                profundidade: m(a.profundidade, nova.profundidade),
                brilho: m(a.brilho, nova.brilho),
                soluvel: m(a.soluvel, nova.soluvel),
                meio: m(a.meio, nova.meio),
                corpo: m(a.corpo, nova.corpo),
                pincel_de_oleo: nova.pincel_de_oleo,
            }
        } else {
            nova
        };
    }
}

/// O alcance da orla virgem: o do material (a mesma tinta, a mesma água — medido em [`super::material::ORLA_VIRGEM`]).
pub const ORLA_VIRGEM: i32 = super::material::ORLA_VIRGEM;

/// A orla VIRGEM (sem tinta) à volta da caixa que o rasto acabou de pousar recebe `nova` — o par de
/// [`super::material::veste_virgens`]: o pigmento que a água leva para fora do rasto não leva o campo, e uma célula
/// sem ele subia ou brilhava com o NEUTRO (um aro de relevo de fábrica à volta de um traço raso). Uma célula virgem
/// não se desenha: nunca muda um byte do que já está na tela.
pub fn veste_virgens(g: &mut Grid, (x0, y0, x1, y1): (i32, i32, i32, i32), nova: OleoCelula) {
    if g.oleo.is_empty() {
        return;
    }
    let r = ORLA_VIRGEM;
    let (x0, y0) = ((x0 - r).max(1), (y0 - r).max(1));
    let (x1, y1) = ((x1 + r).min(g.w as i32), (y1 + r).min(g.h as i32));
    for y in y0..=y1 {
        let o = y as usize * g.s;
        for i in o + x0 as usize..=o + x1 as usize {
            if g.susp[i] <= 0.0 && g.sett[i] <= 0.0 {
                g.oleo[i] = nova;
            }
        }
    }
}

/// Devolve a janela de um recorte (`linhas`); vazio = a grade não tinha o campo ⇒ a janela volta a NEUTRA.
/// Um desfazer que devolve a grade a «sem óleo em parte nenhuma» LARGA o campo (o composite volta ao caminho sem óleo,
/// ao byte, e larga a memória dele).
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[OleoCelula]) {
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(&mut g.oleo, n, s, janela, linhas, OleoCelula::NEUTRA);
    if linhas.is_empty()
        && g.oleo.iter().all(|c| {
            c.oleo <= 0.0
                && !c.pincel_de_oleo
                && c.soluvel <= 0.0
                && c.meio <= 0.0
                && c.corpo <= 0.0
        })
    {
        g.oleo = Vec::new();
    }
}

#[cfg(test)]
#[path = "oleo_celula_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "meio_corpo_tests.rs"]
mod meio_corpo_tests;
