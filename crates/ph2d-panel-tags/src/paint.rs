//! O quadro do painel TAGS — moldura, título, rolagem e o corpo.
//!
//! ⚠️ **A ordem é a dos painéis irmãos**: porta de visibilidade (+ limpeza do rect ao esconder) ·
//! rect do encaixe · publicar o rect · moldura canónica · corpo dentro do recorte da rolagem ·
//! barra · publicar as alturas.

use crate::state::{self, TagsPanelState};
use crate::{TagsPanel, rows};
use ph2d_editor_core::ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    SCROLLBAR_W, TAGS_SCROLLBAR_ID, paint_scrollbar, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

pub(crate) fn paint(state: &mut TagsPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(TagsPanel::ID) {
        // Limpeza simétrica do rect: sem ela o `panel_at` continuaria a devolver este painel no
        // sítio onde ele esteve, e a roda comeria o gesto de outro.
        ctx.host.store_mut().clear_panel_rect(ids::TAGS_PANEL);
        return;
    }
    // ⭐⭐ **A tag acabada de nascer abre o campo de renomear por cima dela** — o fluxo do Blender:
    // o gesto é um clique e o nome escreve-se por cima do de omissão.
    //
    // ⚠️ **Aqui e não no `apply_event`:** quem cria é a SHELL (ela é que tem a árvore), então o id
    // da tag nova só existe no quadro a seguir ao clique. Um `renaming` escrito no evento apontaria
    // para uma tag que ainda não tem id.
    if let Some(nova) = state::take_born_tag() {
        state.renaming = Some(nova);
        state.focus = Some(nova);
        crate::seam::open_rename(ctx.host.store_mut(), "");
    }
    let rect: Rect = ctx.slot;
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(ids::TAGS_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);

    let title_size = paint_panel_title(
        rect,
        ph2d_i18n::tr("panel.tags.title"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        crate::ids::TAGS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    // ⭐⭐⭐ **As linhas registam-se AQUI, antes de serem pintadas** — quem pinta uma linha é quem
    // a torna clicável. Ver o [`crate::seam::register_rows`] para a razão de isto NÃO vir da
    // shell, que é onde a Hierarquia o tem.
    let ids_das_linhas: Vec<u64> = state::with_current(|i| i.rows.iter().map(|r| r.id).collect());
    crate::seam::register_rows(ctx.host.store_mut(), &ids_das_linhas);

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);

    let (focus, renaming) = (state.focus, state.renaming);
    let content_h = state::with_current(|info| {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let scroll_y = store.panel_scroll(ids::TAGS_PANEL).max(0.0);
        let clip = rect_to_vello(Rect::new(rect.x, body_top, rect.w, body_h));
        scene.push_clip(&clip);
        let top = body_top - scroll_y;

        let em_maos = focus.and_then(|f| info.rows.iter().find(|r| r.id == f));
        let verbos = rows::verbs(em_maos);
        let mut y = rows::verb_bar(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            inner_x,
            inner_w,
            top,
            &verbos,
        );
        if info.rows.is_empty() {
            y = rows::empty_line(scene, text_system, theme, inner_x, inner_w, y);
        }
        for row in &info.rows {
            y = rows::tag_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                inner_x,
                inner_w,
                y,
                row,
                focus == Some(row.id),
                renaming == Some(row.id),
            );
            // ⚠️ A recusa é pintada DEBAIXO da linha dela, e por isso a árvore abaixo desce um
            // degrau enquanto ela está lá — *uma queixa que empurra o conteúdo é uma queixa que se
            // vê*, e a alternativa (pintá-la por cima) taparia a linha seguinte.
            if let Some((tag, texto)) = &info.problem
                && *tag == row.id
            {
                y = rows::problem_line(
                    scene,
                    text_system,
                    theme,
                    inner_x,
                    inner_w,
                    y,
                    row.depth,
                    texto,
                );
            }
        }
        // A recusa sem linha (criar uma raiz com nome vazio) vai para o fim, onde o gesto acabou.
        if let Some((tag, texto)) = &info.problem
            && *tag == 0
        {
            y = rows::problem_line(scene, text_system, theme, inner_x, inner_w, y, 0, texto);
        }

        let content_h = (y - top + PANEL_HEAD_PAD).max(0.0);
        if scrollbar_is_needed(content_h, body_h) {
            let body = Rect::new(rect.x, body_top, rect.w, body_h);
            let thumb =
                scrollbar_thumb_rect(scrollbar_track_rect(body), scroll_y, content_h, body_h);
            paint_scrollbar(
                body,
                scroll_y,
                content_h,
                body_h,
                store.scrollbar_visual(TAGS_SCROLLBAR_ID),
                scene,
                theme,
            );
            hit_index.register(TAGS_SCROLLBAR_ID, thumb);
        }
        scene.pop_layer();
        content_h
    });

    state::set_last_content_h(content_h);
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::TAGS_PANEL, content_h);
    store.set_panel_visible_h(ids::TAGS_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::TAGS_PANEL) > max_scroll {
        store.set_panel_scroll(ids::TAGS_PANEL, max_scroll);
    }
}
