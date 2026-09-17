//! **O despacho da secção PARTICLES** (TOP-20 #18, W3).
//!
//! ⚠️ **A linha `i` edita o campo `i`** — a ordem sai do modelo (`PARTICLES_NUMBERS` /
//! `PARTICLES_TEXTS`), nunca de uma segunda lista escrita aqui.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::particles_edits::{
    PARTICLES_NUMBERS, PARTICLES_TEXTS, ParticlesFieldEdit as E,
};
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

/// Despacha um evento da secção. `true` = consumido.
pub(crate) fn apply_particles_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_particles() else {
        return false;
    };
    let bits = info.entity_bits;
    let linha = |tabela: &[ph2d_a11y::NodeId], id| tabela.iter().position(|&o| o == id);

    if let WidgetEvent::Click(id) = ev {
        let edit = if let Some(i) = linha(&crate::ids::INSP_PART_SHAPE, id) {
            u8::try_from(i).ok().map(E::Shape)
        } else if let Some(i) = linha(&crate::ids::INSP_PART_SPACE, id) {
            u8::try_from(i).ok().map(E::Space)
        } else if id == crate::ids::INSP_PART_COLOR || id == crate::ids::INSP_PART_COLOR_END {
            // ⭐ Uma amostra ABRE o selector partilhado, semeado com a cor do documento — a mesma
            // porta da secção Color & Tint. A escolha volta pelo `widget_color`, e é a semente
            // (`sync_particles`) que a devolve ao documento.
            let fim = id == crate::ids::INSP_PART_COLOR_END;
            let atual = if fim { info.color_end } else { info.color };
            let seed = crate::state_tint::tint_f32_to_u8(atual);
            host.store_mut().set_widget_color(id, seed);
            host.store_mut().set_picker_target(Some(id));
            host.store_mut().set_blender_value(
                ph2d_editor_core::ids::INSP_BLENDER_PICKER,
                ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
            );
            demote(host, id);
            return true;
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
        let on = matches!(
            host.store().get(id),
            Some(InteractiveState::Checkbox {
                value: CheckboxValue::Checked,
                ..
            })
        );
        let edit = if id == crate::ids::INSP_PART_EMITTING {
            E::Emitting(on)
        } else if id == crate::ids::INSP_PART_ONE_SHOT {
            E::OneShot(on)
        } else {
            return false;
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && let Some(i) = linha(&crate::ids::INSP_PART_TEXT, id)
        && let Some(&campo) = PARTICLES_TEXTS.get(i)
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(host, bits, E::Text(campo, text));
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev
        && let Some(i) = linha(&crate::ids::INSP_PART_NUM, id)
        && let Some(&campo) = PARTICLES_NUMBERS.get(i)
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
    host.bus_mut()
        .push(EditorAction::InspectorParticlesEdit { entity_bits, edit });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
