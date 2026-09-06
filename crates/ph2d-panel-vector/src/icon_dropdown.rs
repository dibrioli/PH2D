//! **O PICKER de ícone** — a lista de glifos de um `IconButton` autorado (plano UI/UX W8b, §6.2).
//!
//! Irmão do [`crate::font_dropdown`], e a razão de o ser é a mesma que faz aquele existir: **cada
//! linha desenha a própria coisa**. Uma família de fonte escolhe-se vendo o desenho dela, e um
//! ícone escolhe-se olhando — uma lista de nomes (`chevron-down`, `bolt`, `zen`) obrigaria o
//! artista a traduzir palavras em figuras de cabeça, que é precisamente o trabalho que um picker
//! existe para poupar.
//!
//! ⚠️ **A primeira linha é `Drawing`, e ela não é um item a mais:** ela é como se TIRA a escolha.
//! O default de um `IconButton` autorado é desenhar a forma que o veste, então *voltar ao desenho*
//! tem de ser alcançável pela mesma porta por onde se escolhe — um segundo botão *"Clear icon"* ao
//! lado seria um verbo que só existe depois de o primeiro ter sido usado, e a lista já tem o
//! lugar natural para ele.
//!
//! # O que vem de graça, e por isso não está aqui
//!
//! Abrir/fechar, o *light-dismiss*, a roda e o arrasto do scrollbar são do `Dropdown` genérico
//! (`InteractiveState::Dropdown`); o popover é pintado num **passe diferido**, depois de todas as
//! seções, senão o `push_clip` do scroll do painel o cortaria na borda da seção.

use crate::ids;
use crate::state;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ButtonState, DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_scrollbar, resolve_opaque,
    scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};

/// Lado do quadrado do glifo, como fração da altura da linha — deixa folga para o rótulo ao lado.
const GLYPH_RATIO: f32 = 0.72; // LITERAL-PX-OK: lado do glifo ÷ altura da linha (métrica de lista)

/// **O rótulo da linha `i`** — `Drawing` na primeira, o slug do glifo nas outras.
///
/// ⚠️ O slug e não um nome traduzido: ele É o que o documento guarda, e ver no picker exactamente
/// a palavra que o arquivo carrega é o que torna um projeto legível quando alguém o abre num
/// editor de texto. (E a chave i18n de 136 glifos seria uma tabela que ninguém manteria.)
fn row_label(i: usize) -> &'static str {
    match i.checked_sub(1) {
        None => DRAWING,
        Some(n) => IconId::all()[n].slug(),
    }
}

/// **A palavra para *nenhum glifo escolhido***, e ela tem um dono só.
///
/// ⚠️ Ela aparece em DOIS lugares — a primeira linha da lista e o chip fechado — e escrevê-la duas
/// vezes é como o dia em que um deles fosse traduzido e o outro não: o artista escolheria
/// *"Drawing"* na lista e o chip passaria a dizer outra coisa.
pub(crate) const DRAWING: &str = "Drawing";

/// **O que o chip FECHADO mostra** — `None` quando este tipo não tem face de ícone (sem row).
pub(crate) fn chip_label(s: &state::WidgetSkinState) -> Option<String> {
    s.icon
        .as_ref()
        .map(|c| c.clone().unwrap_or_else(|| DRAWING.to_string()))
}

/// Quantas linhas a lista tem: *Drawing* + o catálogo inteiro.
fn row_count() -> usize {
    IconId::all().len() + 1
}

/// A linha vigente: `0` quando não há escolha (o desenho manda).
fn selected_row() -> usize {
    state::widget_skin_state()
        .and_then(|s| s.icon.flatten())
        .and_then(|slug| IconId::from_slug(&slug))
        .and_then(|id| IconId::all().iter().position(|&x| x == id))
        .map_or(0, |n| n + 1)
}

