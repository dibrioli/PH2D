//! **O despacho da secção COUNTER WATCH.**
//!
//! ⚠️ **Clicar numa linha NÃO vai ao barramento** — qual regra se edita é um facto da UI e fica no
//! painel, a lei que o `Timers` escreveu: as N regras avaliam todas ao mesmo tempo, logo a cena não
//! tem onde guardar *«a regra actual»* e publicá-la seria um passo de undo por clique.
//!
//! ⚠️ **O valor que uma edição afirma vem do SNAPSHOT, nunca do store** — ler o store faz o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto anterior.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::counter_watch_edits::CounterWatchFieldEdit as E;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::ButtonState;

use crate::state::InspectorState;

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_counter_watch_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = crate::state_components::current_inspector_counter_watch() else {
        return false;
    };
    let bits = info.entity_bits;
    let sel = panel.watch_selected.min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        if let Some(i) = crate::ids::INSP_WATCH_ROW.iter().position(|&o| o == id)
            && i < info.rows.len()
        {
            panel.watch_selected = i;
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_WATCH_ADD {
            push(host, bits, E::Add);
            // ⚠️ **Abre a que acabou de nascer** — senão o `+` parece não ter feito nada.
            panel.watch_selected = info.rows.len();
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_WATCH_REMOVE && !info.rows.is_empty() {
            push(host, bits, E::Remove(sel_u8));
            // ⚠️ **Recua um** — senão o índice aberto aponta para além do fim e o editor some.
            panel.watch_selected = sel.saturating_sub(1);
            demote(host, id);
            return true;
        }
        // ⚠️ **A posição na tabela de ids É o valor do enum** — a lei que o array declara, com
        // gate na shell a prendê-la à ordem do `ph2d_ecs::Compare`.
        if let Some(i) = crate::ids::INSP_WATCH_CMP_OPT.iter().position(|&o| o == id)
            && !info.rows.is_empty()
        {
            push(host, bits, E::Compare(sel_u8, u8::try_from(i).unwrap_or(0)));
            fecha(host);
            demote(host, id);
            return true;
        }
    }

    if let WidgetEvent::Toggled(id) = ev
        && id == crate::ids::INSP_WATCH_ONCE
        && !info.rows.is_empty()
    {
        push(host, bits, E::Once(sel_u8, !info.rows[sel].once));
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && !info.rows.is_empty()
    {
        let texto = host.store().text(id).unwrap_or("").to_string();
        // ⚠️ **Os dois campos por uma porta só** — um `if` por campo é como o segundo acaba a
        // escrever no primeiro, e trocar o nome do contador por um nome de sinal é o defeito que
        // não se vê até o projecto reabrir.
        let edit = if id == crate::ids::INSP_WATCH_COUNTER {
            E::Counter(sel_u8, texto)
        } else if id == crate::ids::INSP_WATCH_SIGNAL {
            E::Signal(sel_u8, texto)
        } else {
            return false;
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev
        && id == crate::ids::INSP_WATCH_VALUE
        && !info.rows.is_empty()
    {
        let v = host.store().number_value(id).unwrap_or(0.0);
        // ⚠️ **O limiar é INTEIRO porque o contador é** — a lei do `Counter`: *somar `0,1`
        // trezentas vezes em vírgula flutuante não dá `30`*. A truncagem mora na entrada, para o
        // commit receber sempre um número honesto.
        #[allow(clippy::cast_possible_truncation)]
        let n = v.trunc() as i64;
        push(host, bits, E::Value(sel_u8, n));
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut()
        .push(EditorAction::InspectorCounterWatchEdit { entity_bits, edit });
}

/// Fecha o popover depois de uma escolha.
fn fecha(host: &mut dyn PanelHostInternal) {
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(crate::ids::INSP_WATCH_CMP_PICK)
    {
        *open = false;
    }
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
