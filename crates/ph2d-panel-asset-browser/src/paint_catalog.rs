//! ⭐⭐ **A COLUNA DE CATÁLOGOS** (plano 07, wave A3) — irmão por assunto do [`super::paint`], que
//! tinha 61 linhas de folga contra o tecto de 600 dos painéis.
//!
//! # ⛔ Ela contradiz a decisão D2 do plano, e a reversão está MEDIDA
//!
//! A D2 escolheu split **vertical** (catálogos em cima) com um motivo explícito — *«`HSPLIT` só faz
//! sentido com largura que não temos»* —, e essa premissa era sobre um **dock estreito**. Este
//! painel nasceu **flutuante, mais largo que alto e redimensionável**. A tabela da medição e o que
//! sobra da D2 (o botão *só-grade*, que virou o interruptor desta coluna) vivem na
//! [§10 do plano](../../../docs/Components/07_plano_do_navegador_de_assets.md).
//!
//! # ⭐⭐⭐ A largura é DERIVADA, e é isso que impede um cartão cortado
//!
//! O `cols` da grade é `⌊(inner_w + gap) / (cell + gap)⌋` com um `.max(1.0)` no fim — ele **nunca
//! devolve zero**, devolve **um**, e um cartão de 84–160 px num vão de 64 px *não reflui: é cortado
//! pelo recorte*. O defeito já lá estava; uma coluna de largura fixa é que o punha ao alcance de um
//! redimensionamento normal.
//!
//! ⇒ [`col_w`] é o mínimo entre a largura nominal e o que sobra depois de a grade guardar **um
//! cartão inteiro**, e **colapsa a zero** quando nem isso cabe.

use crate::ids;
use crate::state::{AssetBrowserState, CatalogPick, with_catalogs};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{ButtonState, scrollbar_is_needed, scrollbar_thumb_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Density, Radius, Spacing, StrokeToken, TypeToken};

/// A largura nominal da coluna, em px.
///
/// ⚠️ **Medida contra o conteúdo, não escolhida**: `Spacing::Md` de recuo + até três níveis de
/// indentação (`Spacing::Md` cada) + um rótulo de ~12 caracteres a `TypeToken::Sm` + a contagem à
/// direita. Abaixo disto o nome de um catálogo de 2.º nível deixa de caber, e uma coluna que corta
/// todos os nomes não é uma coluna.
const NOMINAL_W: f32 = 140.0; // LITERAL-PX-OK: largura da coluna, domínio do painel

/// Abaixo disto ela colapsa: uma coluna que não mostra um nome não vale a largura que come.
const MIN_W: f32 = 96.0; // LITERAL-PX-OK: piso da coluna, domínio do painel

/// ⭐⭐ **A largura que a coluna pode ter neste quadro** — `0.0` = colapsada.
///
/// ⚠️ **A grade guarda um cartão INTEIRO**, e é essa subtracção que faz a coluna ceder primeiro. É
/// o oposto de dar a ela uma largura fixa e deixar o cartão ser cortado em silêncio.
#[must_use]
pub(crate) fn col_w(state: &AssetBrowserState, rect: Rect, pad: f32) -> f32 {
    if !state.show_catalogs {
        return 0.0;
    }
    let avail = rect.w - pad * 2.0;
    let w = NOMINAL_W.min(avail - state.cell_px);
    if w < MIN_W { 0.0 } else { w }
}

/// Uma linha da coluna, já derivada da taxonomia.
struct Row {
    pick: CatalogPick,
    label: String,
    depth: usize,
    count: usize,
}

