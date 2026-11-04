//! ⭐ **O navegador de Assets em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): a lista
//! flutua no MESMO rect de sempre (arrasta-se e redimensiona-se pelos ids de sempre) e mostra três
//! cartões: Find (busca, família, ordem, tamanho), Catalogs (a árvore) e Library (a grade de miniaturas,
//! que continua a arrancar arrastos e abrir menus pelo `set_asset_cells`). O despacho é o de antes.

use crate::ids;
use crate::paint_catalog::{Row, rows};
use crate::state::{AssetBrowserState, CatalogPick, is_published, with_index};
use ph2d_a11y::NodeId;
use ph2d_asset_index::{AssetRef, Query, Relation, SortBy, Thumb};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text_elided, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::linhas_extra::ListRow;
use ph2d_editor_core::panel::cards::{
    CardCtx, CardDef, Cell, HeadAction, Seg, Widget, dropdown, paint_card_list,
};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::panel_chrome::{
    paint_panel_corner_dot_bl, panel_drag_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{ButtonState, card_ink::CardInk};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, StrokeToken};

const PAINEL: NodeId = ph2d_editor_core::ids::ASSET_PANEL;
/// A seta anda 4 px no tamanho da miniatura; a pega, 1 px por px da mão.
const PASSO_DA_SETA: f64 = 4.0;

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

/// Uma miniatura da grade: o que a consulta devolveu, clonado para fora do índice (o `Thumb` é um `Arc`).
struct Entrada {
    key: AssetRef,
    name: String,
    detail: String,
    swatch: [u8; 4],
    thumb: Option<Thumb>,
}

struct Grade {
    entradas: Vec<Entrada>,
    beyond: usize,
    total: usize,
}

/// A consulta do quadro: a mesma do corpo de antes, com o escopo do catálogo e a pergunta de relação.
fn consulta(state: &AssetBrowserState, store: &WidgetStore) -> Grade {
    let q = Query {
        text: store
            .text(ids::ASSET_SEARCH)
            .unwrap_or_default()
            .to_string(),
        kind: if state.choosing.is_some() {
            Some(ph2d_asset_index::AssetKind::Texture)
        } else {
            state.kind
        },
        catalog: crate::paint_catalog::scope_of(state.pick),
        sort: state.sort,
        related: state.related,
    };
    with_index(|index| {
        let hits = index.query(&q);
        let shown = hits.len().min(ids::MAX_ASSET_CELLS);
        Grade {
            entradas: hits
                .iter()
                .take(shown)
                .map(|e| Entrada {
                    key: e.key,
                    name: e.name.clone(),
                    detail: e.detail.clone(),
                    swatch: e.swatch,
                    thumb: e.thumb.clone(),
                })
                .collect(),
            beyond: hits.len() - shown,
            total: index.len(),
        }
    })
}

/// Com o store mutável: ids por quadro, o que o despachante precisa de saber, números e dicas.
fn prepare(state: &mut AssetBrowserState, store: &mut WidgetStore, grade: &Grade) {
    store.mark_card_list_close(PAINEL, ids::ASSET_CLOSE);
    store.set_collapsed_if_unchosen(ids::ASSET_SEC_CATALOGS, true);
    dropdown::declare(ids::ASSET_KIND_BOX);
    store.mark_card_field(
        ids::ASSET_SIZE_NUM,
        CardFieldSpec {
            step: PASSO_DA_SETA,
            scrub: 1.0,
            default: Some(f64::from(crate::CELL_DEFAULT_PX)),
        },
    );
    if store.number_input_drag().map(|d| d.id) != Some(ids::ASSET_SIZE_NUM) {
        store.set_number_value(ids::ASSET_SIZE_NUM, f64::from(state.cell_px));
    }
    for (id, chave) in [
        (ids::ASSET_CATALOG_TOGGLE, "card.assets.toggle_catalogs"),
        (ids::ASSET_CATALOG_NEW, "card.assets.new_catalog"),
        (ids::ASSET_RELATED_CLEAR, "card.assets.clear_filter"),
        (ids::ASSET_CHOOSE_CANCEL, "card.assets.cancel_choice"),
    ] {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(chave));
        }
    }
    // A coluna de antes tinha sub-região de roda; o cartão rola com a lista.
    crate::paint_catalog::forget_region(store);
    if state.show_catalogs {
        crate::paint_catalog_rename::seed_for_cards(state, store);
    } else {
        crate::paint_catalog_rename::abandon(state, store);
    }
    publica_a_grade(store, grade);
}

