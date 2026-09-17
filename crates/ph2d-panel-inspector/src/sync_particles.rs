//! **A semente dos campos da secção PARTICLES** (TOP-20 #18, W3).
//!
//! ⚠️ **De ARESTA, nunca por quadro** — reescrever sempre apagaria o que o artista está a digitar
//! antes de o commit da shell chegar. A aresta é a ASSINATURA do instantâneo.
//!
//! ⚠️ **As caixas semeiam-se em TODO quadro**: um clique numa caixa é um commit instantâneo, e não
//! há texto a meio para apagar (a lei que a secção da câmera escreve).

use std::hash::{Hash, Hasher};

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::particles_edits::{
    InspectorParticlesInfo, PARTICLES_NUMBERS, PARTICLES_TEXTS,
};
use ph2d_editor_core::widget::CheckboxValue;

use crate::state::InspectorState;

fn assinatura(info: &InspectorParticlesInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    info.entity_bits.hash(&mut h);
    for n in PARTICLES_NUMBERS {
        info.number(n).to_bits().hash(&mut h);
    }
    for t in PARTICLES_TEXTS {
        info.text(t).hash(&mut h);
    }
    (info.shape, info.space).hash(&mut h);
    for c in [info.color, info.color_end] {
        for ch in c {
            ch.to_bits().hash(&mut h);
        }
    }
    h.finish()
}

/// **A semente da secção** — chamada pelo [`crate::sync_sections`].
pub(crate) fn sync(
    host: &mut dyn PanelHostInternal,
    inspector_state: &mut InspectorState,
    entity_changed: bool,
) {
    let Some(info) = crate::state_components::current_inspector_particles() else {
        inspector_state.last_particles_sig = None;
        return;
    };
    for (id, on) in [
        (crate::ids::INSP_PART_EMITTING, info.emitting),
        (crate::ids::INSP_PART_ONE_SHOT, info.one_shot),
    ] {
        if let Some(InteractiveState::Checkbox { value, .. }) = host.store_mut().get_mut(id) {
            *value = if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            };
        }
    }
    // ⭐⭐ **As amostras de cor têm DOIS regimes**, e é a lei que a secção Color & Tint já paga:
    // com o selector apontado a esta amostra o artista está A ESCOLHER, e a divergência contra o
    // documento vai ao barramento (pré-visualização viva); fora disso a amostra segue o documento,
    // para que o undo e uma edição de fora se vejam.
    // ⛔ **Sem o primeiro braço a cor escolhida NUNCA chega ao emissor** — o fio estaria completo
    // até ao `widget_color` e morreria ali, que é a espécie de controlo morto mais difícil de ver.
    let picker = host.store().picker_target();
    for (id, fim, c) in [
        (crate::ids::INSP_PART_COLOR, false, info.color),
        (crate::ids::INSP_PART_COLOR_END, true, info.color_end),
    ] {
        let gravada = crate::state_tint::tint_f32_to_u8(c);
        if picker == Some(id) {
            // ⚠️ A comparação é em BYTES: a ida-e-volta `u8 → /255 → ×255` é exacta, logo o fluxo
            // pára no quadro em que o commit aterra em vez de girar o barramento para sempre.
            if let Some(escolhida) = host.store().widget_color(id)
                && escolhida != gravada
            {
                host.bus_mut().push(
                    ph2d_editor_core::action_bus::EditorAction::InspectorParticlesEdit {
                        entity_bits: info.entity_bits,
                        edit: ph2d_editor_core::particles_edits::ParticlesFieldEdit::Color(
                            fim,
                            crate::state_tint::tint_u8_to_f32(escolhida),
                        ),
                    },
                );
            }
        } else {
            host.store_mut().set_widget_color(id, gravada);
        }
    }
    let sig = assinatura(&info);
    if !entity_changed && inspector_state.last_particles_sig == Some(sig) {
        return;
    }
    inspector_state.last_particles_sig = Some(sig);
    let focus = host.store().focus_id();
    let drag = host.store().number_input_drag().map(|d| d.id);
    for (i, n) in PARTICLES_NUMBERS.into_iter().enumerate() {
        let Some(&id) = crate::ids::INSP_PART_NUM.get(i) else {
            continue;
        };
        if focus == Some(id) || drag == Some(id) {
            continue; // a mão do artista ganha ao instantâneo
        }
        host.store_mut()
            .set_number_value(id, f64::from(info.number(n)));
    }
    for (i, t) in PARTICLES_TEXTS.into_iter().enumerate() {
        let Some(&id) = crate::ids::INSP_PART_TEXT.get(i) else {
            continue;
        };
        crate::sync_text_field::escreve_texto(host, focus, id, info.text(t));
    }
}
