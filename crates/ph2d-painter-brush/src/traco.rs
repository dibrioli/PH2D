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
//! franja de fora, `0` no fundo). A molhada é o *Wet Edges* do Procreate MEDIDO (bug #58, 5.ª volta): o miolo
//! nivela-se num PATAMAR (`PATAMAR_MOLHADO` do nível de uma passagem) — as sobreposições e o grão do miolo somem,
//! a silhueta (o contorno `τ`) não mexe, nenhuma orla; a queimada escurece em Linear Burn com o peso
//! `queimada · e · v` (`docs/Painter/BUGS_painter_fechados_41_a_46.md`).
//!
//! ⚠️ **O neutro não arma o plano** ([`TracoComposto::is_active`]): o carimbo de sempre, ao bit.

use crate::blend::{BrushBlend, blend_over};
use crate::blend_vazio::blend_over_pigment_modo;

/// A largura máxima da orla (px) — a faixa do Spread da Watercolor (`edge_spread` 1..48), a mesma conta.
pub const LARGURA_MAX_PX: f32 = 48.0;
/// A largura de fábrica da orla (px) — o Spread de fábrica da Watercolor.
pub const LARGURA_DE_FABRICA_PX: f32 = 7.0;
/// O patamar do miolo com Wet 1, em fracção do nível de uma passagem: medido no Procreate (Intense Glaze, Flow
/// Max, `procreate_ref/ebb_0162` × `ebb_0164`), a cobertura do miolo cai para `0,92` da do Wet 0.
pub const PATAMAR_MOLHADO: f32 = 0.92;
/// A fracção do grão do miolo que fica com Wet 1: no mesmo par o desvio-padrão do miolo cai de 16–20 para 4–9
/// (`÷3`).
pub const GRAO_MOLHADO: f32 = 1.0 / 3.0;

/// O traço como uma camada: o tecto e as orlas. Tudo a zero = desligado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TracoComposto {
    /// O **tecto** de opacidade do traço, `0..1` (`0` = sem tecto): a tinta do traço nunca passa dele, nem
    /// onde o traço se cruza (o *uniform glaze*).
    pub tecto: f32,
    /// A **orla molhada**, `0..1` (*Wet edges* do Procreate): o miolo do traço nivela-se no patamar (as
    /// sobreposições e o grão somem); a silhueta fica.
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
    /// (`0..1`: `1` orla cheia, `0` fundo do traço ou sem orla). Sem o nível a molhada não age
    /// ([`Self::compoe_molhado`]).
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

    /// ⭐ [`Self::compoe`] com a **orla molhada**: `(aro, nivel)` — `nivel` é o nível `L` de UMA passagem da máscara
    /// visível (a fronteira `τ = L/2`). A cobertura visível `v` vai para o patamar `P = PATAMAR_MOLHADO · L` (com
    /// `GRAO_MOLHADO` do vale do grão) com o peso `molhada · smoothstep(τ, ¾L, v)`: no contorno `τ` o peso é `0` (a
    /// silhueta não mexe), a sobreposição vai ao patamar, o grão do miolo cai a ⅓, a tinta ténue abaixo de `τ`
    /// guarda-o. Nada nasce onde `v = 0`.
    #[must_use]
    #[inline]
    pub fn compoe_molhado(
        &self,
        modo: BrushBlend,
        vazio: bool,
        pigmento: impl Into<crate::blend::Pigmento>,
        base: [f32; 4],
        plano: [f32; 4],
        (aro, nivel): (f32, f32),
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
        let m = self.molhada.clamp(0.0, 1.0);
        if m > 0.0 && nivel > 0.0 {
            a = molhado(v, nivel, m);
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

/// ⭐ **A lei da molhada** sobre a cobertura visível `v`, com o nível de uma passagem `nivel` e a força `m`.
#[must_use]
#[inline]
pub fn molhado(v: f32, nivel: f32, m: f32) -> f32 {
    let tau = 0.5 * nivel;
    // O peso sobe de `τ` a `¾L`: com `L` o grão de ±10 % caía só `÷2` (medido no gate (b)); a `¾L` cai `÷3`.
    let s = ((v - tau) / (0.25 * nivel)).clamp(0.0, 1.0);
    let k = s * s * (3.0 - 2.0 * s);
    // O alvo: o patamar com `GRAO_MOLHADO` do vale do grão abaixo do nível; acima dele (a sobreposição) o patamar.
    let vale = 1.0 - (v / nivel).min(1.0);
    let alvo = PATAMAR_MOLHADO * nivel * (1.0 - GRAO_MOLHADO * vale);
    v + m * k * (alvo - v)
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

    /// ⭐ A orla molhada: o contorno `τ` não mexe, o miolo e a sobreposição vão ao patamar, `v = 0` fica base, sem
    /// nível a molhada não age.
    #[test]
    fn a_molhada_nivela_o_miolo_e_guarda_o_contorno() {
        let t = TracoComposto {
            molhada: 1.0,
            ..TracoComposto::NEUTRO
        };
        let c = |a: f32, nivel| {
            t.compoe_molhado(
                BrushBlend::Mix,
                false,
                0.0,
                [0.0; 4],
                [0.1, 0.2, 0.9, a],
                (0.0, nivel),
            )[3]
        };
        assert!((c(0.3, 0.6) - 0.3).abs() < 1e-6, "o contorno mexeu");
        assert!(
            (c(0.6, 0.6) - 0.552).abs() < 1e-5,
            "o miolo: {}",
            c(0.6, 0.6)
        );
        assert!(
            (c(0.9, 0.6) - 0.552).abs() < 1e-5,
            "a sobreposição: {}",
            c(0.9, 0.6)
        );
        assert!(c(0.0, 0.6) == 0.0);
        assert!((c(0.9, 0.0) - 0.9).abs() < 1e-6, "sem nível a molhada agiu");
    }
}