/// Pinta o popover aberto para `chip_rect`. Só é chamado com o chip aberto.
pub(crate) fn paint(ctx: &mut PaintCtx, chip_rect: Rect, theme: Theme) {
    let id = ids::VECTOR_WIDGET_ICON_DD;
    let count = row_count();
    let sel = selected_row();
    let options: Vec<DropdownOption<usize>> = (0..count)
        .map(|i| DropdownOption::new(ids::vector_widget_icon_option_id(i), i, String::new()))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip_rect, ctx.layout.popover_region());
    let content_h = dd.content_height(chip_rect.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent

    let radius = Radius::Md.px();
    fill_rounded_rect(
        ctx.scene,
        panel,
        radius,
        resolve_opaque(ColorToken::BgElev, theme),
    );
    // ⭐ Pela porta do TEMA: o popover é plano num tema moderno.
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        panel,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Default.px(),
        resolve_opaque(ColorToken::Border, theme),
    );

    let clip = ph2d_vector::Rect::new(
        panel.x as f64,
        panel.y as f64,
        (panel.x + panel.w) as f64,
        (panel.y + panel.h) as f64,
    );
    ctx.scene.push_clip(&clip);
    paint_rows(ctx, &dd, chip_rect, panel, scroll, sel, theme);
    ctx.scene.pop_layer();

    if scrollbar_is_needed(content_h, visible_h) {
        paint_scrollbar(
            panel,
            scroll,
            content_h,
            visible_h,
            ctx.host.store().scrollbar_visual(DROPDOWN_SCROLLBAR_ID),
            ctx.scene,
            theme,
        );
    }

    {
        let store = ctx.host.store_mut();
        for i in 0..count {
            store.register_if_absent(
                ids::vector_widget_icon_option_id(i),
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    // ⚠️ Hit-registar só a parte VISÍVEL da linha: uma linha rolada para fora do popover ainda
    // tem retângulo, e registá-lo inteiro faria um clique no corpo do painel cair nela.
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..count {
        let r = dd.option_rect_in_scrolled(chip_rect, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::vector_widget_icon_option_id(i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}

/// Cada linha visível: o realce do vigente, o GLIFO, e o slug ao lado.
fn paint_rows(
    ctx: &mut PaintCtx,
    dd: &Dropdown<usize>,
    chip_rect: Rect,
    panel: Rect,
    scroll: f32,
    sel: usize,
    theme: Theme,
) {
    let text1 = resolve(ColorToken::Text1, theme);
    let accent_fg = resolve(ColorToken::AccentFg, theme);
    let accent = resolve_opaque(ColorToken::Accent, theme);
    let pad = Spacing::Md.px();
    for i in 0..row_count() {
        let r = dd.option_rect_in_scrolled(chip_rect, panel, i, scroll);
        if r.y + r.h <= panel.y || r.y >= panel.y + panel.h {
            continue; // fora do popover — o clip trata os parciais
        }
        let is_sel = i == sel;
        if is_sel {
            fill_rounded_rect(
                ctx.scene,
                r,
                ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px()),
                accent,
            );
        }
        let color = if is_sel { accent_fg } else { text1 };
        let d = r.h * GLYPH_RATIO;
        let mut text_x = r.x + pad;
        if let Some(n) = i.checked_sub(1) {
            let g = Rect::new(r.x + pad, r.y + (r.h - d) * 0.5, d, d);
            paint_icon(
                ctx.scene,
                IconId::all()[n],
                g,
                color,
                StrokeToken::Default.px(),
            );
            text_x = g.x + g.w + pad;
        }
        paint_text(
            ctx.text_system,
            ctx.scene,
            row_label(i),
            text_x,
            r.y + (r.h - TypeToken::Base.px()) * 0.5,
            TypeToken::Base.px(),
            (r.x + r.w - pad - text_x).max(1.0),
            color,
        );
    }
}

#[cfg(test)]
#[path = "icon_dropdown_tests.rs"]
mod tests;
