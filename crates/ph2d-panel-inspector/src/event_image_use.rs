//! **Os controlos do USO da imagem** sob a linha Image (planos GEF 14 D5 e 15 D1–D3): o seletor «Use», as células
//! da miniatura da folha e Width/Height — registo, espelho do snapshot e despacho, num sítio só. Irmão do
//! [`crate::event`] pelo tecto de LOC.

use crate::{ids, state};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::{
    ImageUseEdit, InspectorDraw, InspectorImageUse, InspectorSpriteInfo, SpriteFieldEdit,
};

/// O passo de arrasto de Width/Height (m).
const PASSO_M: f64 = 0.1;
/// O menor lado (m): o mesmo da lei (`ph2d_app_sprite_anim::tiled::MIN_SIDE_M`, que o painel não importa).
const MIN_M: f64 = 0.01;
/// O maior lado (m): dez quilómetros, muito além de qualquer nível.
const MAX_M: f64 = 1.0e4;

pub(crate) fn populate(store: &mut WidgetStore) {
    crate::populate::register_button_ids(store, &ids::INSP_IMAGE_USE);
    crate::populate::register_button_ids(store, &ids::INSP_IMAGE_FRAME_STEP);
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

/// Quantas células a folha de `sp` tem.
pub(crate) fn sheet_cells(sp: &InspectorSpriteInfo) -> u32 {
    sp.hframes.max(1).saturating_mul(sp.vframes.max(1))
}

/// Uma camada de fundo repete sempre (`ScrollRepeat`, outra lei): o seletor não se lhe oferece.
pub(crate) fn use_is_fixed(sp: &InspectorSpriteInfo) -> bool {
    sp.image_use == InspectorImageUse::Repeated && sp.draw == InspectorDraw::Fixed
}

pub(crate) fn sync(host: &mut dyn PanelHostInternal, sp: &InspectorSpriteInfo) {
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
    let entity_bits = info.entity_bits;
    let action = match ev {
        WidgetEvent::Click(id) if ids::INSP_IMAGE_USE.contains(&id) => {
            let i = ids::INSP_IMAGE_USE.iter().position(|&o| o == id);
            let to = i.map_or(info.image_use, |i| InspectorImageUse::ALL[i]);
            if to == info.image_use || use_is_fixed(&info) {
                return true;
            }
            EditorAction::ImageUseEdit {
                entity_bits,
                edit: ImageUseEdit::Use(to),
            }
        }
        // «‹ ›» trocam o quadro de uma parada numa folha, em roda.
        WidgetEvent::Click(id)
            if ids::INSP_IMAGE_FRAME_STEP.contains(&id)
                && info.image_use == InspectorImageUse::Still
                && sheet_cells(&info) > 1 =>
        {
            let n = sheet_cells(&info);
            let passo = if id == ids::INSP_IMAGE_FRAME_STEP[0] {
                n - 1
            } else {
                1
            };
            EditorAction::InspectorSpriteEdit {
                entity_bits,
                edit: SpriteFieldEdit::Frame((info.frame.min(n - 1) + passo) % n),
            }
        }
        WidgetEvent::ValueChanged(id) if matches!(info.draw, InspectorDraw::Tiled { .. }) => {
            let Some(axis) = ids::INSP_IMAGE_TILED_SIZE.iter().position(|&o| o == id) else {
                return false;
            };
            let Some(m) = host.store().number_value(id) else {
                return false;
            };
            let m = m as f32;
            EditorAction::ImageUseEdit {
                entity_bits,
                edit: if axis == 0 {
                    ImageUseEdit::Width(m)
                } else {
                    ImageUseEdit::Height(m)
                },
            }
        }
        _ => return false,
    };
    host.bus_mut().push(action);
    true
}
