//! **O clique e a digitação da secção HUD** (TOP-20 #20) — cada widget vira uma edição no
//! barramento, e mais nada.

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::hud_edits::{HUD_NUMBERS, HUD_TEXTS, HudFieldEdit as E};
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

pub(crate) fn apply_hud_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_hud() else {
        return false;
    };
    let bits = info.entity_bits;
    let linha = |tabela: &[ph2d_a11y::NodeId], id| tabela.iter().position(|&o| o == id);

    if let WidgetEvent::Click(id) = ev {
        let edit = if let Some(i) = linha(&crate::ids::INSP_HUD_FIT, id) {
            u8::try_from(i).ok().map(E::Fit)
        } else if let Some(i) = linha(&crate::ids::INSP_HUD_SOURCE, id) {
            u8::try_from(i).ok().map(E::Source)
        } else {
            return false;
        };
        if let Some(edit) = edit {
            push(host, bits, edit);
        }
        demote(host, id);
        return true;
    }

    if let WidgetEvent::Toggled(id) = ev {
        // ⚠️ **As DUAS caixas desta secção, e elas são de componentes diferentes** — a `Disabled` é
        // do `UiButton` e a `Keep on restart` é do `Counter`. Um `match` com um braço só engoliria
        // o clique da segunda em silêncio, que é o defeito que esta crate já pagou sete vezes.
        let edit = if id == crate::ids::INSP_HUD_DISABLED {
            E::Disabled
        } else if id == crate::ids::INSP_HUD_COUNTER_KEEP {
            E::CounterKeep
        } else {
            return false;
        };
        let on = matches!(
            host.store().get(id),
            Some(InteractiveState::Checkbox {
                value: CheckboxValue::Checked,
                ..
            })
        );
        push(host, bits, edit(on));
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && let Some(i) = linha(&crate::ids::INSP_HUD_TEXT, id)
        && let Some(&campo) = HUD_TEXTS.get(i)
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(host, bits, E::Text(campo, text));
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(i) = linha(&crate::ids::INSP_HUD_NUM, id)
        && let Some(&campo) = HUD_NUMBERS.get(i)
    {
        // ⚠️ O store fala em `f64` e o componente em `f32` — a conversão é AQUI, num sítio só.
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o campo do componente é f32"
        )]
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        push(host, bits, E::Number(campo, v));
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: E) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Hud(edit),
    });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
