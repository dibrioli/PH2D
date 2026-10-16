//! A barra de estado do editor: `linha:coluna` do cursor primário, a contagem de problemas e o selo
//! do último salvar. Todo texto vem do `ph2d-i18n` (chaves `code.status.*`).

use crate::frame::{Badge, CodeFrame, Severity};
use crate::palette::Palette;
use crate::view::CodeView;
use ph2d_editor_core::paint::{label_column_width, paint_text_elided, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, StrokeToken, TypeToken};
use ph2d_vector::VectorScene;

/// O tom do texto de uma peça da barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Success,
    Warning,
    Error,
}

/// Os textos da barra, já traduzidos (a pintura e os testes lêem os MESMOS).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLine {
    pub position: String,
    pub problems: (String, Tone),
    pub badge: Option<(String, Tone)>,
}

impl StatusLine {
    #[must_use]
    pub fn of(frame: &CodeFrame<'_>) -> Self {
        let t = frame.buffer.text();
        let head = frame.buffer.selection().primary().head;
        Self::at(frame, t.display_col(head, frame.tab.max(1)))
    }

    /// Com a coluna (0-based) do cursor já sabida (a pintura guarda-a por revisão e cursor: numa
    /// linha de 1 MB contá-la custa a linha).
    #[must_use]
    pub fn at(frame: &CodeFrame<'_>, col0: usize) -> Self {
        let t = frame.buffer.text();
        let head = frame.buffer.selection().primary().head;
        let line = t.line_of_byte(head) + 1;
        let col = col0 + 1;
        let position = tr_with("code.status.position", &[("line", &line), ("col", &col)]);
        let errors = frame
            .problems
            .iter()
            .filter(|p| p.severity == Severity::Error)
            .count();
        let warnings = frame.problems.len() - errors;
        let problems = match (errors, warnings) {
            (0, 0) => (tr("code.status.no_problems").to_owned(), Tone::Plain),
            (e, w) => {
                let tone = if e > 0 { Tone::Error } else { Tone::Warning };
                let errors = match e {
                    1 => tr("code.status.errors_one").to_owned(),
                    n => tr_with("code.status.errors", &[("count", &n)]),
                };
                let warnings = match w {
                    1 => tr("code.status.warnings_one").to_owned(),
                    n => tr_with("code.status.warnings", &[("count", &n)]),
                };
                (
                    tr_with(
                        "code.status.problems",
                        &[("errors", &errors), ("warnings", &warnings)],
                    ),
                    tone,
                )
            }
        };
        let badge = frame.badge.map(|b| match b {
            Badge::Reloaded { objects: 1 } => {
                (tr("code.status.reloaded_one").to_owned(), Tone::Success)
            }
            Badge::Reloaded { objects } => (
                tr_with("code.status.reloaded", &[("count", objects)]),
                Tone::Success,
            ),
            Badge::Refused { line: Some(l) } => (
                tr_with("code.status.refused_line", &[("line", l)]),
                Tone::Error,
            ),
            Badge::Refused { line: None } => (tr("code.status.refused").to_owned(), Tone::Error),
            Badge::Saved => (tr("code.status.saved_unused").to_owned(), Tone::Plain),
        });
        Self {
            position,
            problems,
            badge,
        }
    }
}

fn color(pal: &Palette, tone: Tone) -> ph2d_vector::Color {
    match tone {
        Tone::Plain => pal.status_text,
        Tone::Success => pal.success,
        Tone::Warning => pal.warn,
        Tone::Error => pal.error,
    }
}

pub(crate) fn paint(
    view: &CodeView,
    frame: &CodeFrame<'_>,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    pal: &Palette,
) {
    let a = view.area;
    let h = view.m.status_h.min(a.h);
    if h <= 0.0 {
        return;
    }
    let bar = Rect {
        x: a.x,
        y: a.y + a.h - h,
        w: a.w,
        h,
    };
    scene.fill_rect(rect_to_vello(bar), pal.status_bg);
    scene.fill_rect(
        rect_to_vello(Rect {
            h: StrokeToken::Thin.px(),
            ..bar
        }),
        pal.border,
    );
    let s = StatusLine::at(frame, view.status_col.map_or(0, |c| c.1));
    let font = ph2d_text::displayed_font_px(TypeToken::Sm.px());
    let y = bar.y + (h - font) * 0.5;
    let pad = Spacing::Md.px();
    let gap = Spacing::Xl.px();
    let right = bar.x + bar.w - pad;
    let mut x = bar.x + pad;
    for (text, tone) in [
        (s.position.as_str(), Tone::Plain),
        (s.problems.0.as_str(), s.problems.1),
    ] {
        let w = label_column_width(ts, font, [text]);
        paint_text_elided(
            ts,
            scene,
            text,
            x,
            y,
            font,
            (right - x).max(0.0),
            color(pal, tone),
        );
        x += w + gap;
    }
    if let Some((text, tone)) = &s.badge {
        let w = label_column_width(ts, font, [text.as_str()]);
        let bx = (right - w).max(x);
        paint_text_elided(
            ts,
            scene,
            text,
            bx,
            y,
            font,
            (right - bx).max(0.0),
            color(pal, *tone),
        );
    }
}
