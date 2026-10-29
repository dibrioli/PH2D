//! **Os cliques do cartão «Animation»** (plano GameEngineFuncional 07) — o ▶ de cada linha,
//! «Edit Animations…», «Who Picks», «Run From» e «Flip»; a lista de regras está no
//! [`crate::event_anim_rules`]. Irmão do [`crate::event_anim`] pelo tecto de função.

use ph2d_editor_core::interaction::WidgetEvent;
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
    if crate::event_anim_rules::apply_rules_event(host, info, ev) {
        return true;
    }
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
        let v = crate::unidade_mundo::le(host, id).unwrap_or(0.0);
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

/// A linha `row` mostra ⏸ (e não ▶): é a que está no ecrã E a tocar. O pintor e o clique leem
/// esta mesma pergunta.
pub(crate) fn row_shows_pause(info: &InspectorAnimInfo, row: usize) -> bool {
    info.playing && info.rows.get(row).is_some_and(|r| r.name == info.current)
}

/// `true` = o clique era do cartão e foi consumido.
fn apply_card_click(
    panel: &mut InspectorState,
    host: &mut dyn PanelHostInternal,
    info: &InspectorAnimInfo,
    id: ph2d_a11y::NodeId,
) -> bool {
    let bits = info.entity_bits;
    // ▶ noutra linha: escolhe E põe a tocar (a linha, sozinha, só escolhe). Na linha que está no
    // ecrã o botão é o que ela DESENHA ([`row_shows_pause`]): ⏸ pausa, ▶ retoma de onde parou.
    if let Some(i) = crate::ids::INSP_ANIM_PLAY_ROW.iter().position(|&o| o == id)
        && let Some(row) = info.rows.get(i)
    {
        panel.anim_selected = i;
        if info.current == row.name {
            push(host, bits, AnimFieldEdit::Playing(!info.playing));
        } else {
            push(host, bits, AnimFieldEdit::SetCurrent(row.name.clone()));
            push(host, bits, AnimFieldEdit::Playing(true));
        }
        demote(host, id);
        return true;
    }
    if id == crate::ids::INSP_ANIM_OPEN_EDITOR {
        push(host, bits, AnimFieldEdit::OpenEditor);
        demote(host, id);
        return true;
    }
    // ⚠️ A POSIÇÃO no array é o `AnimChooser::tag`. «Rules» vale sem Platform Player (teclas,
    // factos de vida, sinais — plano 08); as poses só aparecem no menu com ele.
    if let Some(i) = crate::ids::INSP_ANIM_CHOOSER.iter().position(|&o| o == id) {
        let choice = AnimChooser::from_tag(i as u8);
        push(host, bits, AnimFieldEdit::Chooser(choice.tag()));
        demote(host, id);
        return true;
    }
    false
}
