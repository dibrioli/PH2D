//! ⭐⭐ **Os cliques das linhas novas** ([`crate::rows`]) — a Cena, o conteúdo e os ícones de
//! componente. Corre ANTES do [`crate::event::apply_event`]: o que é daqui não chega ao caminho
//! dos objectos (selecção por linha, renomear, enquadrar).

use ph2d_editor_core::action_bus::{EditorAction, HierRequest, SelectModifier};
use ph2d_editor_core::hier_rows::{ContentOpens, RowRole};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::screens::hero::HeroSelection;
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;

use crate::state;

/// `Some` = o evento era de uma linha nova e foi tratado aqui.
pub(crate) fn apply(host: &mut dyn PanelHostInternal, ev: &WidgetEvent) -> Option<EventOutcome> {
    match *ev {
        WidgetEvent::Click(id) => click(host, id),
        // A Cena e o conteúdo não se renomeiam nem se enquadram.
        WidgetEvent::DoubleClick(id) => {
            let row = ids::hier_icon_companion_to_row(id).unwrap_or(id);
            let entry = state::live_entry_for(row)?;
            (entry.role != RowRole::Object).then_some(EventOutcome::Consumed)
        }
        _ => None,
    }
}

fn click(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> Option<EventOutcome> {
    // O galo de um grupo de conteúdo abre-o (o conteúdo nasce recolhido — `rows::is_collapsed`).
    if let Some(row) = ids::hier_expand_companion_to_row(id)
        && state::live_entry_for(row)
            .is_some_and(|e| matches!(e.role, RowRole::ContentGroup { .. }))
    {
        host.store_mut().toggle_hierarchy_content_open(row);
        return Some(EventOutcome::Consumed);
    }
    if let Some((row, category, entry)) = component_icon_at(id) {
        return component_click(host, row, category, &entry);
    }
    let entry = state::live_entry_for(id)?;
    match entry.role {
        RowRole::Object => None,
        RowRole::Scene => {
            host.bus_mut().push(EditorAction::ClearSelection);
            *host.selection_mut() = None;
            Some(EventOutcome::Consumed)
        }
        RowRole::ContentGroup { .. } => {
            host.store_mut().toggle_hierarchy_content_open(id);
            Some(EventOutcome::Consumed)
        }
        RowRole::ContentItem {
            owner,
            key,
            opens: ContentOpens::Mode(mode),
            ..
        } => {
            host.bus_mut()
                .push(EditorAction::Hierarchy(HierRequest::OpenContent {
                    owner,
                    mode,
                    key,
                }));
            Some(EventOutcome::Consumed)
        }
        // Num painel (os clips → a Timeline): o dono fica seleccionado, como no ícone, e o painel
        // abre-se aqui; a peça é a shell que a entrega.
        RowRole::ContentItem {
            owner,
            key,
            opens: ContentOpens::Panel(panel),
            ..
        } => {
            if owner != crate::ids::HIER_SCENE_ROW
                && let Some(entry) = state::live_entry_for(owner)
            {
                select_row(host, owner, &entry);
            }
            host.set_panel_visible(panel, true);
            host.bus_mut()
                .push(EditorAction::Hierarchy(HierRequest::OpenContentInPanel {
                    owner,
                    panel,
                    key,
                }));
            Some(EventOutcome::Consumed)
        }
    }
}

/// A linha e a família de um ícone de componente — o id é derivado ([`crate::ids::hier_component_icon`]),
/// então a resposta vem de percorrer as linhas vivas.
fn component_icon_at(
    id: ph2d_a11y::NodeId,
) -> Option<(
    ph2d_a11y::NodeId,
    ph2d_component_desc::ComponentCategory,
    HierarchyEntity,
)> {
    let live = state::current_live_entries()?;
    live.into_iter().find_map(|(row, entry)| {
        let category = *entry
            .components
            .iter()
            .find(|c| crate::ids::hier_component_icon(row, **c) == id)?;
        Some((row, category, entry))
    })
}

/// Selecciona a linha `row` (o que um clique no nome faz) — a porta do ícone e do conteúdo.
fn select_row(host: &mut dyn PanelHostInternal, row: ph2d_a11y::NodeId, entry: &HierarchyEntity) {
    host.bus_mut()
        .push(EditorAction::Hierarchy(HierRequest::SelectRow {
            row,
            modifier: SelectModifier::Replace,
        }));
    *host.selection_mut() = Some(HeroSelection {
        label: entry.name.clone(),
        kind: entry
            .badge
            .clone()
            .unwrap_or_else(|| ph2d_i18n::tr("panel.hierarchy.badge.entity").to_string()),
        world_pos: (0.0, 0.0),
    });
}

/// ⭐ O ícone da Cena abre o painel que É o cartão daquele componente; o de um objecto selecciona-o
/// e pede ao Inspector o cartão da família (doc 01 §7.1).
fn component_click(
    host: &mut dyn PanelHostInternal,
    row: ph2d_a11y::NodeId,
    category: ph2d_component_desc::ComponentCategory,
    entry: &HierarchyEntity,
) -> Option<EventOutcome> {
    if entry.role == RowRole::Scene {
        host.set_panel_visible(crate::rows::scene_panel_of(category)?, true);
        return Some(EventOutcome::Consumed);
    }
    select_row(host, row, entry);
    // O cartão de um componente que se edita num PAINEL é o painel (a Animação → a Timeline, que
    // segue a selecção); o Inspector fica para os que têm cartão.
    // ⚠️ Lido nas SUB-LINHAS e não no `entry.content`: a expansão já tirou o conteúdo da linha do
    // dono para o fazer linhas (`rows::expand`), então ali ele está sempre vazio.
    let panel = (category == ph2d_component_desc::ComponentCategory::Animation)
        .then(state::current_live_entries)
        .flatten()
        .and_then(|live| {
            live.into_iter().find_map(|(_, e)| match e.role {
                RowRole::ContentItem {
                    owner,
                    opens: ContentOpens::Panel(p),
                    ..
                } if owner == row => Some(p),
                _ => None,
            })
        });
    if let Some(panel) = panel {
        host.set_panel_visible(panel, true);
        return Some(EventOutcome::Consumed);
    }
    // O pedido leva a ENTIDADE (o Inspector casa por ela); uma linha sem mundo não tem cartão.
    if let Some(entity) = entry.entity {
        host.store_mut().request_inspector_reveal(category, entity);
    }
    host.set_panel_visible("inspector", true);
    Some(EventOutcome::Consumed)
}
