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
//! `min` no alfa do plano; as orlas pesam-se por `e ∈ [0, 1]`, a orla do traço sobre a máscara VISÍVEL
//! (`traco_orla` no tool: a distância exacta ao contorno `v = ½` do nível de uma passagem, `e = 1 − smoothstep(0, W, d)`; `1` na
//! franja de fora, `0` no fundo). A molhada é o ARO DA AGUADA (bug #58, 7.ª volta: o Watercolor é o oráculo): o
//! tool calcula por texel a densidade do aro da lei da aguada (`ΔD ≥ 0`, `traco_aguada` no tool, em unidades do
//! corpo × a densidade de UMA passagem) e ela SOMA-SE à do texel — `1 − (1 − v)·e^(−ΔD)`, Beer–Lambert sobre o alfa,
//! como a aguada soma `edge` a `cw · fill`; o miolo (`ΔD = 0`) é o traço seco ao bit, a silhueta não mexe. A queimada escurece em Linear Burn com o peso
//! `queimada · e · v` (`docs/Painter/BUGS_painter_fechados_41_a_46.md`).
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
    /// A **orla molhada**, `0..1` (*Wet edges*): o aro da aguada — o Edge Darkening do Watercolor `0..8` — por
    /// dentro da silhueta; o miolo e a silhueta ficam.
    pub molhada: f32,
    /// A **orla queimada**, `0..1`: a orla escurece em Linear Burn sobre o que está por baixo (*Burnt edges*).
    pub queimada: f32,
    /// A **largura** `W` da orla em px, `1..`[`LARGURA_MAX_PX`]: a distância ao contorno visível a que `e` chega a 0.
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

    /// A largura `W` da orla em px, presa à faixa.
    #[must_use]
    pub fn largura_px(&self) -> f32 {
        self.largura.clamp(1.0, LARGURA_MAX_PX)
    }

    /// ⭐ **A composição de um texel**: o plano `plano` (RGBA reto, o traço em Normal sobre o transparente)
    /// sobre a base `base`, no `modo` do pincel (`vazio` = a fórmula geral do W3C; `pigmento` = a mistura do
    /// pincel, `BrushSpec::pigmento` — a do carimbo de sempre; um `f32` é o K–M), com o termo da orla `aro`
    /// (`0..1`: `1` orla cheia, `0` fundo do traço ou sem orla). Sem aro da aguada ([`Self::compoe_molhado`]).
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
        self.compoe_molhado(modo, vazio, pigmento, base, plano, (aro, 0.0))
    }

    /// ⭐ [`Self::compoe`] com a **orla molhada**: `(aro, aguada)` — `aguada` é a densidade óptica `ΔD ≥ 0` do aro
    /// da aguada no texel (já com a força da molhada): a cobertura visível `v` vai a [`orla_da_aguada`]. `ΔD = 0` (o
    /// miolo, ou sem molhada) é o traço seco ao bit. Nada nasce onde `v = 0`.
    #[must_use]
    #[inline]
    pub fn compoe_molhado(
        &self,
        modo: BrushBlend,
        vazio: bool,
        pigmento: impl Into<crate::blend::Pigmento>,
        base: [f32; 4],
        plano: [f32; 4],
        (aro, aguada): (f32, f32),
    ) -> [f32; 4] {
        let mut a = plano[3].clamp(0.0, 1.0);
        if self.tecto > 0.0 {
            a = a.min(self.tecto.clamp(0.0, 1.0));
        }
        // A máscara visível: a orla lê-a e a queimada pesa-se por ela (nunca o alfa cru por baixo do tecto).
        let v = a;
        if a <= 0.0 {
            return base;
        }
        if aguada > 0.0 {
            a = orla_da_aguada(v, aguada);
        }
        let e = aro.clamp(0.0, 1.0);
        let cor = [plano[0], plano[1], plano[2]];
        let mut out = blend_over_pigment_modo(modo, base, cor, a, pigmento, vazio);
        let q = self.queimada.clamp(0.0, 1.0);
        if q > 0.0 && e > 0.0 {
            out = blend_over(BrushBlend::LinearBurn, out, cor, q * e * v);
        }
        out
    }
}

/// ⭐ **O aro da aguada sobre a cobertura** `v`: a densidade `ΔD ≥ 0` soma-se à do texel, `−ln(1 − v)` —
/// `1 − (1 − v)·e^(−ΔD)`. Opaco fica opaco.
#[must_use]
#[inline]
pub fn orla_da_aguada(v: f32, delta: f32) -> f32 {
    1.0 - (1.0 - v.clamp(0.0, 1.0)) * (-delta.max(0.0)).exp()
}

/// A densidade óptica `−ln(1 − L)` de UMA passagem de nível `L` (a unidade do aro da aguada no Digital), presa ao
/// que um `u8` distingue do opaco.
#[must_use]
#[inline]
pub fn densidade_de_uma_passagem(nivel: f32) -> f32 {
    -(1.0 - nivel.clamp(0.0, 1.0 - 1.0 / 255.0)).ln()
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

    /// ⭐ A orla molhada: sem aro (`ΔD = 0`) o texel é o seco ao bit; com aro a densidade soma-se (Beer–Lambert,
    /// `1 − (1 − v)·e^(−ΔD)`), opaco fica opaco, `v = 0` fica base.
    #[test]
    fn a_molhada_e_o_aro_da_aguada_sobre_a_cobertura() {
        let t = TracoComposto {
            molhada: 1.0,
            ..TracoComposto::NEUTRO
        };
        let c = |a: f32, k| {
            t.compoe_molhado(
                BrushBlend::Mix,
                false,
                0.0,
                [0.0; 4],
                [0.1, 0.2, 0.9, a],
                (0.0, k),
            )[3]
        };
        assert!(c(0.6, 0.0).to_bits() == c(0.6, -1.0).to_bits());
        assert!((c(0.6, 0.0) - 0.6).abs() < 1e-6, "o miolo mexeu");
        let um = densidade_de_uma_passagem(0.6);
        assert!((c(0.6, um) - 0.84).abs() < 1e-5, "o aro: {}", c(0.6, um));
        assert!((c(1.0, 3.0) - 1.0).abs() < 1e-6, "opaco");
        assert!(c(0.0, 3.0) == 0.0);
    }
}
