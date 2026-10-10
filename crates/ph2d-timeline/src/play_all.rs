//! ⭐ **Play All** (onda 4, item 1 — pedido do dono no smoke de 09/10: *«se o Arrange ainda não tem
//! nada, como pode tocar alguma coisa? Um segundo Play na Timeline para Play All»*).
//!
//! O Play de sempre toca o que a aba mostra (Keys: o clipe em edição; Arrange: só o arranjado). O
//! Play All é um MODO (2.º smoke do dono, 09/10: *«ao apertar Play, o Play All é desmarcado»* era o
//! defeito): marcado, a CENA inteira toca — cada objecto a Animação dele (a entrada, ou o clipe
//! pedido), o Arrange (ou, vazio, o clipe da Cena) e as cutscenes — em qualquer aba, no relógio da
//! CENA, que o Play/Pause/voltar-ao-início comandam como sempre. Só o próprio botão o desmarca.

use ph2d_core::Playhead;

use crate::{TimelineIntent, TimelineState};

/// Drena os pedidos de Play All deste quadro (antes de a shell escolher o relógio da vista): marcar
/// põe o relógio da CENA a tocar; desmarcar pausa-o. O Pause não o desmarca.
pub fn take_play_all(
    state: &mut TimelineState,
    intents: &mut Vec<TimelineIntent>,
    scene: &mut Playhead,
) {
    let mut pedidos = 0usize;
    intents.retain(|i| {
        let e = matches!(i, TimelineIntent::PlayAll);
        pedidos += usize::from(e);
        !e
    });
    if pedidos % 2 == 1 {
        state.play_all = !state.play_all;
        if state.play_all {
            scene.play();
        } else {
            scene.pause();
        }
    }
}

#[cfg(test)]
#[path = "play_all_tests.rs"]
mod tests;
