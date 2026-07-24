//! **O Contour** — [`VecContour`], N anéis concêntricos com progressão de cor (item #9 da pesquisa
//! `20_pesquisa_ferramentas_de_artista`).
//!
//! É o efeito que a própria Corel publica como não tendo equivalente no Illustrator, e a pesquisa
//! o resume numa frase: *"não é o nosso offset — é offset × N + cor"*. A forma fonte continua
//! autorada (é ela que o modo Node edita); os anéis são **desenho derivado**, re-cozidos por frame
//! numa `LiveGeometry`, como o [`crate::VecOffset`] de que este é a generalização.
//!
//! # Por que um componente, e não um efeito da pilha (ADR-0132)
//!
//! Porque `PathEffect::apply` é `VecPath -> **UM** VecPath`, e um Contour precisa de N caminhos
//! com **tintas DIFERENTES** — a progressão de cor é metade do efeito. Um compound único tem uma
//! `fill` só. Não é limitação da pilha: é a pilha a dizer que este efeito não é um dos seus.
//!
//! # A cor mora aqui como BYTES, e isso é a fronteira sendo respeitada
//!
//! `ph2d-ecs` não depende de `ph2d-color` (nem `ph2d-vec-scene`, que o recusa explicitamente no
//! próprio `Cargo.toml`). Então o alvo da rampa viaja como `[u8; 4]` sRGB — o mesmo que o
//! [`crate::VecShape`] já faz com os eixos — e quem interpola é a shell, em **Oklab**, no cozimento.
//! Interpolar em sRGB cru é o que deixa o meio da rampa lamacento; o módulo já tem picker OKLCH.
//!
//! # O `d` é POR PASSO, e a razão é MEDIDA
//!
//! O anel `k` fica em `k · d` (o modelo do CorelDRAW). A alternativa — `d` como alcance TOTAL, com
//! o anel externo a ficar parado quando a contagem muda — parecia melhor de usar, e foi **medida e
//! recusada**: com alcance total, mexer nos passos **move todos os anéis**, então arrastar aquele
//! slider re-cozinha N offsets por frame. Por passo, acrescentar um anel custa **UM** offset
//! (~0,7 ms) em vez de N, e o gesto mais comum do efeito fica O(1).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Teto de anéis. ⚠️ **MEDIDO, e o recurso é TEMPO** (`probe_contour_cost`, release, um offset por
/// anel):
///
/// | forma | quina | 8 | 16 | 32 |
/// |---|---|---|---|---|
/// | donut | Miter | 0,55 ms | 1,04 | 2,02 |
/// | donut | Round | 1,85 | 3,55 | 6,95 |
/// | estrela | Miter | 2,25 | 4,33 | 8,59 |
/// | estrela | Round | **5,76** | **11,24** | 22,15 |
///
/// O custo é **linear nos anéis** e o caso que morde é a estrela com quina Round. A 16, três das
/// quatro combinações ficam sob o kill de 8 ms do re-cook por-frame; a quarta (estrela+Round,
/// 11,24 ms) **treme durante um arrasto do `d`** — e essa é a degradação honesta que o número 16
/// compra, em vez de capar todo mundo em 8 por causa da forma mais cara (§0: o fallback não define
/// o produto).
///
/// ⚠️ Arrastar os **passos** não paga isso: o `d` é por-passo, então os anéis que já existem são
/// reusados do memo e só o novo é cozido. Quem re-coza tudo é mexer no `d`, na quina ou no lado.
pub const MAX_CONTOUR_STEPS: u16 = 16;

/// **O Contour de uma forma:** N anéis concêntricos, do original até uma cor-alvo.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecContour {
    /// Quantos anéis, **além** da fonte. `0` não acontece: a shell REMOVE o componente em vez de
    /// guardar um contour inerte (a lei do `VecOffset`, e a mesma da rotação do pattern).
    pub steps: u16,
    /// A distância **por passo**, em unidades de MUNDO. O anel `k` fica em `k · d` (§ do módulo).
    /// Negativo encolhe — é o *Inside* do Corel, e cai da mesma aritmética sem um segundo modo.
    pub d: f64,
    /// O estilo de quina, no código do painel: `0` Miter · `1` Round · `2` Bevel. Mesmos códigos
    /// do [`crate::VecOffset`], resolvidos pela MESMA porta (`vec_expand::join_of_code`).
    pub join: u8,
    /// Que contorno anda, no código do painel: `0` Outer · `1` Inner · `2` Both.
    pub side: u8,
    /// A cor do ÚLTIMO anel, em **sRGB** `[r, g, b, a]`. O primeiro parte da cor da FONTE, então a
    /// rampa tem os dois extremos sem o artista autorar o de partida.
    pub to: [u8; 4],
    /// **Aceleração** da progressão. `1.0` é linear; `>1` espalha os anéis para longe; `<1` os
    /// amontoa perto da fonte. É o knob que o Corel tem e o Illustrator não.
    ///
    /// ⚠️ Governa DUAS coisas por fórmulas diferentes, e a diferença é load-bearing:
    /// [`Self::ring_distance`] (independente da contagem — é o que salva o memo) e
    /// [`Self::ramp_t`] (dependente dela — é o que faz o último anel chegar ao alvo).
    ///
    /// ⚠️ Um `accel` só governa **cor e espaçamento juntos**. O Corel os separa; aqui ficam
    /// ligados de propósito, porque desligá-los é um segundo knob para um efeito que o artista lê
    /// como uma coisa só. Separar é aditivo se o uso pedir.
    pub accel: f32,
}

