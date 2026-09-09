//! O quadro do painel do ESQUELETO — moldura, título, rolagem e o corpo.
//!
//! ⚠️ **A ordem é a dos painéis irmãos**: porta de visibilidade (+ limpeza do rect ao esconder) ·
//! rect do encaixe · publicar o rect (para o despacho o achar) · moldura canónica · corpo dentro do
//! recorte da rolagem · barra · publicar as alturas.
//!
//! ⚠️ **O passe DIFERIDO no fim** pinta a lista de acções POR CIMA de tudo: o corpo rola, e uma
//! lista pintada lá dentro seria cortada na borda.

use crate::state::{self, SkeletonPanelState};
use crate::{SkeletonPanel, section};
use ph2d_editor_core::ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel, RowCtx};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{
    SCROLLBAR_W, VECTOR_SCROLLBAR_ID, paint_scrollbar, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

pub(crate) fn paint(_state: &mut SkeletonPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(SkeletonPanel::ID) {
        // Limpeza simétrica do rect: sem ela o `panel_at` continuaria a devolver este painel no
        // sítio onde ele esteve, e a roda comeria o gesto de outro.
        ctx.host.store_mut().clear_panel_rect(ids::SKELETON_PANEL);
        return;
    }
    let rect: Rect = ctx.slot;
    let theme = ctx.host.theme();
    ctx.host
        .store_mut()
        .set_panel_rect(ids::SKELETON_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);

    let title_size = paint_panel_title(
        rect,
        ph2d_i18n::tr("panel.vector.section.bone"),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::VECTOR_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let inner_x = rect.x + PANEL_HEAD_PAD;
    // A calha da barra fica reservada sempre, para o corpo nunca pintar por baixo dela.
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);

    let content_h = {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let scroll_y = store.panel_scroll(ids::SKELETON_PANEL).max(0.0);
        let clip = rect_to_vello(Rect::new(rect.x, body_top, rect.w, body_h));
        scene.push_clip(&clip);
        let body_top_y = body_top - scroll_y;
        let mut r = RowCtx {
            scene,
            text_system,
            store,
            hit_index,
            theme,
            inner_x,
            inner_w,
            row_h: ROW_H_PX,
            row_gap: Spacing::Xs.px(),
            font: TypeToken::Base.px(),
            open_fold: None,
        };
        let y = section::body(&mut r, body_top_y);
        // ⚠️ **A dobra fecha aqui**: ela é aberta pelo cabeçalho e, sem isto, o painel ficaria com a
        // secção meio-aberta para sempre.
        let y = r.close_fold(y);
        let content_h = (y - body_top_y + PANEL_HEAD_PAD).max(0.0);
        if scrollbar_is_needed(content_h, body_h) {
            let body = Rect::new(rect.x, body_top, rect.w, body_h);
            let thumb =
                scrollbar_thumb_rect(scrollbar_track_rect(body), scroll_y, content_h, body_h);
            paint_scrollbar(
                body,
                scroll_y,
                content_h,
                body_h,
                r.store.scrollbar_visual(VECTOR_SCROLLBAR_ID),
                r.scene,
                theme,
            );
            r.hit_index.register(VECTOR_SCROLLBAR_ID, thumb);
        }
        r.scene.pop_layer();
        content_h
    };

    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::SKELETON_PANEL, content_h);
    store.set_panel_visible_h(ids::SKELETON_PANEL, body_h);
    // Apara uma rolagem rançosa quando o conteúdo encolheu (uma secção fechada), para o corpo nunca
    // ficar desenhado acima do topo.
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::SKELETON_PANEL) > max_scroll {
        store.set_panel_scroll(ids::SKELETON_PANEL, max_scroll);
    }
    // ⭐⭐⭐ O passe DIFERIDO: a lista de acções por cima de tudo.
    if let Some(chip_rect) = state::take_pending_bone_action_dd() {
        section::paint_action_popover(ctx, chip_rect, theme);
    }
}