/// Os cartões dizem-se ao despachante: as células da consulta, com registo e menu de documento.
/// Fora da janela o corpo não pinta nem regista, logo a célula não tem rect e não apanha o dedo.
fn publica_a_grade(store: &mut WidgetStore, grade: &Grade) {
    let keys: Vec<AssetRef> = grade.entradas.iter().map(|e| e.key).collect();
    let celulas: std::collections::BTreeMap<NodeId, usize> = (0..keys.len())
        .map(|i| (ph2d_editor_core::ids::asset_cell_id(i), i))
        .collect();
    for id in celulas.keys() {
        store.register_if_absent(
            *id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    crate::paint_thumb::retain_painted(&keys);
    let docs = celulas
        .iter()
        .filter(|(_, i)| keys.get(**i).is_some_and(|k| crate::state::is_document(*k)))
        .map(|(id, _)| *id)
        .collect();
    store.set_asset_doc_cells(docs);
    store.set_asset_cells(celulas);
    crate::state::set_painted(keys);
}

fn publica_as_linhas(store: &mut WidgetStore, linhas: &[Row]) {
    let n = linhas.len().min(ph2d_editor_core::ids::MAX_CATALOG_ROWS);
    for i in 0..n {
        store.register_if_absent(
            ph2d_editor_core::ids::catalog_row_id(i),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    crate::state::set_painted_rows(linhas.iter().take(n).map(|r| r.pick).collect());
}

/// ⭐ Pinta a lista dentro do rect flutuante, e os dois gestos do painel que flutua: mover e redimensionar.
pub(crate) fn paint(state: &mut AssetBrowserState, ctx: &mut PaintCtx, rect: Rect) {
    let grade = consulta(state, ctx.host.store());
    prepare(state, ctx.host.store_mut(), &grade);
    let linhas = if state.show_catalogs {
        rows("")
    } else {
        Vec::new()
    };
    publica_as_linhas(ctx.host.store_mut(), &linhas);
    crate::state::set_last_visible_h(rect.h);
    let janela = (ctx.viewport.y, ctx.viewport.y + ctx.viewport.h);
    let cartoes = cards(state, linhas, grade, janela);
    let encaixe = ctx.slot;
    ctx.slot = rect;
    let lista = paint_card_list(ctx, PAINEL, crate::AssetBrowserPanel::TITLE.tr(), &cartoes);
    ctx.slot = encaixe;
    crate::state::set_last_content_h(lista.h);
    let store = ctx.host.store_mut();
    store.set_panel_rect(PAINEL, lista);
    let theme = ctx.host.theme();
    paint_panel_corner_dot_bl(lista, ctx.scene, theme);
    // O «…» e o «×» ficam à direita da faixa de arrasto.
    let reserva = (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 2.0;
    let hit = ctx.host.hit_index_mut();
    hit.register(
        ids::ASSET_DRAG_HANDLE,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
    hit.register(
        ids::ASSET_RESIZE_HANDLE_BL,
        panel_resize_handle_rect_bl(lista),
    );
}

fn cards(
    state: &AssetBrowserState,
    linhas: Vec<Row>,
    grade: Grade,
    janela: (f32, f32),
) -> Vec<CardDef<'static>> {
    let mut out = vec![cartao_de_busca(state)];
    if state.show_catalogs {
        out.push(cartao_de_catalogos(state, linhas));
    }
    out.push(cartao_da_biblioteca(state, grade, janela));
    out
}

fn nome_da_ordem(i: usize) -> &'static str {
    SortBy::ALL.get(i).map_or("", |s| tr(s.label_key()))
}

fn cartao_de_busca(state: &AssetBrowserState) -> CardDef<'static> {
    let (kind, sort, com_catalogos) = (state.kind, state.sort, state.show_catalogs);
    let resumo = AssetBrowserState::kind_chip_label(escolha_da_familia(kind));
    CardDef {
        section: ids::ASSET_SEC_FIND,
        title: tr("card.assets.find"),
        actions: Vec::new(),
        add: None,
        summary: Some(resumo.to_string()),
        body: Some(Box::new(move |c, y| {
            let alternar = [Seg {
                id: ids::ASSET_CATALOG_TOGGLE,
                icon: Some(IconId::Folder),
                text: "",
                on: com_catalogos,
            }];
            let mut y = c.row(
                &[
                    Cell::new(Widget::Text {
                        id: ids::ASSET_SEARCH,
                        placeholder: tr("panel.asset_browser.panel.search_assets"),
                    }),
                    Cell::new(Widget::Bare(&alternar)),
                ],
                y,
            );
            let acesa = escolha_da_familia(kind);
            let familias: Vec<Seg<'_>> = ids::ASSET_KIND
                .iter()
                .enumerate()
                .map(|(i, &id)| Seg {
                    id,
                    icon: None,
                    text: AssetBrowserState::kind_chip_label(i),
                    on: i == acesa,
                })
                .collect();
            y = c.row(
                &[
                    Cell::new(Widget::Dropdown {
                        id: ids::ASSET_KIND_BOX,
                        text: AssetBrowserState::kind_chip_label(acesa),
                        options: &familias,
                    })
                    .label(tr("card.assets.kind")),
                    Cell::new(Widget::Field {
                        id: ids::ASSET_SIZE_NUM,
                        glyph: Glyph::Text(letra_da_grandeza(tr("card.assets.thumb_size"))),
                    })
                    .label(tr("card.assets.thumb_size")),
                ],
                y,
            );
            let ordens: Vec<Seg<'_>> = ids::ASSET_SORT
                .iter()
                .enumerate()
                .map(|(i, &id)| Seg {
                    id,
                    icon: None,
                    text: nome_da_ordem(i),
                    on: SortBy::ALL.get(i) == Some(&sort),
                })
                .collect();
            c.full(Some(tr("card.assets.sort")), Widget::Choice(&ordens), y)
        })),
    }
}

fn escolha_da_familia(kind: Option<ph2d_asset_index::AssetKind>) -> usize {
    (0..ids::ASSET_KIND_FILTERS)
        .find(|i| AssetBrowserState::kind_for_chip(*i) == kind)
        .unwrap_or(0)
}

fn cartao_de_catalogos(state: &AssetBrowserState, linhas: Vec<Row>) -> CardDef<'static> {
    let (pick, renaming) = (state.pick, state.renaming.map(|r| r.id));
    let resumo = linhas
        .iter()
        .find(|r| r.pick == pick)
        .map(|r| r.label.clone());
    CardDef {
        section: ids::ASSET_SEC_CATALOGS,
        title: tr("card.assets.catalogs"),
        actions: vec![HeadAction {
            id: ids::ASSET_CATALOG_NEW,
            icon: IconId::Plus,
            on: false,
        }],
        add: None,
        summary: resumo,
        body: Some(Box::new(move |c, top| {
            let mut y = top;
            for (i, r) in linhas
                .iter()
                .enumerate()
                .take(ph2d_editor_core::ids::MAX_CATALOG_ROWS)
            {
                let n = (r.count > 0).then(|| r.count.to_string());
                let aberta = renaming.is_some_and(|id| r.pick == CatalogPick::One(id));
                y = c.list_row(
                    &ListRow {
                        depth: r.depth,
                        trailing: n.as_deref(),
                        selected: pick == r.pick,
                        edit: aberta.then_some(ids::ASSET_CATALOG_RENAME),
                        ..ListRow::new(ph2d_editor_core::ids::catalog_row_id(i), &r.label)
                    },
                    y,
                );
            }
            y
        })),
    }
}

/// A frase da grade vazia: vazio e por-publicar dizem coisas diferentes, e a relação fala primeiro.
fn frase_do_vazio(total: usize, related: Option<(AssetRef, Relation)>) -> &'static str {
    if !is_published() {
        tr("panel.asset_browser.panel.loading_assets")
    } else if total == 0 {
        tr("panel.asset_browser.panel.no_assets_yet")
    } else {
        match related {
            Some((_, Relation::Uses)) => {
                tr("panel.asset_browser.panel.this_asset_uses_nothing_else")
            }
            Some((_, Relation::UsedBy)) => {
                tr("panel.asset_browser.panel.nothing_in_the_library_uses_this")
            }
            None => tr("panel.asset_browser.panel.nothing_matches_this_search"),
        }
    }
}

/// A faixa do filtro que só um menu liga: nomeia-o e traz o próprio «×».
fn faixa(c: &mut CardCtx<'_>, texto: &str, fechar: NodeId, y: f32) -> f32 {
    let x = [Seg {
        id: fechar,
        icon: Some(IconId::Close),
        text: "",
        on: false,
    }];
    c.row(
        &[
            Cell::new(Widget::Readout { text: texto }),
            Cell::new(Widget::Bare(&x)),
        ],
        y,
    )
}

fn frase_da_relacao(related: Option<(AssetRef, Relation)>) -> Option<String> {
    let (anchor, dir) = related?;
    let nome = with_index(|ix| ix.get(&anchor).map(|e| e.name.clone()))
        .unwrap_or_else(|| tr("panel.asset_browser.related.this_asset").to_string());
    let chave = match dir {
        Relation::Uses => "panel.asset_browser.related.what_name_uses",
        Relation::UsedBy => "panel.asset_browser.related.what_uses_name",
    };
    Some(tr_with(chave, &[("name", &nome)]))
}

fn cartao_da_biblioteca(
    state: &AssetBrowserState,
    grade: Grade,
    janela: (f32, f32),
) -> CardDef<'static> {
    let (lado, related) = (state.cell_px, state.related);
    let relacao = frase_da_relacao(related);
    let escolha = state
        .choosing
        .as_ref()
        .map(|ch| tr_with("panel.asset_browser.pick.choose_for", &[("name", &ch.name)]));
    let resumo = grade.entradas.len().to_string();
    CardDef {
        section: ids::ASSET_SEC_LIBRARY,
        title: tr("card.assets.library"),
        actions: Vec::new(),
        add: None,
        summary: Some(resumo),
        body: Some(Box::new(move |c, top| {
            let mut y = top;
            if let Some(t) = &relacao {
                y = faixa(c, t, ids::ASSET_RELATED_CLEAR, y);
            }
            if let Some(t) = &escolha {
                y = faixa(c, t, ids::ASSET_CHOOSE_CANCEL, y);
            }
            if grade.entradas.is_empty() {
                return c.hint(frase_do_vazio(grade.total, related), y);
            }
            if y > top {
                y += m::FIELD_ROW_GAP;
            }
            y = grelha(c, &grade.entradas, lado, janela, y);
            if grade.beyond > 0 {
                y = c.hint(
                    &tr_with(
                        "panel.asset_browser.panel.n_more",
                        &[("beyond", &grade.beyond)],
                    ),
                    y,
                );
            }
            y
        })),
    }
}

