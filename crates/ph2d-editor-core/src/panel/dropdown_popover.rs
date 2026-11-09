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
    paint_dropdown_popover_scrolled, paint_popover_drop_mark, paint_popover_row_flying,
    paint_scrollbar, scrollbar_is_needed, scrollbar_track_rect,
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
    porta(
        ctx,
        dd,
        chip,
        region,
        theme,
        Conteudo {
            rows,
            row_text_w: None,
            in_hand: None,
        },
    )
}

/// ⭐⭐ **A porta para linhas que NÃO se pintam no rótulo** (o dropdown de fontes do Pixel Art pinta
/// cada nome na própria fonte): `row_text_w` é a largura REAL do conteúdo mais largo, como `rows` o
/// pinta, e entra na MESMA lei da largura ([`crate::widget::popover_width_for`] +
/// [`Dropdown::popover_rect_placed`]: `max(chip, conteúdo + recuos)`, desloca à esquerda se não
/// cabe, nunca sai da `region`) no lugar do rótulo medido na fonte da interface.
pub fn paint_dropdown_list_measured<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
    theme: Theme,
    rows: Option<PopoverRowPainter<'_>>,
    row_text_w: f32,
) -> PaintedPopover {
    porta(
        ctx,
        dd,
        chip,
        region,
        theme,
        Conteudo {
            rows,
            row_text_w: Some(row_text_w),
            in_hand: None,
        },
    )
}

/// A linha de uma lista aberta que a mão leva: a lei do voo do cartão do Kanban
/// ([`crate::paint_flight`]), com a ordem prevista e a marca de acento onde ela cai.
pub struct DropdownRowInHand<'a, T: Clone + PartialEq> {
    /// A lista como vai ficar, com o vão na linha `gap`. Mesmas linhas, mesmas alturas.
    pub preview: &'a Dropdown<T>,
    pub gap: usize,
    /// Há onde cair? Sem isto, o vão fica no sítio dela e não há marca: largar não muda nada.
    pub lands: bool,
    /// A linha que voa, em `dd.options`, e onde ela está: a mão manda, não a ordem prevista.
    pub flying: usize,
    pub at: Rect,
}

/// A lista aberta que se reordena à mão. Os alvos ficam os de `dd`, a ordem de AGORA: o que se pinta
/// mexe-se, e o sítio da queda não pode mexer-se com ele.
pub fn paint_dropdown_list_reordering<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    dd: &Dropdown<T>,
    in_hand: &DropdownRowInHand<'_, T>,
    chip: Rect,
    region: Rect,
    theme: Theme,
) -> PaintedPopover {
    porta(
        ctx,
        dd,
        chip,
        region,
        theme,
        Conteudo {
            rows: None,
            row_text_w: None,
            in_hand: Some(in_hand),
        },
    )
}

/// As faixas visíveis de `dd` onde a lista se vai pintar neste quadro, antes de a pintar: quem
/// reordena à mão precisa delas para prever a ordem.
pub fn dropdown_list_bands<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
) -> Vec<(NodeId, Rect)> {
    let panel = dd.popover_rect_fitted(chip, Some(region), ctx.text_system);
    let scroll = rolagem(ctx.host.store(), dd, chip, panel);
    visible_option_bands(dd, chip, panel, scroll).collect()
}

/// A altura, do topo de `panel`, que as primeiras `rows` linhas de `dd` ocupam — a faixa fixa de quem
/// reserva linhas no topo da lista (o campo da busca do Preset do Painter).
#[must_use]
pub fn dropdown_list_top_rows_h<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    chip: Rect,
    panel: Rect,
    rows: usize,
) -> f32 {
    let Some(ultima) = rows.checked_sub(1) else {
        return 0.0;
    };
    let r = dd.option_rect_in(chip, panel, ultima);
    r.y + r.h - panel.y
}

/// Rola a lista aberta de `dd` (sob `chip`, em `region`) até a linha `index` caber por baixo das
/// `fixed_top_rows` linhas fixas do topo; ao subir, a linha de cima (o título da família) vem com ela.
pub fn dropdown_list_scroll_to_row<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
    index: usize,
    fixed_top_rows: usize,
) {
    let panel = dd.popover_rect_fitted(chip, Some(region), ctx.text_system);
    let r = dd.option_rect_in(chip, panel, index);
    let topo = r.y - panel.y;
    let faixa = dropdown_list_top_rows_h(dd, chip, panel, fixed_top_rows);
    let store = ctx.host.store_mut();
    let mut s = store.panel_scroll(dd.id);
    if topo - faixa < s {
        s = (topo - r.h - faixa).max(0.0);
    } else if topo + r.h > s + panel.h {
        s = topo + r.h - panel.h;
    }
    store.set_panel_scroll(dd.id, s);
}

