//! ⭐ **A Hierarquia em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`, bloco F): a lista
//! flutua, a busca e o filtro ficam presos debaixo do título, e a árvore mora num cartão. As linhas têm
//! a densidade de uma árvore, não a altura de um campo, e só se pinta o que se vê. Ids, despacho e
//! `populate` são os de antes; `PH2D_CARDS=0` pinta o painel de antes (`paint.rs`).

use crate::row::paint_hierarchy_row;
use crate::row_ink::RowInk;
use crate::search::compute_match_filter;
use crate::{HierarchyPanel, filter, ids, rows, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::hier_rows::RowRole;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{WidgetStore, toggle_paint};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text_elided};
use ph2d_editor_core::panel::cards::linhas_extra::ListRow;
use ph2d_editor_core::panel::cards::{
    CardCtx, CardDef, ListTop, dicas, paint_card_list_top, scroll_at_edge,
};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::screens::hero::fixture::HierarchyEntity;
use ph2d_editor_core::widget::{RowHighlight, paint_row_stripe};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, list_row_gap_px};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

const PAINEL: NodeId = core_ids::HIER_PANEL;

thread_local! {
    static FORCADO: Cell<Option<bool>> = const { Cell::new(None) };
    /// Onde a 1.ª linha cai no conteúdo rolável, e o topo da banda visível: a vista de quem traz uma
    /// linha ao olhar e de quem rola na borda. São do quadro anterior: o conteúdo mede-se ao pintar.
    static GEOMETRIA: Cell<Geometria> = const { Cell::new(Geometria { arvore: 0.0, topo: 0.0 }) };
}

#[derive(Copy, Clone)]
struct Geometria {
    arvore: f32,
    topo: f32,
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (os testes de um processo correm em paralelo).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

type Linhas = BTreeMap<NodeId, HierarchyEntity>;

/// Uma linha mostrada, já resolvida a dobra e a busca.
#[derive(Copy, Clone)]
struct Mostrada {
    id: NodeId,
    /// Índice na ordem inteira (as guias de parentesco olham para a frente nela).
    i: usize,
    depth: u32,
    filhos: bool,
    recolhida: bool,
    casa: bool,
}

/// O que se resolveu antes de pintar: custa `O(n)`, sem copiar uma única linha.
struct Arvore {
    entradas: Rc<Linhas>,
    ordem: Vec<NodeId>,
    depths: Vec<u32>,
    mostradas: Vec<Mostrada>,
    renomeia: Option<NodeId>,
}

fn monta(store: &WidgetStore, filtro: Option<u8>, renomeia: Option<NodeId>) -> Arvore {
    let vivas = state::live_entries_shared();
    let viva = vivas.is_some();
    let entradas = vivas.unwrap_or_else(|| Rc::new(crate::paint::entries_now()));
    let ordem = store.hierarchy_order().to_vec();
    let depths: Vec<u32> = ordem
        .iter()
        .map(|id| {
            if viva {
                entradas.get(id).map_or(0, |e| u32::from(e.indent))
            } else {
                store.hierarchy_depth_of(*id)
            }
        })
        .collect();
    let busca = store
        .text(ids::HIER_SEARCH)
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let buscando = !busca.is_empty() || filtro.is_some();
    let (mostra, casa) = if buscando {
        compute_match_filter(&ordem, &depths, &entradas, &busca, filtro)
    } else {
        (Vec::new(), Vec::new())
    };
    let mut mostradas = Vec::with_capacity(ordem.len());
    let mut fechada: Option<u32> = None;
    for (i, id) in ordem.iter().enumerate() {
        let Some(e) = entradas.get(id) else { continue };
        let depth = depths[i];
        if fechada.is_some_and(|f| depth <= f) {
            fechada = None;
        }
        if fechada.is_some() || (buscando && !mostra.get(i).copied().unwrap_or(false)) {
            continue;
        }
        let filhos = depths.get(i + 1).is_some_and(|&d| d > depth);
        let recolhida = filhos && !buscando && rows::is_collapsed(store, *id, e);
        if recolhida {
            fechada = Some(depth);
        }
        mostradas.push(Mostrada {
            id: *id,
            i,
            depth,
            filhos,
            recolhida,
            casa: buscando && casa.get(i).copied().unwrap_or(false),
        });
    }
    Arvore {
        entradas,
        ordem,
        depths,
        mostradas,
        renomeia,
    }
}

/// A altura de uma linha mais o vão: o mesmo do painel de antes.
fn passo() -> f32 {
    crate::paint::HIER_ROW_H + list_row_gap_px()
}

/// As linhas que caem dentro de `(topo, fundo)`, com a 1.ª em `y0`.
fn faixa(y0: f32, (topo, fundo): (f32, f32), n: usize) -> std::ops::Range<usize> {
    let p = passo();
    let de = (((topo - y0) / p).floor().max(0.0) as usize).min(n);
    let ate = (((fundo - y0) / p).ceil().max(0.0) as usize).min(n);
    de..ate.max(de)
}

/// O texto da contagem: objectos e, se houver, componentes.
fn contagem(entradas: &Linhas, viva: bool) -> String {
    let (objetos, componentes) = if viva {
        (
            rows::object_count(entradas),
            state::current_component_count(),
        )
    } else {
        ph2d_editor_core::screens::hero::fixture::hierarchy_counts()
    };
    if componentes > 0 {
        ph2d_i18n::tr_with(
            "panel.hierarchy.count.entities_components",
            &[("entities", &objetos), ("components", &componentes)],
        )
    } else {
        ph2d_i18n::tr_with("panel.hierarchy.count.entities", &[("entities", &objetos)])
    }
}

/// Com o store mutável: o que o corpo não pode escrever. Corre uma vez por quadro.
fn prepare(
    st: &mut state::HierarchyState,
    store: &mut WidgetStore,
    a: &Arvore,
    janela: (f32, f32),
) {
    toggle_paint::limpa_grupo(ids::HIER_TOGGLE_PAINT_GROUP);
    store.set_hierarchy_row_ids(rows::object_rows(&a.ordem, &a.entradas));
    state::set_last_hierarchy_content_h(a.mostradas.len() as f32 * passo());
    for (id, chave) in [
        (core_ids::HIERARCHY_ADD, "card.hierarchy.add"),
        (ids::HIER_FILTER_DD, "card.hierarchy.filter"),
    ] {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(chave));
        }
    }
    let g = GEOMETRIA.with(Cell::get);
    if let Some(d) = store.hierarchy_drag().filter(|d| d.active)
        && store.hierarchy_drag_in_own_panel()
    {
        scroll_at_edge(store, PAINEL, d.cursor_y, g.topo);
    }
    traz_a_selecao(st, store, a, g);
    let y0 = g.arvore - store.panel_scroll(PAINEL).max(0.0);
    for l in &a.mostradas[faixa(y0, janela, a.mostradas.len())] {
        dicas_da_linha(store, l.id, a.entradas.get(&l.id));
    }
}

