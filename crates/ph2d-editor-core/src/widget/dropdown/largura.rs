//! ⭐⭐⭐ **A LARGURA da lista aberta — UMA lei** (dono 25/10: *«todas as dropdowns devem ter a lista
//! na largura necessária para mostrar os nomes inteiros, mesmo que no painel sejam estreitas»*).
//!
//! A lista toma `max(chip, rótulo mais largo MEDIDO + recuos)`, limitada à região: se não cabe à
//! direita, desloca-se para a esquerda; nunca sai dela; só aí o rótulo elide. O chip não muda.
//! A pintura e as bandas clicáveis leem o MESMO rect ([`Dropdown::popover_rect_fitted`]): as linhas
//! derivam do painel ([`Dropdown::option_rect_in`]), logo pintadas e clicáveis não divergem.

use super::*;
use crate::paint::label_column_width;

/// **A largura de uma lista em que cabe um rótulo de `text_w`** — o inverso do
/// [`option_icon_split`]: o recuo do painel dos dois lados, o glifo, e a folga de
/// [`Spacing::Lg`] de cada lado para o nome centrado não encostar.
#[must_use]
pub fn popover_width_for(text_w: f32, has_icon: bool) -> f32 {
    let glyph = if has_icon {
        POPOVER_PANEL_PAD_X + Spacing::Xs.px() + TypeToken::Base.px()
    } else {
        0.0
    };
    (POPOVER_PANEL_PAD_X * 2.0 + glyph + text_w + Spacing::Lg.px() * 2.0).ceil()
}

/// A largura de lista em que cabe um TÍTULO de `text_w` (o inverso do recuo `Sm` do
/// `paint_popover_header_row`), conferida contra a conta da pintura — um ULP a menos elide.
fn header_width_for(text_w: f32) -> f32 {
    let chrome = POPOVER_PANEL_PAD_X * 2.0 + Spacing::Sm.px() * 2.0;
    let mut w = (chrome + text_w).ceil();
    while w - chrome < text_w {
        w = w.next_up();
    }
    w
}

impl<T: Clone + PartialEq> Dropdown<T> {
    /// ⭐⭐⭐ **A largura que a lista PEDE** — a do chip, ou a do rótulo mais largo medido no peso em
    /// que se pinta (opções em `Base`, títulos em caixa alta `Xs`), com o glifo se alguma opção o
    /// tem.
    #[must_use]
    pub fn list_width(&self, chip_w: f32, text_system: &mut TextSystem) -> f32 {
        let has_icon = self.options.iter().any(|o| o.icon.is_some());
        let opcoes = label_column_width(
            text_system,
            TypeToken::Base.px(),
            self.options
                .iter()
                .filter(|o| !o.header)
                .map(|o| o.label.as_str()),
        );
        let titulos: Vec<String> = self
            .options
            .iter()
            .filter(|o| o.header)
            .map(|o| o.label.to_uppercase())
            .collect();
        let mut w = chip_w.max(popover_width_for(opcoes, has_icon));
        if !titulos.is_empty() {
            let t = label_column_width(
                text_system,
                TypeToken::Xs.px(),
                titulos.iter().map(String::as_str),
            );
            w = w.max(header_width_for(t));
        }
        w
    }

    /// **Onde a lista de largura `list_w` assenta** — a vertical é a do
    /// [`Self::popover_rect_clamped`] (ou a do [`Self::popover_rect`] sem região); na horizontal ela
    /// ancora no chip, desloca-se para a esquerda se passa da borda direita, e nunca é mais larga
    /// que a região. Uma lista que cabe no chip fica exactamente onde estava.
    #[must_use]
    pub fn popover_rect_placed(&self, chip: Rect, viewport: Option<Rect>, list_w: f32) -> Rect {
        let Some(vp) = viewport else {
            let r = self.popover_rect(chip);
            return Rect::new(r.x, r.y, list_w.max(chip.w), r.h);
        };
        let r = self.popover_rect_clamped(chip, vp);
        let w = list_w.max(chip.w).min(vp.w.max(chip.w));
        let right = vp.x + vp.w;
        let x = if w > chip.w && chip.x + w > right {
            (right - w).max(vp.x)
        } else {
            chip.x
        };
        Rect::new(x, r.y, w, r.h)
    }

    /// ⭐⭐⭐ **O rect da lista aberta** — a porta que a pintura e o hit-test leem.
    #[must_use]
    pub fn popover_rect_fitted(
        &self,
        chip: Rect,
        viewport: Option<Rect>,
        text_system: &mut TextSystem,
    ) -> Rect {
        let w = self.list_width(chip.w, text_system);
        self.popover_rect_placed(chip, viewport, w)
    }
}
