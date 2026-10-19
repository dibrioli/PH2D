//! A barra de estado do editor: `linha:coluna` do cursor primário, o contador de problemas (ícone e
//! número de erros e de avisos; um clique abre a lista) e o selo do último salvar. Todo texto vem do
//! `ph2d-i18n` (chaves `code.status.*`).

use crate::frame::{Assist, Badge, CodeFrame, Severity};
use crate::palette::Palette;
use crate::view::CodeView;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{label_column_width, paint_icon, paint_text_elided, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{INLINE_ICON_PX, Spacing, StrokeToken, TypeToken};
use ph2d_vector::VectorScene;

/// O tom do texto de uma peça da barra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Success,
    Warning,
    Error,
}

/// O contador de problemas da barra: erros e avisos (informação e dica não contam) e o que o leitor
/// de ecrã diz dele (o molde do VS Code, «Errors: 3, Warnings: 2»).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Counter {
    pub errors: usize,
    pub warnings: usize,
    pub label: String,
}

/// Os ids do leitor de ecrã somados ao do `Document` (ao lado dos da faixa de baixo).
const A11Y_COUNTER: u64 = (1 << 31) | (1 << 28) | (1 << 27);

impl CodeView {
    /// O contador pintado no último quadro ao leitor de ecrã: um `Button` (um clique abre a lista),
    /// filho do `Document` `base`.
    pub(crate) fn counter_a11y(&self, base: NodeId) -> Option<(NodeId, Node)> {
        (self.counter.w > 0.0).then(|| {
            let r = self.counter;
            let n = NodeBuilder::new(Role::Button)
                .label(self.counter_label.clone())
                .bounds(
                    f64::from(r.x),
                    f64::from(r.y),
                    f64::from(r.w),
                    f64::from(r.h),
                )
                .action(Action::Click)
                .build();
            (NodeId(base.0 + A11Y_COUNTER), n)
        })
    }
}

/// Os textos da barra, já traduzidos (a pintura e os testes lêem os MESMOS).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLine {
    pub position: String,
    pub counter: Counter,
    pub assist: Option<(String, Tone)>,
    pub badge: Option<(String, Tone)>,
    /// O modo de tipos do script (I14): «Strict» com `--!strict`.
    pub mode: Option<String>,
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
        let warnings = frame
            .problems
            .iter()
            .filter(|p| p.severity == Severity::Warning)
            .count();
        let counter = Counter {
            errors,
            warnings,
            label: tr_with(
                "code.status.counter",
                &[("errors", &errors), ("warnings", &warnings)],
            ),
        };
        Self {
            position,
            counter,
            assist: frame.assist.map(assist_line),
            badge: badge_line(frame.badge, frame.project_unsaved),
            mode: frame.strict.then(|| tr("code.status.strict").to_owned()),
        }
    }
}

/// O texto e o tom da peça do assistente (a pintura e as sondas lêem o MESMO).
#[must_use]
pub fn assist_line(a: Assist) -> (String, Tone) {
    let (key, tone) = match a {
        Assist::Starting => ("code.status.assist_starting", Tone::Plain),
        Assist::Restarting => ("code.status.assist_restarting", Tone::Warning),
        Assist::Missing => ("code.status.assist_missing", Tone::Warning),
        Assist::Stopped => ("code.status.assist_stopped", Tone::Warning),
    };
    (tr(key).to_owned(), tone)
}

/// O texto e o tom do selo (a pintura e as sondas lêem o MESMO). `project_unsaved` = o script foi
/// aplicado mas o projecto ainda não tem ficheiro: o selo di-lo e aponta o File ▸ Save As….
#[must_use]
pub fn badge_line(badge: Option<&Badge>, project_unsaved: bool) -> Option<(String, Tone)> {
    let base = badge.map(|b| match b {
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
    if !project_unsaved {
        return base;
    }
    Some(match base {
        Some((text, tone)) => (
            tr_with("code.status.project_unsaved_after", &[("badge", &text)]),
            if tone == Tone::Error {
                Tone::Error
            } else {
                Tone::Warning
            },
        ),
        None => (tr("code.status.project_unsaved").to_owned(), Tone::Warning),
    })
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
    view: &mut CodeView,
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
    let w = label_column_width(ts, font, [s.position.as_str()]);
    paint_text_elided(
        ts,
        scene,
        &s.position,
        x,
        y,
        font,
        (right - x).max(0.0),
        pal.status_text,
    );
    x += w + gap;
    // O contador: ⊗ N ⚠ M, cada ícone no tom da gravidade quando há algum.
    let c0 = x;
    let icon = INLINE_ICON_PX.min(h);
    for (id, n, tone) in [
        (IconId::Error, s.counter.errors, Tone::Error),
        (IconId::Warning, s.counter.warnings, Tone::Warning),
    ] {
        let ir = Rect {
            x,
            y: bar.y + (h - icon) * 0.5,
            w: icon,
            h: icon,
        };
        let c = color(pal, if n > 0 { tone } else { Tone::Plain });
        paint_icon(scene, id, ir, c, StrokeToken::Default.px());
        x += icon + Spacing::Xs.px();
        let t = n.to_string();
        let w = label_column_width(ts, font, [t.as_str()]);
        paint_text_elided(ts, scene, &t, x, y, font, (right - x).max(0.0), c);
        x += w + Spacing::Sm.px();
    }
    view.counter = Rect {
        x: c0,
        y: bar.y,
        w: (x - c0).max(0.0),
        h,
    };
    view.counter_label.clone_from(&s.counter.label);
    x += gap;
    if let Some(text) = s.mode.as_deref() {
        let w = label_column_width(ts, font, [text]);
        paint_text_elided(
            ts,
            scene,
            text,
            x,
            y,
            font,
            (right - x).max(0.0),
            pal.status_text,
        );
        x += w + gap;
    }
    // O selo (o resultado do salvar) ganha o seu lugar à direita; o assistente cabe no que sobra.
    let mut end = right;
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
        end = bx - gap;
    }
    if let Some((text, tone)) = &s.assist {
        paint_text_elided(
            ts,
            scene,
            text,
            x,
            y,
            font,
            (end - x).max(0.0),
            color(pal, *tone),
        );
    }
}
