//! **O despacho da secção TIMERS** (TOP-20 #2, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_anim`].
//!
//! # ⚠️ Clicar numa linha NÃO vai ao barramento — ao contrário da §11
//!
//! Aqui a §12 é o precedente: qual timer se edita é um facto da UI e fica no painel. Na §11 a
//! linha aberta **é** a animação que toca, e isso é estado da cena. Um `Timers` não tem «o timer
//! actual» — os N correm todos ao mesmo tempo —, logo publicar a escolha faria um passo de undo
//! por clique sobre um facto que a cena nem tem onde guardar.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::TimerFieldEdit;
use ph2d_editor_core::widget::ButtonState;

use crate::state::{self, InspectorState};

/// Despacha um evento da secção TIMERS. `true` = consumido.
pub(crate) fn apply_timer_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = state::current_inspector_timer() else {
        return false;
    };
    let sel = panel.timer_selected.min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        // Uma linha da lista: abre a ficha, e **só** isso.
        if let Some(i) = ids::INSP_TIMER_ROW.iter().position(|&o| o == id)
            && i < info.rows.len()
        {
            panel.timer_selected = i;
            demote(host, id);
            return true;
        }
        if id == ids::INSP_TIMER_ADD {
            push(host, info.entity_bits, TimerFieldEdit::Add);
            // ⚠️ **Abre o que acabou de nascer.** Sem isto o artista carrega em `+`, a lista
            // cresce, e o editor continua a mostrar o timer anterior — que se lê como o botão
            // não ter feito nada.
            panel.timer_selected = info.rows.len();
            demote(host, id);
            return true;
        }
        if id == ids::INSP_TIMER_REMOVE && !info.rows.is_empty() {
            push(host, info.entity_bits, TimerFieldEdit::Remove(sel_u8));
            // ⚠️ **Recua um** — senão o índice aberto passa a apontar para além do fim, e o editor
            // desaparece sem o artista ter pedido nada.
            panel.timer_selected = sel.saturating_sub(1);
            demote(host, id);
            return true;
        }
    }

    // ⚠️ **O clique afirma o CONTRÁRIO do que está no ecrã**, e o que está no ecrã vem do
    // SNAPSHOT — nunca do store, que guarda o visual. É a lei que a §11 pagou com um report:
    // ler o store fazia o primeiro clique depois de trocar de objecto mandar o valor do objecto
    // anterior.
    if let WidgetEvent::Toggled(id) = ev
        && !info.rows.is_empty()
        && matches!(id, ids::INSP_TIMER_REPEAT | ids::INSP_TIMER_AUTOSTART)
    {
        let edit = if id == ids::INSP_TIMER_REPEAT {
            TimerFieldEdit::Repeat(sel_u8, !info.rows[sel].repeat)
        } else {
            TimerFieldEdit::Autostart(sel_u8, !info.rows[sel].autostart)
        };
        push(host, info.entity_bits, edit);
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && !info.rows.is_empty()
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        // ⚠️ **Os dois campos de texto por uma porta só.** Um `if` por campo é como o segundo
        // acaba a chamar o `Rename` do primeiro — e renomear um timer por escrever um nome de
        // sinal é o defeito que não se vê até o projeto reabrir.
        let edit = if id == ids::INSP_TIMER_NAME {
            TimerFieldEdit::Rename(sel_u8, text)
        } else if id == ids::INSP_TIMER_SIGNAL {
            TimerFieldEdit::Signal(sel_u8, text)
        } else {
            return false;
        };
        push(host, info.entity_bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev
        && id == ids::INSP_TIMER_DURATION
        && !info.rows.is_empty()
    {
        let v = host.store().number_value(id).unwrap_or(0.0);
        // ⚠️ **SEGUNDOS saem daqui**; quem converte para microssegundos é a shell, que é a única
        // que conhece o teto do motor. Duas conversões dariam duas respostas a *«quanto é 1,5 s?»*,
        // e a que o artista vê seria a que envelhece.
        //
        // ⚠️ O `max(0.0)` mora aqui, na entrada: uma duração negativa não existe, e o commit tem
        // de receber sempre um número honesto.
        #[allow(clippy::cast_possible_truncation)]
        let secs = v.max(0.0) as f32;
        push(
            host,
            info.entity_bits,
            TimerFieldEdit::DurationSecs(sel_u8, secs),
        );
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: TimerFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorTimerEdit { entity_bits, edit });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
