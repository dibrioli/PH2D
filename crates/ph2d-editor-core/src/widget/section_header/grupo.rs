//! ⭐ **O cabeçalho de um GRUPO dentro de um cartão** — cada parte de um cartão composto (a câmera e
//! o seu `Camera Follow`, a paralaxe e o seu `Parallax Repeat`…). A lei vem do oráculo Godot
//! corrido (`docs/CodeEditor/12_cartoes_compostos.md`): chevron + rótulo em negrito, mais baixo
//! que uma linha, conteúdo recuado. Sem placa e sem pega — o cartão já é a placa, e um grupo não
//! se reordena.

use super::SectionHeader;
use super::fold::paint_chevron;
use crate::paint::paint_text_title;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A altura do cabeçalho de um grupo — o corpo do rótulo de uma linha mais a margem `Sm`: mais
/// baixo que a linha, como o do Godot (23 px contra a linha).
#[must_use]
pub fn group_header_h() -> f32 {
    TypeToken::Sm.px() + Spacing::Sm.px()
}

/// Pinta o cabeçalho de um grupo em `rect`: o chevron encostado à coluna dos rótulos e o nome em
/// `SemiBold`, `Text1`.
pub fn paint_group_header(
    header: &SectionHeader,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let font = TypeToken::Sm.px();
    let icon_w = font;
    let chev = Rect::new(rect.x, rect.y + (rect.h - icon_w) * 0.5, icon_w, icon_w);
    paint_chevron(
        scene,
        chev,
        header.fold_t(),
        crate::widget::card_ink::tinta(ColorToken::Text2, theme),
    );
    let label_x = rect.x + icon_w + ph2d_tokens::icon_label_gap_px();
    paint_text_title(
        text_system,
        scene,
        &header.label,
        label_x,
        titulo_y(rect, font),
        font,
        (rect.x + rect.w - label_x).max(0.0),
        crate::widget::card_ink::tinta(ColorToken::Text1, theme),
    );
}

/// O topo do título: as maiúsculas no meio da faixa, onde o chevron está centrado (doc 18, U2).
fn titulo_y(rect: Rect, font: f32) -> f32 {
    rect.y + crate::paint::folga_do_texto(rect.h, font)
}

#[cfg(test)]
#[test]
fn as_maiusculas_do_grupo_ficam_no_meio_do_chevron() {
    let (rect, font) = (Rect::new(0.0, 10.0, 200.0, 24.0), TypeToken::Sm.px());
    let meio = titulo_y(rect, font) + ph2d_text::cap_centre_px(font)
        - (ph2d_text::displayed_font_px(font) - font) * 0.5;
    assert!((meio - (rect.y + rect.h * 0.5)).abs() < 0.01, "{meio}");
}
