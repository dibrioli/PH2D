//! ⭐⭐ **A porta de uma lista de dropdown aberta que ROLA** — a pintura, os hits e o que se publica.
//!
//! ⚠️ Mora no `panel`, e não no `widget` ao lado do pintor, porque lê o [`WidgetStore`] e regista no
//! [`HitIndex`]: a aresta `widget → interaction` é uma catraca que só encolhe
//! (`architecture_the_foundation_modules_form_a_dag`), e o `panel` já está acima dos dois.
//!
//! ⛔ Catorze painéis traziam cópias desta lei; o censo `architecture_dropdown_lists_use_the_door`
//! reprova a décima-quinta.

use crate::interaction::{HitIndex, InteractiveState, WidgetStore};
use crate::panel::PaintCtx;
use crate::widget::{
    ButtonState, DROPDOWN_SCROLLBAR_ID, Dropdown, PaintedPopover, PopoverRowPainter,
    paint_dropdown_popover_scrolled, paint_scrollbar, scrollbar_is_needed, scrollbar_track_rect,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// ⭐⭐⭐ **A PORTA de um painel** — a lista aberta de `dd` sob o `chip`, clampada a `region`, com
/// TUDO o que a faz viva: pintura (`rows` pinta o conteúdo de cada linha; `None` = rótulo), hits
/// da faixa visível e da barra, cada linha visível registada como `Button` focável, e a
/// publicação (rect + alturas + rolagem clampada gravada de volta) por onde a roda e o polegar a
/// acham.
pub fn paint_dropdown_list<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
    theme: Theme,
    rows: Option<PopoverRowPainter<'_>>,
) -> PaintedPopover {
    let painted = {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_list(
            dd,
            chip,
            region,
            store,
            ctx.scene,
            ctx.text_system,
            theme,
            hit_index,
            rows,
        )
    };
    let store = ctx.host.store_mut();
    for (id, _) in visible_option_bands(dd, chip, painted.panel, painted.scroll) {
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.publish_dropdown_popover(painted);
    painted
}

/// ⭐⭐ **A lei da FAIXA VISÍVEL** — cada opção ativa cuja linha rolada mostra pelo menos um pixel
/// dentro do `panel`, com o recorte dela. Uma linha rolada para fora apanharia o clique por baixo
/// do popover; as `disabled` e os títulos (`header`) pintam-se e nunca se oferecem — nem ao clique,
/// nem ao foco do teclado (o registo da porta passa por aqui).
pub fn visible_option_bands<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip: Rect,
    panel: Rect,
    scroll: f32,
) -> impl Iterator<Item = (NodeId, Rect)> + '_ {
    dd.options
        .iter()
        .enumerate()
        .filter(|(_, opt)| !opt.disabled && !opt.header)
        .filter_map(move |(i, opt)| {
            let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
            let top = r.y.max(panel.y);
            let bot = (r.y + r.h).min(panel.y + panel.h);
            // LITERAL-PX-OK: um pixel é o piso de uma faixa clicável, não uma medida de desenho
            (bot - top >= 1.0).then(|| (opt.id, Rect::new(r.x, top, r.w, bot - top)))
        })
}

/// ⭐⭐ **O NÚCLEO para quem não tem o store mutável no passe** (o Inspector publica no fim do
/// `paint`): clamp a `region`, rolagem lida do store, pintura, hits da faixa visível e da trilha
/// da barra sob o [`DROPDOWN_SCROLLBAR_ID`]. Quem chama publica o devolvido. Com o store mutável,
/// a porta é o [`paint_dropdown_list`].
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
    paint_list(
        dd,
        chip,
        region,
        store,
        scene,
        text_system,
        theme,
        hit_index,
        None,
    )
}

/// ⭐ **A barra de um popover que ROLA sem ser lista** (o painel do traço da barra do vetor): pinta a
/// trilha e o polegar de `p` e regista a trilha sob o [`DROPDOWN_SCROLLBAR_ID`] — a mesma barra da
/// lista, que o arrasto acha pelo `publish_dropdown_popover` do mesmo `p`. Sem excesso, nada.
pub fn paint_popover_scrollbar(
    p: &PaintedPopover,
    store: &WidgetStore,
    scene: &mut VectorScene,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let visual = store.scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(p.id));
    if paint_scrollbar(
        p.panel,
        p.scroll,
        p.content_h,
        p.visible_h,
        visual,
        scene,
        theme,
    )
    .is_some()
    {
        hit_index.register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(p.panel));
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_list<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
    store: &WidgetStore,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    rows: Option<PopoverRowPainter<'_>>,
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
        rows,
    );
    for (id, band) in visible_option_bands(dd, chip, panel, scroll) {
        hit_index.register(id, band);
    }
    if scrollbar_is_needed(content_h, visible_h) {
        hit_index.register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
    PaintedPopover {
        id: dd.id,
        panel,
        content_h,
        visible_h,
        scroll,
    }
}
