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
use crate::widget::scroll_area::ScrollArea;
use crate::zones::Rect;
use ph2d_a11y::NodeId;

/// Abre o corpo de um painel: recorta o desenho **e** o clique pela mesma banda.
pub fn open(ctx: &mut PaintCtx, panel: NodeId, bar: NodeId, body: Rect) -> ScrollArea {
    let scroll = ctx.host.store().panel_scroll(panel).max(0.0);
    ctx.scene.push_clip(&rect_to_vello(body));
    ctx.host.hit_index_mut().push_clip(body);
    ScrollArea::opened(panel, bar, body, scroll)
}

/// Fecha o corpo de um painel: desfaz os recortes, pinta a barra fora do recorte, regista a TRILHA,
/// e publica as alturas (com a margem do fim e o clamp) e o dono da barra.
///
/// ⭐ **Delega no fecho das partes soltas** ([`crate::widget::scroll_area::close_parts`]): as duas
/// formas repetiam a mesma lei, e a margem do fim (2026-09-29) teve de ser escrita nas DUAS — *uma
/// lei escrita em dois sítios ainda não é uma lei; só uma porta é*. Hoje este fecho é a costura com
/// o `PaintCtx` e mais nada.
pub fn close(area: ScrollArea, ctx: &mut PaintCtx, content_h: f32) {
    let theme = ctx.host.theme();
    let content_h = notas_do_fim(&area, ctx, content_h, theme);
    let pending = {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        crate::widget::scroll_area::close_parts(area, ctx.scene, hit_index, store, content_h, theme)
    };
    pending.publish(ctx.host.store_mut());
}

/// ⭐⭐ **As notas do fim do corpo** (2026-10-01, ordem do dono: *«a possibilidade de criar notas
/// deve existir em quaisquer painéis de qualquer tipo»*) — pintadas DENTRO do recorte, a seguir ao
/// conteúdo, e somadas à altura que a porta publica (senão uma nota comprida ficava fora da
/// rolagem). Ver [`crate::widget::showcase::notes_chrome`]. Devolve a altura com elas.
fn notas_do_fim(
    area: &ScrollArea,
    ctx: &mut PaintCtx,
    content_h: f32,
    theme: ph2d_tokens::Theme,
) -> f32 {
    let body = area.body();
    let gap = ph2d_tokens::Spacing::Md.px();
    let inner_x = body.x + gap;
    let inner_w = (crate::widget::scrollbar_track_rect(body).x - gap - inner_x).max(0.0);
    let y0 = area.top() + content_h + gap;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let y1 = crate::widget::showcase::notes_chrome::pinta_as_que_sobram(
        ctx.scene,
        ctx.text_system,
        hit_index,
        store,
        area.panel(),
        theme,
        inner_x,
        inner_w,
        y0,
    );
    if y1 > y0 { y1 - area.top() } else { content_h }
}

/// Fecha o corpo da GALERIA de widgets — a porta, e depois o cromo dela por cima (os pontos dos
/// cantos, a alça de arrasto, os dois redimensionamentos e o fechar).
pub fn close_showcase_body(body: crate::widget::showcase::ShowcaseBody, ctx: &mut PaintCtx) {
    let theme = ctx.host.theme();
    let (area, content_h, rect) = body.into_parts();
    close(area, ctx, content_h);
    crate::widget::showcase::finish_chrome(rect, ctx.scene, ctx.host.hit_index_mut(), theme);
}
