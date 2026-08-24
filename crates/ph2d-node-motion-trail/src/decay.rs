//! **OS ALVOS DA CAUDA e as taxas que eles viram** — o `Decay` do `motion.trail`.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o corte é por
//! RESPONSABILIDADE: o `lib.rs` responde *quem sobrevive a este tique* e este responde *quanto
//! cada sobrevivente desbota*.

use super::{colour, rate_for, step_for};
use crate::carry::{add_scalar, fade_alpha, scale_vec2, tint_op};
use ph2d_nodegraph::attr::Stream;

/// **Tudo o que um eco sofre ao longo da CAUDA INTEIRA**, num lugar só.
///
/// ⚠️ Os cinco campos são o que o artista AUTORA — o estado do eco mais VELHO, relativo à
/// cabeça viva (os multiplicativos) e o total percorrido (os angulares). Eles **não** são
/// taxas por tick: quem as deriva é [`Decay::per_tick`], e é essa derivação que torna o
/// número do slider independente do Length e do Spacing. Um knob no neutro é a identidade
/// **ao bit** — nenhum deles toca um byte no default.
#[derive(Copy, Clone, Debug)]
pub struct Decay {
    /// Alfa do eco mais velho, relativa à cabeça viva.
    pub fade: f32,
    /// Tamanho do eco mais velho, relativo à cabeça viva.
    pub shrink: f32,
    /// Graus de matiz que a cauda inteira percorre (rotação luma-preservante, RGB linear).
    pub hue_shift: f32,
    /// Saturação do eco mais velho, relativa à cabeça viva.
    pub saturation: f32,
    /// Graus de giro que a cauda inteira percorre.
    pub spin: f32,
    /// **O TETO DA CAUDA** — ver [`ALPHA_MAX`]. `1` é o de sempre.
    pub alpha_max: f32,
}

impl Decay {
    /// O ponto neutro — todo operador na identidade.
    pub const NEUTRAL: Self = Self {
        alpha_max: 1.0,
        fade: 1.0,
        shrink: 1.0,
        hue_shift: 0.0,
        saturation: 1.0,
        spin: 0.0,
    };

    /// Só o par que sempre existiu (para as fixtures que não falam de cor).
    #[must_use]
    pub fn new(fade: f32, shrink: f32) -> Self {
        Self {
            fade,
            shrink,
            ..Self::NEUTRAL
        }
    }

    /// **Converte os alvos AUTORADOS nas taxas POR TICK que os alcançam.**
    ///
    /// `span` é a idade do eco mais VELHO — `(length − 1) × spacing` —, então
    /// `rate^span == target` por construção: o que o artista digita é o que ele vê na
    /// ponta da cauda, e o número **não se move** quando o Length ou o Spacing mudam.
    ///
    /// ⚠️ O `span` sai do `k` **CLAMPADO** (o orçamento de instâncias pode encurtar a
    /// cauda), porque o alvo pertence à cauda que de fato existe. Se o `k` mudar no meio
    /// de um traço, as linhas já carregadas guardam o que a taxa anterior lhes deu e as
    /// novas seguem a nova — transitório, e ele se cura sozinho quando as velhas saem.
    #[must_use]
    pub(crate) fn per_tick(self, span: u32) -> Self {
        if span == 0 {
            return Self::NEUTRAL;
        }
        let inv = 1.0 / span as f32;
        Self {
            // ⚠️ O teto NÃO é uma taxa: ele nao se converte por vao nenhum -- viaja intacto.
            alpha_max: self.alpha_max,
            fade: rate_for(self.fade, inv),
            shrink: rate_for(self.shrink, inv),
            saturation: rate_for(self.saturation, inv),
            hue_shift: step_for(self.hue_shift, span),
            spin: step_for(self.spin, span),
        }
    }

    /// **O estado ABSOLUTO de um eco de idade `age`**, em vez do passo de um tick.
    ///
    /// O ring aplica [`Self::per_tick`] uma vez por tick, então uma linha com `n`
    /// ticks levou-a `n` vezes; o rastro RE-COZIDO não tem passado nenhum a que
    /// somar, e tem de chegar ao mesmo sítio de uma vez.
    ///
    /// ⚠️ **`at_age(span, 1)` é `per_tick(span)`, expressão por expressão** — a
    /// generalização contém o caso antigo, e é por isso que existe uma lei e não
    /// duas. (Não ao BIT ao longo da cauda, de propósito: `rate^n` por `n`
    /// multiplicações e `rate_for(alvo, n/span)` são a mesma matemática por dois
    /// caminhos de arredondamento, e o modo do ring continua a ser o primeiro.)
    #[must_use]
    pub(crate) fn at_age(self, span: u32, age: u32) -> Self {
        if span == 0 {
            return Self::NEUTRAL;
        }
        let f = age as f32 / span as f32;
        Self {
            // ⚠️ O teto NÃO é uma taxa: ele nao se converte por vao nenhum -- viaja intacto.
            alpha_max: self.alpha_max,
            fade: rate_for(self.fade, f),
            shrink: rate_for(self.shrink, f),
            saturation: rate_for(self.saturation, f),
            hue_shift: step_for(self.hue_shift, span) * age as f32,
            spin: step_for(self.spin, span) * age as f32,
        }
    }

    /// Envelhece um conjunto de linhas carregadas em UM tick. **Recebe as taxas já
    /// derivadas** — chamá-la com os alvos autorados é o defeito que o smoke de
    /// 2026-08-08 reportou.
    pub(crate) fn apply(self, carried: &mut Stream) {
        fade_alpha(carried, "tint", self.fade);
        scale_vec2(carried, "size", self.shrink);
        // ⚠️ UMA matriz para os dois operadores de cor: compor antes do laço deixa o
        // caminho por-linha com nove multiplicações, sejam zero, um ou dois knobs armados
        // — e no neutro a matriz É a identidade, então o `tint` não se move.
        let m = colour::compose(
            colour::hue_rotation(self.hue_shift),
            colour::saturation(self.saturation),
        );
        if m != colour::IDENTITY {
            tint_op(carried, "tint", m);
        }
        // ⚠️ Gateado em `!= 0`, ao contrário dos outros: o `rot` é MATERIALIZADO quando
        // ausente, e materializá-lo sem o artista ter pedido giro acrescentaria uma coluna
        // que ninguém pediu a todo rastro do app.
        if self.spin != 0.0 {
            add_scalar(carried, "rot", self.spin);
        }
    }
}
