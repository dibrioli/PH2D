//! **A semente dos campos da secção MOTION INPUT** (ciclo 19 do Motion).
//!
//! Os campos semeiam-se de ARESTA (a assinatura do instantâneo), nunca por quadro — senão apagariam
//! o que o artista está a digitar. A amostra de cor segue a lei das partículas: com o selector
//! apontado a ela, a escolha vai ao barramento; fora disso ela segue o documento.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::motion_input_edits::{InspectorMotionInputInfo, MotionInputFieldEdit};
use ph2d_editor_core::panel::PanelHostInternal;

use crate::state::InspectorState;

fn assinatura(info: &InspectorMotionInputInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    (info.entity_bits, &info.name, info.source, &info.from).hash(&mut h);
    for v in [info.value, info.remap_from, info.remap_to, info.smooth] {
        v.to_bits().hash(&mut h);
    }
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_motion_input() else {
        inspector_state.last_motion_input_sig = None;
        return;
    };
    let id = crate::ids::INSP_MI_COLOR;
    let gravada = crate::state_tint::tint_f32_to_u8(info.color);
    if host.store().picker_target() == Some(id) {
        // Comparação em BYTES: a ida-e-volta `u8 → /255 → ×255` é exacta, o fluxo pára.
        if let Some(escolhida) = host.store().widget_color(id)
            && escolhida != gravada
        {
            host.bus_mut().push(EditorAction::InspectorComponentEdit {
                entity_bits: info.entity_bits,
                edit: ComponentEdit::MotionInput(MotionInputFieldEdit::Color(
                    crate::state_tint::tint_u8_to_f32(escolhida),
                )),
            });
        }
    } else {
        host.store_mut().set_widget_color(id, gravada);
    }
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_motion_input_sig == Some(sig) {
        return;
    }
    inspector_state.last_motion_input_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (id, v) in [
        (crate::ids::INSP_MI_VALUE, info.value),
        (crate::ids::INSP_MI_REMAP_FROM, info.remap_from),
        (crate::ids::INSP_MI_REMAP_TO, info.remap_to),
        (crate::ids::INSP_MI_SMOOTH, info.smooth),
    ] {
        if focus != Some(id) && drag != Some(id) {
            crate::unidade_mundo::semeia(host, id, f64::from(v));
        }
    }
    crate::sync_text_field::escreve_texto(host, focus, crate::ids::INSP_MI_NAME, &info.name);
    crate::sync_text_field::escreve_texto(host, focus, crate::ids::INSP_MI_FROM, &info.from);
}
