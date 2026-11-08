//! **O dente do papel NA tinta do Óleo Real** (W3b, `docs/Painter/49_realshader/oraculo/README.md`
//! §Transmission): o Relevo do papel (a tinta segue o grão) e a Transmission (o papel visto pelo filme) mostram o
//! dente onde a tinta OPACA o enterra (§BUG-papel). A tinta do Wet Paint guarda-se em píxeis (a grade morre com a
//! sessão), por isso o grão ASSA-SE no píxel da tinta aqui, pela MESMA óptica do passe do relevo
//! ([`super::super::impasto_shade::Rig::shade`] com o dente do substrato) — o grão sob o filme casa com o do papel
//! nu ao lado, e a luz do relevo, que lê o alfa da tinta, não o soma outra vez.

use super::super::impasto_shade::{MatShade, Rig};
use super::super::substrate_relief::Substrate;

/// U2 (auditoria W3b): o PISO da profundidade do papel NA tinta do Óleo Real — o Relief do Cold com que o Relevo do
/// papel e o grão do vidro foram calibrados (`oraculo/README.md` §Transmission). Com o Relief do papel a 0 (a fábrica)
/// o Paper Relief e a Transmission mostram a textura do papel a esta profundidade; acima dela, a do papel.
pub(in crate::tool::paint) const RELIEF_NA_TINTA_MIN: f32 = 0.6;

/// A profundidade do papel NA tinta: de [`RELIEF_NA_TINTA_MIN`] no Relief 0 a `1` no Relief 1, sempre a crescer com
/// ele (dono 08/11: «independente da intensidade do relevo, o efeito sobre a tinta é o mesmo» — o piso `max` de antes
/// achatava todo Relief abaixo de 0,6). As pontas são as de antes.
///
/// ⚠️ EM PROVA (dono 08/11, «melhor testar os dois modos»): `PH2D_PAPEL_NA_TINTA=b` troca pela [`relief_na_tinta_b`].
#[must_use]
pub(in crate::tool::paint) fn relief_na_tinta(relief: f32) -> f32 {
    static B: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if *B.get_or_init(|| std::env::var("PH2D_PAPEL_NA_TINTA").is_ok_and(|v| v == "b")) {
        relief_na_tinta_b(relief)
    } else {
        relief_na_tinta_rampa(relief)
    }
}

/// (a) A rampa de 01fb209238: [`RELIEF_NA_TINTA_MIN`] no Relief 0, `1` no 1. A Relief 0,6 dá 0,84 — o Relevo do papel
/// do Óleo Real mostra o dente 2,24 × o papel nu, contra 1,71 do Cycles (`a_transmissao_e_o_relevo_do_papel_mostram_o_grao`).
#[must_use]
pub(in crate::tool::paint) fn relief_na_tinta_rampa(relief: f32) -> f32 {
    RELIEF_NA_TINTA_MIN + (1.0 - RELIEF_NA_TINTA_MIN) * relief.clamp(0.0, 1.0)
}

/// (b) O papel na tinta É o do papel de [`RELIEF_NA_TINTA_MIN`] para cima (a lei calibrada contra o Cycles, igual à de
/// antes de 01fb209238); abaixo continua a crescer, da metade do piso no Relief 0 até ao piso.
#[must_use]
pub(in crate::tool::paint) fn relief_na_tinta_b(relief: f32) -> f32 {
    let r = relief.clamp(0.0, 1.0);
    if r >= RELIEF_NA_TINTA_MIN {
        r
    } else {
        0.5 * RELIEF_NA_TINTA_MIN * (1.0 + r / RELIEF_NA_TINTA_MIN)
    }
}

/// As duas leis em prova sobem sempre com o Relief e acabam juntas no 1; a (b) passa pelo piso nele.
#[test]
fn as_duas_leis_do_papel_na_tinta_sobem_com_o_relief() {
    for lei in [relief_na_tinta_rampa, relief_na_tinta_b] {
        let v: Vec<f32> = (0..=10).map(|k| lei(k as f32 / 10.0)).collect();
        assert!(v.windows(2).all(|w| w[1] > w[0]), "{v:?}");
        assert!((v[10] - 1.0).abs() < 1e-6 && v[0] > 0.0, "{v:?}");
    }
    assert!((relief_na_tinta_b(RELIEF_NA_TINTA_MIN) - RELIEF_NA_TINTA_MIN).abs() < 1e-6);
    assert!((relief_na_tinta_b(0.0) - 0.3).abs() < 1e-6);
}

/// O papel que o filme transmissivo mostra: o do documento (a textura escolhida) ou, sem papel escolhido, o de
/// fábrica (o Cold que o Relief arma), à profundidade [`relief_na_tinta`].
pub(super) fn papel_do_filme<'a>(
    tex: ph2d_painter_brush::texture::TextureSettings,
    relief: f32,
    rough: f32,
    image: Option<ph2d_painter_brush::texture::ImageMask<'a>>,
) -> Option<Substrate<'a>> {
    let tex = super::super::substrate_relief::papel_do_documento(tex);
    Substrate::resolve(tex, relief_na_tinta(relief), rough, image)
}

