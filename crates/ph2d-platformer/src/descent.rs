//! **O TETO DA DESCIDA** (`W-Fall`, plano 10 §4) — a lei ÚNICA que limita quão
//! depressa o personagem cai, e as duas coisas que a autoram.
//!
//! # ⚠️ Não existia velocidade terminal, e o número é desta wave
//!
//! A auditoria afirmava-o e uma premissa herdada não é uma medição. Medido pela
//! porta do produto (`ph2d-physics-ecs/tests/measure_terminal.rs`), largando de
//! mil metros, a descida por segundo é:
//!
//! | s | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
//! |---|---|---|---|---|---|---|---|---|
//! | m | 5,94 | 24,86 | 44,48 | 64,10 | 83,72 | 103,33 | 122,95 | **142,57** |
//!
//! Monotónica, **nos dois modos**. Um personagem que caia de alto o bastante
//! atravessa o cenário a velocidades que nenhum colisor discreto resolve, e o
//! artista não tinha número nenhum para dizer *"não mais depressa que isto"*.
//!
//! # ⚠️ O plano prescrevia DUAS implementações, e a medição as colapsou numa
//!
//! Ele escreveu *"sob Spring é um boost contra a gravidade do solver; sob Snap é
//! um clamp no `KinematicState`"*. Mas o freio do planeio sai desta mesma lei
//! como um [`Motor`], e o `kinematic_advance` **já consome `motor.boost`** —
//! medido com o planeio armado a `4,0`, os dois modos assentam numa descida
//! constante (**4,25** Spring · **4,33** Snap) em vez de continuarem a acelerar.
//! Logo **uma porta serve os dois modos**, e a segunda implementação seria a
//! segunda resposta à mesma pergunta.
//!
//! # ⚠️ O "menor vence" é um `min`, e não uma regra a lembrar
//!
//! Duas leis autoram um teto de descida — o planeio (enquanto o dedo dura) e o
//! teto de queda (sempre). Se cada uma tivesse a sua porta e o seu `boost`, os
//! dois motores somariam, e o planeio ganharia o poder de **acelerar** uma queda
//! que o teto já tinha limitado. Aqui elas não produzem motor nenhum: elas
//! **propõem um número**, o [`descent_ceiling`] fica com o menor, e existe um
//! único [`descent_motor`]. A composição deixa de ser algo que alguém tem de
//! acertar e passa a ser o que a representação permite escrever.
//!
//! # ⚠️ A lei não consegue empurrar para BAIXO, por construção
//!
//! O guard (`rel_up >= −ceiling` ⇒ silêncio) é o teto, e ele também é o que
//! torna `delta` **sempre positivo**. Não é uma verificação a mais: é a mesma
//! linha a responder às duas perguntas, e é por isso que não existe um instante
//! em que este módulo possa acelerar uma queda. Foi ela que descartou a forma
//! *alvo* na wave do planeio — ver o topo do [`crate::glide`], onde a medição
//! que escolheu **teto** entre as três formas candidatas está guardada.
//!
//! # ⚠️ O teto assenta ~6% ACIMA do número autorado, e isso é integração
//!
//! Medido: `4,0` autorado assenta em **4,25** (Spring) e **4,33** (Snap). O
//! freio é aplicado no topo do tique e a gravidade soma **dentro** dele, então a
//! média do intervalo fica acima do teto instantâneo. Está nomeado aqui porque
//! um gate que pedisse igualdade exata nasceria **vermelho sobre produto
//! correto** — o oráculo honesto é *a descida PARA de crescer*, não um `==`.

use crate::{GlideConfig, Motor, Vec2};

/// **O teto de queda do personagem** — a velocidade terminal que ele tem.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FallConfig {
    /// **A velocidade máxima de descida**, em m/s, em queda livre.
    ///
    /// `0.0` **desliga** — sem teto, que é o mundo de antes desta wave ao bit —
    /// e é o idioma dos irmãos `coyote_time` · `corner_reach` ·
    /// `wall_slide_speed` · `air_jumps` · `ledge_grab` · `glide_fall_speed`: um
    /// zero que significa *"esta assistência não existe"*.
    ///
    /// ⚠️ **Não confundir com `glide_fall_speed`:** os dois são tetos de descida
    /// e é por isso que passam pela mesma porta, mas o do planeio existe
    /// **enquanto o dedo dura** e este vale **sempre**. Quando os dois estão
    /// vivos, vence o menor — ver [`descent_ceiling`].
    ///
    /// ⚠️ **Não confundir com `fall_gravity`:** aquele é uma ACELERAÇÃO e diz
    /// quão depressa a descida cresce; este é uma VELOCIDADE e diz onde ela
    /// para de crescer.
    pub max_speed: f32,
}

impl FallConfig {
    /// O ponto de partida — **DESLIGADO**.
    ///
    /// ⚠️ **Nasce em zero porque toda capacidade autorada deste módulo nasce**,
    /// e aqui isso tem um preço executável: um projeto salvo antes desta wave
    /// reabre a cair exatamente como caía, e o hash de determinismo
    /// (`physics_ecs_c9`) fica byte-idêntico — a prova de que o degrau de schema
    /// não move física nenhuma.
    pub const STARTING_POINT: Self = Self { max_speed: 0.0 };

    /// A assistência está autorada?
    #[must_use]
    pub fn armed(&self) -> bool {
        self.max_speed > 0.0
    }
}

impl Default for FallConfig {
    fn default() -> Self {
        Self::STARTING_POINT
    }
}

/// **Que teto de descida está em vigor neste tique** — o MENOR dos vivos, em
/// m/s, ou `None` se nenhuma lei limita esta queda.
///
/// `held` é o nível do botão de pulo, que é o que arma o planeio (ver
/// [`crate::glide`]); o teto de queda não pergunta nada ao jogador.
///
/// ⚠️ **É aqui que a composição acontece, e é de propósito que ela seja um
/// `min` sobre NÚMEROS** — ver o topo deste módulo.
#[must_use]
pub fn descent_ceiling(glide: &GlideConfig, fall: &FallConfig, held: bool) -> Option<f32> {
    let from_glide = (glide.armed() && held).then_some(glide.fall_speed);
    let from_fall = fall.armed().then_some(fall.max_speed);
    match (from_glide, from_fall) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// **O freio** — `Motor::default()` quando não há nada a fazer.
///
/// `rel_up` é a velocidade vertical **relativa ao suporte** (a mesma que o
/// [`crate::wall_slide`] consome), e `up` é o eixo para cima.
///
/// ⚠️ **Uma linha, duas perguntas:** o guard é o TETO (só age sobre quem cai
/// mais depressa que o limite) e é também o que garante que `delta` seja
/// positivo — ou seja, que este módulo nunca consiga acelerar uma queda.
#[must_use]
pub fn descent_motor(ceiling: Option<f32>, rel_up: f32, up: Vec2) -> Motor {
    let Some(ceiling) = ceiling else {
        return Motor::default();
    };
    if rel_up >= -ceiling {
        return Motor::default();
    }
    let delta = -ceiling - rel_up;
    Motor {
        accel: [0.0, 0.0],
        boost: [up[0] * delta, up[1] * delta],
    }
}

#[cfg(test)]
#[path = "descent_tests.rs"]
mod tests;
