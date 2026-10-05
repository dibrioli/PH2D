//! ⭐⭐⭐ **A PARTIDA DO FOCO — uma lei, dois chamadores.**
//!
//! # O defeito
//!
//! Um campo de painel com chip numérico ([`crate::interaction::InteractiveState::NumberInput`])
//! toma o foco do teclado ao toque, como em todo campo deste app. O que faltava era a metade de
//! sair: um consumidor de canvas que **toma** o clique e devolve `return` **antes** do
//! `forward_to_hero` da shell faz com que o bloco de partida de foco do [`super::pointer_down`]
//! nunca corra, e o `focus_id` fica preso naquele chip **para o resto da sessão** — e com ele
//! morrem `Delete`, `Ctrl+Z` e todo atalho do canvas. (Achado em 2026-09-07 no módulo 3D, que saiu
//! com o ADR-0179; a lei é de todo consumidor que devolve cedo.)
//!
//! *Uma porta que toma o gesto herda TODAS as obrigações da porta que ela
//! saltou — e a que se esquece é sempre a que não se vê acontecer.*
//!
//! # ⚠️ Por que uma PORTA, e não a mesma meia-dúzia de linhas repetida
//!
//! A partida de foco não é `set_focus(None)`: ela **compromete** o buffer
//! numérico e o hexadecimal (senão o número que o artista digitou e não
//! confirmou evapora em silêncio), repõe o visual do widget (senão ele fica a
//! desenhar o cursor de texto sem ter o teclado) e emite o [`WidgetEvent::Blur`]
//! que o painel escuta. Cinco passos com ordem — copiá-los para o segundo
//! chamador é como as duas cópias começam a divergir, e a que diverge é a que
//! não tem gate.

use super::{commit_hex_buffer, commit_number_buffer, reset_focused_visual_state};
use crate::interaction::{WidgetEvent, WidgetStore};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;

/// **O foco parte do widget que o tinha** — comprometendo o que estava por
/// confirmar. `keep` é o foco NOVO: quando ele é o mesmo widget, não há partida.
///
/// ⚠️ **Idempotente por construção**: sem foco, ou com o mesmo foco, é um no-op
/// exacto. É isso que deixa a shell chamá-la à frente de um consumidor de canvas
/// sem duplicar o trabalho que o `dispatch_down` faria a seguir.
pub(super) fn depart_focus<'a>(
    store: &mut WidgetStore,
    keep: Option<ph2d_a11y::NodeId>,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    let Some(old) = store.focus_id() else {
        return;
    };
    if keep == Some(old) {
        return;
    }
    commit_number_buffer(store, old, events, false);
    commit_hex_buffer(store, old, events);
    reset_focused_visual_state(store, old);
    events.push(WidgetEvent::Blur(old));
    store.set_focus(None);
}

/// **A porta pública: largue o teclado que um campo estava a segurar.**
///
/// Para quem **toma** um gesto de canvas e devolve cedo, sem deixar o evento
/// chegar ao despachante — a alça do gizmo de âncora, por exemplo. Devolve os eventos emitidos, que o chamador drena como
/// draina os do ponteiro.
///
/// ⚠️ **Não é «cancelar»**: um número digitado e não confirmado é
/// **comprometido**, exactamente como quando o clique cai em espaço morto.
pub fn blur_focus<'frame>(store: &mut WidgetStore, arena: &'frame Bump) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    depart_focus(store, None, &mut events);
    events.into_bump_slice()
}
