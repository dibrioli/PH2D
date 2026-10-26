//! **Os controlos do USO da imagem** em Texture & Atlas (planos GEF 13 D5 e 14 D5): Make it Animated, a caixa
//! Tiled e Width/Height — registo, espelho do snapshot e despacho, num sítio só. Irmão do [`crate::event`] pelo
//! tecto de LOC.

use crate::{ids, state};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::{ButtonState, CheckboxState, CheckboxValue, TextInputState};
use ph2d_editor_core::{ImageUseEdit, InspectorDraw, InspectorImageUse, InspectorSpriteInfo};
use std::cell::Cell;

/// O passo de arrasto de Width/Height (m).
const PASSO_M: f64 = 0.1;
/// O menor lado (m): o mesmo da lei (`ph2d_app_sprite_anim::tiled::MIN_SIDE_M`, que o painel não importa).
const MIN_M: f64 = 0.01;
/// O maior lado (m): dez quilómetros, muito além de qualquer nível.
const MAX_M: f64 = 1.0e4;

pub(crate) fn populate(store: &mut WidgetStore) {
    store.register(
        ids::INSP_IMAGE_MAKE_ANIMATED,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    store.register(
        ids::INSP_IMAGE_TILED,
        InteractiveState::Checkbox {
            state: CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    for id in ids::INSP_IMAGE_TILED_SIZE {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 1.0,
                buffer: "1".to_owned(),
                caret: 0,
                last_committed: 1.0,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, MIN_M, MAX_M, PASSO_M);
    }
}

thread_local! {
    /// O `(entidade, tiled)` publicado da última vez: a caixa só se re-semeia quando o MUNDO muda (troca de
    /// objecto, desfazer) — re-semear sempre desfaria o clique antes de o commit chegar ao snapshot.
    static VISTO: Cell<Option<(u64, bool)>> = const { Cell::new(None) };
}

pub(crate) fn sync(host: &mut dyn PanelHostInternal, sp: &InspectorSpriteInfo) {
    let tiled = matches!(sp.draw, InspectorDraw::Tiled { .. });
    if VISTO.with(|v| v.replace(Some((sp.entity_bits, tiled)))) != Some((sp.entity_bits, tiled))
        && let Some(InteractiveState::Checkbox { value, .. }) =
            host.store_mut().get_mut(ids::INSP_IMAGE_TILED)
    {
        *value = if tiled {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        };
    }
    let InspectorDraw::Tiled { size } = sp.draw else {
        return;
    };
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, v) in ids::INSP_IMAGE_TILED_SIZE.into_iter().zip(size) {
        if focus != Some(id) && drag != Some(id) {
            host.store_mut().set_number_value(id, f64::from(v));
        }
    }
}

/// Despacha um evento destes controlos. `true` = consumido.
pub(crate) fn event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = state::current_inspector_sprite() else {
        return false;
    };
    let edit = match ev {
        // Só onde o aviso se pinta (uso repetido, imagem com células).
        WidgetEvent::Click(id)
            if id == ids::INSP_IMAGE_MAKE_ANIMATED
                && info.image_use == InspectorImageUse::Repeated
                && info.image_cells > 1 =>
        {
            ImageUseEdit::MakeAnimated
        }
        WidgetEvent::Toggled(id)
            if id == ids::INSP_IMAGE_TILED && info.draw != InspectorDraw::Fixed =>
        {
            ImageUseEdit::Tiled(matches!(
                host.store().checkbox(id).map(|(_, v)| v),
                Some(CheckboxValue::Checked)
            ))
        }
        WidgetEvent::ValueChanged(id) if matches!(info.draw, InspectorDraw::Tiled { .. }) => {
            let Some(axis) = ids::INSP_IMAGE_TILED_SIZE.iter().position(|&o| o == id) else {
                return false;
            };
            let Some(m) = host.store().number_value(id) else {
                return false;
            };
            let m = m as f32;
            if axis == 0 {
                ImageUseEdit::Width(m)
            } else {
                ImageUseEdit::Height(m)
            }
        }
        _ => return false,
    };
    host.bus_mut().push(EditorAction::ImageUseEdit {
        entity_bits: info.entity_bits,
        edit,
    });
    true
}
