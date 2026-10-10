//! ⭐ **Play All** (onda 4, item 1 — pedido do dono no smoke de 09/10: *«se o Arrange ainda não tem
//! nada, como pode tocar alguma coisa? Um segundo Play na Timeline para Play All»*).
//!
//! O Play de sempre toca o que a aba mostra (Keys: o clipe em edição; Arrange: só o arranjado). O
//! Play All toca a CENA inteira, como no jogo: cada objecto a Animação dele (a entrada, ou o clipe
//! pedido), o Arrange da Cena e as cutscenes — em qualquer aba, no relógio da CENA. Acaba quando esse
//! relógio pára (o mesmo botão, o Pause, o Play normal que o pausa).

use ph2d_core::Playhead;

use crate::{TimelineIntent, TimelineState};

/// Drena os pedidos de Play All deste quadro (antes de a shell escolher o relógio da vista) e
/// mantém o estado: ligar põe o relógio da CENA a tocar; pedir com ele a correr desliga-o e pausa; o
/// relógio parado por outra via acaba-o.
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
        if state.play_all && scene.is_playing() {
            scene.pause();
            state.play_all = false;
        } else {
            state.play_all = true;
            scene.play();
        }
    }
    if state.play_all && !scene.is_playing() {
        state.play_all = false;
    }
}

#[cfg(test)]
#[path = "play_all_tests.rs"]
mod tests;