/// A faixa do rótulo: as duas linhas que a célula desenha, dos mesmos tokens que as pintam.
const ROTULO_H: f32 =
    m::LABEL_BOTTOM + m::VALUE_PX + m::LABEL_BOTTOM + m::LABEL_PX + m::LABEL_BOTTOM;

/// A grade estica as colunas até à largura do cartão; o tamanho pedido decide quantas cabem.
fn grelha(
    c: &mut CardCtx<'_>,
    entradas: &[Entrada],
    lado_pedido: f32,
    (topo, fundo): (f32, f32),
    top: f32,
) -> f32 {
    let vao = m::FIELD_GAP;
    let colunas = (((c.w + vao) / (lado_pedido + vao)).floor() as usize).max(1);
    let lado = (c.w - vao * (colunas - 1) as f32) / colunas as f32;
    let passo = lado + ROTULO_H + vao;
    for (i, e) in entradas.iter().enumerate() {
        let y = top + (i / colunas) as f32 * passo;
        if y + lado + ROTULO_H < topo || y > fundo {
            continue;
        }
        let x = c.x + (i % colunas) as f32 * (lado + vao);
        celula(c, i, e, Rect::new(x, y, lado, lado + ROTULO_H), lado);
    }
    top + entradas.len().div_ceil(colunas) as f32 * passo - vao
}

