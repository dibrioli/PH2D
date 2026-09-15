//! **O despacho da secção AUDIO** (TOP-20 #4, W3).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — mesmo padrão do [`crate::event_timer`].
//!
//! # ⚠️ Ela não tem linha aberta, e por isso não escreve no `InspectorState`
//!
//! Um objecto tem UMA fonte de som (ver o doc do [`ph2d_ecs::AudioSource2D`]). Sem lista não há
//! escolha de linha a lembrar, e este despacho é o único da família que não toca no estado do
//! painel — ele só traduz gestos em edições.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual. É a lei que a §11 pagou com um report: ler o store fazia o
//! primeiro clique depois de trocar de objecto mandar o valor do objecto **anterior**.

use crate::ids;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::AudioFieldEdit;
use ph2d_editor_core::widget::ButtonState;

/// O maior valor que um `u8` guarda — a cerca da conversão, não um número de desenho.
const U8_TOP: f64 = 255.0; // LITERAL-PX-OK: o teto de um `u8`, não um pixel

/// Despacha um evento da secção AUDIO. `true` = consumido.
pub(crate) fn apply_audio_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_audio() else {
        return false;
    };
    let bits = info.entity_bits;
    let Some(src) = info.source.as_ref() else {
        // ⚠️ **Um objecto que só tem as ORELHAS não tem gesto nenhum** — o marcador não tem campos.
        // Devolver `false` deixa o evento seguir para quem o queira, em vez de o engolir.
        return false;
    };

    if let WidgetEvent::Click(id) = ev {
        let edit = if id == ids::INSP_AUDIO_BROWSE {
            AudioFieldEdit::Browse
        } else if id == ids::INSP_AUDIO_PREVIEW {
            AudioFieldEdit::Preview
        } else if id == ids::INSP_AUDIO_STOP {
            AudioFieldEdit::StopPreview
        } else if let Some(i) = ids::INSP_AUDIO_BUS_OPT.iter().position(|&o| o == id) {
            close_bus_popover(host);
            AudioFieldEdit::Bus(u8::try_from(i).unwrap_or(0))
        } else {
            return false;
        };
        push(host, bits, edit);
        demote(host, id);
        return true;
    }

    if let WidgetEvent::Toggled(id) = ev
        && matches!(id, ids::INSP_AUDIO_LOOP | ids::INSP_AUDIO_AUTOPLAY)
    {
        let edit = if id == ids::INSP_AUDIO_LOOP {
            AudioFieldEdit::Looping(!src.looping)
        } else {
            AudioFieldEdit::Autoplay(!src.autoplay)
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev
        && id == ids::INSP_AUDIO_SOUND
    {
        let text = host.store().text(id).unwrap_or("").to_string();
        push(host, bits, AudioFieldEdit::Sound(text));
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        let v = host.store().number_value(id).unwrap_or(0.0);
        #[allow(clippy::cast_possible_truncation)]
        let f = v as f32;
        // ⚠️ **Um `if` por campo seria como o terceiro acaba a escrever no primeiro** — a mesma
        // razão que pôs os três campos de texto da tabela de acções numa porta só.
        let edit = match id {
            ids::INSP_AUDIO_VOLUME => AudioFieldEdit::VolumeDb(f),
            ids::INSP_AUDIO_PITCH => AudioFieldEdit::Pitch(f),
            ids::INSP_AUDIO_MAX_DIST => AudioFieldEdit::MaxDistance(f),
            ids::INSP_AUDIO_ATTENUATION => AudioFieldEdit::Attenuation(f),
            ids::INSP_AUDIO_RADIUS => AudioFieldEdit::Radius(f),
            ids::INSP_AUDIO_PANNING => AudioFieldEdit::Panning(f),
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            ids::INSP_AUDIO_POLYPHONY => AudioFieldEdit::Polyphony(v.clamp(1.0, U8_TOP) as u8), // CLAMP-OK: a faixa é a do campo
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: AudioFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorAudioEdit { entity_bits, edit });
}

/// Fecha o popover do seletor de barramento depois de uma escolha.
///
/// ⚠️ **E NÃO escreve o `selected_index`** — quem é dono do barramento é o snapshot. É a lei que a
/// §12 e o seletor do verbo já pagam.
fn close_bus_popover(host: &mut dyn PanelHostInternal) {
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(ids::INSP_AUDIO_BUS_PICK)
    {
        *open = false;
    }
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}
