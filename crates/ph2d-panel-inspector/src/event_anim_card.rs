//! **Os cliques do cartão «Animation»** (plano GameEngineFuncional 07) — o ▶ de cada linha,
//! «Edit Animations…», «Who Picks» e as linhas dos menus de estado. Irmão do [`crate::event_anim`]
//! pelo tecto de função.

use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{AnimChooser, AnimFieldEdit, InspectorAnimInfo};

use crate::event_anim::{demote, push};
use crate::state::InspectorState;

/// `true` = o evento era do cartão (um clique, «Flip», «Run From» ou a barra) e foi consumido.
pub(crate) fn apply_card_event(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    info: &InspectorAnimInfo,
    ev: WidgetEvent,
) -> bool {
    if let WidgetEvent::Click(id) = ev {
        return apply_card_click(panel, host, info, id);
    }
    // «Flip» do modo «The Character» — a mesma lei: o contrário do que está DESENHADO.
    if let WidgetEvent::Toggled(id) = ev
        && id == crate::ids::INSP_ANIM_FLIP
        && let Some(ch) = info.card.character.as_ref()
    {
        push(
            host,
            info.entity_bits,
            AnimFieldEdit::CharacterFlip(!ch.flip),
        );
        return true;
    }
    // ⚠️ **A BARRA DE FRAMES** — o `0..1` do slider vira célula pela INVERSA da mesma lei que a
    // pinta (`InspectorAnimInfo::scrub_cell` ↔ `scrub_position`).
    //
    // ⚠️ **A régua vivia aqui, no pintor e no `sync`, em três cópias** — e uma mutação que mudou só
    // a do pintor **sobreviveu** à suíte inteira, porque nenhum gate ligava o que se desenha ao que
    // o clique produz. Hoje é uma função só, com gate de ida-e-volta no modelo.
    if let WidgetEvent::ValueChanged(id) = ev
        && id == crate::ids::INSP_ANIM_FRAME_SCRUB
    {
        let v = host.store().slider(id).map_or(0.0, |(_, v)| v);
        // ⭐ Numa lista de quadros o arrasto escolhe o PASSO (`SetStep`), num intervalo a célula.
        if let Some(edit) = info.scrub_edit(v) {
            push(host, info.entity_bits, edit);
        }
        return true;
    }

    if let WidgetEvent::ValueChanged(id) = ev
        && id == crate::ids::INSP_ANIM_RUN_FROM
    {
        let v = host.store().number_value(id).unwrap_or(0.0);
        if info.card.character.is_some() {
            #[allow(clippy::cast_possible_truncation)]
            push(
                host,
                info.entity_bits,
                AnimFieldEdit::CharacterRunFrom(v as f32),
            );
        }
        return true;
    }
    false
}

/// `true` = o clique era do cartão e foi consumido.
fn apply_card_click(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    info: &InspectorAnimInfo,
    id: ph2d_a11y::NodeId,
) -> bool {
    let bits = info.entity_bits;
    // ▶: escolhe E põe a tocar (a linha, sozinha, só escolhe).
    if let Some(i) = crate::ids::INSP_ANIM_PLAY_ROW.iter().position(|&o| o == id)
        && let Some(row) = info.rows.get(i)
    {
        panel.anim_selected = i;
        push(host, bits, AnimFieldEdit::SetCurrent(row.name.clone()));
        push(host, bits, AnimFieldEdit::Playing(true));
        demote(host, id);
        return true;
    }
    if id == crate::ids::INSP_ANIM_OPEN_EDITOR {
        push(host, bits, AnimFieldEdit::OpenEditor);
        demote(host, id);
        return true;
    }
    // ⚠️ A POSIÇÃO no array é o `AnimChooser::tag`. «The Character» sem Platform Player não pega: o
    // cartão diz porquê, e aqui não se emite nada que a cena recusaria.
    if let Some(i) = crate::ids::INSP_ANIM_CHOOSER.iter().position(|&o| o == id) {
        let choice = AnimChooser::from_tag(i as u8);
        if choice != AnimChooser::Character || info.card.has_character_body {
            push(host, bits, AnimFieldEdit::Chooser(choice.tag()));
        }
        demote(host, id);
        return true;
    }
    // Uma linha do popover de ESTADO: o estado é o do menu que está ABERTO (as linhas são partilhadas).
    if let Some(j) = crate::ids::INSP_ANIM_STATE_OPT
        .iter()
        .position(|&o| o == id)
    {
        let open = crate::ids::INSP_ANIM_STATE_PICK.iter().position(|&pick| {
            matches!(
                host.store().get(pick),
                Some(InteractiveState::Dropdown { open: true, .. })
            )
        });
        if let (Some(state), Some(row)) = (open, info.rows.get(j)) {
            push(
                host,
                bits,
                AnimFieldEdit::CharacterState(state as u8, row.name.clone()),
            );
        }
        for pick in crate::ids::INSP_ANIM_STATE_PICK {
            if let Some(InteractiveState::Dropdown { open, .. }) = host.store_mut().get_mut(pick) {
                *open = false;
            }
        }
        demote(host, id);
        return true;
    }
    false
}