/// Uma miniatura: quadrado + nome + detalhe. O gesto é a célula inteira.
fn celula(c: &mut CardCtx<'_>, i: usize, e: &Entrada, r: Rect, lado: f32) {
    let id = ph2d_editor_core::ids::asset_cell_id(i);
    let ink: CardInk = c.ink;
    let quente = c.store.button_visual(id).0 != ButtonState::Normal;
    let q = Rect::new(r.x, r.y, lado, lado);
    fill_rounded_rect(
        c.scene,
        q,
        m::FIELD_RADIUS,
        ink.color(ColorToken::CardField),
    );
    let folga = m::GRID_GAP;
    fill_rounded_rect(
        c.scene,
        Rect::new(
            q.x + folga,
            q.y + folga,
            (q.w - folga * 2.0).max(0.0),
            (q.h - folga * 2.0).max(0.0),
        ),
        m::SEG_RADIUS,
        ph2d_editor_core::paint::token_to_vello(crate::card_backdrop::card_backdrop(
            ink.theme,
            e.swatch,
            e.thumb.is_some(),
        )),
    );
    if let Some(t) = &e.thumb {
        crate::paint_thumb::paint_thumb(c.scene, e.key, t, q, folga);
    } else if let Some(icone) = crate::paint::doc_icon(e.key) {
        let meio = lado * 0.5;
        ph2d_editor_core::paint::paint_icon(
            c.scene,
            icone,
            Rect::new(q.x + meio * 0.5, q.y + meio * 0.5, meio, meio),
            ink.color(ColorToken::Text2),
            StrokeToken::Default.px(),
        );
    }
    if quente {
        // FRAME-RAW-OK: a célula sob a mão, contornada no acento do cartão
        stroke_rounded_rect(
            c.scene,
            q,
            m::FIELD_RADIUS,
            StrokeToken::Default.px(),
            ink.color(ColorToken::Accent),
        );
    }
    let nome_y = q.y + lado + m::LABEL_BOTTOM;
    paint_text_elided(
        c.text_system,
        c.scene,
        &e.name,
        r.x,
        nome_y,
        m::VALUE_PX,
        r.w,
        ink.color(ColorToken::Text1),
    );
    paint_text_elided(
        c.text_system,
        c.scene,
        &e.detail,
        r.x,
        nome_y + m::VALUE_PX + m::LABEL_BOTTOM,
        m::LABEL_PX,
        r.w,
        ink.color(ColorToken::CardLabel),
    );
    c.hit_index.register(id, r);
}
