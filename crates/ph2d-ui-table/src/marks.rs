//! As marcas que se põem por cima de uma tabela já pintada: a moldura da célula sob o rato e a da
//! premida (o acabamento comum dos quadros do plano 14), e a geometria do quadrado de cor.

use crate::{Layout, Row};
use ph2d_editor_core::paint::{frame_radius, resolve, stroke_rounded_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O lado do quadrado de cor de uma célula (`Cell.swatch`): o corpo da letra da célula.
#[must_use]
pub fn swatch_side() -> f32 {
    TypeToken::Sm.px()
}

impl Layout {
    /// A célula `(chave da linha, coluna)` recuada `Spacing::Xxs`: onde a moldura e a banda da
    /// célula mudada assentam.
    #[must_use]
    pub fn cell_frame(&self, rows: &[Row], row: &str, col: &str) -> Option<Rect> {
        let i = rows.iter().position(|r| r.key == row)?;
        let r = self.cell_rect(i, col)?;
        Some(inset(r, Spacing::Xxs.px()))
    }

    /// A coluna `col` no corpo inteiro (a banda da coluna acesa).
    #[must_use]
    pub fn col_band(&self, col: &str) -> Option<Rect> {
        let c = self.col(col)?;
        Some(Rect::new(c.x, self.body.y, c.w, self.body.h))
    }
}

pub(crate) fn inset(r: Rect, d: f32) -> Rect {
    Rect::new(
        r.x + d,
        r.y + d,
        (r.w - d * 2.0).max(0.0),
        (r.h - d * 2.0).max(0.0),
    )
}

/// ⭐ A moldura da célula debaixo do rato (`hover`: `BorderStrong`, fina) e da premida (`pressed`:
/// `Accent`, grossa — vence o hover), `(chave da linha, coluna)`, por cima do texto sem o tapar.
pub fn paint_cell_frames(
    lay: &Layout,
    rows: &[Row],
    hover: Option<(&str, &str)>,
    pressed: Option<(&str, &str)>,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let radius = frame_radius(theme, Radius::Sm.px());
    let at = |(row, col): (&str, &str)| lay.cell_frame(rows, row, col);
    if let Some(r) = pressed.and_then(at) {
        let ink = resolve(ColorToken::Accent, theme);
        stroke_rounded_rect(scene, r, radius, StrokeToken::Thick.px(), ink);
    } else if let Some(r) = hover.and_then(at) {
        let ink = resolve(ColorToken::BorderStrong, theme);
        stroke_rounded_rect(scene, r, radius, StrokeToken::Thin.px(), ink);
    }
}
