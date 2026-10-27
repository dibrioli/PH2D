//! **O material no sombreado do Óleo Real** (W2 E3-B, filho de [`super`]): os lóbulos do Principled do
//! Cycles sobre a tinta — metal, rugosidade, specular, tint do reflexo, verniz, veludo, anisotropia e a
//! translucidez da película fina —, a partir do [`MaterialCelula`] da célula combinado com o ajuste GLOBAL
//! do cartão Real Oil ([`MaterialCartao`]). Medido contra o Cycles em `docs/Painter/49_realshader/oraculo/`
//! (README §E3). Com a célula e o cartão no neutro o sombreado é o de antes, bit a bit.

use super::{Ambiente, CampoAltura};
use crate::material::MaterialCelula;

/// O material GLOBAL do cartão Real Oil. Regra de combinação com o da célula ([`efectivo`]): os lóbulos de
/// neutro `0` (metal, tint, verniz, veludo, anisotropia, translucidez) tomam o MAIOR dos dois — o cartão
/// pode pôr verniz em toda a tinta, a célula metálica continua metálica; os de neutro ≠ 0 (rugosidade,
/// specular, rugosidade do verniz) MULTIPLICAM-se (`1` = o da célula). Os quatro do «Metallic Materials» do
/// Rebelle são só do cartão.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCartao {
    /// Metallic global (`0..1`, max com a célula).
    pub metal: f64,
    /// Multiplicador da rugosidade (`× a da célula × a que o Gloss dá`).
    pub rugosidade: f64,
    /// Multiplicador do Specular da célula (`1` = o dela; o Principled `0,5` = IOR 1,5).
    pub especular: f64,
    /// Tint do reflexo dieléctrico pela cor da tinta (`0..1`, max).
    pub tinta_reflexo: f64,
    /// Verniz (clearcoat, `0..1`, max).
    pub verniz: f64,
    /// Multiplicador da rugosidade do verniz da célula.
    pub verniz_rugosidade: f64,
    /// Sheen (`0..1`, max).
    pub veludo: f64,
    /// Anisotropia ao longo da estria (`0..1`, max).
    pub anisotropia: f64,
    /// Translucidez da película fina (`0..1`, max).
    pub translucidez: f64,
    /// **Reflectivity** (Rebelle): multiplicador do F0 do metal (`F0 = cor × isto`, preso a 1).
    pub refletividade: f64,
    /// **Reflection Map** (Rebelle): o ambiente que o REFLEXO usa (`None` = o Environment). Com o Clássico
    /// e sem mapa, o metal e o verniz reflectem o céu uniforme.
    pub mapa_reflexo: Option<Ambiente>,
    /// **Scale** (Rebelle): a escala do mapa de reflexo — a direcção reflectida inclina-se `× isto` (maior =
    /// o mapa varia mais depressa sobre o relevo).
    pub escala_mapa: f64,
    /// **Strength** (Rebelle): multiplicador do reflexo do AMBIENTE (todos os lóbulos).
    pub forca_reflexo: f64,
}

impl MaterialCartao {
    pub const NEUTRO: Self = Self {
        metal: 0.0,
        rugosidade: 1.0,
        especular: 1.0,
        tinta_reflexo: 0.0,
        verniz: 0.0,
        verniz_rugosidade: 1.0,
        veludo: 0.0,
        anisotropia: 0.0,
        translucidez: 0.0,
        refletividade: 1.0,
        mapa_reflexo: None,
        escala_mapa: 1.0,
        forca_reflexo: 1.0,
    };
}

impl MaterialCartao {
    /// Os 12 ajustes NUMÉRICOS do cartão, pela ordem da UI (`PAINTER_WETPAINT_OLEO_FIELDS[8..]`) e com os
    /// nomes da macro: primeiro os do «Metallic Materials» do Rebelle, depois os lóbulos do Principled.
    pub const CAMPOS: [&'static str; 12] = [
        "metal",
        "refletividade",
        "escala_mapa",
        "forca_reflexo",
        "rugosidade",
        "especular",
        "tinta_reflexo",
        "verniz",
        "verniz_rugosidade",
        "veludo",
        "anisotropia",
        "translucidez",
    ];

    /// Os valores de [`Self::CAMPOS`], pela mesma ordem.
    #[must_use]
    pub const fn valores(&self) -> [f64; 12] {
        [
            self.metal,
            self.refletividade,
            self.escala_mapa,
            self.forca_reflexo,
            self.rugosidade,
            self.especular,
            self.tinta_reflexo,
            self.verniz,
            self.verniz_rugosidade,
            self.veludo,
            self.anisotropia,
            self.translucidez,
        ]
    }

