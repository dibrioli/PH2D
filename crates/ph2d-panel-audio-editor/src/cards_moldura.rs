//! A moldura da janela flutuante da onda, nos cartões: a pílula da lista e o título dela. O que há dentro (régua, onda,
//! seleção, cabeça) é da shell e não muda. A shell pergunta aqui, para não crescer.

use super::*;
use ph2d_editor_core::panel::cards::banda;
use ph2d_editor_core::panel::cards::ferramenta::cards_on;
use ph2d_editor_core::widget::panel_chrome::paint_panel_surface_floating;
use ph2d_text::TextSystem;
use ph2d_tokens::{Radius, TypeToken};
use ph2d_vector::VectorScene;

/// A altura da barra de título da janela na cara de antes.
const ANTES_HEADER_H: f32 = 26.0; // LITERAL-PX-OK: a mesma de `audio_overlay.rs`

/// A moldura e o título da janela. Sem cartões, a de antes.
pub fn pinta(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    rect: Rect,
    titulo: &str,
    theme: Theme,
) {
    if cards_on() {
        banda::pinta_pilula_na_cena(scene, theme, rect);
        let max_w = rect.w - m::LIST_PAD_X * 2.0;
        // A barra de título da janela é a de sempre: o título centra-se nela.
        let topo = Rect {
            y: rect.y + (ANTES_HEADER_H - m::LIST_HEAD_H) * 0.5,
            ..rect
        };
        banda::pinta_titulo_da_lista(scene, text, theme, topo, titulo, max_w);
        return;
    }
    ph2d_editor_core::paint::fill_rounded_rect(
        scene,
        rect,
        Radius::Sm.px(),
        ph2d_editor_core::paint::resolve(ColorToken::BgElev, theme),
    );
    paint_panel_surface_floating(rect, scene, theme);
    let header = Rect::new(rect.x, rect.y, rect.w, ANTES_HEADER_H);
    ph2d_editor_core::paint::paint_text_centered(
        text,
        scene,
        titulo,
        header,
        TypeToken::Xs.px(),
        ph2d_editor_core::paint::resolve(ColorToken::Text2, theme),
    );
}

/// O fundo da régua da onda.
#[must_use]
pub fn fundo_da_regua(theme: Theme) -> Color {
    if cards_on() {
        CardInk::of_list(theme).color(ColorToken::KanbanCard)
    } else {
        ph2d_editor_core::paint::resolve(ColorToken::Bg1, theme)
    }
}
