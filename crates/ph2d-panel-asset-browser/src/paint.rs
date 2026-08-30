//! O desenho do navegador — cabeçalho, controlos, e a **grade de cartões**.
//!
//! ⚠️ **A grade é um widget NOVO** (plano 07 §8 risco 2): o catálogo de widgets tem `tree_view` e
//! **nada** de grade. Ela nasce aqui, dentro do painel, e a promoção para
//! `ph2d-editor-core/src/widget/` acontece quando existir um SEGUNDO consumidor — antes disso a
//! promoção é adivinhar a forma da generalização.
//!
//! ⚠️ **O corpo é recortado nos DOIS canais**: `scene.push_clip` para os pixels e
//! `HitIndex::push_clip` para o dedo. Só o primeiro é a doença que o painel do Motion pagou — um
//! cartão rolado para fora do corpo continua **clicável** e o artista instancia o que não vê.

use crate::AssetBrowserPanel;
use crate::ids;
use crate::state::{AssetBrowserState, is_published, painted_at, with_index};
use ph2d_asset_index::{AssetEntry, Query};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text, paint_text_centered, rect_to_vello, resolve,
};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE, clamp_panel_rect,
    paint_panel_corner_dot, paint_panel_surface, paint_panel_title, panel_drag_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    ASSET_BROWSER_SCROLLBAR_ID, Button, ButtonKind, Slider, SliderOrientation, TextInput,
    TextInputState, paint_button, paint_scrollbar, paint_slider, paint_text_input_with_buffer,
    scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Density, Radius, Spacing, StrokeToken, TypeToken};

/// O estado do campo de busca — o mesmo idioma da Hierarquia.
fn read_search(
    store: &ph2d_editor_core::interaction::WidgetStore,
) -> (TextInputState, String, usize, Option<usize>) {
    match store.get(ids::ASSET_SEARCH) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    }
}

/// Espaçamento interno do painel.
fn pad() -> f32 {
    Spacing::Md.px()
}

/// Folga entre fileiras.
fn gap() -> f32 {
    Spacing::Sm.px()
}

/// Geometria com que o painel nasce.
///
/// ⚠️ **Mais largo que alto**, ao contrário dos irmãos: uma grade precisa de colunas, e um painel
/// estreito degenera na lista que ele veio substituir.
#[must_use]
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    let w = 420.0_f32.min(viewport_w - 16.0).max(300.0); // LITERAL-PX-OK: largura do navegador + margem/piso (chrome)
    let h = 520.0_f32.min(viewport_h - 16.0).max(320.0); // LITERAL-PX-OK: altura do navegador + margem/piso (chrome)
    let x = (viewport_w - w - 24.0).max(8.0); // LITERAL-PX-OK: encosta à direita, onde o pill vive
    let y = ((viewport_h - h) * 0.5).max(8.0); // LITERAL-PX-OK: inset de borda
    Rect::new(x, y, w, h)
}

pub(crate) fn paint(state: &mut AssetBrowserState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(AssetBrowserPanel::ID) {
        // Limpeza simétrica do rect: sem ela o `panel_at` continua a devolver este painel depois
        // de fechado, e a roda do rato de um painel por baixo vai parar a um painel invisível.
        ctx.host.store_mut().clear_panel_rect(ids::ASSET_PANEL);
        // ⛔ E os cartões deixam de existir para o despachante — senão um `Down` no sítio onde o
        // painel ESTAVA arrancaria um arrasto de um cartão que já ninguém pinta.
        ctx.host
            .store_mut()
            .set_asset_cells(std::collections::BTreeMap::new());
        crate::state::set_painted(Vec::new());
        return;
    }
    let base = match state.rect {
        Some(r) => r,
        None => {
            let r = default_rect(ctx.layout.viewport.w, ctx.layout.viewport.h);
            state.rect = Some(r);
            r
        }
    };
    let viewport = ctx.viewport;
    let off = ctx.host.store().blender_picker_offset(ids::ASSET_PANEL);
    let resize = ctx.host.store().panel_resize_delta(ids::ASSET_PANEL);
    let (rect, clamped_off, clamped_resize) = clamp_panel_rect(base, off, resize, viewport);
    {
        let store = ctx.host.store_mut();
        if (clamped_off.0 - off.0).abs() > f32::EPSILON
            || (clamped_off.1 - off.1).abs() > f32::EPSILON
        {
            store.set_blender_picker_offset(ids::ASSET_PANEL, clamped_off.0, clamped_off.1);
        }
        if (clamped_resize.0 - resize.0).abs() > f32::EPSILON
            || (clamped_resize.1 - resize.1).abs() > f32::EPSILON
        {
            store.set_panel_resize_delta(ids::ASSET_PANEL, clamped_resize.0, clamped_resize.1);
        }
        store.set_panel_rect(ids::ASSET_PANEL, rect);
    }
    paint_chrome(ctx, rect);
    let body_top = paint_controls(state, ctx, rect);
    paint_grid(state, ctx, rect, body_top);
}

