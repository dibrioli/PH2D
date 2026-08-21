//! **A sprite como FONTE DE LUZ** — plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md) W8.
//!
//! > Enio, 2026-08-21: *"1) sprite como fonte de luz."* — o item que o §6.1 daquele plano tinha
//! > devolvido como **decisão de produto**, agora decidido.
//!
//! # As duas coisas que «fonte de luz» pode querer dizer, e qual é esta
//!
//! | leitura | o que exige | é isto? |
//! |---|---|---|
//! | a sprite **brilha** — a luz dela sangra sobre o que está à volta | um passe de bloom sobre os emissores, que **já existe** | ✅ **é esta** |
//! | a sprite **ilumina** outras sprites — sombra, oclusão, atenuação | um sistema de propagação de luz 2D, do zero | ⛔ não |
//!
//! ⚠️ **A diferença não é de grau, é de sistema.** O `ph2d-light` que existe é um rig de
//! **sombreamento por normais** (lâmpadas + ambiente, para o impasto e o sculpt) — ele acende uma
//! superfície que tem relevo, e não sabe nada sobre uma sprite iluminar a vizinha. A segunda leitura
//! é uma frente própria; esta wave entrega a primeira, que é a que o `Rgba16Float` do `GameRt`
//! tornava gratuita e que ninguém tinha ligado.
//!
//! # Por que isto pertence à wave dos 16 bits
//!
//! O plano 18 §6.1 já o tinha antecipado: *"o `Rgba16Float` dá a folga acima de 1.0 **de graça**"*.
//! Um alvo de 8 bits satura em branco — não há como dizer «isto é mais brilhante que branco», que é
//! a única coisa que um emissor precisa de dizer. É por isso que este componente é **um
//! multiplicador** e não uma cor: o que ele faz é empurrar a cor da sprite para além de 1.0, onde o
//! bright-pass do bloom a encontra.
//!
//! ⚠️ **E há um caminho que nem precisa deste componente:** uma textura de 16 bits importada de um
//! EXR/HDR pode **já** guardar valores acima de 1.0, e nesse caso ela brilha sozinha. O componente é
//! a autoria — o botão do artista — e não a única porta.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// O multiplicador neutro: a sprite não emite.
pub const EMISSIVE_OFF: f32 = 0.0;

/// **O maior multiplicador que o autor pode pedir.**
///
/// ⚠️ **É um limite de REPRESENTAÇÃO, medido, e não um palpite de conforto** (`CLAUDE.md` §0). O
/// `GameRt` é `Rgba16Float`: o maior meio-float finito é **65504**. Uma cor de sprite chega no
/// máximo a 1.0, logo qualquer multiplicador até 65504 é representável — e acima disso o produto
/// satura em **infinito**, que o bright-pass propaga como NaN pelo blur e pinta o ecrã de buracos.
///
/// O tecto real é bem mais baixo que o da representação, e também é medido: a partir de ~32× o
/// bright-pass já leva a sprite inteira (não só os realces) e o resultado deixa de ser um halo para
/// ser um borrão branco. **64** dá uma oitava de folga acima disso, e é o número que o slider mostra.
pub const EMISSIVE_MAX: f32 = 64.0;

/// **Quanto esta sprite EMITE** — o multiplicador aplicado à cor dela no passe de emissão.
///
/// `0.0` (o default, e a ausência do componente) = não emite, e o quadro é **byte-idêntico** ao de
/// antes deste componente existir. Há gate.
///
/// ⚠️ **Multiplica a cor, não a substitui.** Uma sprite vermelha a `4.0` emite vermelho, não branco —
/// o halo herda a cor da arte, que é o que faz uma lâmpada desenhada parecer acesa em vez de
/// recortada. Quem quiser um halo de outra cor pinta a arte de outra cor: *o emissor não é um segundo
/// sítio onde a cor se autora*.
///
/// ⚠️ **A sprite continua a ser desenhada normalmente.** Este componente **acrescenta** um halo; ele
/// não troca o caminho de desenho nem toca no `Sprite`. É o que mantém o emissor ortogonal a tudo o
/// resto — atlas ou individual, 8 ou 16 bits, com ou sem máscara.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpriteEmissive(pub f32);

impl Default for SpriteEmissive {
    fn default() -> Self {
        Self(EMISSIVE_OFF)
    }
}

impl SpriteEmissive {
    /// A intensidade útil, presa à faixa que a representação suporta.
    #[must_use]
    pub fn clamped(self) -> f32 {
        if self.0.is_finite() {
            self.0.clamp(EMISSIVE_OFF, EMISSIVE_MAX)
        } else {
            // ⚠️ Um NaN vindo de um projeto corrompido (ou de um slider dividido por zero) tem de
            // virar «apagado», nunca «infinito»: infinito atravessa o blur e apaga o quadro.
            EMISSIVE_OFF
        }
    }

    /// **Esta sprite chega a emitir alguma coisa?** É o que decide se o passe corre de todo.
    #[must_use]
    pub fn emits(self) -> bool {
        self.clamped() > EMISSIVE_OFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_does_not_emit() {
        assert_eq!(SpriteEmissive::default().0, EMISSIVE_OFF);
        assert!(!SpriteEmissive::default().emits());
    }

    /// ⚠️ **O teto vem da representação, e o gate nomeia-a.** Um `f32` acima de 65504 não cabe num
    /// meio-float e satura em infinito; o `EMISSIVE_MAX` fica três oitavas abaixo disso de propósito
    /// (ver o doc-comment), e este teste é o que impede alguém subi-lo sem ler porquê.
    #[test]
    fn the_ceiling_is_below_what_the_half_float_can_hold() {
        // ⚠️ **O maior meio-float finito é DERIVADO da definição do formato, nunca digitado:**
        // 11 bits de mantissa e expoente máximo 15 dão `(2 − 2⁻¹⁰) × 2¹⁵`. Escrever `65504.0` aqui
        // seria um número a acreditar — e o clippy nem sequer deixaria (dois literais comparados
        // são uma asserção de valor constante, que passa sem medir nada).
        let half_max = (2.0_f32 - 2.0_f32.powi(-10)) * 2.0_f32.powi(15);
        assert!(
            EMISSIVE_MAX < half_max,
            "o teto do emissor ({EMISSIVE_MAX}) tem de caber num meio-float ({half_max}) — acima \
             disso o produto satura em infinito, e um infinito atravessa o blur do bloom e apaga \
             o quadro"
        );
        assert_eq!(SpriteEmissive(1.0e9).clamped(), EMISSIVE_MAX);
        assert_eq!(SpriteEmissive(-5.0).clamped(), EMISSIVE_OFF);
    }

    /// ⚠️ **NaN vira APAGADO, nunca infinito.** Um projeto corrompido não pode apagar o quadro.
    #[test]
    fn a_nan_becomes_off_and_not_infinite() {
        assert_eq!(SpriteEmissive(f32::NAN).clamped(), EMISSIVE_OFF);
        assert!(!SpriteEmissive(f32::NAN).emits());
        assert_eq!(SpriteEmissive(f32::INFINITY).clamped(), EMISSIVE_OFF);
    }

    #[test]
    fn a_positive_intensity_emits() {
        assert!(SpriteEmissive(1.0).emits());
        assert!(SpriteEmissive(0.001).emits());
        assert!(!SpriteEmissive(0.0).emits());
    }
}