/// ⭐ Uma selecção nova vinda de fora (o canvas, o Desfazer) rola a lista até à linha dela; a linha
/// que já se vê não mexe a lista, logo o clique na própria lista não a faz saltar.
fn traz_a_selecao(
    st: &mut state::HierarchyState,
    store: &mut WidgetStore,
    a: &Arvore,
    g: Geometria,
) {
    let na_ordem = a.ordem.iter().position(|id| {
        a.entradas
            .get(id)
            .is_some_and(|e| e.selected && e.role == RowRole::Object)
    });
    let id = na_ordem.map(|i| a.ordem[i]);
    let primeira = a.mostradas.iter().position(|l| Some(l.id) == id);
    // Antes do 1.º quadro medido não há vista: a selecção do arranque espera por ela. O cartão solto
    // rola por dentro, não pela lista.
    let Some(visivel) = store.panel_visible_h(PAINEL) else {
        return;
    };
    if id == st.revealed || store.is_loose_in(ids::HIER_SEC_TREE, PAINEL) {
        return;
    }
    // Num ramo recolhido abre-o agora e rola no quadro seguinte. Recolher depois com a mesma
    // selecção não reabre: essa já se revelou.
    if let (Some(i), None) = (na_ordem, primeira) {
        abre_os_ramos(store, a, i);
        return;
    }
    st.revealed = id;
    let Some(k) = primeira else { return };
    // Em coordenadas do conteúdo, onde a rolagem é o topo da vista.
    let origem = g.arvore - g.topo;
    let (cima, baixo) = (
        origem + k as f32 * passo(),
        origem + (k + 1) as f32 * passo(),
    );
    let agora = store.panel_scroll_target(PAINEL);
    let alvo = if cima < agora {
        cima
    } else if baixo > agora + visivel {
        baixo - visivel
    } else {
        return;
    };
    let max = store
        .panel_content_h(PAINEL)
        .map_or(alvo, |c| (c - visivel).max(0.0));
    store.set_panel_scroll(PAINEL, alvo.clamp(0.0, max));
}

