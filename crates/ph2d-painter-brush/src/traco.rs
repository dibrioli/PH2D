//! **O traço como uma CAMADA** (L8, `docs/Painter/Pinceis/03_MOTORES.md` §4 e `06_LEIS_DAS_LACUNAS.md` §6): o
//! tecto de opacidade POR TRAÇO e as orlas molhada e queimada — o *light/uniform glaze*, o *Wet edges* e o
//! *Burnt edges* do Procreate (e do Photoshop).
//!
//! ## A lei medida
//!
//! As aguadas e os glazes do Rebelle TAPAM a opacidade dentro de UM traço: na espiral da bateria os braços
//! sobrepõem-se e `k = ln T(sobreposição) ÷ ln T(recto)` fica em **1,00–1,15** (Watercolor Round 2 1,09, Wash
//! Round 1,11, os 7 Glaze); os aerógrafos e o Marker Dry ACUMULAM (`k` 1,8–4,5). O tecto do Accumulate
//! desligado (`stroke_cover`) já tapa a opacidade num pincel de dabs que convergem; o que faltava é o tecto de
//! um pincel que ACUMULA por dentro (o glaze uniforme: o ombro suave sobe, o miolo pára no tecto) e a orla.
//!
//! ## O desenho
//!
//! O traço pinta num PLANO transparente só dele (todas as rotas de carimbo, em `Mix`), e cada lote compõe o
//! plano sobre a base congelada no pen-down — com o MODO do pincel, uma vez, como uma camada. O tecto é um
//! `min` no alfa do plano; as orlas são uma passagem sobre a cobertura do plano: o ARO da Watercolor
//! (`cw − inner`, a MESMA conta, `watercolor_rim::aro_da_cobertura` no tool) — positivo logo dentro da
//! fronteira, negativo na franja de fora, zero no miolo.
//!
//! ⚠️ **O neutro não arma o plano** ([`TracoComposto::is_active`]): o carimbo de sempre, ao bit.

use crate::blend::{BrushBlend, blend_over};
use crate::blend_vazio::blend_over_pigment_modo;

/// A largura máxima da orla (px) — a faixa do Spread da Watercolor (`edge_spread` 1..48), a mesma conta.
pub const LARGURA_MAX_PX: f32 = 48.0;
/// A largura de fábrica da orla (px) — o Spread de fábrica da Watercolor.
pub const LARGURA_DE_FABRICA_PX: f32 = 7.0;

/// O traço como uma camada: o tecto e as orlas. Tudo a zero = desligado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TracoComposto {
    /// O **tecto** de opacidade do traço, `0..1` (`0` = sem tecto): a tinta do traço nunca passa dele, nem
    /// onde o traço se cruza (o *uniform glaze*).
    pub tecto: f32,
    /// A **orla molhada**, `0..1`: o miolo clareia até `1 − molhada` da opacidade do traço e a orla guarda-a
    /// (*Wet edges*).
    pub molhada: f32,
    /// A **orla queimada**, `0..1`: a orla escurece em Linear Burn sobre o que está por baixo (*Burnt edges*).
    pub queimada: f32,
    /// A **largura** da orla em px, `1..`[`LARGURA_MAX_PX`] (o raio do borrão da Watercolor).
    pub largura: f32,
    /// ⭐ **O tecto POR DAB** (J3, `docs/Painter/Pinceis/exames/basicos.md` 2.ª onda): cada texel do traço pára na
    /// opacidade do dab que o cobre — a da mão (pressão, inclinação, velocidade) —, e os dabs não somam além dela:
    /// a *Opacity* do Krita/Photoshop (o Flow é a taxa). É o tecto do Accumulate desligado (`stroke_cover`) armado
    /// também com a Strength a 1; com o Accumulate ligado os dabs somam por definição e isto não age. Não arma o
    /// plano ([`Self::is_active`]).
    pub tecto_por_dab: bool,
}

