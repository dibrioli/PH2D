//! O corpo do painel — uma linha por token de `ColorToken::ALL`.
//!
//! ⚠️ **A tabela é a lista, e não uma cópia dela.** `paint`, `populate` e `event` percorrem
//! `ColorToken::ALL`; um token novo no design system nasce pintado, registado e vivo, e não há
//! segunda lista para driftar.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, ColorSwatch, SwatchSize, TOKENS_SCROLLBAR_ID, paint_button,
    paint_color_swatch, paint_scrollbar, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::overrides::{color_override, overridden_count};
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::TokensPanel;
use crate::state::{TokensPanelState, set_last_content_h, set_last_visible_h};

/// Largura do botão *Reset* de uma linha. Estreito de propósito: ele é a exceção, não a coluna.
const RESET_W: f32 = 48.0; // LITERAL-PX-OK: panel grid metric (per-row reset button width)

pub(crate) fn paint(_state: &mut TokensPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(TokensPanel::ID) {
        // Limpeza simétrica do rect: sem ela o `panel_at` continua a devolver TOKENS_PANEL depois
        // de fechado, e a roda do rato roda um painel que não está na tela.
        ctx.host.store_mut().clear_panel_rect(ids::TOKENS_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(ids::TOKENS_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(ids::INSP_RESIZE_HANDLE, panel_resize_handle_rect(rect));
        hit_index.register(
            ids::INSP_RESIZE_HANDLE_BL,
            panel_resize_handle_rect_bl(rect),
        );
    }

    let title_size = paint_panel_title(
        rect,
        tr("panel.tokens.title"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::TOKENS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::TOKENS_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let y_after = paint_body(ctx, theme, x, w, body_top - scroll);
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);
}

/// O readout do modo + o *Reset All* + a lista.
fn paint_body(ctx: &mut PaintCtx, theme: Theme, x: f32, w: f32, mut y: f32) -> f32 {
    let n = overridden_count(theme);
    let font = TypeToken::Sm.px();
    // ⚠️ O modo VIGENTE é dito, e não inferido: o mesmo token tem quatro valores, e um painel que
    // não nomeia qual deles está a mostrar torna o `M` (que cicla o tema) indistinguível de um bug.
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!(
            "{}  —  {n} {}",
            theme_label(theme),
            tr("panel.tokens.authored")
        ),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y += ROW_H_PX + Spacing::Xs.px();

    // ⚠️ Só com o que resetar. Um *Reset All* sobre um modo de fábrica é um clique que não faz
    // nada, e o artista não tem como o saber antes de o dar.
    if n > 0 {
        y = command(
            ctx,
            ids::TOKENS_RESET_ALL,
            tr("panel.tokens.reset_all"),
            x,
            w,
            y,
        ) + Spacing::Sm.px();
    }

    for (row, &token) in ColorToken::ALL.iter().enumerate() {
        y = paint_token_row(ctx, theme, row, token, x, w, y);
    }
    y
}

/// `[swatch]  chave-do-token           [Reset]`
fn paint_token_row(
    ctx: &mut PaintCtx,
    theme: Theme,
    row: usize,
    token: ColorToken,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let authored = color_override(theme, token).is_some();
    let swatch_w = SwatchSize::Md.px();
    let font = TypeToken::Sm.px();

    let swatch_rect = Rect::new(x, y, swatch_w, ROW_H_PX);
    // ⚠️ A swatch mostra a cor EFETIVA (a que o app está a usar), nunca a de fábrica sob um token
    // autorado: uma swatch que afirmasse um valor que o desenho não usa é a rachura que a row de
    // Token do vetor já documenta.
    let colour = token.resolve(theme);
    let sw = ColorSwatch::new(
        ids::tokens_swatch_id(row),
        "Design token colour",
        [colour.r, colour.g, colour.b, colour.a],
    )
    .size(SwatchSize::Md);
    paint_color_swatch(&sw, swatch_rect, ctx.scene, theme);
    ctx.host
        .hit_index_mut()
        .register(ids::tokens_swatch_id(row), swatch_rect);

    let label_x = x + swatch_w + Spacing::Sm.px();
    let label_w = (w - swatch_w - Spacing::Sm.px() - if authored { RESET_W } else { 0.0 }).max(1.0);
    // A cor do rótulo DIZ se a linha está autorada — sem isso, "este está diferente da fábrica"
    // só se descobre carregando em Reset e vendo o que muda.
    let label_token = if authored {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        token.key(),
        label_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        label_w,
        resolve(label_token, theme),
    );

    if authored {
        command(
            ctx,
            ids::tokens_reset_id(row),
            tr("panel.tokens.reset"),
            x + w - RESET_W,
            RESET_W,
            y,
        );
    }
    y + ROW_H_PX + Spacing::Xxs.px()
}

/// Um botão de ação simples — o mesmo desenho do irmão no painel de física.
fn command(ctx: &mut PaintCtx, id: ph2d_a11y::NodeId, label: &str, x: f32, w: f32, y: f32) -> f32 {
    let theme = ctx.host.theme();
    let rect = Rect::new(x, y, w, ROW_H_PX);
    let state = ctx
        .host
        .store()
        .button_state(id)
        .unwrap_or(ButtonState::Normal);
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (_, hit_index) = ctx.host.store_and_hit_index_mut();
    paint_button(
        &Button::new(id, label).state(state),
        rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, rect);
    y + ROW_H_PX
}

/// O nome do modo, como o artista o conhece.
const fn theme_label(theme: Theme) -> &'static str {
    match theme {
        Theme::Forge => "Forge",
        Theme::Workshop => "Workshop",
        Theme::Sunstone => "Sunstone",
        Theme::Blueprint => "Blueprint",
    }
}

fn paint_scrollbar_and_publish(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active = matches!(
            ctx.host.store().scrollbar_drag(),
            Some(d) if d.panel == ids::TOKENS_PANEL
        );
        paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, ctx.scene, theme,
        );
        ctx.host
            .hit_index_mut()
            .register(TOKENS_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::TOKENS_PANEL, content_h);
    store.set_panel_visible_h(ids::TOKENS_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::TOKENS_PANEL) > max_scroll {
        store.set_panel_scroll(ids::TOKENS_PANEL, max_scroll);
    }
}
