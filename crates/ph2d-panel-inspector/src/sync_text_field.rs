//! **A SEMENTE de um campo de texto do Inspector** — a porta única, e o ficheiro que ela habita.
//!
//! ⚠️ **Irmão do [`super::sync_sections`] por CAP de FICHEIRO**: pôr a porta lá levava-o a `607`
//! contra `600`. ⛔ *A cura de um tecto é o CORTE* — e aqui ele cai sozinho, porque isto não é o
//! `sync` de secção nenhuma: é o primitivo que três delas usam.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;

/// **Escreve um campo de texto do Inspector, menos o que está em FOCO** — ele é do dedo, e
/// reescrevê-lo enquanto se digita apagaria a letra.
///
/// ⚠️ **`pub(crate)` e UMA porta desde o TOP-20 #18:** ela vivia em DUAS cópias (o cérebro e o
/// script) e a terceira secção com campos de texto ia escrever a terceira — *três respostas à
/// mesma pergunta divergem no dia em que uma delas aprende alguma coisa*, e o que se perde aqui é
/// precisamente a cerca do foco, que é invisível até alguém estar a digitar.
pub(crate) fn escreve_texto(
    host: &mut dyn PanelHostInternal,
    focus: Option<ph2d_a11y::NodeId>,
    id: ph2d_a11y::NodeId,
    value: &str,
) {
    if focus == Some(id) {
        return;
    }
    if let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        ..
    }) = host.store_mut().get_mut(id)
    {
        text.clear();
        text.push_str(value);
        *caret = text.len();
        *selection_anchor = None;
    }
}
