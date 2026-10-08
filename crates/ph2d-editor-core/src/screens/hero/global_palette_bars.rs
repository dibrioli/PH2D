//! **A PALETA PROJECTA AS BARRAS DO MÓDULO** (F4, D10) — ① a coluna e o flyout, ② a fila e os
//! pulldowns da área, ④ as ações da seleção. Os grupos têm o nome do LUGAR onde o comando mora, e
//! escolher faz o botão piscar lá ([`super::float_bar::flash`]).
//!
//! ⚠️ **O mesmo id, a mesma porta**: o item é o id do botão e quem executa re-toca o clique
//! (`HeroScreen::apply_event`). Nenhuma tabela `id → acção` nova.
//!
//! ⛔ **Um campo numérico não é comando** (o clique sem Down não o opera): fica fora. **Uma amostra
//! de cor é**, pela porta dela — abre o seletor ([`WidgetStore::open_picker_on`]).

use crate::interaction::{BarButton, BarFace, WidgetStore};
use crate::widget::ToolRailEntry;
use crate::widget::command_palette::{PaletteGroup, PaletteItem, PaletteSub};
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

/// Um item da paleta a partir de uma entrada da fila — `None` para o que não é comando.
fn row_item(e: &ToolRailEntry) -> Option<PaletteItem> {
    match e {
        ToolRailEntry::Number { .. } | ToolRailEntry::Divider => None,
        _ => Some(PaletteItem {
            label: e.label()?.to_string(),
            id: e.node_id()?,
            keywords: "",
        }),
    }
}

fn button_item(b: &BarButton) -> PaletteItem {
    PaletteItem {
        label: b.label.clone(),
        id: b.id,
        keywords: "",
    }
}

/// Um sub com os itens ainda não vistos (o frequente mora em dois lugares; a paleta mostra-o no
/// primeiro, o da prioridade mais alta).
fn sub(
    title: Option<String>,
    items: impl Iterator<Item = PaletteItem>,
    seen: &mut Vec<NodeId>,
) -> Option<PaletteSub> {
    let items: Vec<PaletteItem> = items
        .filter(|i| {
            let fresh = !seen.contains(&i.id);
            seen.push(i.id);
            fresh
        })
        .collect();
    (!items.is_empty()).then_some(PaletteSub { title, items })
}

fn group(title: &str, color: ColorToken, subs: Vec<Option<PaletteSub>>) -> Option<PaletteGroup> {
    let subs: Vec<PaletteSub> = subs.into_iter().flatten().collect();
    (!subs.is_empty()).then(|| PaletteGroup {
        title: tr(title).into(),
        color,
        subs,
    })
}

/// ⭐ Os grupos das barras publicadas NESTE quadro — vazio sem módulo com barras.
#[must_use]
pub fn groups(store: &WidgetStore) -> Vec<PaletteGroup> {
    let Some(bars) = store.tool_bars() else {
        return Vec::new();
    };
    let mut seen = Vec::new();
    let mut out = Vec::new();
    let flyout = bars.flyout.iter().flat_map(|(_, f)| f.iter());
    out.extend(group(
        "chrome.palette.left_bar",
        ColorToken::NodeCatSource,
        vec![
            sub(
                None,
                bars.column.iter().flatten().map(button_item),
                &mut seen,
            ),
            sub(None, flyout.map(button_item), &mut seen),
        ],
    ));
    let row = bars.row_lead.iter().chain(&bars.row_tail).filter_map(row_item);
    let mut top = vec![sub(None, row, &mut seen)];
    for m in store.area_menus() {
        top.push(sub(
            Some(m.label.clone()),
            m.rows.iter().filter_map(row_item),
            &mut seen,
        ));
    }
    out.extend(group("chrome.palette.top_bar", ColorToken::NodeCatTransform, top));
    out.extend(group(
        "chrome.palette.near_selection",
        ColorToken::NodeCatFx,
        vec![sub(
            None,
            bars.actions.iter().flatten().map(button_item),
            &mut seen,
        )],
    ));
    out
}

/// `id` é uma amostra de cor das barras (abre o seletor em vez de clicar)?
#[must_use]
pub fn is_swatch(store: &WidgetStore, id: NodeId) -> bool {
    let Some(bars) = store.tool_bars() else {
        return false;
    };
    let in_buttons = bars
        .column
        .iter()
        .chain(&bars.actions)
        .flatten()
        .any(|b| b.id == id && matches!(b.face, BarFace::Swatch { .. }));
    let in_row = bars
        .row_lead
        .iter()
        .chain(&bars.row_tail)
        .any(|e| matches!(e, ToolRailEntry::Swatch { .. }) && e.node_id() == Some(id));
    in_buttons || in_row
}

/// `id` é um comando que estas barras oferecem (e por isso tem onde piscar)?
#[must_use]
pub fn owns(store: &WidgetStore, id: NodeId) -> bool {
    groups(store)
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .any(|i| i.id == id)
}