impl TracoComposto {
    /// Desligado: o plano não arma, o carimbo é o de sempre ao bit.
    pub const NEUTRO: Self = Self {
        tecto: 0.0,
        molhada: 0.0,
        queimada: 0.0,
        largura: LARGURA_DE_FABRICA_PX,
        tecto_por_dab: false,
    };

    /// Se algum campo pede o plano do traço.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.tecto > 0.0 || self.tem_orla()
    }

    /// Se há orla (a passagem do aro corre).
    #[must_use]
    pub fn tem_orla(&self) -> bool {
        self.molhada > 0.0 || self.queimada > 0.0
    }

    /// O raio do borrão do aro para um pincel de raio `radius_px` — a regra da Watercolor:
    /// `min(spread, raio/2)`, pelo menos 1.
    #[must_use]
    pub fn core_r(&self, radius_px: f32) -> usize {
        let spread = self.largura.clamp(1.0, LARGURA_MAX_PX).round() as usize;
        spread.min(((radius_px * 0.5).round() as usize).max(1))
    }

    /// ⭐ **A composição de um texel**: o plano `plano` (RGBA reto, o traço em Normal sobre o transparente)
    /// sobre a base `base`, no `modo` do pincel (`vazio` = a fórmula geral do W3C; `pigmento` = a mistura do
    /// pincel, `BrushSpec::pigmento` — a do carimbo de sempre; um `f32` é o K–M), com o `aro` da cobertura do plano
    /// (`0` sem orla).
    #[must_use]
    #[inline]
    pub fn compoe(
        &self,
        modo: BrushBlend,
        vazio: bool,
        pigmento: impl Into<crate::blend::Pigmento>,
        base: [f32; 4],
        plano: [f32; 4],
        aro: f32,
    ) -> [f32; 4] {
        let mut a = plano[3].clamp(0.0, 1.0);
        if a <= 0.0 {
            return base;
        }
        if self.tecto > 0.0 {
            a = a.min(self.tecto.clamp(0.0, 1.0));
        }
        let m = self.molhada.clamp(0.0, 1.0);
        if m > 0.0 {
            a *= (1.0 - m + m * 2.0 * aro).clamp(0.0, 1.0);
        }
        let cor = [plano[0], plano[1], plano[2]];
        let mut out = blend_over_pigment_modo(modo, base, cor, a, pigmento, vazio);
        let q = self.queimada.clamp(0.0, 1.0);
        if q > 0.0 && aro > 0.0 {
            let aq = q * (2.0 * aro).min(1.0) * plano[3].clamp(0.0, 1.0);
            out = blend_over(BrushBlend::LinearBurn, out, cor, aq);
        }
        out
    }
}

impl Default for TracoComposto {
    fn default() -> Self {
        Self::NEUTRO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O tecto é um `min`: o alfa composto nunca passa dele.
    #[test]
    fn o_tecto_e_um_teto() {
        let t = TracoComposto {
            tecto: 0.4,
            ..TracoComposto::NEUTRO
        };
        let o = t.compoe(
            BrushBlend::Mix,
            false,
            0.0,
            [0.0; 4],
            [0.1, 0.2, 0.9, 1.0],
            0.0,
        );
        assert!((o[3] - 0.4).abs() < 1e-6);
    }

    /// A orla molhada: o miolo (`aro 0`) clareia, a orla (`aro 0,5`) guarda o alfa do traço.
    #[test]
    fn a_orla_molhada_guarda_a_borda_e_clareia_o_miolo() {
        let t = TracoComposto {
            molhada: 0.6,
            ..TracoComposto::NEUTRO
        };
        let p = [0.1, 0.2, 0.9, 0.8];
        let miolo = t.compoe(BrushBlend::Mix, false, 0.0, [0.0; 4], p, 0.0)[3];
        let orla = t.compoe(BrushBlend::Mix, false, 0.0, [0.0; 4], p, 0.5)[3];
        assert!((miolo - 0.8 * 0.4).abs() < 1e-6, "{miolo}");
        assert!((orla - 0.8).abs() < 1e-6, "{orla}");
    }
}
