//! As cores de UM quadro, resolvidas uma vez pelo tema: cada papel tem o seu token, e o texto
//! pergunta à porta única [`Ink::token`].

use crate::style::Ink;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::widget::section_cards::CardDepth;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Color, VectorScene};

pub(crate) struct Palette {
    inks: [Color; Ink::ALL.len()],
    pub bg: Color,
    pub gutter: Color,
    pub border: Color,
    pub current_line: Color,
    pub selection: Color,
    pub match_: Color,
    pub match_current: Color,
    pub match_edge: Color,
    pub bracket: Color,
    pub caret: Color,
    pub caret_other: Color,
    pub number: Color,
    pub number_current: Color,
    pub error: Color,
    pub warn: Color,
    pub info: Color,
    pub fold: Color,
    pub pill: Color,
    pub pill_text: Color,
    pub status_bg: Color,
    pub status_text: Color,
    pub success: Color,
    /// O tema do quadro — as quinas perguntam-lhe ([`ph2d_editor_core::paint::frame_radius`]).
    pub theme: Theme,
}

impl Palette {
    pub fn new(theme: Theme) -> Self {
        let c = |t: ColorToken| resolve(t, theme);
        Self {
            inks: Ink::ALL.map(|i| c(i.token())),
            bg: c(ColorToken::CodeBg),
            gutter: c(ColorToken::CodeGutter),
            border: c(ColorToken::Border),
            current_line: c(ColorToken::CodeLineCurrent),
            selection: c(ColorToken::Selection),
            match_: c(ColorToken::InfoSoft),
            match_current: c(ColorToken::WarnSoft),
            match_edge: c(ColorToken::Warn),
            bracket: c(ColorToken::BorderStrong),
            caret: c(ColorToken::Text1),
            caret_other: c(ColorToken::Text2),
            number: c(ColorToken::Text3),
            number_current: c(ColorToken::Text2),
            error: c(ColorToken::Danger),
            warn: c(ColorToken::Warn),
            info: c(ColorToken::Info),
            fold: c(ColorToken::Text2),
            pill: c(ColorToken::Bg3),
            pill_text: c(ColorToken::Text2),
            // A barra de estado assenta na raiz do documento, não dentro de um cartão: é ela a
            // superfície de nível de cartão, e o tom vem da porta.
            status_bg: CardDepth::Section.fill(theme),
            status_text: c(ColorToken::Text2),
            success: c(ColorToken::Success),
            theme,
        }
    }

    pub fn ink(&self, ink: Ink) -> Color {
        self.inks[ink as usize]
    }
}

/// O marcador de dobra na calha: ▸ fechada, ▾ aberta, centrado em `(cx, cy)`.
pub(crate) fn fold_marker(scene: &mut VectorScene, cx: f32, cy: f32, folded: bool, color: Color) {
    let h = f64::from(Spacing::Xs.px());
    let (cx, cy) = (f64::from(cx), f64::from(cy));
    let pts = if folded {
        [(cx - h * 0.5, cy - h), (cx + h, cy), (cx - h * 0.5, cy + h)]
    } else {
        [(cx - h, cy - h * 0.5), (cx + h, cy - h * 0.5), (cx, cy + h)]
    };
    let mut p = BezPath::new();
    p.move_to(pts[0]);
    p.line_to(pts[1]);
    p.line_to(pts[2]);
    p.close_path();
    scene.fill_path(&p, &Brush::Solid(color), Affine::IDENTITY);
}
