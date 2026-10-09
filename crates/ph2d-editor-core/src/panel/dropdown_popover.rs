//! ⭐⭐ **A porta de uma lista de dropdown aberta que ROLA** — a pintura, os hits e o que se publica.
//!
//! ⚠️ Mora no `panel`, e não no `widget` ao lado do pintor, porque lê o [`WidgetStore`] e regista no
//! [`HitIndex`]: a aresta `widget → interaction` é uma catraca que só encolhe
//! (`architecture_the_foundation_modules_form_a_dag`), e o `panel` já está acima dos dois.

use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, PaintedPopover, paint_dropdown_popover_scrolled,
    scrollbar_is_needed, scrollbar_track_rect,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// ⭐⭐ **A PORTA de uma lista aberta que ROLA** — clamp a `region`, rolagem lida do store, pintura,
/// e os hits: só a faixa VISÍVEL de cada linha (uma linha rolada para fora apanharia o clique por
/// baixo do popover; as `disabled` nunca) e a trilha da barra sob o [`DROPDOWN_SCROLLBAR_ID`],
/// que o despacho liga ao dropdown publicado. Quem chama publica o devolvido — logo, ou no fim do
/// `paint` quando o passe não tem o store mutável (o Inspector).
#[allow(clippy::too_many_arguments)]
pub fn paint_dropdown_popover_scrolled_with_hits<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
    store: &WidgetStore,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> PaintedPopover {
    let panel = dd.popover_rect_clamped(chip, region);
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    let scroll = store.panel_scroll(dd.id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        dd,
        chip,
        panel,
        scroll,
        store.scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(dd.id)),
        scene,
        text_system,
        theme,
    );
    for (i, opt) in dd.options.iter().enumerate() {
        if opt.disabled {
            continue;
        }
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            // LITERAL-PX-OK: um pixel é o piso de uma faixa clicável, não uma medida de desenho
            hit_index.register(opt.id, Rect::new(r.x, top, r.w, bot - top));
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        hit_index.register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
    PaintedPopover {
        id: dd.id,
        panel,
        content_h,
        visible_h,
    }
}
