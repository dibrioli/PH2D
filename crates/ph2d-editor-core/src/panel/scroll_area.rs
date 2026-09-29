//! **A porta da rolagem com um [`PaintCtx`]** — a forma de todo `Panel::paint` (spec
//! `docs/UI_New_and_Simple/spec/04_a_rolagem_unica.md`).
//!
//! A lei vive em [`crate::widget::scroll_area`] (o que a porta garante está no cabeçalho de lá);
//! aqui mora só a costura com o `PaintCtx`. ⚠️ **Mora no `panel` e não no `widget` de propósito:**
//! o `widget` não pode conhecer o `panel` — `panel → action_bus → interaction → widget` já existe,
//! e `widget → panel` fechava um ciclo que o gate `the_foundation_modules_form_a_dag` reprova.
//!
//! ```text
//! let area = panel::scroll_area::open(ctx, PANEL, BAR, body);
//! … o corpo pinta a partir de  area.top()  …
//! panel::scroll_area::close(area, ctx, content_h);
//! ```

use super::PaintCtx;
use crate::paint::rect_to_vello;
use crate::widget::scroll_area::{ScrollArea, publish};
use crate::widget::{
    paint_scrollbar, scrollbar_is_needed as is_needed, scrollbar_track_rect as track_rect,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;

/// Abre o corpo de um painel: recorta o desenho **e** o clique pela mesma banda.
pub fn open(ctx: &mut PaintCtx, panel: NodeId, bar: NodeId, body: Rect) -> ScrollArea {
    let scroll = ctx.host.store().panel_scroll(panel).max(0.0);
    ctx.scene.push_clip(&rect_to_vello(body));
    ctx.host.hit_index_mut().push_clip(body);
    ScrollArea::opened(panel, bar, body, scroll)
}

/// Fecha o corpo de um painel: desfaz os recortes, publica as alturas (com o clamp), pinta a barra
/// fora do recorte, regista a TRILHA e publica o dono dela.
pub fn close(area: ScrollArea, ctx: &mut PaintCtx, content_h: f32) {
    let theme = ctx.host.theme();
    ctx.scene.pop_layer();
    ctx.host.hit_index_mut().pop_clip();
    let body = area.body();
    let visible_h = body.h;
    publish(ctx.host.store_mut(), area.panel(), content_h, visible_h);
    if is_needed(content_h, visible_h) {
        let track = track_rect(body);
        let visual = ctx.host.store().scrollbar_visual(area.bar());
        let _ = paint_scrollbar(
            body,
            area.scroll(),
            content_h,
            visible_h,
            visual,
            ctx.scene,
            theme,
        );
        ctx.host.hit_index_mut().register(area.bar(), track);
        ctx.host
            .store_mut()
            .publish_scroll_bar(area.bar(), area.panel(), track);
    }
}

/// Fecha o corpo da GALERIA de widgets — a porta, e depois o cromo dela por cima (os pontos dos
/// cantos, a alça de arrasto, os dois redimensionamentos e o fechar).
pub fn close_showcase_body(body: crate::widget::showcase::ShowcaseBody, ctx: &mut PaintCtx) {
    let theme = ctx.host.theme();
    let (area, content_h, rect) = body.into_parts();
    close(area, ctx, content_h);
    crate::widget::showcase::finish_chrome(rect, ctx.scene, ctx.host.hit_index_mut(), theme);
}
