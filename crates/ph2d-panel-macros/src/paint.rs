//! O quadro do painel MACROS — a ordem das irmãs: porta de visibilidade (+ limpeza do rect) · rect
//! do encaixe · moldura · semente · corpo dentro do recorte da rolagem · barra.

use crate::state::{self, MacrosPanelState};
use crate::{MacrosPanel, corpo};
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_surface, paint_panel_title,
};
use ph2d_editor_core::widget::{MACROS_SCROLLBAR_ID, SCROLLBAR_W};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

pub(crate) fn paint(st: &mut MacrosPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(MacrosPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(ids::MACROS_PANEL);
        return;
    }
    // A escolhida de outro objecto, ou de uma etapa que já não existe, cai: os verbos e os campos
    // nunca apontam para o que não se vê.
    if st.escolhida.is_some_and(|e| state::etapa(e).is_none()) {
        st.escolhida = None;
        st.renomeando = false;
    }
    let rect: Rect = ctx.slot;
    let theme = ctx.host.theme();
    ctx.host.store_mut().set_panel_rect(ids::MACROS_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    let title_size = paint_panel_title(
        rect,
        MacrosPanel::TITLE.tr(),
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        crate::ids::MACROS_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let linhas: Vec<(usize, usize)> = state::with_current(|i| {
        i.objeto
            .iter()
            .flat_map(|o| o.etapas.iter().map(|e| (e.trilha, e.k)))
            .collect()
    });
    crate::seam::register_rows(ctx.host.store_mut(), &linhas);
    crate::seam::semeia(st, ctx.host);

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let area = ph2d_editor_core::panel::scroll_area::open(
        ctx,
        ids::MACROS_PANEL,
        MACROS_SCROLLBAR_ID,
        Rect::new(rect.x, body_top, rect.w, body_h),
    );
    let top = area.top();
    let content_h = state::with_current(|info| {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let mut c = corpo::Corpo {
            scene: &mut *ctx.scene,
            text_system: &mut *ctx.text_system,
            store,
            hit_index,
            theme,
            x: inner_x,
            w: inner_w,
        };
        let y = c.tudo(info, st, top);
        (y - top + PANEL_HEAD_PAD).max(0.0)
    });
    state::set_last_content_h(content_h);
    ph2d_editor_core::panel::scroll_area::close(area, ctx, content_h);
}
