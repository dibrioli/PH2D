//! **Os gestos da lista de regras do cartão «Animation»** (plano GameEngineFuncional 08): as linhas
//! dos dois popovers, o ×, «+ Rule», a pega e o arrasto que reordena. Irmão do
//! [`crate::event_anim_card`] pelo tecto de função.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, PanelRowDrop, PanelRowFamily, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{AnimFieldEdit, InspectorAnimInfo};

use crate::event_anim::{demote, push};
use crate::ids::{
    INSP_ANIM_RULE_ADD, INSP_ANIM_RULE_DEL, INSP_ANIM_RULE_GRIP, INSP_ANIM_RULE_PLAY,
    INSP_ANIM_RULE_PLAY_OPT, INSP_ANIM_RULE_WHEN, INSP_ANIM_RULE_WHEN_OPT,
};

/// `true` = o evento era da lista de regras e foi consumido.
pub(crate) fn apply_rules_event(
    host: &mut dyn PanelHostInternal,
    info: &InspectorAnimInfo,
    ev: WidgetEvent,
) -> bool {
    match ev {
        WidgetEvent::Click(id) => apply_rules_click(host, info, id),
        WidgetEvent::PanelRowReparent {
            family: PanelRowFamily::AnimRule,
            dragged,
            drop,
        } => {
            if let Some((de, para)) = rule_move(info, dragged, drop) {
                push(host, info.entity_bits, AnimFieldEdit::RuleMove(de, para));
            }
            true
        }
        _ => false,
    }
}

/// **A posição FINAL de uma regra arrastada**, ou `None` se ela fica onde estava. O alvo é o vão
/// ANTES do qual ela cai (`Before(r)` = o de `r`, `After(r)` = o seguinte, `End`/`Inside` = o fim);
/// tirá-la de cima desloca os vãos de baixo uma posição para cima.
pub(crate) fn rule_move(
    info: &InspectorAnimInfo,
    dragged: NodeId,
    drop: PanelRowDrop,
) -> Option<(u8, u8)> {
    let n = crate::sections::anim_rules::shown_rules(info);
    let grips = &INSP_ANIM_RULE_GRIP[..n];
    let at = |id: NodeId| grips.iter().position(|&g| g == id);
    let de = at(dragged)?;
    let alvo = match drop {
        PanelRowDrop::Before(id) => at(id)?,
        PanelRowDrop::After(id) => at(id)? + 1,
        PanelRowDrop::Inside(_) | PanelRowDrop::End => n,
    };
    let fim = if alvo > de { alvo - 1 } else { alvo };
    if fim == de {
        return None;
    }
    Some((u8::try_from(de).ok()?, u8::try_from(fim).ok()?))
}

/// O menu de `table` que está ABERTO (as linhas do popover são partilhadas pelas regras todas).
fn open_menu(host: &dyn PanelHostInternal, table: &[NodeId]) -> Option<usize> {
    table.iter().position(|&id| {
        matches!(
            host.store().get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        )
    })
}

/// Fecha todos os menus das regras.
fn close_menus(host: &mut dyn PanelHostInternal) {
    for id in INSP_ANIM_RULE_WHEN.into_iter().chain(INSP_ANIM_RULE_PLAY) {
        if let Some(InteractiveState::Dropdown { open, .. }) = host.store_mut().get_mut(id) {
            *open = false;
        }
    }
}

/// `true` = o clique era da lista de regras e foi consumido.
fn apply_rules_click(
    host: &mut dyn PanelHostInternal,
    info: &InspectorAnimInfo,
    id: NodeId,
) -> bool {
    let bits = info.entity_bits;
    let at = |table: &[NodeId]| table.iter().position(|&o| o == id);
    let edit = if let Some(j) = at(&INSP_ANIM_RULE_WHEN_OPT) {
        let edit = open_menu(host, &INSP_ANIM_RULE_WHEN)
            .zip(info.card.when_options().get(j).cloned())
            .map(|(i, when)| AnimFieldEdit::RuleWhen(i as u8, when));
        close_menus(host);
        edit
    } else if let Some(j) = at(&INSP_ANIM_RULE_PLAY_OPT) {
        let edit = open_menu(host, &INSP_ANIM_RULE_PLAY)
            .zip(info.rows.get(j))
            .map(|(i, row)| AnimFieldEdit::RulePlay(i as u8, row.name.clone()));
        close_menus(host);
        edit
    } else if let Some(i) = at(&INSP_ANIM_RULE_DEL) {
        Some(AnimFieldEdit::RuleRemove(i as u8))
    } else if id == INSP_ANIM_RULE_ADD {
        info.card
            .when_options()
            .into_iter()
            .next()
            .zip(info.rows.first())
            .map(|(when, row)| AnimFieldEdit::RuleAdd(when, row.name.clone()))
    } else if at(&INSP_ANIM_RULE_GRIP).is_some() {
        // A pega só arrasta: o clique sem movimento é consumido sem edição.
        None
    } else {
        return false;
    };
    if let Some(edit) = edit {
        push(host, bits, edit);
    }
    demote(host, id);
    true
}
