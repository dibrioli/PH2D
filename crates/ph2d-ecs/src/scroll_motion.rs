//! **O MOVIMENTO PRÓPRIO de um fundo de paralaxe** (plano 24, W4) — nuvens que andam sozinhas.
//!
//! ```text
//! offset = velocidade × playhead
//! ```
//!
//! # ⭐⭐⭐ Aqui ganha-se por DESENHO, e não por afinação
//!
//! O *autoscroll* do alvo **não é observável** — medido, e com os dois controlos a dizê-lo (§4.6 da
//! pesquisa): os quadros correm (`10` somaram `0,0667 s`), o observável responde (mexer o
//! `scroll_offset` à mão move `+333,0`), e `120 px/s` durante `60` quadros movem **`+0,000`** nos
//! **quatro** observáveis (transformada · offset · global · o próprio filho). ⇒ *ele vive no
//! caminho de DESENHO*: não compõe com o resto do estado, não sobrevive a um scrub, e nenhum teste
//! o alcança.
//!
//! O nosso é uma função **pura do playhead**, e as três consequências são a razão da wave:
//!
//! | propriedade | o que ela compra |
//! |---|---|
//! | **puro** | sobrevive ao scrub e ao rebobinar **sem uma linha de estado** |
//! | **medível** | tem gate — e é por isso que ele existe neste ficheiro |
//! | **determinista** | entra no replay, e duas máquinas vêem a mesma nuvem |
//!
//! # ⚠️ Ele é um SOMANDO do deslocamento, nunca um segundo condutor
//!
//! Medido antes da 1.ª linha ([`super::super::parallax_w4_probe_tests`] na ponte): um segundo motor
//! a escrever o mesmo `Transform` — um tween de pose, por exemplo — entra no ledger com **outra
//! chave**, e a paralaxe lê a escrita dele como se tivesse sido o artista a arrastar. ⇒ *a deriva
//! tem de viver dentro da MESMA lei*, somada ao deslocamento antes de o ledger o ver.
//!
//! ⛔ **E não há `ScrollMotionRuntime`:** um acumulador daria uma nuvem que anda ao contrário quando
//! o artista arrasta a régua para trás, e duas máquinas com quadros diferentes veriam nuvens
//! diferentes. *O que não tem estado não pode sobreviver errado* — a mesma frase do `ScrollFactor`.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

/// **Quanto este fundo anda sozinho, em metros por segundo.**
///
/// ⚠️ **Por EIXO**, como o `k`: uma camada de nuvens corre de lado e não sobe.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScrollMotion {
    pub velocity: [f32; 2],
}

impl ScrollMotion {
    /// ⭐ **O deslocamento próprio no instante `t`** (segundos do playhead).
    ///
    /// ⚠️ **`t` é `f64` e o produto é feito em `f64`**, e isso é medido: ao fim de uma hora de
    /// relógio (`3 600 s`) a `1 m/s`, um `f32` tem ULP de `2,4e-4` e o erro de arredondar o
    /// PRODUTO cresce com ele. *A deriva é a única grandeza desta família que cresce sem limite com
    /// o tempo*, e é a única que paga o `f64`.
    #[must_use]
    pub fn deslocamento(&self, t: f64) -> [f32; 2] {
        if !t.is_finite() {
            return [0.0, 0.0];
        }
        [
            (f64::from(self.velocity[0]) * t) as f32,
            (f64::from(self.velocity[1]) * t) as f32,
        ]
    }

    /// `true` se ele não move nada — a omissão, e o que o deixa fora do trabalho.
    #[must_use]
    pub fn e_inerte(&self) -> bool {
        self.velocity == [0.0, 0.0]
    }
}