/// Abre os antepassados recolhidos da linha `i` da ordem inteira.
fn abre_os_ramos(store: &mut WidgetStore, a: &Arvore, i: usize) {
    let mut nivel = a.depths[i];
    for j in (0..i).rev() {
        if a.depths[j] >= nivel {
            continue;
        }
        nivel = a.depths[j];
        let id = a.ordem[j];
        if store.is_hierarchy_collapsed(id) {
            store.toggle_hierarchy_collapsed(id);
        }
    }
}

/// Todo ícone mudo da linha diz o que faz (T31). Só nas linhas à vista: é uma por quadro no pior caso.
fn dicas_da_linha(store: &mut WidgetStore, row: NodeId, e: Option<&HierarchyEntity>) {
    let Some(e) = e else { return };
    let mut dica = |id: NodeId, chave: &str| {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(chave));
        }
    };
    dica(
        core_ids::hier_expand_companion(row),
        "card.hierarchy.expand",
    );
    dica(ids::hier_icon_companion(row), "card.hierarchy.icon");
    if e.role == RowRole::Object {
        dica(core_ids::hier_eye_companion(row), "card.hierarchy.eye");
        dica(ids::hier_lock_companion(row), "card.hierarchy.lock");
        dica(ids::hier_group_companion(row), "card.hierarchy.group");
    }
    for c in &e.components {
        dica(
            ids::hier_component_icon(row, *c),
            "card.hierarchy.component",
        );
    }
}

/// O rato está sobre a linha ou sobre um dos ícones dela? (`hover_live` diz 1 a quem o relógio nunca viu.)
fn sob_o_rato(store: &WidgetStore, id: NodeId) -> bool {
    store.hot_id().is_some_and(|h| {
        h == id
            || core_ids::hier_expand_companion_to_row(h) == Some(id)
            || core_ids::hier_eye_companion_to_row(h) == Some(id)
            || core_ids::hier_lock_companion_to_row(h) == Some(id)
            || core_ids::hier_group_companion_to_row(h) == Some(id)
            || core_ids::hier_icon_companion_to_row(h) == Some(id)
    })
}

/// O fundo de uma linha na largura inteira do cartão: à Kanban/Figma, sem moldura.
fn fundo(c: &mut CardCtx<'_>, r: Rect, k: usize, e: &HierarchyEntity, id: NodeId) {
    paint_row_stripe(c.scene, r, c.ink.theme, ColorToken::KanbanCard, k);
    let tom = if e.selected {
        RowHighlight::Selected
    } else if e.hovered || sob_o_rato(c.store, id) {
        RowHighlight::Hovered
    } else {
        RowHighlight::None
    };
    let cor = match tom {
        RowHighlight::Selected => Some(ColorToken::KanbanCardRaised),
        RowHighlight::Hovered => Some(ColorToken::CardFieldHover),
        RowHighlight::None => None,
    };
    if let Some(t) = cor {
        fill_rounded_rect(c.scene, r, m::SEG_RADIUS, c.ink.color(t));
    }
}