/// A rolagem gravada, clampada ao que `dd` transborda em `panel`.
fn rolagem<T: Clone + PartialEq>(
    store: &WidgetStore,
    dd: &Dropdown<T>,
    chip: Rect,
    panel: Rect,
) -> f32 {
    let max_scroll = (dd.content_height(chip.h) - panel.h).max(0.0);
    store.panel_scroll(dd.id).clamp(0.0, max_scroll) // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
}

/// O que distingue as portas da lista: quem pinta as linhas, a largura medida e a linha na mão.
struct Conteudo<'r, 'm, T: Clone + PartialEq> {
    rows: Option<PopoverRowPainter<'r>>,
    row_text_w: Option<f32>,
    in_hand: Option<&'m DropdownRowInHand<'m, T>>,
}

fn porta<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    dd: &Dropdown<T>,
    chip: Rect,
    region: Rect,
    theme: Theme,
    conteudo: Conteudo<'_, '_, T>,
) -> PaintedPopover {
    let Conteudo {
        rows,
        row_text_w,
        in_hand,
    } = conteudo;
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
            row_text_w,
            in_hand,
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
        None,
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
    let pintou = super::camada_das_listas::por_cima(scene, |scene| {
        paint_scrollbar(
            p.panel,
            p.scroll,
            p.content_h,
            p.visible_h,
            visual,
            scene,
            theme,
        )
    });
    if pintou.is_some() {
        super::camada_das_listas::alvo(
            hit_index,
            DROPDOWN_SCROLLBAR_ID,
            scrollbar_track_rect(p.panel),
        );
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
    row_text_w: Option<f32>,
    in_hand: Option<&DropdownRowInHand<'_, T>>,
) -> PaintedPopover {
    // ⭐ A largura da lista é a da porta ([`Dropdown::popover_rect_fitted`]); os hits abaixo e o
    // clique-fora (o `panel` publicado) leem este MESMO rect.
    let panel = match row_text_w {
        Some(w) => {
            let has_icon = dd.options.iter().any(|o| o.icon.is_some());
            let list_w = chip.w.max(crate::widget::popover_width_for(w, has_icon));
            dd.popover_rect_placed(chip, Some(region), list_w)
        }
        None => dd.popover_rect_fitted(chip, Some(region), text_system),
    };
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let scroll = rolagem(store, dd, chip, panel);
    let cara = in_hand.map_or(dd, |m| m.preview);
    let barra = store.scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(dd.id));
    // O chip veio de um cartão ⇒ a lista tem a cara dele, tema próprio incluído (T25, 05 §4).
    match crate::widget::card_ink::tinta_do_popover(dd.id) {
        Some(ink) => crate::widget::card_ink::com(ink, || {
            paint_dropdown_popover_scrolled(
                cara,
                chip,
                panel,
                scroll,
                barra,
                scene,
                text_system,
                ink.theme,
                rows,
            );
            na_mao(
                dd,
                in_hand,
                (chip, panel, scroll),
                scene,
                text_system,
                ink.theme,
            );
        }),
        None => {
            paint_dropdown_popover_scrolled(
                cara,
                chip,
                panel,
                scroll,
                barra,
                scene,
                text_system,
                theme,
                rows,
            );
            na_mao(
                dd,
                in_hand,
                (chip, panel, scroll),
                scene,
                text_system,
                theme,
            );
        }
    }
    for (id, band) in visible_option_bands(dd, chip, panel, scroll) {
        super::camada_das_listas::alvo(hit_index, id, band);
    }
    if scrollbar_is_needed(content_h, visible_h) {
        super::camada_das_listas::alvo(
            hit_index,
            DROPDOWN_SCROLLBAR_ID,
            scrollbar_track_rect(panel),
        );
    }
    PaintedPopover {
        id: dd.id,
        panel,
        content_h,
        visible_h,
        scroll,
    }
}

/// O que a mão acrescenta à lista pintada: a marca da queda e, por cima dela, a linha que voa.
fn na_mao<T: Clone + PartialEq>(
    dd: &Dropdown<T>,
    in_hand: Option<&DropdownRowInHand<'_, T>>,
    (chip, panel, scroll): (Rect, Rect, f32),
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let Some(m) = in_hand else {
        return;
    };
    if m.lands {
        let vao = dd.option_rect_in_scrolled(chip, panel, m.gap, scroll);
        paint_popover_drop_mark(scene, theme, vao);
    }
    if let Some(opt) = dd.options.get(m.flying) {
        paint_popover_row_flying(scene, text_system, theme, opt, m.at);
    }
}
