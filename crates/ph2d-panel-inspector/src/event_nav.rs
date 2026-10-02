//! **O despacho das secções NAV REGION e NAV AGENT** (plano 30, W4).
//!
//! ⚠️ **Irmão do [`crate::event`] por CAP de função** — o mesmo padrão das irmãs.
//!
//! # ⚠️ O CLIQUE afirma o contrário do que está no ECRÃ, e o ecrã vem do SNAPSHOT
//!
//! Nunca do store, que guarda o visual: a máscara nova é a do snapshot com UM bit trocado, e o
//! interruptor é o INVERTIDO do snapshot (a lei que a §11 pagou com um report).

use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::nav_edits::{NavAlvoModo, NavFieldEdit};
use ph2d_editor_core::panel::PanelHostInternal;

/// Despacha um evento das secções NAV. `true` = consumido.
pub(crate) fn apply_nav_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = crate::state_components::current_inspector_nav() else {
        return false;
    };
    let bits = info.entity_bits;

    if let WidgetEvent::Click(id) = ev {
        if let Some(r) = info.region
            && let Some(i) = crate::ids::INSP_NAV_LAYERS.iter().position(|&b| b == id)
        {
            push(
                host,
                bits,
                NavFieldEdit::ObstacleLayers(r.obstacle_layers ^ (1u8 << i)),
            );
            return true;
        }
        if let Some(i) = crate::ids::INSP_NAV_TARGET_MODE
            .iter()
            .position(|&b| b == id)
        {
            push(host, bits, NavFieldEdit::AlvoModo(NavAlvoModo::ALL[i]));
            return true;
        }
        // (W6) Uma opção da TAG — a lista é a MESMA que o pintor derivou.
        if let Some(i) = crate::ids::INSP_NAV_TAG_OPT.iter().position(|&o| o == id) {
            if let Some(o) = crate::sections::nav_tag_row::nav_tag_options().get(i) {
                push(host, bits, NavFieldEdit::AlvoTag(o.value));
            }
            return true;
        }
    }

    if let WidgetEvent::Toggled(id) = ev
        && id == crate::ids::INSP_NAV_ACTIVE
        && let Some(a) = info.agent.as_ref()
    {
        push(host, bits, NavFieldEdit::Active(!a.active));
        return true;
    }
    if let WidgetEvent::Toggled(id) = ev
        && id == crate::ids::INSP_NAV_AVOIDANCE
        && let Some(a) = info.agent.as_ref()
    {
        push(host, bits, NavFieldEdit::Avoidance(!a.avoidance));
        return true;
    }

    if let WidgetEvent::TextChanged(id) = ev {
        // ⚠️ **O texto CRU** — quem o apara é quem o lê.
        let t = host.store().text(id).unwrap_or("").to_string();
        let edit = match id {
            crate::ids::INSP_NAV_TARGET_NAME => NavFieldEdit::AlvoNome(t),
            crate::ids::INSP_NAV_ON_ARRIVED => NavFieldEdit::OnArrived(t),
            crate::ids::INSP_NAV_ON_NO_PATH => NavFieldEdit::OnNoPath(t),
            crate::ids::INSP_NAV_ON_STUCK => NavFieldEdit::OnStuck(t),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev {
        #[allow(clippy::cast_possible_truncation)]
        let f = host.store().number_value(id).unwrap_or(0.0) as f32;
        // ⚠️ **Um `match` e não um `if` por campo** — com `if`s, o terceiro acaba a escrever no
        // primeiro (a lição dos três campos de texto da tabela de acções).
        let edit = match id {
            crate::ids::INSP_NAV_HALF_W => NavFieldEdit::HalfW(f),
            crate::ids::INSP_NAV_HALF_H => NavFieldEdit::HalfH(f),
            crate::ids::INSP_NAV_TARGET_X => NavFieldEdit::AlvoX(f),
            crate::ids::INSP_NAV_TARGET_Y => NavFieldEdit::AlvoY(f),
            crate::ids::INSP_NAV_RADIUS => NavFieldEdit::Radius(f),
            crate::ids::INSP_NAV_ARRIVE => NavFieldEdit::Arrive(f),
            crate::ids::INSP_NAV_REPATH => NavFieldEdit::Repath(f),
            crate::ids::INSP_NAV_STUCK => NavFieldEdit::StuckAfter(f),
            _ => return false,
        };
        push(host, bits, edit);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: NavFieldEdit) {
    host.bus_mut().push(EditorAction::InspectorComponentEdit {
        entity_bits,
        edit: ComponentEdit::Nav(edit),
    });
}
