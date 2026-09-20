//! **A semente dos campos da secção HUD** (TOP-20 #20).
//!
//! ⚠️ **De ARESTA, nunca por quadro** — reescrever sempre apagaria o que o artista está a digitar
//! antes de o commit da shell chegar. A aresta é a ASSINATURA do instantâneo.
//!
//! ⚠️ **A caixa semeia-se em TODO quadro**: um clique nela é um commit instantâneo, e não há texto
//! a meio para apagar (a lei que a secção da câmera escreve).

use std::hash::{Hash, Hasher};

use ph2d_editor_core::hud_edits::{HUD_NUMBERS, HUD_TEXTS, InspectorHudInfo};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::widget::CheckboxValue;

use crate::state::InspectorState;

fn assinatura(info: &InspectorHudInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    for n in HUD_NUMBERS {
        info.number(n).to_bits().hash(&mut h);
    }
    for t in HUD_TEXTS {
        info.text(t).hash(&mut h);
    }
    (info.fit, info.source, info.has_canvas, info.has_label).hash(&mut h);
    (info.has_button, info.has_counter).hash(&mut h);
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_hud() else {
        inspector_state.last_hud_sig = None;
        return;
    };
    for (id, ligada) in [
        (crate::ids::INSP_HUD_DISABLED, info.disabled),
        (crate::ids::INSP_HUD_COUNTER_KEEP, info.counter_keep),
    ] {
        if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
            *value = if ligada {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
        }
    }
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_hud_sig == Some(sig) {
        return;
    }
    inspector_state.last_hud_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (i, n) in HUD_NUMBERS.into_iter().enumerate() {
        let Some(&id) = crate::ids::INSP_HUD_NUM.get(i) else {
            continue;
        };
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut()
            .set_number_value(id, f64::from(info.number(n)));
    }
    for (i, t) in HUD_TEXTS.into_iter().enumerate() {
        let Some(&id) = crate::ids::INSP_HUD_TEXT.get(i) else {
            continue;
        };
        crate::sync_text_field::escreve_texto(host, focus, id, info.text(t));
    }
}
