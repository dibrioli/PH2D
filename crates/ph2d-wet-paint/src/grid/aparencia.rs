//! O campo PREGUIÇOSO [`Grid::aparencia`] (W3b item 3, dono 27/10: «se o usuário muda as configurações no painel,
//! não afeta a tinta já colocada no papel»): os controlos do PINCEL que o ecrã lê ao COMPOR, guardados na célula no
//! pouso — o render e as transferências verticais lêem o da célula, nunca o pincel de agora. VAZIO (=
//! [`AparenciaCelula::NEUTRA`], os valores de fábrica) até o primeiro pincel fora deles pintar: quem não mexe neles
//! não paga nada. Mistura-se pela massa como o [`Grid::relevo`]; como ele, só o pouso do rasto o escreve (o arrasto,
//! o Smear e o Blend não o movem). Doc: `docs/Painter/49_realshader/W3b_painel_so_no_futuro.md`.

use super::Grid;
use super::campo_esparso::CampoEsparso;
use crate::sim::Params;
use crate::tuning::Knob;

/// A aparência da tinta de uma célula (28 B).
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
    /// O **Edge Darkening** da tinta (dono 05/11, Bug #69: é do TRAÇO): a borda desta célula seca por ele, e trocar de
    /// pincel não a endurece. [`BORDA_DO_FUNDO`] = o da grade ([`Grid::borda_fundo`]).
    pub borda: f32,
}

/// O campo da grade (B12.3): pedaços esparsos, o ausente lê [`AparenciaCelula::NEUTRA`].
pub type CampoAparencia = CampoEsparso<AparenciaCelula>;

/// O campo que ainda não existe (o `Vec::new()` de antes).
#[must_use]
pub fn sem_campo() -> CampoAparencia {
    CampoEsparso::vazio(AparenciaCelula::NEUTRA)
}

/// O denso `v` como campo (os testes, o upload inverso).
#[must_use]
pub fn campo_de(v: &[AparenciaCelula]) -> CampoAparencia {
    CampoEsparso::de_vec(AparenciaCelula::NEUTRA, v, mesmo_bit)
}

/// Igualdade ao bit (o `==` de `f32` iguala `-0.0` e `0.0`).
#[must_use]
pub fn mesmo_bit(a: &AparenciaCelula, b: &AparenciaCelula) -> bool {
    let f = |c: &AparenciaCelula| {
        [
            c.papel, c.grao, c.vidrado, c.textura, c.opaca, c.suave, c.borda,
        ]
        .map(f32::to_bits)
    };
    f(a) == f(b)
}

/// A célula sem borda própria lê a da grade: a tela de um pincel só não aloca o campo por causa disto.
pub const BORDA_DO_FUNDO: f32 = -1.0;

impl AparenciaCelula {
    /// Os valores de fábrica (os do registo e os do produto).
    pub const NEUTRA: Self = Self {
        papel: 1.0,
        grao: 1.0,
        vidrado: 0.0,
        textura: 0.0,
        opaca: 0.0,
        suave: 0.0,
        borda: BORDA_DO_FUNDO,
    };
}

impl Default for AparenciaCelula {
    fn default() -> Self {
        Self::NEUTRA
    }
}

/// A aparência que o pincel pousa agora. O primeiro pincel a tocar a grade dá-lhe o fundo da borda, e esse valor não muda
/// mais: o histórico e o Clear podem devolver células que o lêem.
#[must_use]
pub fn do_pincel(g: &mut Grid, p: &Params) -> AparenciaCelula {
    let borda = p.k(Knob::EdgeDarkening);
    if g.borda_fundo < 0.0 {
        g.borda_fundo = borda;
    }
    AparenciaCelula {
        papel: p.k(Knob::PaperVisibility) as f32,
        grao: p.k(Knob::VisualGrain) as f32,
        vidrado: p.vidrado,
        textura: p.textura,
        opaca: p.k(Knob::Opaque) as f32,
        suave: p.suave,
        borda: if borda == g.borda_fundo {
            BORDA_DO_FUNDO
        } else {
            borda as f32
        },
    }
}

/// A aparência da célula `i` — [`AparenciaCelula::NEUTRA`] sem o campo.
#[inline]
#[must_use]
pub fn da_celula(campo: &CampoAparencia, i: usize) -> AparenciaCelula {
    campo.get(i).unwrap_or(AparenciaCelula::NEUTRA)
}

/// O vidrado da célula `i` (a lei do ecrã de cada célula, [`crate::assenta::Ecra::com_vidrado`]).
#[inline]
#[must_use]
pub fn vidrado(campo: &CampoAparencia, i: usize) -> f32 {
    campo.get(i).map_or(0.0, |a| a.vidrado)
}

/// A opacidade da célula `i` ([`crate::assenta::Ecra::com_opaca`]).
#[inline]
#[must_use]
pub fn opaca(campo: &CampoAparencia, i: usize) -> f32 {
    campo.get(i).map_or(0.0, |a| a.opaca)
}

