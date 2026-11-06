//! **Os gestos da lista de sons** (SFX Lab W1). Irmão do [`crate::event_audio`] pelo tecto de função.
//!
//! ⚠️ Os valores afirmados vêm do snapshot, nunca do store: é a mesma lei das caixas da secção.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{AudioFieldEdit, InspectorSoundSet};
use ph2d_editor_core::widget::ButtonState;

fn fecha(host: &mut dyn PanelHostInternal, id: NodeId) {
    if let Some(InteractiveState::Dropdown { open, .. }) = host.store_mut().get_mut(id) {
        *open = false;
    }
}

fn clique(
    host: &mut dyn PanelHostInternal,
    set: &InspectorSoundSet,
    id: NodeId,
) -> Option<AudioFieldEdit> {
    if let Some(i) = ids::INSP_AUDIO_SET_DEL.iter().position(|&o| o == id) {
        return (i < set.rows.len()).then_some(AudioFieldEdit::SetRemove(i as u8));
    }
    if let Some(i) = ids::INSP_AUDIO_PICK_OPT.iter().position(|&o| o == id) {
        fecha(host, ids::INSP_AUDIO_PICK);
        return set.offer.get(i).cloned().map(AudioFieldEdit::SetAdd);
    }
    if let Some(i) = ids::INSP_AUDIO_MODE_OPT.iter().position(|&o| o == id) {
        fecha(host, ids::INSP_AUDIO_MODE);
        return Some(AudioFieldEdit::SetStrategy(i as u8));
    }
    None
}

fn da_lista(id: NodeId) -> bool {
    ids::INSP_AUDIO_SET_DEL.contains(&id)
        || ids::INSP_AUDIO_PICK_OPT.contains(&id)
        || ids::INSP_AUDIO_MODE_OPT.contains(&id)
}

/// Despacha um evento da lista. `true` = consumido.
pub(crate) fn apply_set_event(
    host: &mut dyn PanelHostInternal,
    entity_bits: u64,
    set: &InspectorSoundSet,
    ev: WidgetEvent,
) -> bool {
    let edit = match ev {
        WidgetEvent::Click(id) if da_lista(id) => {
            let edit = clique(host, set, id);
            if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
                *state = ButtonState::Normal;
            }
            edit
        }
        WidgetEvent::Toggled(id) if id == ids::INSP_AUDIO_NO_REPEAT => set
            .no_repeat_decides()
            .then_some(AudioFieldEdit::SetNoRepeat(!set.no_repeat)),
        WidgetEvent::Toggled(id) if ids::INSP_AUDIO_SET_ON.contains(&id) => {
            let i = ids::INSP_AUDIO_SET_ON.iter().position(|&o| o == id);
            i.and_then(|i| {
                let on = set.rows.get(i)?.enabled;
                Some(AudioFieldEdit::SetEnabled(i as u8, !on))
            })
        }
        WidgetEvent::ValueChanged(id) => {
            #[allow(clippy::cast_possible_truncation)]
            let v = host.store().number_value(id).unwrap_or(0.0) as f32;
            if id == ids::INSP_AUDIO_PITCH_JITTER {
                Some(AudioFieldEdit::SetPitchJitter(v))
            } else if id == ids::INSP_AUDIO_GAIN_JITTER {
                Some(AudioFieldEdit::SetGainJitter(v))
            } else if let Some(i) = ids::INSP_AUDIO_SET_WEIGHT.iter().position(|&o| o == id) {
                Some(AudioFieldEdit::SetWeight(i as u8, v))
            } else {
                return false;
            }
        }
        _ => return false,
    };
    if let Some(edit) = edit {
        host.bus_mut()
            .push(EditorAction::InspectorAudioEdit { entity_bits, edit });
    }
    true
}
