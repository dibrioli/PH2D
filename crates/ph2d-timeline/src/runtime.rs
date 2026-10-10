//! ⭐ **A cena a CORRER** (onda 4, item 1 — dono no 3.º smoke de 09/10: *«na Godot não é necessário
//! Play All: os marcados com Autoplay tocam no Play do runtime, o Play da barra superior»*).
//!
//! O **Play da barra superior** (e `Run ▸ Play`) põe a cena a correr, como o jogo: cada objecto a
//! Animação dele (a entrada, ou o clipe pedido), o Arrange da Cena — ou, vazio, o clipe dela — e as
//! cutscenes, em qualquer aba, no relógio da CENA. O Pause de cima pausa sem sair; o Reset de cima
//! sai. O **transporte da timeline** é de EDIÇÃO (toca o que a aba mostra): usá-lo sai da corrida —
//! o relógio da cena pára, e a vista volta a ser a da aba.

use ph2d_core::Playhead;

use crate::{TimelineIntent, TimelineState};

/// Drena os pedidos da corrida deste quadro (antes de a shell escolher o relógio da vista):
/// `RunScene` liga-a e toca o relógio da cena; `StopScene` desliga-a; um Play/Pause do transporte da
/// timeline com ela ligada desliga-a e pára o relógio da cena — e segue para a vista, como sempre.
pub fn take_runtime(
    state: &mut TimelineState,
    intents: &mut Vec<TimelineIntent>,
    scene: &mut Playhead,
) {
    let mut liga = None;
    intents.retain(|i| match i {
        TimelineIntent::RunScene => {
            liga = Some(true);
            false
        }
        TimelineIntent::StopScene => {
            liga = Some(false);
            false
        }
        _ => true,
    });
    match liga {
        Some(true) => {
            state.runtime = true;
            scene.play();
        }
        Some(false) => state.runtime = false,
        None => {}
    }
    let edita = intents.iter().any(|i| {
        matches!(
            i,
            TimelineIntent::Play | TimelineIntent::Pause | TimelineIntent::TogglePlay
        )
    });
    if state.runtime && edita {
        state.runtime = false;
        scene.pause();
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