/// O Edge Darkening próprio da célula `i`, ou [`BORDA_DO_FUNDO`].
#[inline]
#[must_use]
pub fn borda(campo: &CampoAparencia, i: usize) -> f32 {
    campo.get(i).map_or(BORDA_DO_FUNDO, |a| a.borda)
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

/// Os bytes que o campo ocupa (o gate do 1.º toque, B12.3).
#[must_use]
pub fn bytes_alocados(g: &Grid) -> usize {
    g.aparencia.bytes_vivos()
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
        g.aparencia = CampoEsparso::novo(AparenciaCelula::NEUTRA, g.cells);
    }
    g.suave_teto = g.suave_teto.max(nova.suave);
    let fundo = g.borda_fundo as f32;
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
                borda: if a.borda < 0.0 && nova.borda < 0.0 {
                    BORDA_DO_FUNDO
                } else {
                    let de = |b: f32| if b < 0.0 { fundo } else { b };
                    m(de(a.borda), de(nova.borda))
                },
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
    let (x0, y0, x1, y1) = janela;
    if linhas.is_empty() && g.aparencia.is_empty() {
        return;
    }
    if g.aparencia.is_empty() {
        g.aparencia = CampoEsparso::novo(AparenciaCelula::NEUTRA, n);
    }
    let w = (x1 - x0 + 1) as usize;
    let neutra = vec![AparenciaCelula::NEUTRA; w];
    for (r, y) in (y0..=y1).enumerate() {
        let o = y as usize * s + x0 as usize;
        let l = linhas.get(r * w..(r + 1) * w).unwrap_or(&neutra);
        g.aparencia.escreve_intervalo_poupando(o, l, mesmo_bit);
    }
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
    // O rasto só de água não pousa: sem isto, a tinta que chegasse depois lia um degrau acima do tecto (Bug #65).
    g.suave_teto = g.suave_teto.max(nova.suave);
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

#[cfg(test)]
mod veste_tests {
    use super::*;

    /// O tecto cobre o que a orla virgem veste: um rasto sem pouso não deixa um degrau acima dele.
    #[test]
    fn vestir_a_orla_virgem_sobe_o_tecto_do_suave() {
        let mut g = Grid::new(16, 16);
        g.aparencia = CampoEsparso::novo(AparenciaCelula::NEUTRA, g.cells);
        let nova = AparenciaCelula {
            suave: 0.5,
            ..AparenciaCelula::NEUTRA
        };
        veste_virgens(&mut g, (4, 4, 6, 6), nova);
        assert_eq!(g.suave_teto, 0.5);
    }

    fn params_com_borda(ed: f64) -> Params {
        let mut e = crate::painter::Engine::with_flow_ratio(8, 8, 1);
        e.tuning.set(Knob::EdgeDarkening, ed);
        e.sim.gather_params(&e.tuning)
    }

    /// B12.3: o 1.º dab fora da fábrica (raio 30) aloca os pedaços das linhas dele, não a tela (era `plano_cheio`).
    #[test]
    fn o_primeiro_dab_nao_neutro_aloca_a_aparencia_das_linhas_dele() {
        // A largura da tela 4K (a linha cabe num pedaço); a altura só tem de passar o dab.
        let mut g = Grid::new(3840, 600);
        let nova = AparenciaCelula {
            vidrado: 1.0,
            ..AparenciaCelula::NEUTRA
        };
        let (cx, cy, r) = (1900i32, 300i32, 30i32);
        for y in cy - r..=cy + r {
            for x in cx - r..=cx + r {
                if (x - cx).pow(2) + (y - cy).pow(2) <= r * r {
                    let i = x as usize + y as usize * g.s;
                    pousa(&mut g, i, 0.0, nova, 1.0);
                }
            }
        }
        let denso = g.cells * std::mem::size_of::<AparenciaCelula>();
        let vivos = bytes_alocados(&g);
        let linhas = ((2 * r + 1) as usize * g.s).div_ceil(super::super::campo_esparso::PEDACO) + 1;
        let teto =
            linhas * super::super::campo_esparso::PEDACO * std::mem::size_of::<AparenciaCelula>();
        assert!(
            vivos > 0 && vivos <= teto && vivos * 8 < denso,
            "{vivos} B: as linhas do dab cabem em {teto} B, o denso {denso} B"
        );
        assert_eq!(vidrado(&g.aparencia, cx as usize + cy as usize * g.s), 1.0);
        assert_eq!(vidrado(&g.aparencia, 3 + 3 * g.s), 0.0);
    }

    /// Bug #69: o fundo é o do 1.º pincel, e o 2.º guarda o dele na célula: a tinta de A não passa a secar pelo de B.
    #[test]
    fn o_segundo_pincel_nao_muda_o_fundo_da_borda() {
        let mut g = Grid::new(16, 16);
        let a = do_pincel(&mut g, &params_com_borda(12.0));
        assert_eq!((a.borda, g.borda_fundo), (BORDA_DO_FUNDO, 12.0));
        let b = do_pincel(&mut g, &params_com_borda(40.0));
        assert_eq!((b.borda, g.borda_fundo), (40.0, 12.0));
    }

    /// A célula com tinta mistura a borda pela massa, e o lado sem borda própria entra pelo fundo.
    #[test]
    fn a_borda_mistura_pela_massa_em_celula_com_tinta() {
        let mut g = Grid::new(16, 16);
        g.borda_fundo = 10.0;
        g.aparencia = CampoEsparso::novo(AparenciaCelula::NEUTRA, g.cells);
        let com = |borda| AparenciaCelula {
            borda,
            ..AparenciaCelula::NEUTRA
        };
        // Do fundo (10) com massa 3 para 30 com massa 1: (3·10 + 30) / 4.
        pousa(&mut g, 7, 3.0, com(30.0), 1.0);
        assert_eq!(g.aparencia[7].borda, 15.0);
        // Dois valores próprios, massas iguais.
        pousa(&mut g, 7, 1.0, com(25.0), 1.0);
        assert_eq!(g.aparencia[7].borda, 20.0);
    }
}
