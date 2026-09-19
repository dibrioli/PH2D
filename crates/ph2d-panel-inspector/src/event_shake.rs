//! **O despacho das DUAS secções do ABANÃO** (suplente #25).
//!
//! ⚠️ **Clicar numa fonte NÃO vai ao barramento** — qual fonte se edita é um facto da UI e fica no
//! painel, a lei que o `Timers` escreveu: as N fontes ouvem todas ao mesmo tempo, logo a cena não
//! tem onde guardar *«a fonte actual»* e publicá-la seria um passo de undo por clique.
//!
//! ⚠️ **O valor que uma edição afirma vem do SNAPSHOT, nunca do store** — ler o store faz o primeiro
//! clique depois de trocar de objecto mandar o valor do objecto anterior.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::shake_edits::{EmitterFieldEdit as EE, ShakeFieldEdit as SE};
use ph2d_editor_core::widget::ButtonState;

use crate::state::InspectorState;

/// Despacha um evento da secção **CAMERA SHAKE**. `true` = consumido.
pub(crate) fn apply_shake_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_shake() else {
        return false;
    };
    let bits = info.entity_bits;

    // ⚠️⚠️ **`i + EXPOENTE_MIN` e NUNCA `i`** — a faixa da lei começa em `1`, e mandar o índice cru
    // entregaria um `0`, que é o valor que ela recusa **por apagar o trauma**. Há gate na shell a
    // prender a ida e a volta.
    if let WidgetEvent::Click(id) = ev
        && let Some(i) = crate::ids::INSP_SHAKE_EXPOENTE
            .iter()
            .position(|&o| o == id)
    {
        let n = u8::try_from(i).unwrap_or(0) + ph2d_shake::EXPOENTE_MIN;
        push_shake(host, bits, SE::Expoente(n));
        demote(host, id);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let edit = if id == crate::ids::INSP_SHAKE_AMPLITUDE {
            SE::Amplitude(v as f32)
        } else if id == crate::ids::INSP_SHAKE_FREQUENCIA {
            SE::Frequencia(v as f32)
        } else if id == crate::ids::INSP_SHAKE_DECAIMENTO {
            SE::Decaimento(v as f32)
        } else if id == crate::ids::INSP_SHAKE_SEMENTE {
            // ⚠️ **A semente não tem valor CERTO, só tem de ser DIFERENTE** — a conversão perde os
            // bits acima de `2^53` e isso não custa nada aqui, ao contrário de todo outro `u64`
            // desta casa. O `max(0)` é a cerca do domínio.
            SE::Semente(v.max(0.0) as u64)
        } else {
            return false;
        };
        push_shake(host, bits, edit);
        return true;
    }
    false
}

/// Despacha um evento da secção **SHAKE EMITTER**. `true` = consumido.
pub(crate) fn apply_emitter_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = crate::state_components::current_inspector_emitter() else {
        return false;
    };
    let bits = info.entity_bits;
    let sel = panel
        .emitter_selected
        .min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        if let Some(i) = crate::ids::INSP_EMITTER_ROW.iter().position(|&o| o == id)
            && i < info.rows.len()
        {
            panel.emitter_selected = i;
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_EMITTER_ADD {
            push_emitter(host, bits, EE::Add);
            // ⚠️ **Abre o que acabou de nascer** — senão o `+` parece não ter feito nada.
            panel.emitter_selected = info.rows.len();
            demote(host, id);
            return true;
        }
        if id == crate::ids::INSP_EMITTER_REMOVE && !info.rows.is_empty() {
            push_emitter(host, bits, EE::Remove(sel_u8));
            // ⚠️ **Recua um** — senão o índice aberto aponta para além do fim e o editor some.
            panel.emitter_selected = sel.saturating_sub(1);
            demote(host, id);
            return true;
        }
        // ⚠️ **A posição na tabela de ids É a tag do `SignalFrom`** — com gate na shell.
        if let Some(i) = crate::ids::INSP_EMITTER_DE.iter().position(|&o| o == id)
            && !info.rows.is_empty()
        {
            push_emitter(host, bits, EE::De(sel_u8, u8::try_from(i).unwrap_or(0)));
            demote(host, id);
            return true;
        }
    }

    if info.rows.is_empty() {
        return false;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == crate::ids::INSP_EMITTER_ON
    {
        let texto = host.store().text(id).unwrap_or("").to_string();
        push_emitter(host, bits, EE::On(sel_u8, texto));
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        #[allow(clippy::cast_possible_truncation)]
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        // ⚠️ **Os três campos por uma porta só** — um `if` por campo é como o segundo acaba a
        // escrever no primeiro, e trocar um raio pelo outro é o defeito que só se vê quando a
        // explosão deixa de abanar.
        let edit = if id == crate::ids::INSP_EMITTER_FORCA {
            EE::Forca(sel_u8, v)
        } else if id == crate::ids::INSP_EMITTER_DENTRO {
            EE::Dentro(sel_u8, v)
        } else if id == crate::ids::INSP_EMITTER_FORA {
            EE::Fora(sel_u8, v)
        } else {
            return false;
        };
        push_emitter(host, bits, edit);
        return true;
    }
    false
}

fn push_shake(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: SE) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Shake(edit),
    });
}

fn push_emitter(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: EE) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::ShakeEmitter(edit),
    });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
