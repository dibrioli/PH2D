//! ⭐ **O painel Tags em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): a árvore é uma
//! lista dentro do cartão «All tags», com o «+» no cabeçalho; os verbos da linha em mãos moram no cartão
//! «Selected tag». Os ids, a linha arrastável e o despacho (`seam.rs`) são os de antes.

use crate::{ids, rows, seam, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{PanelRowFamily, WidgetStore};
use ph2d_editor_core::panel::cards::linhas_extra::ListRow;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Widget};
use ph2d_editor_core::{TagsPanelInfo, TagsPanelRow};
use ph2d_i18n::tr;
use state::TagsPanelState;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (os testes de um processo correm em paralelo).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// Antes de pintar, com o store mutável: as linhas clicáveis, o fecho e a dica do «+».
pub(crate) fn prepare(store: &mut WidgetStore) {
    store.mark_card_list_close(ph2d_editor_core::ids::TAGS_PANEL, ids::TAGS_CLOSE);
    let linhas: Vec<u64> = state::with_current(|i| i.rows.iter().map(|r| r.id).collect());
    seam::register_rows(store, &linhas);
    if store.tooltip_for(ids::TAGS_NEW).is_none() {
        store.set_tooltip(ids::TAGS_NEW, tr("card.tags.new"));
    }
}

/// O arrasto de linha em curso: o id que voa e onde está a mão.
fn voo(store: &WidgetStore) -> Option<(NodeId, (f32, f32), f32)> {
    let (familia, d) = store.panel_row_drag()?;
    (familia == PanelRowFamily::TagTree && d.active).then_some((
        d.dragged,
        (d.cursor_x, d.cursor_y),
        d.down_x,
    ))
}

fn linhas(c: &mut CardCtx<'_>, info: &TagsPanelInfo, st: &TagsPanelState, top: f32) -> f32 {
    let mut y = top;
    if info.rows.is_empty() {
        y = c.hint(tr("card.tags.empty"), y);
    }
    let arrasto = voo(c.store);
    let mut no_ar: Option<(&TagsPanelRow, String)> = None;
    for row in &info.rows {
        let id = ids::row_id(row.id);
        let n = row.members.to_string();
        let debaixo =
            arrasto.is_some_and(|(voa, (_, cy), _)| voa != id && cy >= y && cy < y + m::FIELD_H);
        let voa = arrasto.is_some_and(|(v, ..)| v == id);
        y = c.list_row(
            &ListRow {
                id,
                label: &row.label,
                depth: row.depth,
                trailing: Some(&n),
                selected: st.focus == Some(row.id),
                target: debaixo,
                dim: voa,
                edit: (st.renaming == Some(row.id)).then_some(ids::TAGS_RENAME_INPUT),
                icon: None,
            },
            y,
        );
        if voa {
            no_ar = Some((row, n));
        }
        // A recusa debaixo da linha dela: empurra a árvore para se ver.
        if let Some((tag, texto)) = &info.problem
            && *tag == row.id
        {
            y = c.problem(texto, y);
        }
    }
    // A recusa sem linha (criar uma raiz com nome vazio) vai para o fim, onde o gesto acabou.
    if let Some((0, texto)) = &info.problem {
        y = c.problem(texto, y);
    }
    if let (Some((row, _)), Some((_, (cx, cy), down_x))) = (no_ar, arrasto) {
        let linha = ListRow::new(ids::row_id(row.id), &row.label);
        c.list_row_flight(&linha, cx - (down_x - c.x), cy - m::FIELD_H * 0.5);
    }
    y
}

fn verbos(c: &mut CardCtx<'_>, row: &TagsPanelRow, top: f32) -> f32 {
    let tabela = rows::verbs(Some(row));
    let cells: Vec<Cell<'_>> = tabela
        .iter()
        .filter(|v| v.id != ids::TAGS_NEW && v.id != ids::TAGS_DELETE)
        .map(|v| {
            Cell::new(Widget::Button {
                id: v.id,
                text: &v.label,
                primary: false,
            })
        })
        .collect();
    let y = cells.chunks(2).fold(top, |y, par| c.row(par, y));
    let apagar = tabela.iter().find(|v| v.id == ids::TAGS_DELETE);
    apagar.map_or(y, |v| c.danger_button(v.id, &v.label, y))
}

/// ⭐ Os cartões do painel: a árvore e os verbos da linha em mãos.
pub(crate) fn cards(st: &TagsPanelState) -> Vec<CardDef<'static>> {
    let info = state::current_tags();
    let st = st.clone();
    let em_maos = st
        .focus
        .and_then(|f| info.rows.iter().find(|r| r.id == f).cloned());
    let total = info.rows.len().to_string();
    let resumo = em_maos.as_ref().map(|r| r.label.clone());
    vec![
        CardDef {
            section: ids::TAGS_SEC_TREE,
            title: tr("card.tags.all"),
            actions: vec![HeadAction {
                id: ids::TAGS_NEW,
                icon: IconId::Plus,
                on: false,
            }],
            add: None,
            summary: Some(total),
            body: Some(Box::new(move |c, y| linhas(c, &info, &st, y))),
        },
        CardDef {
            section: ids::TAGS_SEC_SELECTED,
            title: tr("card.tags.selected"),
            actions: Vec::new(),
            add: None,
            summary: resumo,
            body: Some(Box::new(move |c, y| match &em_maos {
                Some(row) => verbos(c, row, y),
                None => c.hint(tr("card.tags.pick_hint"), y),
            })),
        },
    ]
}
