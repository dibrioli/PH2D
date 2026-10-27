//! **O material no sombreado do Óleo Real** (filho de [`super`]): o Principled SIMPLIFICADO do Cycles sobre a
//! tinta (W3: Metallic · Roughness · IOR Level · Coat · Coat Roughness · Sheen · Subsurface · Subsurface Radius),
//! a partir do [`MaterialCelula`] da célula combinado com o ajuste GLOBAL do cartão Real Oil ([`MaterialCartao`],
//! regra em [`crate::material::efectivo`]). Medido contra o Cycles em `docs/Painter/49_realshader/oraculo/`
//! (README §E3-B). Com a célula NEUTRA e o cartão no padrão o sombreado é o de antes, bit a bit. O reflexo usa o
//! Environment (o Principled não tem mapa de reflexo à parte).

use crate::material::MaterialCelula;

/// A Roughness do cartão no PADRÃO: a `Afinacao::rugosidade` de antes da W3 (0,28, a régua M4) — a imagem de antes.
pub const RUGOSIDADE_PADRAO: f64 = 0.28;

/// O material GLOBAL do cartão Real Oil, os 8 controlos do Blender em valores ABSOLUTOS. [`Self::NEUTRO`] é o
/// padrão do cartão (= a imagem de antes). Conversão do cartão de antes da W3 (multiplicadores): Roughness =
/// `0,28 × m`, IOR Level = `0,5 × s`, Coat Roughness = `0,03 × r`; Translucency → Subsurface
/// ([`Self::poe_campo_antigo`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaterialCartao {
    /// Metallic (`0..1`, max com a célula).
    pub metal: f64,
    /// Roughness absoluta (`0..1`).
    pub rugosidade: f64,
    /// IOR Level (`0,5` = IOR 1,5).
    pub nivel_ior: f64,
    /// Coat (`0..1`, max).
    pub verniz: f64,
    /// Coat Roughness absoluta.
    pub verniz_rugosidade: f64,
    /// Sheen (`0..1`, max).
    pub veludo: f64,
    /// Subsurface (`0..1`, max).
    pub subsuperficie: f64,
    /// Subsurface Radius, px de tela.
    pub raio_subsuperficie: f64,
}

impl MaterialCartao {
    pub const NEUTRO: Self = Self {
        metal: 0.0,
        rugosidade: RUGOSIDADE_PADRAO,
        nivel_ior: 0.5,
        verniz: 0.0,
        // O neutro da célula em `f64` (a razão padrão/neutro é `1` exacto: a célula passa ao bit).
        verniz_rugosidade: MaterialCelula::NEUTRO.verniz_rugosidade as f64,
        veludo: 0.0,
        subsuperficie: 0.0,
        raio_subsuperficie: crate::material::RAIO_SUBSUPERFICIE_PADRAO as f64,
    };

    /// Os 8 campos, pela ordem da UI (`PAINTER_WETPAINT_OLEO_FIELDS[8..]`) e com os nomes da macro.
    pub const CAMPOS: [&'static str; 8] = [
        "metal",
        "rugosidade",
        "nivel_ior",
        "verniz",
        "verniz_rugosidade",
        "veludo",
        "subsuperficie",
        "raio_subsuperficie",
    ];

    /// Os valores de [`Self::CAMPOS`], pela mesma ordem.
    #[must_use]
    pub const fn valores(&self) -> [f64; 8] {
        [
            self.metal,
            self.rugosidade,
            self.nivel_ior,
            self.verniz,
            self.verniz_rugosidade,
            self.veludo,
            self.subsuperficie,
            self.raio_subsuperficie,
        ]
    }

    /// Põe o campo `k` de [`Self::CAMPOS`] (fora da faixa = sem efeito).
    pub fn poe(&mut self, k: usize, v: f64) {
        let alvo = match k {
            0 => &mut self.metal,
            1 => &mut self.rugosidade,
            2 => &mut self.nivel_ior,
            3 => &mut self.verniz,
            4 => &mut self.verniz_rugosidade,
            5 => &mut self.veludo,
            6 => &mut self.subsuperficie,
            7 => &mut self.raio_subsuperficie,
            _ => return,
        };
        *alvo = v;
    }

    /// O LEITOR TOLERANTE do cartão de antes da W3 (multiplicadores, nomes da macro sem o `mat_`): converte
    /// `rugosidade`/`especular`/`verniz_rugosidade` (× o padrão) e `translucidez` (→ Subsurface); os outros
    /// campos do cartão de então (metal, verniz, veludo) têm o mesmo sentido; os que saíram são ignorados.
    pub fn poe_campo_antigo(&mut self, nome: &str, v: f64) {
        match nome {
            "metal" => self.metal = v,
            "rugosidade" => self.rugosidade = RUGOSIDADE_PADRAO * v,
            "especular" => self.nivel_ior = Self::NEUTRO.nivel_ior * v,
            "verniz" => self.verniz = v,
            "verniz_rugosidade" => self.verniz_rugosidade = Self::NEUTRO.verniz_rugosidade * v,
            "veludo" => self.veludo = v,
            "translucidez" => self.subsuperficie = v,
            _ => {}
        }
    }
}

impl Default for MaterialCartao {
    fn default() -> Self {
        Self::NEUTRO
    }
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

#[cfg(test)]
#[path = "oleo_material_tests.rs"]
mod oleo_material_tests;