/// Superfície, título, faixa de arrasto, fechar e alça de redimensionar.
fn paint_chrome(ctx: &mut PaintCtx, rect: Rect) {
    let theme = ctx.host.theme();
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    let drag = panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
    let resize_bl = panel_resize_handle_rect_bl(rect);
    {
        let hit = ctx.host.hit_index_mut();
        hit.register(ids::ASSET_DRAG_HANDLE, drag);
        hit.register(ids::ASSET_RESIZE_HANDLE_BL, resize_bl);
    }
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    paint_panel_title(
        rect,
        "Assets",
        Spacing::Xl3.px(),
        ctx.scene,
        ctx.text_system,
        theme,
    );
    let close_size = Density::Compact.row_h_px();
    let close_rect = Rect::new(
        rect.x + rect.w - close_size - pad(),
        title_y - 2.0, // LITERAL-PX-OK: alinhamento óptico do X com a linha de base do título
        close_size,
        close_size,
    );
    paint_icon(
        ctx.scene,
        ph2d_editor_core::icons::IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );
    ctx.host
        .hit_index_mut()
        .register(ids::ASSET_CLOSE, close_rect);
}

/// A busca, os chips de família, os de ordenação e o slider de tamanho. Devolve o `y` do corpo.
fn paint_controls(state: &AssetBrowserState, ctx: &mut PaintCtx, rect: Rect) -> f32 {
    let theme = ctx.host.theme();
    let x = rect.x + pad();
    let w = rect.w - pad() * 2.0;
    let row_h = Density::Compact.row_h_px();
    let mut y = rect.y + PANEL_TITLE_BASELINE + row_h + gap();

    // ── A busca da grade ───────────────────────────────────────────────────────────────────────
    let (st, value, caret, anchor) = read_search(ctx.host.store());
    let input = TextInput::new(ids::ASSET_SEARCH, "")
        .placeholder("Search assets\u{2026}")
        .visual((st, ctx.host.store().hover_live(ids::ASSET_SEARCH)));
    let search_rect = Rect::new(x, y, w, row_h);
    paint_text_input_with_buffer(
        &input,
        Some(value.as_str()),
        Some(caret),
        anchor,
        search_rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(ids::ASSET_SEARCH, search_rect);
    y += row_h + gap();

    // ── Família + ordenação, numa fileira de chips cada ────────────────────────────────────────
    y = paint_chip_row(
        ctx,
        x,
        w,
        y,
        row_h,
        ids::ASSET_KIND_FILTERS,
        |i| AssetBrowserState::kind_chip_label(i).to_string(),
        |i| AssetBrowserState::kind_for_chip(i) == state.kind,
        ids::ASSET_KIND,
    );
    y = paint_chip_row(
        ctx,
        x,
        w,
        y,
        row_h,
        ids::ASSET_SORT_MODES,
        |i| {
            ph2d_asset_index::SortBy::ALL
                .get(i)
                .map_or_else(String::new, |s| s.label().to_string())
        },
        |i| ph2d_asset_index::SortBy::ALL.get(i) == Some(&state.sort),
        ids::ASSET_SORT,
    );

    // ── O slider do tamanho do cartão ──────────────────────────────────────────────────────────
    let slider_rect = Rect::new(x, y, w, row_h);
    let mut slider = Slider::new(ids::ASSET_SIZE, "");
    slider.value = state.size_slider_value();
    slider.orientation = SliderOrientation::Horizontal;
    slider.state = ctx
        .host
        .store()
        .slider(ids::ASSET_SIZE)
        .map_or(ph2d_editor_core::widget::SliderState::Normal, |(s, _)| s);
    slider.hover_t = ctx.host.store().hover_live(ids::ASSET_SIZE);
    paint_slider(&slider, slider_rect, ctx.scene, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::ASSET_SIZE, slider_rect);
    y + row_h + gap()
}

/// Uma fileira de chips que reparte a largura por igual. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
fn paint_chip_row(
    ctx: &mut PaintCtx,
    x: f32,
    w: f32,
    y: f32,
    row_h: f32,
    count: usize,
    label: impl Fn(usize) -> String,
    selected: impl Fn(usize) -> bool,
    table: [ph2d_a11y::NodeId; 3],
) -> f32 {
    let theme = ctx.host.theme();
    let step = (w + gap()) / count as f32;
    for (i, id) in table
        .iter()
        .copied()
        .enumerate()
        .take(count.min(table.len()))
    {
        let r = Rect::new(x + step * i as f32, y, step - gap(), row_h);
        let mut b = Button::new(id, label(i));
        b.kind = if selected(i) {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let (st, hover) = ctx.host.store().button_visual(id);
        b.state = st;
        b.hover_t = hover;
        paint_button(&b, r, ctx.scene, ctx.text_system, theme);
        ctx.host.hit_index_mut().register(id, r);
    }
    y + row_h + gap()
}

/// A grade de cartões, recortada e rolável.
fn paint_grid(state: &AssetBrowserState, ctx: &mut PaintCtx, rect: Rect, body_top: f32) {
    let theme = ctx.host.theme();
    let x = rect.x + pad();
    let body_h = (rect.y + rect.h - body_top - pad()).max(0.0);
    let body = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::ASSET_PANEL);

    let q = Query {
        text: ctx
            .host
            .store()
            .text(ids::ASSET_SEARCH)
            .unwrap_or_default()
            .to_string(),
        kind: state.kind,
        catalog: None,
        sort: state.sort,
    };

    // ⚠️ **Uma consulta, uma travessia**: o que se pinta e o que se despacha saem da MESMA lista.
    let (keys, cards, beyond, total) = with_index(|index| {
        let hits = index.query(&q);
        let total = index.len();
        let shown = hits.len().min(ids::MAX_ASSET_CELLS);
        let beyond = hits.len() - shown;
        let keys: Vec<_> = hits.iter().take(shown).map(|e| e.key).collect();
        // ⚠️ **A miniatura é CLONADA para fora do índice**, e o clone é barato de propósito: o
        // `Thumb` é um `Arc` + dois `u32`, então o que viaja é uma contagem de referências. É isso
        // que mantém o `Arc::ptr_eq` do memo de textura a acertar — copiar os bytes aqui daria um
        // ponteiro novo por quadro e o `vello` reenviaria cada cartão ao atlas.
        let cards: Vec<(String, String, [u8; 4], Option<ph2d_asset_index::Thumb>)> = hits
            .iter()
            .take(shown)
            .map(|e| (e.name.clone(), e.detail.clone(), e.swatch, e.thumb.clone()))
            .collect();
        (keys, cards, beyond, total)
    });

    let cell = state.cell_px;
    // A faixa do rótulo: **as duas linhas que o cartão desenha**, derivadas dos mesmos tokens que
    // as pintam (`paint_card`) — não de um múltiplo escolhido à mão.
    let label_h = TypeToken::Sm.px() + Spacing::Xxs.px() + TypeToken::Xs.px() + Spacing::Xs.px();
    let card_h = cell + label_h;
    let inner_w = rect.w - pad() * 2.0;
    let cols = ((inner_w + gap()) / (cell + gap())).floor().max(1.0) as usize;
    let rows = cards.len().div_ceil(cols);
    let beyond_h = if beyond > 0 {
        Density::Compact.row_h_px()
    } else {
        0.0
    };
    let content_h = rows as f32 * (card_h + gap()) + beyond_h;

    // ⛔ **Os DOIS canais de recorte.** Sem o segundo, um cartão rolado para fora continua
    // clicável e o artista instancia o que não vê.
    ctx.scene.push_clip(&rect_to_vello(body));
    ctx.host.hit_index_mut().push_clip(body);

    if cards.is_empty() {
        // ⚠️ **Vazio e por-publicar dizem coisas diferentes.** Um balde que ninguém encheu lê-se
        // como «não tenho nada», e é o defeito que a memória `a_bucket_nobody_fills` nomeia.
        let msg = if !is_published() {
            "Loading assets\u{2026}"
        } else if total == 0 {
            "No assets yet \u{2014} right-click an object and choose Make Prefab"
        } else {
            "Nothing matches this search"
        };
        paint_text_centered(
            ctx.text_system,
            ctx.scene,
            msg,
            Rect::new(x, body_top, inner_w, Density::Comfortable.row_h_px() * 2.0),
            TypeToken::Sm.px(),
            resolve(ColorToken::Text2, theme),
        );
    }

    // Os cartões que este quadro de facto pinta — enchido pelo laço, e por isso honesto.
    let mut painted_cells: std::collections::BTreeMap<ph2d_a11y::NodeId, usize> =
        std::collections::BTreeMap::new();
    for (i, (name, detail, swatch, thumb_img)) in cards.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let cx = x + col as f32 * (cell + gap());
        let cy = body_top + row as f32 * (card_h + gap()) - scroll;
        // Fora do corpo: nada a pintar, e — mais importante — nada a registar.
        if cy + card_h < body.y || cy > body.y + body.h {
            continue;
        }
        let id = ids::asset_cell_id(i);
        painted_cells.insert(id, i);
        // ⛔⛔ **`register_if_absent`, e não `register` — a auditoria apanhou-me a usar o segundo.**
        // O `register` SUBSTITUI sempre, então o `state: Normal` deste quadro apagaria o `Pressed`
        // que o `pointer_down` escreveu no quadro anterior: o cartão ficaria a piscar e o clique a
        // meio caminho. `register` é a chamada certa para fiação de construção, uma vez; um laço
        // de paint precisa do irmão.
        ctx.host.store_mut().register_if_absent(
            id,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
        paint_card(
            ctx,
            theme,
            id,
            Rect::new(cx, cy, cell, card_h),
            cell,
            name,
            detail,
            *swatch,
            thumb_img.as_ref(),
            keys[i],
        );
    }

    if beyond > 0 {
        let y = body_top + rows as f32 * (card_h + gap()) - scroll;
        paint_text(
            ctx.text_system,
            ctx.scene,
            &format!("+{beyond} more \u{2014} narrow the search to reach them"),
            x,
            y,
            TypeToken::Sm.px(),
            inner_w,
            resolve(ColorToken::Text2, theme),
        );
    }

    ctx.host.hit_index_mut().pop_clip();
    ctx.scene.pop_layer();

    // ⭐⭐ **Os cartões dizem-se ao despachante** (etapa B): o `pointer_down` precisa de responder
    // *«este id é um cartão?»* sem conhecer este painel — o mesmo idioma do
    // `set_hierarchy_row_ids`, e pela mesma razão (quantos existem só se sabe em runtime).
    //
    // ⛔ **Só os que foram DE FACTO pintados**, e a 1.ª versão dizia isso e fazia outra coisa: ela
    // publicava `0..keys.len()`, que inclui os cartões que o laço acima **saltou** por estarem
    // rolados para fora do corpo. Hoje o laço enche `painted_cells`, então a lista é o que ela diz
    // ser. *Um comentário que nomeia uma propriedade que o código não tem é a próxima armadilha.*
    ctx.host.store_mut().set_asset_cells(painted_cells);
    crate::state::set_painted(keys);
    crate::state::set_last_content_h(content_h);
    crate::state::set_last_visible_h(body_h);

    // A barra, e o clamp da rolagem.
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let visual = ctx
            .host
            .store()
            .scrollbar_visual_for(ASSET_BROWSER_SCROLLBAR_ID, Some(ids::ASSET_PANEL));
        paint_scrollbar(body, scroll, content_h, body_h, visual, ctx.scene, theme);
        ctx.host
            .hit_index_mut()
            .register(ASSET_BROWSER_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::ASSET_PANEL, content_h);
    store.set_panel_visible_h(ids::ASSET_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::ASSET_PANEL) > max_scroll {
        store.set_panel_scroll(ids::ASSET_PANEL, max_scroll);
    }
}

/// **Um cartão.** Quadrado de cor + nome + detalhe.
///
/// ⚠️ **A cor é informação, não decoração:** ela é a redução da imagem a um pixel (A2), e é o que
/// permite reconhecer um asset antes de a miniatura verdadeira existir (A6).
#[allow(clippy::too_many_arguments)]
fn paint_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    cell: f32,
    name: &str,
    detail: &str,
    swatch: [u8; 4],
    thumb_img: Option<&ph2d_asset_index::Thumb>,
    key: ph2d_asset_index::AssetRef,
) {
    let hot = ctx.host.store().button_visual(id).0 != ph2d_editor_core::widget::ButtonState::Normal;
    let thumb = Rect::new(rect.x, rect.y, cell, cell);
    fill_rounded_rect(
        ctx.scene,
        thumb,
        Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let inset = Spacing::Xs.px();
    fill_rounded_rect(
        ctx.scene,
        Rect::new(
            thumb.x + inset,
            thumb.y + inset,
            (thumb.w - inset * 2.0).max(0.0),
            (thumb.h - inset * 2.0).max(0.0),
        ),
        Radius::Sm.px(),
        ph2d_vector::Color::from_rgba8(swatch[0], swatch[1], swatch[2], swatch[3]), // LITERAL-COLOR-OK: ponte — a cor É o dado do asset
    );
    // ⭐⭐ **A miniatura por CIMA da cor** (wave A6) — e a cor fica por baixo de propósito: uma
    // imagem com alfa mostra o fundo dela, que é a cor dominante do próprio asset, e não um
    // xadrez nem um cinzento de chrome.
    if let Some(t) = thumb_img {
        paint_thumb(ctx, key, t, thumb, inset);
    }
    ph2d_editor_core::paint::stroke_rounded_rect(
        ctx.scene,
        thumb,
        Radius::Sm.px(),
        StrokeToken::Default.px(),
        resolve(
            if hot {
                ColorToken::Accent
            } else {
                ColorToken::Border
            },
            theme,
        ),
    );
    // ⚠️ **As duas linhas do rótulo saem de TOKENS, não de múltiplos do tamanho da fonte.** A 1.ª
    // versão usava `font * 1.2` para o avanço e `font * 0.85` para o detalhe, e o gate dos números
    // mágicos apanhou-a: um múltiplo escolhido à mão não segue a escala tipográfica quando ela
    // mudar.
    let name_y = thumb.y + thumb.h + Spacing::Xs.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        name,
        rect.x,
        name_y,
        TypeToken::Sm.px(),
        rect.w,
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        ctx.text_system,
        ctx.scene,
        detail,
        rect.x,
        name_y + TypeToken::Sm.px() + Spacing::Xxs.px(),
        TypeToken::Xs.px(),
        rect.w,
        resolve(ColorToken::Text2, theme),
    );
    // ⚠️ **O rectângulo de gesto é o CARTÃO INTEIRO**, não só a miniatura — o nome faz parte do
    // alvo, que é o que todo navegador de ficheiros faz.
    ctx.host.hit_index_mut().register(id, rect);
}