/// (Testes) Desliga o grão assado — a régua do grão que o filme mostra (`material_do_pincel_tests`).
#[cfg(test)]
pub(in crate::tool::paint) mod sem_grao {
    thread_local!(static S: std::cell::Cell<bool> = const { std::cell::Cell::new(false) });
    pub(in crate::tool::paint) fn get() -> bool {
        S.with(std::cell::Cell::get)
    }
    pub(in crate::tool::paint) fn set(v: bool) {
        S.with(|s| s.set(v));
    }
}

/// O papel e a luz do passe do relevo, resolvidos uma vez por composite.
pub(super) struct GraoNoFilme<'a> {
    papel: Substrate<'a>,
    luz: Rig,
    mat: MatShade,
    w: i64,
    h: i64,
}

impl<'a> GraoNoFilme<'a> {
    /// `None` sem papel ou sem lâmpada acesa: não há grão que se veja em lado nenhum.
    pub(super) fn de(
        papel: Option<Substrate<'a>>,
        rig: &super::super::impasto_rig::LightRig,
        (w, h): (usize, usize),
    ) -> Option<Self> {
        #[cfg(test)]
        if sem_grao::get() {
            return None;
        }
        let papel = papel?;
        let luz = Rig::new(rig)?;
        let mat = luz.resolve(ph2d_painter_brush::material::Material::NEUTRAL.to_bytes());
        Some(Self {
            papel,
            luz,
            mat,
            w: w as i64,
            h: h as i64,
        })
    }

    /// [`Self::de`] sobre o [`papel_do_filme`] do documento: `(relief, rough)` do substrato.
    pub(super) fn do_papel(
        tex: ph2d_painter_brush::texture::TextureSettings,
        (relief, rough): (f32, f32),
        image: Option<ph2d_painter_brush::texture::ImageMask<'a>>,
        rig: &super::super::impasto_rig::LightRig,
        tela: (usize, usize),
    ) -> Option<Self> {
        Self::de(papel_do_filme(tex, relief, rough, image), rig, tela)
    }

    /// A cor RECTA `cor` (`0..1`) do píxel `(x, y)` com o dente do papel × `d` (`0..1`, a fracção que a tinta
    /// mostra, [`ph2d_wet_paint::oleo_real::grao_do_papel`]): a luz do relevo sobre o papel nu, com a inclinação
    /// do dente escalada por `d`. `d = 0` não chega aqui (quem chama salta: ao bit).
    pub(super) fn sombreia(&self, x: usize, y: usize, d: f32, cor: [f32; 3]) -> [f32; 3] {
        let at = |x: i64, y: i64| {
            self.papel
                .tooth_loads(x.clamp(0, self.w - 1), y.clamp(0, self.h - 1))
        };
        let (x, y) = (x as i64, y as i64);
        let dhx = (at(x + 1, y) - at(x - 1, y)) * 0.5 * d;
        let dhy = (at(x, y + 1) - at(x, y - 1)) * 0.5 * d;
        let (mul, add) = self.luz.shade(&self.mat, 1.0, 0.0, dhx, dhy, cor);
        [0, 1, 2].map(|c| super::super::impasto_light::light_pixel(cor[c], mul[c], add[c]))
    }
}

/// W3b: o que a tinta do Óleo Real lê FORA do cartão e do motor — o papel (o dente e a cor) e a lâmpada do relevo.
/// Mudou desde o último quadro da sessão viva ⇒ a tinta pousada recompõe-se inteira (a varredura do cartão).
#[derive(Clone, Copy, PartialEq)]
pub(in crate::tool::paint) struct ChaveDoGrao {
    substrato: super::super::substrate_relief::SubstrateKey,
    luz: super::super::impasto_rig::LightRig,
    papel: Option<[u8; 3]>,
}

impl ChaveDoGrao {
    pub(super) fn de(t: &crate::tool::PainterTool) -> Self {
        Self {
            substrato: t.substrate_key(),
            luz: t.paint.impasto_rig,
            papel: t.papel(),
        }
    }
}

impl crate::tool::PainterTool {
    /// A porta do [`Self::reconcile_substrate`] (a cada quadro): com o Óleo Real ligado e a sessão viva, o papel ou a
    /// lâmpada que mudaram desde o último quadro recompõem a tela inteira pela varredura do cartão
    /// ([`Self::wet_recompoe_no_quadro`], o orçamento por quadro). A tinta de uma sessão que já acabou é píxel.
    pub(in crate::tool::paint) fn reconcilia_o_grao_do_oleo(&mut self) {
        if !self.paint.wetpaint.oleo_na_tela {
            return;
        }
        let agora = ChaveDoGrao::de(self);
        let Some(s) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        if s.chave_do_grao.replace(agora).is_some_and(|c| c != agora) {
            self.wet_recompoe_no_quadro();
        }
    }
}