    /// Põe o campo `k` de [`Self::CAMPOS`] (fora da faixa = sem efeito).
    pub fn poe(&mut self, k: usize, v: f64) {
        let alvo = match k {
            0 => &mut self.metal,
            1 => &mut self.refletividade,
            2 => &mut self.escala_mapa,
            3 => &mut self.forca_reflexo,
            4 => &mut self.rugosidade,
            5 => &mut self.especular,
            6 => &mut self.tinta_reflexo,
            7 => &mut self.verniz,
            8 => &mut self.verniz_rugosidade,
            9 => &mut self.veludo,
            10 => &mut self.anisotropia,
            11 => &mut self.translucidez,
            _ => return,
        };
        *alvo = v;
    }

    /// O wire do **Reflection Map**: `0` = o Environment, senão o índice em `Ambiente::TODOS` (o Clássico não
    /// tem mapa, por isso o `0` é dele).
    #[must_use]
    pub fn mapa_indice(&self) -> u8 {
        self.mapa_reflexo.map_or(0, Ambiente::indice)
    }

    /// O inverso de [`Self::mapa_indice`] (`None` = fora da faixa).
    #[must_use]
    pub fn mapa_de_indice(i: u8) -> Option<Option<Ambiente>> {
        match i {
            0 => Some(None),
            _ => Ambiente::de_indice(i).map(Some),
        }
    }
}

impl Default for MaterialCartao {
    fn default() -> Self {
        Self::NEUTRO
    }
}

/// O material efectivo de uma célula (célula ∘ cartão), em `f64`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Efectivo {
    pub metal: f64,
    pub rugosidade: f64,
    pub especular: f64,
    pub tinta_reflexo: f64,
    pub verniz: f64,
    pub verniz_rugosidade: f64,
    pub veludo: f64,
    pub anisotropia: f64,
    pub translucidez: f64,
}

/// A regra de combinação de [`MaterialCartao`].
pub(super) fn efectivo(c: &MaterialCelula, k: &MaterialCartao) -> Efectivo {
    let mx = |a: f32, b: f64| f64::from(a).max(b).clamp(0.0, 1.0);
    Efectivo {
        metal: mx(c.metal, k.metal),
        rugosidade: f64::from(c.rugosidade) * k.rugosidade,
        especular: f64::from(c.especular) * k.especular,
        tinta_reflexo: mx(c.tinta_reflexo, k.tinta_reflexo),
        verniz: mx(c.verniz, k.verniz),
        verniz_rugosidade: f64::from(c.verniz_rugosidade) * k.verniz_rugosidade,
        veludo: mx(c.veludo, k.veludo),
        anisotropia: mx(c.anisotropia, k.anisotropia),
        translucidez: mx(c.translucidez, k.translucidez),
    }
}

/// A espessura (células de relevo) em que a película deixa passar `1/e` da luz até ao papel.
pub const ESPESSURA_TRANSLUCIDA: f64 = 1.0;

/// A fracção de luz que atravessa a película na célula `(x, y)` e volta do papel: `exp(−h/h₀)` pela altura
/// fina (a espessura da tinta em células), `1` no papel nu.
pub fn passagem(campo: &CampoAltura, x: isize, y: isize) -> f64 {
    libm::exp(-campo.at_fina(x, y).max(0.0) / ESPESSURA_TRANSLUCIDA)
}

/// A direcção da ESTRIA na célula `(x, y)` (unitária no plano da tela, x direita, y baixo): a
/// perpendicular ao eixo dominante do tensor de estrutura do gradiente da altura fina numa janela 3×3 — o
/// relevo sobe e desce ATRAVÉS da estria. `None` onde o relevo é plano.
pub fn estria(campo: &CampoAltura, x: isize, y: isize) -> Option<[f64; 2]> {
    let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let h = |a: isize, b: isize| campo.at_fina(x + dx + a, y + dy + b);
            let gx = (h(1, 0) - h(-1, 0)) / 2.0;
            let gy = (h(0, 1) - h(0, -1)) / 2.0;
            sxx += gx * gx;
            syy += gy * gy;
            sxy += gx * gy;
        }
    }
    if sxx + syy < 1e-12 {
        return None;
    }
    // O eixo do gradiente dominante e a estria perpendicular a ele.
    let th = 0.5 * libm::atan2(2.0 * sxy, sxx - syy);
    Some([-libm::sin(th), libm::cos(th)])
}

