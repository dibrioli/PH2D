//! **Os gestos dos marcadores em quadro** (SFX Lab W1). Irmão do [`crate::event_anim`] pelo tecto de função.

use crate::event_anim::{demote, push};
use crate::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{AnimFieldEdit, InspectorAnimInfo};

/// `true` = o evento era dos marcadores da animação `row` e foi consumido.
pub(crate) fn apply_markers_event(
    host: &mut dyn PanelHostInternal,
    info: &InspectorAnimInfo,
    row: u8,
    ev: WidgetEvent,
) -> bool {
    let at = |t: &[ph2d_a11y::NodeId], id| t.iter().position(|&o| o == id);
    let edit = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_ANIM_MARKER_ADD => {
            demote(host, id);
            // Em cartões os marcadores moram no editor: o botão é a porta dele.
            if crate::cards::active() {
                push(host, info.entity_bits, AnimFieldEdit::OpenEditor);
                return true;
            }
            AnimFieldEdit::MarkerAdd(row)
        }
        WidgetEvent::Click(id) => {
            let Some(i) = at(&ids::INSP_ANIM_MARKER_DEL, id) else {
                return false;
            };
            demote(host, id);
            AnimFieldEdit::MarkerRemove(row, i as u8)
        }
        WidgetEvent::ValueChanged(id) => {
            let Some(i) = at(&ids::INSP_ANIM_MARKER_FRAME, id) else {
                return false;
            };
            // Na tela o quadro conta de 1; no dado, de 0.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let step = (host.store().number_value(id).unwrap_or(1.0).round() - 1.0).max(0.0) as u32;
            AnimFieldEdit::MarkerStep(row, i as u8, step)
        }
        WidgetEvent::TextChanged(id) => {
            let Some(i) = at(&ids::INSP_ANIM_MARKER_SIGNAL, id) else {
                return false;
            };
            let text = host.store().text(id).unwrap_or("").to_string();
            AnimFieldEdit::MarkerSignal(row, i as u8, text)
        }
        _ => return false,
    };
    if !info.rows.is_empty() {
        push(host, info.entity_bits, edit);
    }
    true
}
