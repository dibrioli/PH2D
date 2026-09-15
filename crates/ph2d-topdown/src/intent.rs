//! **A INTENÇÃO vira VELOCIDADE** — a rampa de arranque e a de travagem.
//!
//! Uma porta só, e ela governa **as três coisas de uma vez**: acelerar, travar e
//! **virar**. Uma lei que acelerasse só ao longo do eixo do movimento deixaria a
//! viragem instantânea num corpo com rampa — o carro que muda de direcção sem
//! desacelerar, que é o defeito que se lê como *«escorrega»*.
//!
//! ```text
//! alvo = direcção_de_mundo · speed
//! com intenção   →  anda-se de `v` para `alvo`, no máximo `acceleration·dt`
//! sem intenção   →  anda-se de `v` para zero,  no máximo `deceleration·dt`
//! ```
//!
//! # ⚠️ Zero DESLIGA a rampa, e é isso que faz o arranque arcade
//!
//! `acceleration = 0` não quer dizer *«nunca acelera»* — quer dizer
//! **instantâneo**. É a mesma convenção que o `step_height` do `CharacterParams`
//! desta casa já declara por escrito (*«Zero DESLIGA»*), e a alternativa seria um
//! componente que, nos defaults, não anda.
//!
//! ⭐ Os dois nascem a zero: a resposta imediata é o que um jogo de vista de cima
//! quer no primeiro clique (Zelda, sokoban, twin-stick), e quem quer peso escreve
//! o número.

use crate::{Vec2, len};

/// **A porta**: velocidade de agora + direcção de mundo ⇒ velocidade nova.
///
/// `dir_mundo` já vem quantizada e reprojectada (`crate::world_direction`), com
/// comprimento em `[0, 1]` — é ele que faz um manípulo a meio curso andar a meia
/// velocidade.
#[must_use]
pub fn advance(v: Vec2, dir_mundo: Vec2, speed: f32, accel: f32, decel: f32, dt: f32) -> Vec2 {
    if !dt.is_finite() || dt <= 0.0 {
        return v;
    }
    let pedido = len(dir_mundo);
    if pedido < 1.0e-6 {
        return aproximar(v, [0.0, 0.0], decel, dt);
    }
    let alvo = [dir_mundo[0] * speed, dir_mundo[1] * speed];
    aproximar(v, alvo, accel, dt)
}

/// Anda de `de` para `ate`, no máximo `taxa · dt`. ⚠️ `taxa <= 0` é **chegar já**.
fn aproximar(de: Vec2, ate: Vec2, taxa: f32, dt: f32) -> Vec2 {
    if !(taxa.is_finite() && taxa > 0.0) {
        return ate;
    }
    let delta = [ate[0] - de[0], ate[1] - de[1]];
    let falta = len(delta);
    let passo = taxa * dt;
    if falta <= passo || falta < 1.0e-9 {
        return ate;
    }
    [de[0] + delta[0] / falta * passo, de[1] + delta[1] / falta * passo]
}