/// O corpo do cartão da árvore: pinta só o que cai em `janela` e devolve o fundo da árvore inteira.
fn arvore(c: &mut CardCtx<'_>, a: &Arvore, janela: (f32, f32), y0: f32) -> f32 {
    // O cartão que voa ou anda solto pinta-se noutra origem: essa não é a da lista.
    if c.store.card_drag().is_none() && !c.store.is_loose_in(ids::HIER_SEC_TREE, PAINEL) {
        GEOMETRIA.with(|g| {
            let mut v = g.get();
            v.arvore = y0 + c.store.panel_scroll(PAINEL).max(0.0);
            g.set(v);
        });
    }
    let (x0, w) = (c.x - m::TREE_ROW_BLEED, c.w + m::TREE_ROW_BLEED * 2.0);
    let (p, indent) = (passo(), ph2d_tokens::list_indent_px());
    let tinta = RowInk::Card(c.ink);
    let arrasto = c.store.hierarchy_drag().filter(|d| d.active);
    let leva_selecao =
        arrasto.is_some_and(|d| a.entradas.get(&d.dragged).is_some_and(|e| e.selected));
    let faixa_vista = faixa(y0, janela, a.mostradas.len());
    let mut retas: Vec<(NodeId, Rect)> = Vec::with_capacity(faixa_vista.len());
    for k in faixa_vista {
        let l = a.mostradas[k];
        let Some(molde) = a.entradas.get(&l.id) else {
            continue;
        };
        let y = y0 + k as f32 * p;
        let recuo = l.depth as f32 * indent;
        let inteira = Rect::new(x0, y, w, crate::paint::HIER_ROW_H);
        let linha = Rect::new(x0 + recuo, y, w - recuo, crate::paint::HIER_ROW_H);
        c.hit_index.register(l.id, inteira);
        fundo(c, inteira, k, molde, l.id);
        let mut e = molde.clone();
        e.muted = e.muted
            || arrasto.is_some_and(|d| {
                crate::paint::row_rides_the_drag(l.id == d.dragged, leva_selecao, e.selected)
            });
        let renomeia = a.renomeia == Some(l.id);
        if renomeia {
            e.name = String::new();
        }
        let nome_fim = paint_hierarchy_row(
            &e,
            linha,
            c.scene,
            c.text_system,
            tinta,
            Some(l.id),
            Some(&mut *c.hit_index),
            l.filhos,
            l.recolhida,
            l.casa,
        );
        marca_os_icones(l.id, &e, l.filhos);
        crate::paint::paint_parentage_lines(
            c.scene, tinta, linha, x0, indent, l.depth, &a.depths, l.i,
        );
        if renomeia {
            let x = crate::row::name_x(linha.x) - Spacing::Xs.px();
            let fim_caixa = nome_fim - Spacing::Xxs.px();
            let caixa = Rect::new(
                x,
                y + 1.0,
                (fim_caixa - x).max(m::FIELD_VALUE_MIN_W),
                linha.h - 2.0,
            );
            c.text_box(caixa, core_ids::HIER_RENAME_INPUT, "");
        }
        retas.push((l.id, linha));
    }
    let fim = y0 + a.mostradas.len() as f32 * p;
    if let Some(d) = arrasto.filter(|_| c.store.hierarchy_drag_in_own_panel()) {
        let resto = Rect::new(x0, fim - 1.0, w, 2.0);
        crate::paint::paint_drop_indicator(d, &retas, resto, c.scene, tinta);
        voa_o_que_vai_na_mao(c, a, d, leva_selecao);
    }
    fim
}

/// As linhas à vista no voo de uma multisselecção; as outras dizem-se por «+n more».
const LINHAS_NO_VOO: usize = 5;

/// O voo mostra o que vai na mão: a linha agarrada e, se ela está escolhida, as outras escolhidas.
fn voa_o_que_vai_na_mao(
    c: &mut CardCtx<'_>,
    a: &Arvore,
    d: ph2d_editor_core::interaction::HierarchyDragState,
    leva_selecao: bool,
) {
    let Some(agarrada) = a.entradas.get(&d.dragged) else {
        return;
    };
    let outras: Vec<&HierarchyEntity> = a
        .ordem
        .iter()
        .filter(|id| leva_selecao && **id != d.dragged)
        .filter_map(|id| a.entradas.get(id))
        .filter(|e| e.selected && e.role == RowRole::Object)
        .collect();
    let total = outras.len() + 1;
    let inteiras = if total > LINHAS_NO_VOO {
        LINHAS_NO_VOO - 1
    } else {
        total
    };
    let mais = total - inteiras;
    let resto = ph2d_i18n::tr_with("panel.hierarchy.flight_more", &[("n", &mais)]);
    let mut linhas: Vec<ListRow<'_>> = std::iter::once(agarrada)
        .chain(outras.iter().copied())
        .take(inteiras)
        .map(|e| ListRow {
            icon: Some(e.icon),
            ..ListRow::new(d.dragged, &e.name)
        })
        .collect();
    if mais > 0 {
        linhas.push(ListRow::new(d.dragged, &resto));
    }
    c.list_rows_flight(
        &linhas,
        d.cursor_x - (d.down_x - c.x),
        d.cursor_y - m::FIELD_H * 0.5,
    );
}