impl SimComponent for VecContour {}

impl Default for VecContour {
    fn default() -> Self {
        Self {
            steps: 4,
            d: 0.1,
            join: 1, // Round: a quina que faz um contour parecer um contour
            side: 0, // Outer
            to: [255, 255, 255, 255],
            accel: 1.0,
        }
    }
}

impl VecContour {
    /// A **distância** do anel `k` (1-based), em unidades de mundo: `d · k^accel`.
    ///
    /// ⚠️ **Não depende de `steps`, e é isso que a torna barata.** Acrescentar um anel deixa todos
    /// os anteriores exatamente onde estavam, então o memo do cozimento reusa o prefixo inteiro e
    /// o arrasto do slider de passos custa UM offset. Normalizar por `steps` (o modelo
    /// "alcance total") daria um anel externo que fica parado — e re-cozeria os N a cada passo.
    #[must_use]
    pub fn ring_distance(&self, k: u16) -> f64 {
        self.d * f64::from(k).powf(f64::from(self.accel).max(Self::MIN_ACCEL))
    }

    /// A fração da rampa de **COR** em que o anel `k` (1-based) cai.
    ///
    /// ⚠️ **Esta SIM depende de `steps`, e as duas coisas não podem ser a mesma função** — o
    /// último anel tem de aterrissar na cor-alvo seja qual for a contagem, e isso é exatamente o
    /// que a distância não pode fazer (acima). Custa nada: a cor é recomputada por frame, fora do
    /// memo, então depender da contagem não re-coze geometria nenhuma.
    #[must_use]
    pub fn ramp_t(&self, k: u16) -> f64 {
        if self.steps == 0 {
            return 0.0;
        }
        let raw = f64::from(k) / f64::from(self.steps);
        raw.powf(f64::from(self.accel).max(Self::MIN_ACCEL))
    }

    /// Piso da aceleração: `0` mataria a progressão (todo anel na mesma distância) e negativo a
    /// inverteria com uma divergência em `k → 0`. Guarda de representação, não de produto.
    const MIN_ACCEL: f64 = 0.01;
}

#[cfg(test)]
mod tests {
    use super::VecContour;

    /// **A distância do anel `k` NÃO depende da contagem** — o invariante que deixa o memo do
    /// cozimento reusar o prefixo inteiro, e a razão de o `d` ser por-passo.
    ///
    /// ⚠️ Este gate existe porque o irmão de integração (`adding_a_step_leaves_the_existing_rings_
    /// where_they_were`, na shell) **não o prova**: lá os anéis vêm do memo, que não é invalidado
    /// por uma mudança de contagem, então eles ficariam parados mesmo com uma fórmula dependente
    /// de `steps` — a diferença é que passariam a estar STALE em vez de certos. A propriedade é da
    /// FUNÇÃO PURA e é aqui que se afirma.
    #[test]
    fn the_ring_distance_ignores_the_step_count() {
        for accel in [0.5_f32, 1.0, 2.0] {
            for k in 1..=4_u16 {
                let d: Vec<f64> = [2_u16, 5, 16]
                    .iter()
                    .map(|&steps| {
                        VecContour {
                            steps,
                            d: 0.1,
                            accel,
                            ..VecContour::default()
                        }
                        .ring_distance(k)
                    })
                    .collect();
                assert!(
                    (d[0] - d[1]).abs() < 1e-15 && (d[1] - d[2]).abs() < 1e-15,
                    "accel {accel}, anel {k}: a distância mudou com a contagem ({d:?}) — o memo \
                     do cozimento passaria a servir anéis STALE"
                );
            }
        }
    }

    /// **A rampa de cor SIM depende da contagem, e sempre aterrissa no alvo** — o par exato do
    /// gate acima, e é o contraste entre os dois que documenta por que são funções diferentes.
    #[test]
    fn the_colour_ramp_lands_on_the_target_whatever_the_count() {
        for steps in [1_u16, 3, 16] {
            for accel in [0.5_f32, 1.0, 2.0] {
                let c = VecContour {
                    steps,
                    accel,
                    ..VecContour::default()
                };
                assert!(
                    (c.ramp_t(steps) - 1.0).abs() < 1e-12,
                    "steps {steps}, accel {accel}: o último anel não chega ao alvo ({})",
                    c.ramp_t(steps)
                );
                assert!(
                    c.ramp_t(1) <= c.ramp_t(steps),
                    "a rampa tem de ser monótona"
                );
            }
        }
    }
}
