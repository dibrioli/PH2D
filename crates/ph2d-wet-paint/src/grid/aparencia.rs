//! O campo PREGUIÇOSO [`Grid::aparencia`] (W3b item 3, dono 27/10: «se o usuário muda as configurações no painel,
//! não afeta a tinta já colocada no papel»): os controlos do PINCEL que o ecrã lê ao COMPOR, guardados na célula no
//! pouso — o render e as transferências verticais lêem o da célula, nunca o pincel de agora. VAZIO (=
//! [`AparenciaCelula::NEUTRA`], os valores de fábrica) até o primeiro pincel fora deles pintar: quem não mexe neles
//! não paga nada. Mistura-se pela massa como o [`Grid::relevo`]; como ele, só o pouso do rasto o escreve (o arrasto,
//! o Smear e o Blend não o movem). Doc: `docs/Painter/49_realshader/W3b_painel_so_no_futuro.md`.

use super::Grid;
use crate::sim::Params;
use crate::tuning::Knob;

/// A aparência da tinta de uma célula (24 B).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AparenciaCelula {
    /// [`Knob::PaperVisibility`] da tinta (o mestre do papel impresso nela).
    pub papel: f32,
    /// [`Knob::VisualGrain`] da tinta (a granulação impressa).
    pub grao: f32,
    /// A fracção da tinta pousada com o «Glaze layering (K-M)» ([`crate::sim::Sim::camadas_transparentes`]):
    /// `1` = a camada tapa só a sua força ([`crate::colorops::TINTA_ESCONDE_EM_VELATURA`]), `0` = a lei da sessão.
    pub vidrado: f32,
    /// A fracção da tinta pousada com a **Paint Texture** (o papel impresso na tinta; dono 28/10: é do TRAÇO):
    /// `1` = imprime o papel pela [`Self::papel`]/[`Self::grao`] dela, `0` = não imprime (a de fábrica).
    pub textura: f32,
    /// [`Knob::Opaque`] da tinta: a fracção com que o ecrã a prende ao segmento das duas tintas
    /// ([`crate::assenta::Ecra::com_opaca`]); `0` = a lei da aguarela.
    pub opaca: f32,
    /// O **Smooth Edges** da tinta (`0..1`; dono 04/11: é do TRAÇO, cada pincel o seu): cada borda amacia pelo dela, na
    /// cor e no relevo; mudar o controlo não recompõe o que já está pintado.
    pub suave: f32,
}

impl AparenciaCelula {
    /// Os valores de fábrica (os do registo e os do produto).
    pub const NEUTRA: Self = Self {
        papel: 1.0,
        grao: 1.0,
        vidrado: 0.0,
        textura: 0.0,
        opaca: 0.0,
        suave: 0.0,
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
        textura: p.textura,
        opaca: p.k(Knob::Opaque) as f32,
        suave: p.suave,
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

/// A opacidade da célula `i` ([`crate::assenta::Ecra::com_opaca`]).
#[inline]
#[must_use]
pub fn opaca(campo: &[AparenciaCelula], i: usize) -> f32 {
    campo.get(i).map_or(0.0, |a| a.opaca)
}

/// Em quantos degraus a força se reparte ao AMACIAR (`raio = degrau ÷ 25 × 6 px`, 0,24 px por degrau): o valor pousado
/// é contínuo (a mistura pela massa) e cada raio presente na região é um desfoque. O recurso é o quadro.
pub const DEGRAUS_DO_SUAVE: u8 = 25;
/// O raio do alisamento no máximo do controlo, px (medido na prancha de 10/10, `05` §4b).
pub const SUAVE_RAIO_MAX_PX: f32 = 6.0;

/// O degrau de uma força `0..1`.
#[inline]
#[must_use]
pub fn degrau_do_suave(forca: f32) -> u8 {
    (forca.clamp(0.0, 1.0) * f32::from(DEGRAUS_DO_SUAVE)).round() as u8
}

/// O raio do degrau `d`, px.
#[inline]
#[must_use]
pub fn raio_do_degrau(d: u8) -> f32 {
    f32::from(d) / f32::from(DEGRAUS_DO_SUAVE) * SUAVE_RAIO_MAX_PX
}

/// O Smooth Edges que se vê na célula `i`: o da camada de baixo com tinta e cada camada visível por cima pela cobertura
/// dela (a lei do Gloss visto, `oleo_campo`); `0` sem tinta.
#[must_use]
pub fn suave_visto(camadas: &[(&Grid, bool)], i: usize) -> f32 {
    let mut q: Option<f64> = None;
    for &(g, visivel) in camadas {
        if !visivel {
            continue;
        }
        let m = f64::from(g.sett[i]) + f64::from(g.susp[i]);
        if m > 0.0 {
            let b = f64::from(da_celula(&g.aparencia, i).suave);
            let alfa = crate::opacity::alpha_of_mass(m);
            q = Some(q.map_or(b, |q| q + (b - q) * alfa));
        }
    }
    q.unwrap_or(0.0) as f32
}

/// O maior Smooth Edges que as camadas visíveis já pousaram (`0` = ninguém: todo o caminho de antes, ao byte). É um
/// tecto, só desce com a tela limpa: apagar uma tinta suave ainda tem de recompor a orla que ela deixou.
#[must_use]
pub fn teto_do_suave(camadas: &[(&Grid, bool)]) -> f32 {
    camadas
        .iter()
        .filter(|&&(_, v)| v)
        .fold(0.0, |t, &(g, _)| t.max(g.suave_teto))
}

/// O raio máximo, px, que o tecto pode dar a uma célula (o degrau arredonda): o alcance de uma mudança e a janela do
/// campo lêem-no; `0` = o caminho de antes.
#[must_use]
pub fn raio_do_teto(camadas: &[(&Grid, bool)]) -> f32 {
    raio_do_degrau(degrau_do_suave(teto_do_suave(camadas)))
}

/// As portas que reescrevem o campo inteiro (o instantâneo, o histórico) recontam o tecto: nunca o baixam.
pub fn reconta_teto(g: &mut Grid) {
    g.suave_teto = g.aparencia.iter().fold(g.suave_teto, |t, a| t.max(a.suave));
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
    g.suave_teto = g.suave_teto.max(nova.suave);
    let a = g.aparencia[i];
    if a != nova {
        g.aparencia[i] = if m_old > 0.0 {
            let t = v / (m_old + v);
            let m = |x: f32, y: f32| (f64::from(x) + (f64::from(y) - f64::from(x)) * t) as f32;
            AparenciaCelula {
                papel: m(a.papel, nova.papel),
                grao: m(a.grao, nova.grao),
                vidrado: m(a.vidrado, nova.vidrado),
                textura: m(a.textura, nova.textura),
                opaca: m(a.opaca, nova.opaca),
                suave: m(a.suave, nova.suave),
            }
        } else {
            nova
        };
    }
}

/// Devolve a janela de um recorte (`linhas`); vazio = a grade não tinha o campo ⇒ a janela volta a NEUTRA.
pub fn restaura_janela(g: &mut Grid, janela: (i32, i32, i32, i32), linhas: &[AparenciaCelula]) {
    let (n, s) = (g.cells, g.s);
    g.suave_teto = linhas.iter().fold(g.suave_teto, |t, a| t.max(a.suave));
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
