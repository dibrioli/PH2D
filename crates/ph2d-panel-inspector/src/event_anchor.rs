//! **O despacho da §12 Sockets / Named Anchors** ([ADR-0072]).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_slice`].
//!
//! # A seleção de linha NÃO vai ao barramento
//!
//! Clicar numa linha da lista muda **estado do painel** (`InspectorState::anchor_selected`) e
//! consome o evento. Publicá-la como ação obrigaria a shell a saber qual ficha está aberta — um
//! facto que só a UI tem — e faria toda troca de linha custar um quadro. É a mesma fronteira
//! que separa «que painel está visível» de «o que a cena contém».
//!
//! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::AnchorFieldEdit;
use ph2d_editor_core::widget::{ButtonState, CheckboxValue};

use crate::state::{self, InspectorState};

/// Despacha um evento da §12. `true` = consumido.
pub(crate) fn apply_anchor_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> bool {
    let Some(info) = state::current_inspector_anchor() else {
        return false;
    };
    let sel = panel.anchor_selected.min(info.rows.len().saturating_sub(1));
    let sel_u8 = u8::try_from(sel).unwrap_or(0);

    if let WidgetEvent::Click(id) = ev {
        // Uma linha da lista: só muda a ficha aberta.
        if let Some(i) = ids::INSP_ANCHOR_ROW.iter().position(|&o| o == id)
            && i < info.rows.len()
        {
            panel.anchor_selected = i;
            demote_button(host, id);
            return true;
        }
        if id == ids::INSP_ANCHOR_ADD {
            push(host, info.entity_bits, AnchorFieldEdit::Add);
            demote_button(host, id);
            return true;
        }
        if id == ids::INSP_ANCHOR_REMOVE && !info.rows.is_empty() {
            push(host, info.entity_bits, AnchorFieldEdit::Remove(sel_u8));
            demote_button(host, id);
            return true;
        }
    }

    // Sem âncoras não há nada que os campos abaixo possam editar.
    if info.rows.is_empty() {
        return false;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == ids::INSP_ANCHOR_NAME
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(
            host,
            info.entity_bits,
            AnchorFieldEdit::Rename(sel_u8, text),
        );
        return true;
    }

    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_ANCHOR_BOUNDS_ON | ids::INSP_ANCHOR_CENTER_ON)
    {
        let on = matches!(
            host.store().checkbox(id).map(|(_, v)| v),
            Some(CheckboxValue::Checked)
        );
        let edit = if id == ids::INSP_ANCHOR_BOUNDS_ON {
            AnchorFieldEdit::BoundsOn(sel_u8, on)
        } else {
            AnchorFieldEdit::CenterOn(sel_u8, on)
        };
        push(host, info.entity_bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0) as f32;
        // ⚠️ Cada campo despacha SÓ o seu eixo — a lei do `PerCornerTintAt`.
        let edit = ids::INSP_ANCHOR_POS
            .iter()
            .position(|&o| o == id)
            .map(|i| AnchorFieldEdit::Pos(sel_u8, i as u8, v))
            .or_else(|| (id == ids::INSP_ANCHOR_ROT).then_some(AnchorFieldEdit::Rot(sel_u8, v)))
            .or_else(|| {
                ids::INSP_ANCHOR_BOUNDS
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| AnchorFieldEdit::Bounds(sel_u8, i as u8, v))
            })
            .or_else(|| {
                ids::INSP_ANCHOR_CENTER
                    .iter()
                    .position(|&o| o == id)
                    .map(|i| AnchorFieldEdit::Center(sel_u8, i as u8, v))
            });
        if let Some(edit) = edit {
            push(host, info.entity_bits, edit);
            return true;
        }
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: AnchorFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorAnchorEdit { entity_bits, edit });
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote_button(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
