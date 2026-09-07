//! ⭐⭐⭐ **A PARTIDA DO FOCO — uma lei, dois chamadores.**
//!
//! # O report
//!
//! Enio, 2026-09-07, a esculpir com o pincel de tecido: *«a tecla del para
//! deletar o mesh parou de funcionar e não temos undo/redo para Cloth»*.
//!
//! # ⛔ Os dois relatos são UM defeito, e ele não é do tecido
//!
//! Cada fileira de slider do painel de escultura regista um **chip numérico**
//! ([`crate::interaction::InteractiveState::NumberInput`]) ao lado do cursor —
//! são 37 deles, e o pincel de tecido acrescentou cinco. Tocar num chip põe o
//! foco do teclado nele, como em todo campo deste app.
//!
//! O que faltava era a metade de sair. O `sculpt3d_pointer_down` da shell
//! **toma** o clique no barro e devolve `return` **antes** do `forward_to_hero`
//! — logo o bloco de partida de foco que vive no [`super::pointer_down`] nunca
//! corre, e `focus_id` fica preso naquele chip **para o resto da sessão**.
//! A partir daí o `sculpt3d_key` recusa na primeira linha (`text_entry_focused`)
//! e morrem, de uma vez: `Delete`, `Ctrl+Z`, `Ctrl+Shift+Z` e **todo** atalho da
//! cena 3D. O artista lê isso como *«o Del parou»* e *«o tecido não desfaz»*,
//! que são as duas teclas que ele de facto usa.
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
/// chegar ao despachante — a cena 3D de escultura, a janela de modelagem, a alça
/// do gizmo de âncora. Devolve os eventos emitidos, que o chamador drena como
/// draina os do ponteiro.
///
/// ⚠️ **Não é «cancelar»**: um número digitado e não confirmado é
/// **comprometido**, exactamente como quando o clique cai em espaço morto.
pub fn blur_focus<'frame>(store: &mut WidgetStore, arena: &'frame Bump) -> &'frame [WidgetEvent] {
    let mut events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    depart_focus(store, None, &mut events);
    events.into_bump_slice()
}
