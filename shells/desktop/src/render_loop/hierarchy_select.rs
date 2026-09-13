//! ⭐ **A selecção pela Hierarquia** — o clique com modificador e o *shift*-intervalo (Fase 0e).
//! Filho por ASSUNTO (por `#[path]`) do [`super`], cujo `dispatch` chama [`apply`] no sítio exacto
//! onde o bloco estava.

use super::HierarchySelectIntent;
use crate::HeroLive;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::action_bus::SelectModifier;

/// Aplica a intenção de selecção da Hierarquia: a linha resolve-se pela ponte, o modificador escolhe
/// a mutação do grupo, e o intervalo anda a ordem canónica entre a âncora e o alvo.
pub(super) fn apply(
    hierarchy_select_intent: Option<HierarchySelectIntent>,
    hero: &mut HeroScreen,
    hero_live: &Option<HeroLive>,
) {
    // Fase 0e: drain pending multi-select-aware hierarchy intent.
    // Bridge resolves row → entity_bits; modifier picks the
    // `GizmoStateGroup` mutation (Replace / Add / Toggle). Range walks
    // the canonical hierarchy order between primary's row and target,
    // adding every entity in between.
    if let Some(intent) = hierarchy_select_intent
        && let Some(live) = hero_live.as_ref()
    {
        match intent {
            HierarchySelectIntent::Row { row, modifier } => {
                if let Some(entity_bits) = live.bridge.entity_for(row) {
                    match modifier {
                        SelectModifier::Replace => {
                            // Smart-click parity with canvas pick (Fase
                            // 0 hotfix): bare click on a row already
                            // part of a multi-selection preserves the
                            // group instead of collapsing to single.
                            let preserves_multi = hero.gizmo.selected_len() > 1
                                && hero.gizmo.is_selected(entity_bits);
                            if !preserves_multi {
                                hero.gizmo.replace_selection(Some(entity_bits));
                            }
                        }
                        SelectModifier::Add => {
                            hero.gizmo.add_to_selection(entity_bits);
                        }
                        SelectModifier::Toggle => {
                            hero.gizmo.toggle_in_selection(entity_bits);
                        }
                    }
                }
            }
            HierarchySelectIntent::Range { row: target_row } => {
                // Onda 2 hotfix v2 — preserves the anchor (Enio: "o
                // shift em múltiplas sprites desselecionou a primeira").
                // Decision based on the TARGET row's current state:
                //   - target NOT selected → ADD every row in
                //     [anchor..target] (anchor stays as primary; new
                //     rows enter via add_to_selection, which is a
                //     no-op for ones already there).
                //   - target selected → REMOVE every row in
                //     [anchor..target] EXCEPT the anchor itself
                //     (anchor never demoted by this gesture; primary
                //     remains stable for the next range click).
                // Without an anchor (selection empty), degenerates to
                // a single add of the target — same as Cmd-click /
                // bare-click on an empty selection.
                let target_bits = live.bridge.entity_for(target_row);
                let anchor_row = hero
                    .gizmo
                    .selection
                    .and_then(|bits| live.bridge.node_for(bits));
                if let (Some(target_bits), Some(anchor_row)) = (target_bits, anchor_row) {
                    let order = hero.store.hierarchy_order();
                    let i_anchor = order.iter().position(|n| *n == anchor_row);
                    let i_target = order.iter().position(|n| *n == target_row);
                    if let (Some(a), Some(t)) = (i_anchor, i_target) {
                        let (lo, hi) = if a <= t { (a, t) } else { (t, a) };
                        let row_range: Vec<_> = order[lo..=hi].to_vec();
                        let target_was_selected = hero.gizmo.is_selected(target_bits);
                        for n in row_range {
                            if n == anchor_row {
                                continue;
                            }
                            if let Some(bits) = live.bridge.entity_for(n) {
                                if target_was_selected {
                                    // Remove from extras only — anchor
                                    // (= primary) skipped above so we
                                    // never demote it.
                                    hero.gizmo.extra_selection.retain(|b| *b != bits);
                                } else {
                                    hero.gizmo.add_to_selection(bits);
                                }
                            }
                        }
                    } else {
                        hero.gizmo.add_to_selection(target_bits);
                    }
                } else if let Some(target_bits) = target_bits {
                    hero.gizmo.add_to_selection(target_bits);
                }
            }
        }
    }
}
