//! O campo PREGUIÇOSO [`Grid::aparencia`] (W3b item 3, dono 27/10: «se o usuário muda as configurações no painel,
//! não afeta a tinta já colocada no papel»): os controlos do PINCEL que o ecrã lê ao COMPOR, guardados na célula no
//! pouso — o render e as transferências verticais lêem o da célula, nunca o pincel de agora. VAZIO (=
//! [`AparenciaCelula::NEUTRA`], os valores de fábrica) até o primeiro pincel fora deles pintar: quem não mexe neles
//! não paga nada. Mistura-se pela massa como o [`Grid::relevo`]; como ele, só o pouso do rasto o escreve (o arrasto,
//! o Smear e o Blend não o movem). Doc: `docs/Painter/49_realshader/W3b_painel_so_no_futuro.md`.

use super::Grid;
use crate::sim::Params;
use crate::tuning::Knob;

/// A aparência da tinta de uma célula (12 B).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AparenciaCelula {
    /// [`Knob::PaperVisibility`] da tinta (o mestre do papel impresso nela).
    pub papel: f32,
    /// [`Knob::VisualGrain`] da tinta (a granulação impressa).
    pub grao: f32,
    /// A fracção da tinta pousada com o «Glaze layering (K-M)» ([`crate::sim::Sim::camadas_transparentes`]):
    /// `1` = a camada tapa só a sua força ([`crate::colorops::TINTA_ESCONDE_EM_VELATURA`]), `0` = a lei da sessão.
    pub vidrado: f32,
}

impl AparenciaCelula {
    /// Os valores de fábrica (os do registo e os do produto).
    pub const NEUTRA: Self = Self {
        papel: 1.0,
        grao: 1.0,
        vidrado: 0.0,
    };
}

impl Default for AparenciaCelula {
    fn default() -> Self {
        Self::NEUTRA
    }
}

/// A aparência que o pincel pousa agora.
#[must_use]
pub fn do_pincel(p: &Params) -> AparenciaCelula {
    AparenciaCelula {
        papel: p.k(Knob::PaperVisibility) as f32,
        grao: p.k(Knob::VisualGrain) as f32,
        vidrado: p.vidrado,
    }
}

/// A aparência da célula `i` — [`AparenciaCelula::NEUTRA`] sem o campo.
#[inline]
#[must_use]
pub fn da_celula(campo: &[AparenciaCelula], i: usize) -> AparenciaCelula {
    campo.get(i).copied().unwrap_or(AparenciaCelula::NEUTRA)
}

/// O vidrado da célula `i` (a lei do ecrã de cada célula, [`crate::assenta::Ecra::com_vidrado`]).
#[inline]
#[must_use]
pub fn vidrado(campo: &[AparenciaCelula], i: usize) -> f32 {
    campo.get(i).map_or(0.0, |a| a.vidrado)
}

/// Pousa `v` de massa com a aparência `nova` sobre a célula `i`, que tinha `m_old` (susp + sett): a média pela massa
/// (a lei do relevo); o papel limpo recebe a do pincel. Sem o campo, a tinta NEUTRA não o aloca.
pub fn pousa(g: &mut Grid, i: usize, m_old: f64, nova: AparenciaCelula, v: f64) {
    if g.aparencia.is_empty() {
        if nova == AparenciaCelula::NEUTRA {
            return;
        }
        g.aparencia = vec![AparenciaCelula::NEUTRA; g.cells];
    }
    let a = g.aparencia[i];
    if a != nova {
        g.aparencia[i] = if m_old > 0.0 {
            let t = v / (m_old + v);
            let m = |x: f32, y: f32| (f64::from(x) + (f64::from(y) - f64::from(x)) * t) as f32;
            AparenciaCelula {
                papel: m(a.papel, nova.papel),
                grao: m(a.grao, nova.grao),
                vidrado: m(a.vidrado, nova.vidrado),
            }
        } else {
            nova
        };
    }
}

/// Devolve a janela de um recorte (`linhas`); vazio = a grade não tinha o campo ⇒ a janela volta a NEUTRA.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[AparenciaCelula]) {
    let (n, s) = (g.cells, g.s);
    super::altura::restaura_campo(
        &mut g.aparencia,
        n,
        s,
        janela,
        linhas,
        AparenciaCelula::NEUTRA,
    );
}

/// O alcance, em células, da orla VIRGEM que a tinta nova veste ([`veste_virgens`]): medido no gate
/// `o_vidrado_da_celula_e_a_lei_de_velatura_ao_bit` (doc W3b §custo) — o pigmento que a água leva a uma célula
/// virgem pára a 1–2 células da que o rasto pousou.
pub const ORLA_VIRGEM: i32 = 2;

/// A orla VIRGEM (sem tinta) à volta da caixa `[x0..=x1] × [y0..=y1]` que o rasto acabou de pousar recebe a aparência
/// `nova`: a advecção não move este campo, e o pigmento que a água leva para fora do rasto chega a células virgens — sem
/// isto elas mostravam a [`AparenciaCelula::NEUTRA`] (um aro de grão de fábrica à volta de um traço sem papel).
/// Uma célula virgem não se desenha, logo isto nunca muda um byte do que já está na tela.
pub fn veste_virgens(g: &mut Grid, (x0, y0, x1, y1): (i32, i32, i32, i32), nova: AparenciaCelula) {
    if g.aparencia.is_empty() {
        return;
    }
    let (x0, y0) = ((x0 - ORLA_VIRGEM).max(1), (y0 - ORLA_VIRGEM).max(1));
    let (x1, y1) = (
        (x1 + ORLA_VIRGEM).min(g.w as i32),
        (y1 + ORLA_VIRGEM).min(g.h as i32),
    );
    for y in y0..=y1 {
        let o = y as usize * g.s;
        for i in o + x0 as usize..=o + x1 as usize {
            if g.susp[i] <= 0.0 && g.sett[i] <= 0.0 {
                g.aparencia[i] = nova;
            }
        }
    }
}