/// ⭐ **As linhas que a coluna desenha** — derivadas da árvore, com as duas fixas à cabeça.
///
/// ⚠️ **`All` e `Unassigned` são LINHAS, e não estados escondidos num chip**: sem a segunda, um
/// asset por arrumar fica inalcançável no dia em que existir um catálogo.
fn rows(needle: &str) -> Vec<Row> {
    with_catalogs(|t| {
        let mut out = vec![
            Row {
                pick: CatalogPick::All,
                label: "All".into(),
                depth: 0,
                count: 0,
            },
            Row {
                pick: CatalogPick::Unassigned,
                label: "Unassigned".into(),
                depth: 0,
                count: 0,
            },
        ];
        for c in t.catalogs() {
            // ⚠️ **A busca da coluna casa o CAMINHO inteiro**, não só o rótulo: procurar
            // *«heróis»* tem de achar `"Personagens/Heróis"` mesmo com o pai fora do resultado.
            if !needle.is_empty() && !c.path.to_lowercase().contains(needle) {
                continue;
            }
            out.push(Row {
                pick: CatalogPick::One(c.id),
                label: c.label().to_string(),
                depth: c.depth(),
                count: t.count_in(c.id),
            });
        }
        out
    })
}

/// Desenha a coluna e devolve a largura que ela ocupou.
///
/// ⚠️ **Ela recorta nos DOIS canais e ANINHA** — o `HitIndex::push_clip` intersecta com o topo da
/// pilha, então o par dela é exacto mesmo dentro do recorte de outra região. Sem o segundo canal,
/// uma linha rolada para fora continua clicável e o artista filtra por um catálogo que não vê.
pub(crate) fn paint(
    state: &mut AssetBrowserState,
    ctx: &mut PaintCtx,
    rect: Rect,
    body_top: f32,
) -> f32 {
    let pad = Spacing::Md.px();
    let w = col_w(state, rect, pad);
    if w <= 0.0 {
        crate::state::set_painted_rows(Vec::new());
        // ⚠️ **Limpeza simétrica:** sem ela a roda continuaria a ser comida no sítio onde a coluna
        // esteve — o mesmo contrato que o `clear_panel_rect` do painel fechado.
        ctx.host
            .store_mut()
            .clear_sub_scroll_region(ids::ASSET_CATALOG_COL);
        return 0.0;
    }
    let theme = ctx.host.theme();
    let body_h = (rect.y + rect.h - body_top - pad).max(0.0);
    let col = Rect::new(rect.x + pad, body_top, w, body_h);

    // O fundo da coluna, para ela se ler como uma região e não como texto solto na grade.
    fill_rounded_rect(
        ctx.scene,
        col,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );

    let needle = ctx
        .host
        .store()
        .text(ids::ASSET_SEARCH)
        .unwrap_or_default()
        .to_lowercase();
    // ⛔ **A busca da GRADE não filtra a coluna.** São duas perguntas (plano 07 D1) e a segunda
    // busca é wave própria; até lá a coluna mostra tudo, que é o honesto — filtrá-la pela busca da
    // grade esconderia catálogos por causa de um texto que fala de assets.
    let _ = needle;
    let list = rows("");
    let row_h = Density::Compact.row_h_px();

    // ⭐ **O `+` fica FORA do recorte da lista**, no cabeçalho da coluna: ele tem de continuar
    // alcançável com a lista rolada até ao fim.
    let new_rect = Rect::new(
        col.x + Spacing::Xs.px(),
        col.y + Spacing::Xs.px(),
        col.w - Spacing::Xs.px() * 2.0,
        row_h,
    );
    let mut new_btn = ph2d_editor_core::widget::Button::new(ids::ASSET_CATALOG_NEW, "+ Catalog");
    new_btn.state = ctx.host.store().button_visual(ids::ASSET_CATALOG_NEW).0;
    ph2d_editor_core::widget::paint_button(&new_btn, new_rect, ctx.scene, ctx.text_system, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::ASSET_CATALOG_NEW, new_rect);

    // A lista começa DEPOIS do cabeçalho — e é esta a altura que a rolagem mede.
    let list_top = new_rect.y + row_h + Spacing::Xs.px();
    let list_rect = Rect::new(col.x, list_top, col.w, (col.y + col.h - list_top).max(0.0));
    let content_h = row_h * list.len() as f32;
    let scroll = ctx.host.store().panel_scroll(ids::ASSET_CATALOG_COL);

    ctx.scene.push_clip(&rect_to_vello(list_rect));
    ctx.host.hit_index_mut().push_clip(list_rect);
    let mut painted: Vec<CatalogPick> = Vec::new();
    // ⚠️ **O `y` da linha a renomear sai deste laço**, e não de uma segunda conta: o campo tem de
    // ficar exactamente onde a linha está, e duas contas divergem no dia em que a lista mudar.
    let renaming = state.renaming.map(|r| r.id);
    let mut rename_y: Option<f32> = None;
    for (i, r) in list.iter().enumerate() {
        if i >= ids::MAX_CATALOG_ROWS {
            break;
        }
        let y = list_rect.y + row_h * i as f32 - scroll;
        painted.push(r.pick);
        if renaming.is_some_and(|id| r.pick == CatalogPick::One(id)) {
            rename_y = Some(y);
        }
        // Fora do corpo: nada a pintar e — mais importante — nada a registar.
        if y + row_h < list_rect.y || y > list_rect.y + list_rect.h {
            continue;
        }
        let id = ids::catalog_row_id(i);
        let row = Rect::new(list_rect.x, y, list_rect.w, row_h);
        ctx.host.store_mut().register_if_absent(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        let chosen = state.pick == r.pick;
        if chosen {
            fill_rounded_rect(
                ctx.scene,
                row,
                Radius::Xs.px(),
                resolve(ColorToken::AccentSoft, theme),
            );
        }
        let indent = Spacing::Sm.px() + Spacing::Md.px() * r.depth as f32;
        paint_text(
            ctx.text_system,
            ctx.scene,
            &r.label,
            row.x + indent,
            // A mesma centragem vertical do `paint_list_item`, que é o primitivo desta forma.
            row.y + (row_h - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            (row.w - indent - Spacing::Lg.px()).max(0.0),
            resolve(
                if chosen {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            ),
        );
        if r.count > 0 {
            paint_text(
                ctx.text_system,
                ctx.scene,
                &r.count.to_string(),
                row.x + row.w - Spacing::Lg.px(),
                row.y + (row_h - TypeToken::Xs.px()) * 0.5,
                TypeToken::Xs.px(),
                Spacing::Lg.px(),
                resolve(ColorToken::Text2, theme),
            );
        }
        ctx.host.hit_index_mut().register(id, row);
    }
    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();
    crate::state::set_painted_rows(painted);
    crate::catalog_rename::paint(state, ctx, list_rect, row_h, rename_y);

    // A barra, DEPOIS do recorte — ela tem de sobreviver ao clip do corpo.
    if scrollbar_is_needed(content_h, list_rect.h) {
        let track = ph2d_editor_core::widget::scrollbar_track_rect(list_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let visual = ctx.host.store().scrollbar_visual_for(
            ph2d_editor_core::widget::ASSET_CATALOG_SCROLLBAR_ID,
            Some(ids::ASSET_CATALOG_COL),
        );
        ph2d_editor_core::widget::paint_scrollbar(
            col, scroll, content_h, body_h, visual, ctx.scene, theme,
        );
        ctx.host
            .hit_index_mut()
            .register(ph2d_editor_core::widget::ASSET_CATALOG_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::ASSET_CATALOG_COL, content_h);
    store.set_panel_visible_h(ids::ASSET_CATALOG_COL, list_rect.h);
    // ⭐ **Onde ela está**, para a roda a achar antes do painel que a contém.
    store.set_sub_scroll_region(ids::ASSET_CATALOG_COL, list_rect);

    // A fronteira entre a coluna e a grade — uma linha, não uma sombra: ela diz onde uma região
    // acaba sem competir com o realce da escolhida.
    ph2d_editor_core::paint::stroke_rounded_rect(
        ctx.scene,
        col,
        Radius::Sm.px(),
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    w
}

/// ⭐ **O escopo que a escolha desta coluna significa** — expandido pela árvore, no quadro.
///
/// ⚠️ Derivado a cada quadro de propósito: guardar a expansão no estado seria uma segunda resposta
/// que envelhece no instante em que alguém cria um filho.
#[must_use]
pub(crate) fn scope_of(pick: CatalogPick) -> ph2d_asset_index::CatalogScope {
    match pick {
        CatalogPick::All => ph2d_asset_index::CatalogScope::All,
        CatalogPick::Unassigned => ph2d_asset_index::CatalogScope::Unassigned,
        CatalogPick::One(id) => with_catalogs(|t| t.scope_of(id)),
    }
}