/// Para o censo das dicas: estes ícones não têm texto a explicá-los.
fn marca_os_icones(row: NodeId, e: &HierarchyEntity, filhos: bool) {
    if filhos {
        dicas::icone(core_ids::hier_expand_companion(row));
    }
    dicas::icone(ids::hier_icon_companion(row));
    if e.role == RowRole::Object {
        dicas::icone(core_ids::hier_eye_companion(row));
        dicas::icone(ids::hier_lock_companion(row));
        dicas::icone(ids::hier_group_companion(row));
    }
    for c in &e.components {
        dicas::icone(ids::hier_component_icon(row, *c));
    }
}

/// A caixa do filtro: a cara de uma lista suspensa do cartão. Abrir e escolher são os de sempre.
fn caixa_do_filtro(c: &mut CardCtx<'_>, r: Rect, texto: &str) {
    c.hit_index.register(ids::HIER_FILTER_DD, r);
    let tom = if c.store.hot_id() == Some(ids::HIER_FILTER_DD) {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(c.scene, r, m::FIELD_RADIUS, c.ink.color(tom));
    let g = m::CARD_CHEVRON;
    paint_text_elided(
        c.text_system,
        c.scene,
        texto,
        r.x + m::FIELD_PAD_R,
        r.y + ph2d_editor_core::paint::folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.w - m::FIELD_PAD_R * 2.0 - g).max(0.0),
        c.ink.color(ColorToken::Text1),
    );
    paint_icon(
        c.scene,
        IconId::ChevronDown,
        Rect::new(r.x + r.w - m::SEG_INSET - g, r.y + (r.h - g) * 0.5, g, g),
        c.ink.color(ColorToken::Text2),
        StrokeToken::Default.px(),
    );
}

/// ⭐ Pinta a lista: título com «+» e «…», a busca presa, a árvore no cartão.
pub(crate) fn paint(st: &mut state::HierarchyState, ctx: &mut PaintCtx) {
    let escolhido = st.component_filter;
    let a = monta(ctx.host.store(), escolhido, st.rename_target_row);
    let janela = (ctx.viewport.y, ctx.viewport.y + ctx.viewport.h);
    prepare(st, ctx.host.store_mut(), &a, janela);
    let contado = contagem(&a.entradas, state::live_entries_shared().is_some());
    let cartoes = [CardDef {
        section: ids::HIER_SEC_TREE,
        title: &contado,
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(|c, y| arvore(c, &a, janela, y))),
    }];
    let filtro_texto = filter::chosen_label(&a.entradas, escolhido);
    let caixa = Cell::new(Rect::new(0.0, 0.0, 0.0, 0.0));
    let topo = |c: &mut CardCtx<'_>, r: Rect| {
        let cw = filter::FILTER_W.min(r.w * 0.5);
        let f = Rect::new(r.x + r.w - cw, r.y, cw, r.h);
        caixa.set(f);
        let busca = Rect::new(r.x, r.y, (r.w - cw - m::FIELD_GAP).max(0.0), r.h);
        c.text_box(busca, ids::HIER_SEARCH, tr("panel.hierarchy.search"));
        caixa_do_filtro(c, f, &filtro_texto);
    };
    let lista = paint_card_list_top(
        ctx,
        PAINEL,
        HierarchyPanel::TITLE.tr(),
        &cartoes,
        None,
        &[],
        Some(ListTop {
            h: m::FIELD_H,
            body: &topo,
            add: Some((core_ids::HIERARCHY_ADD, tr("card.hierarchy.add"))),
        }),
    );
    GEOMETRIA.with(|g| {
        let mut v = g.get();
        v.topo = lista.y + m::LIST_HEAD_H + m::FIELD_H + m::LIST_TOP_GAP;
        g.set(v);
    });
    if filter::is_open(ctx.host.store()) {
        let theme = ctx.host.theme();
        // A lista aberta tem a cara das do cartão: a porta pinta-a assim dentro desta tinta.
        let tinta = ph2d_editor_core::widget::card_ink::CardInk::of_card(None, theme);
        ph2d_editor_core::widget::card_ink::com(tinta, || {
            filter::paint_popover(ctx, &a.entradas, escolhido, caixa.get(), theme);
        });
    }
}