/// A entrada que a célula `index` desenhou — a porta que o `apply_event` usa.
pub(crate) fn cell_target(index: usize) -> Option<ph2d_asset_index::AssetRef> {
    painted_at(index)
}

/// Só para os gates: a lista de nomes que a grade desenharia para esta consulta.
#[must_use]
pub fn probe_query(index: &ph2d_asset_index::AssetIndex, q: &Query) -> Vec<String> {
    index
        .query(q)
        .iter()
        .map(|e: &&AssetEntry| e.name.clone())
        .collect()
}

// ── ⭐⭐ O MEMO DE TEXTURA DAS MINIATURAS (wave A6) ──────────────────────────────────────────────

thread_local! {
    /// `AssetRef → (os bytes que a construíram, a textura estável)`.
    ///
    /// ⛔⛔ **Sem isto o navegador reenviaria CADA cartão ao atlas do `vello`, TODO o quadro.** O
    /// `draw_image_rgba` faz `Blob::new(rgba.clone())` em cada chamada e o `vello` indexa o cache
    /// por `data.id()` — com o tecto de 512 células a 96² isso é ~18 MB de upload + repack por
    /// quadro. O `StableImage` guarda o id, e é o que faz o cache dele acertar.
    ///
    /// ⚠️ **A chave da revalidação é a IDENTIDADE do `Arc`**, não os bytes: o `Thumb` compara em
    /// `O(1)` por `ptr_eq`, e quem produz uma miniatura nova produz um `Arc` novo (a junção
    /// garante-o guardando o `Arc` na memória por conteúdo). ⛔ Mutar um `Arc` em sítio manteria o
    /// id da `Blob` e o atlas serviria os pixels velhos, **sem erro nenhum**.
    static THUMB_TEX: std::cell::RefCell<
        std::collections::BTreeMap<ph2d_asset_index::AssetRef, (ph2d_asset_index::Thumb, ph2d_vector::StableImage)>,
    > = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// Desenha a miniatura dentro do quadrado do cartão, **aspecto preservado e centrada**.
///
/// ⚠️ **Nunca esticada:** uma tira 8:1 esticada num quadrado lê-se como outra forma, e a miniatura
/// existe precisamente para se reconhecer a forma.
fn paint_thumb(
    ctx: &mut PaintCtx,
    key: ph2d_asset_index::AssetRef,
    thumb: &ph2d_asset_index::Thumb,
    square: Rect,
    inset: f32,
) {
    let img = THUMB_TEX.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((cached, img)) = c.get(&key)
            && cached == thumb
        {
            return Some(img.clone());
        }
        let img = ph2d_vector::StableImage::from_rgba(thumb.rgba.clone(), thumb.w, thumb.h)?;
        c.insert(key, (thumb.clone(), img.clone()));
        Some(img)
    });
    let Some(img) = img else { return };

    let (bw, bh) = (
        (square.w - inset * 2.0).max(0.0),
        (square.h - inset * 2.0).max(0.0),
    );
    let (tw, th) = (thumb.w.max(1) as f32, thumb.h.max(1) as f32);
    let s = (bw / tw).min(bh / th).max(0.0);
    let (dw, dh) = (tw * s, th * s);
    let x0 = f64::from(square.x + inset + (bw - dw) * 0.5);
    let y0 = f64::from(square.y + inset + (bh - dh) * 0.5);
    ctx.scene.draw_stable_image(
        &img,
        (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
        // Bilinear: uma miniatura é um render encolhido, e suavizar entre texels lê-se melhor que
        // o `Nearest` que uma MEDIÇÃO (um espectrograma) quereria.
        ph2d_vector::ImageQuality::Medium,
    );
}