/// A base tangente da superfície `(t, b)` pela estria `e` projectada no plano da normal `n`.
pub(super) fn base_tangente(n: [f64; 3], e: [f64; 2]) -> ([f64; 3], [f64; 3]) {
    let d = e[0] * n[0] + e[1] * n[1];
    let t = [e[0] - d * n[0], e[1] - d * n[1], -d * n[2]];
    let l = libm::sqrt(t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).max(1e-12);
    let t = [t[0] / l, t[1] / l, t[2] / l];
    let b = [
        n[1] * t[2] - n[2] * t[1],
        n[2] * t[0] - n[0] * t[2],
        n[0] * t[1] - n[1] * t[0],
    ];
    (t, b)
}

/// GGX ANISOTRÓPICO (a convenção do Principled: `aspecto = √(1 − 0,9·an)`, `αt = α/aspecto`,
/// `αb = α·aspecto`; o lobo alonga-se AO LONGO da tangente).
pub(super) fn ggx_aniso(a2: f64, an: f64, h: [f64; 3], n: [f64; 3], t: [f64; 3], b: [f64; 3]) -> f64 {
    let alfa = libm::sqrt(a2);
    let asp = libm::sqrt(1.0 - 0.9 * an.clamp(0.0, 1.0));
    let (at, ab) = (alfa / asp, alfa * asp);
    let dot = |u: [f64; 3], v: [f64; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let (ht, hb, hn) = (dot(h, t) / at, dot(h, b) / ab, dot(h, n));
    let d = ht * ht + hb * hb + hn * hn;
    1.0 / (core::f64::consts::PI * at * ab * d * d)
}

/// A normal DOBRADA do reflexo anisotrópico (o pré-filtrado é isotrópico): a normal inclina-se para a do
/// plano que contém a estria e a vista, `|an|·sat(5·rug)` (Filament, «bent normal»).
pub(super) fn normal_dobrada(n: [f64; 3], t: [f64; 3], an: f64, rug: f64) -> [f64; 3] {
    // v = (0,0,1): a direcção `t` (a BITANGENTE quando o lobo se alonga ao longo da estria) × v, e a normal
    // do plano dela com a vista.
    let at = [t[1], -t[0], 0.0];
    let an_n = [
        at[1] * t[2] - at[2] * t[1],
        at[2] * t[0] - at[0] * t[2],
        at[0] * t[1] - at[1] * t[0],
    ];
    let k = an.clamp(0.0, 1.0) * (5.0 * rug).clamp(0.0, 1.0);
    let m = [0, 1, 2].map(|i| n[i] + (an_n[i] - n[i]) * k);
    let l = libm::sqrt(m[0] * m[0] + m[1] * m[1] + m[2] * m[2]).max(1e-12);
    let m = [m[0] / l, m[1] / l, m[2] / l];
    if m[2] < 1e-3 { n } else { m }
}

/// O VELUDO (Sheen) relativo, à luz branca de difuso `e` e normal de cosseno com a vista `nz`: `s·(k₀ +
/// k₁·(1−nz)^p)·e`, as constantes ajustadas ao Sheen do Principled (rugosidade 0,5) na cena do dono
/// (`oraculo/README.md` §E3: sheen/e 0,031 de frente → 0,108 a `1−nz ≈ 0,6`).
pub(super) fn veludo(s: f64, nz: f64, e: f64) -> f64 {
    s * (VELUDO_K0 + VELUDO_K1 * libm::pow((1.0 - nz).clamp(0.0, 1.0), VELUDO_P)) * e
}
pub(super) const VELUDO_K0: f64 = 0.031;
pub(super) const VELUDO_K1: f64 = 0.20;
pub(super) const VELUDO_P: f64 = 1.83;

/// O que volta do papel pela película, relativo ao difuso do branco: `G·a^k` (a ida e a volta tingem e o
/// Fresnel das duas faces leva a sua parte) — ajustado ao Transmission Thin Wall do Principled sobre papel
/// branco (`oraculo/README.md` §E3: a razão volta/difuso 0,45 no vermelho × 0,31 no verde fino).
pub(super) const TRANSLUCIDA_K: f64 = 1.2;
pub(super) const TRANSLUCIDA_G: f64 = 0.47;

#[cfg(test)]
#[path = "oleo_material_tests.rs"]
mod oleo_material_tests;
