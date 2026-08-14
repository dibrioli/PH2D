//! **O BANCO DE ENSAIO da caminhada** — a porta única por onde os gates do freio
//! (`W-Brake`) e da superfície (`W-Surface`) dirigem o produto.
//!
//! ⚠️ **Ele existe porque a segunda wave precisava do mesmo laço**, e um
//! integrador copiado é a forma exacta como duas medições do mesmo produto
//! passam a discordar: bastaria uma das cópias esquecer de forçar o
//! `grounded`, ou de somar o `delta` que a porta devolve, para uma tabela
//! publicar números de uma física que ninguém shipa.
//!
//! ⚠️ **A porta do PRODUTO, e não um integrador escrito aqui:** o laço chama
//! [`crate::player_motor`] e [`crate::kinematic_advance`] — as duas funções que a
//! ponte atravessa. Sobre chão plano o modo cinemático absorve a gravidade, então
//! o que sobra no eixo é exactamente `v += accel·dt + boost`, a mesma aritmética
//! que a ponte dinâmica faz do outro lado da cerca.
//!
//! ⚠️ **Módulo FILHO** (via `#[path]`), como os dois que o consomem — é isso que
//! deixa `pub(super)` alcançá-los sem abrir nada para fora do crate.

use super::*;
use crate::{KinematicState, PlayerConfig, PlayerInput, PlayerState, Support, kinematic_advance};

pub(super) const UP: Vec2 = [0.0, 1.0];
pub(super) const G: Vec2 = [0.0, -9.81];
pub(super) const DT: f32 = 1.0 / 60.0;

/// Abaixo disto o personagem está **parado** — um milímetro por segundo, a mesma
/// ordem do `normalized_allowed_linear_error` com que o solver assenta.
pub(super) const STILL: f32 = 1.0e-3;

/// Chão plano, imóvel, com o `grip` neutro — a fixture de partida.
pub(super) fn flat_at(distance: f32) -> GroundSample {
    GroundSample {
        distance,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
        grip: GroundSample::NEUTRAL_GRIP,
    }
}

/// O que uma corrida deixou.
pub(super) struct Run {
    /// Distância percorrida no eixo `x`, em metros, ao fim dos tiques pedidos.
    pub travelled: f32,
    /// A velocidade no eixo `x` no fim.
    pub velocity: f32,
    /// O primeiro tique em que `|v| < STILL` — `None` se nunca aconteceu, que é
    /// a resposta HONESTA para uma superfície que não segura nada.
    pub ticks_to_still: Option<u32>,
    /// Quanto tinha percorrido nesse instante (zero se nunca parou).
    pub travelled_when_still: f32,
}

/// **Dirige o produto** por `ticks` tiques, a partir de `v0` no eixo, com o dedo
/// em `input` (`0.0` = solto), sobre `ground`.
pub(super) fn drive_for(
    cfg: &PlayerConfig,
    ground: &GroundSample,
    v0: f32,
    input: f32,
    ticks: u32,
) -> Run {
    let mut state = KinematicState {
        velocity: [v0, 0.0],
        grounded: true,
    };
    let mut travelled = 0.0_f32;
    let mut ticks_to_still = None;
    let mut travelled_when_still = 0.0_f32;

    for tick in 1..=ticks {
        let step = crate::player_motor(
            cfg,
            Some(ground),
            None,
            None,
            None,
            None,
            PlayerInput {
                drive: input,
                ..PlayerInput::default()
            },
            PlayerState::default(),
            state.velocity,
            G,
            UP,
            DT,
            crate::Buoyed::DRY,
            Support::Snap,
        );
        let (next, delta) = kinematic_advance(
            state,
            step.motor,
            Some(ground),
            G,
            UP,
            DT,
            crate::Fluid::DRY,
        );
        // ⚠️ **O `grounded` é forçado**: o banco mede a caminhada, e um tique em
        // que a perna "solta" trocaria o orçamento do chão pelo do ar no meio da
        // medição — a tabela passaria a descrever duas leis somadas.
        state = KinematicState {
            grounded: true,
            ..next
        };
        travelled += delta[0];
        if ticks_to_still.is_none() && state.velocity[0].abs() < STILL {
            ticks_to_still = Some(tick);
            travelled_when_still = travelled;
        }
    }

    Run {
        travelled,
        velocity: state.velocity[0],
        ticks_to_still,
        travelled_when_still,
    }
}
