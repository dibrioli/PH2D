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
use ph2d_editor_core::paint::{paint_icon, paint_text, resolve};
use ph2d_editor_core::panel::{PaintCtx, paint_dropdown_list};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, PopoverRow};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::TextKey;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Lado do quadrado do glifo, como fração da altura da linha — deixa folga para o rótulo ao lado.
const GLYPH_RATIO: f32 = 0.72; // LITERAL-PX-OK: lado do glifo ÷ altura da linha (métrica de lista)

/// **O rótulo da linha `i`** — `Drawing` na primeira, o slug do glifo nas outras.
///
/// ⚠️ O slug e não um nome traduzido: ele É o que o documento guarda, e ver no picker exactamente
/// a palavra que o arquivo carrega é o que torna um projeto legível quando alguém o abre num
/// editor de texto. (E a chave i18n de 136 glifos seria uma tabela que ninguém manteria.)
fn row_label(i: usize) -> &'static str {
    match i.checked_sub(1) {
        None => DRAWING.tr(),
        Some(n) => IconId::all()[n].slug(),
    }
}

/// **A palavra para *nenhum glifo escolhido***, e ela tem um dono só.
///
/// ⚠️ Ela aparece em DOIS lugares — a primeira linha da lista e o chip fechado — e escrevê-la duas
/// vezes é como o dia em que um deles fosse traduzido e o outro não: o artista escolheria
/// *"Drawing"* na lista e o chip passaria a dizer outra coisa.
pub(crate) const DRAWING: TextKey = TextKey::new("panel.vector.icon.drawing");

/// **O que o chip FECHADO mostra** — `None` quando este tipo não tem face de ícone (sem row).
pub(crate) fn chip_label(s: &state::WidgetSkinState) -> Option<String> {
    s.icon
        .as_ref()
        .map(|c| c.clone().unwrap_or_else(|| DRAWING.tr().to_string()))
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

/// Pinta o popover aberto para `chip_rect` pela porta da fundação. Só é chamado com o chip aberto.
pub(crate) fn paint(ctx: &mut PaintCtx, chip_rect: Rect, theme: Theme) {
    let options: Vec<DropdownOption<usize>> = (0..row_count())
        .map(|i| DropdownOption::new(ids::vector_widget_icon_option_id(i), i, String::new()))
        .collect();
    let dd = Dropdown::new(ids::VECTOR_WIDGET_ICON_DD, "", options)
        .selected(selected_row())
        .open(true);
    let region = ctx.layout.popover_region();
    let mut linha = |scene: &mut VectorScene, text: &mut TextSystem, row: PopoverRow| {
        paint_row(scene, text, row, theme);
    };
    paint_dropdown_list(ctx, &dd, chip_rect, region, theme, Some(&mut linha));
}

/// O conteúdo de uma linha visível: o GLIFO e o slug ao lado (o realce é da lista).
fn paint_row(scene: &mut VectorScene, text_system: &mut TextSystem, row: PopoverRow, theme: Theme) {
    let r = row.rect;
    let color = resolve(row.ink(), theme);
    let pad = Spacing::Md.px();
    let d = r.h * GLYPH_RATIO;
    let mut text_x = r.x + pad;
    if let Some(n) = row.index.checked_sub(1) {
        let g = Rect::new(r.x + pad, r.y + (r.h - d) * 0.5, d, d);
        paint_icon(scene, IconId::all()[n], g, color, StrokeToken::Default.px());
        text_x = g.x + g.w + pad;
    }
    paint_text(
        text_system,
        scene,
        row_label(row.index),
        text_x,
        r.y + (r.h - TypeToken::Base.px()) * 0.5,
        TypeToken::Base.px(),
        (r.x + r.w - pad - text_x).max(1.0),
        color,
    );
}

#[cfg(test)]
#[path = "icon_dropdown_tests.rs"]
mod tests;
