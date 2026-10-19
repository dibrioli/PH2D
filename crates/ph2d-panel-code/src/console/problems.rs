//! ⭐ **O separador Problems da faixa de baixo** (acabamento, item 4 de `docs/CodeEditor/03_onda2.md`):
//! os MESMOS problemas da calha, um por linha — a pintura das linhas, o que o leitor de ecrã diz de
//! cada uma e os nós da lista. A faixa (o cabeçalho, os separadores, o console e o clique) mora em
//! [`super`].

use super::{BandTab, ConsoleView, bounds, row_h};
use crate::chrome::{self, Ui};
use crate::frame::Severity;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::paint::{paint_icon, paint_text_elided, rect_to_vello};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{INLINE_ICON_PX, Spacing, StrokeToken};
use ph2d_vector::VectorScene;

/// Uma linha da lista de problemas, já composta: o texto (a explicação do I8 ou a mensagem crua) e a
/// linha do script (1-based).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProblemRow<'a> {
    pub severity: Severity,
    pub text: &'a str,
    pub line: usize,
}

/// Uma linha da lista pintada no último quadro (a sonda dos testes e o nó do leitor de ecrã).
#[derive(Clone, Debug, PartialEq)]
pub struct PaintedProblem {
    pub index: usize,
    pub rect: Rect,
    pub severity: Severity,
    pub text: String,
    /// 1-based.
    pub line: usize,
    pub selected: bool,
    /// O que o leitor de ecrã diz dela.
    pub label: String,
}

/// O id da `List` do leitor de ecrã, somado ao do `Document` da aba (os `ListItem` vêm a seguir).
const A11Y_PROBLEMS: u64 = (1 << 31) | (1 << 29);

/// O título do separador da lista: «Problems», ou com a contagem («Problems (5)»).
#[must_use]
pub fn problems_title(count: usize) -> String {
    match count {
        0 => tr("code.band.problems").to_owned(),
        n => tr_with("code.band.problems_count", &[("count", &n)]),
    }
}

/// O que o leitor de ecrã diz de uma linha da lista (o molde do VS Code).
#[must_use]
pub fn problem_label(severity: Severity, text: &str, line: usize) -> String {
    let key = match severity {
        Severity::Error => "code.problems.a11y_error",
        Severity::Warning => "code.problems.a11y_warning",
        Severity::Info => "code.problems.a11y_info",
    };
    tr_with(key, &[("text", &text), ("line", &line)])
}

fn icon_of(s: Severity) -> IconId {
    match s {
        Severity::Error => IconId::Error,
        Severity::Warning => IconId::Warning,
        Severity::Info => IconId::Info,
    }
}

impl ConsoleView {
    /// A lista: um problema por linha — o ícone da gravidade, o texto (cortado com «…») e, a seguir,
    /// «Line N» no cinzento. A linha `current` rola para a vista quando muda.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_problems(
        &mut self,
        rows: &[ProblemRow<'_>],
        current: Option<usize>,
        body: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        ui: &Ui,
    ) {
        let pad = Spacing::Sm.px();
        let h = row_h();
        let fit = (((body.h - pad) / h).max(0.0) as usize).max(1);
        if rows.is_empty() {
            let text = tr("code.problems.empty");
            chrome::label_in(
                scene,
                ts,
                text,
                Rect::new(body.x + pad, body.y, body.w, h),
                ui.text3,
            );
            self.empty = Some(text.to_owned());
            self.last_current = current;
            return;
        }
        if current != self.last_current {
            if let Some(c) = current {
                if c < self.top {
                    self.top = c;
                } else if c >= self.top + fit {
                    self.top = c + 1 - fit;
                }
            }
            self.last_current = current;
        }
        self.top = self.top.min(rows.len().saturating_sub(fit));
        let pal = crate::palette::Palette::new(ui.theme);
        let font = chrome::font();
        let gap = Spacing::Md.px();
        for (k, i) in (self.top..rows.len().min(self.top + fit)).enumerate() {
            let row = rows[i];
            let r = Rect::new(body.x, body.y + k as f32 * h, body.w, h);
            let selected = current == Some(i);
            scene.fill_rect(
                rect_to_vello(r),
                if selected { ui.selection } else { ui.hover },
            );
            let s = INLINE_ICON_PX.min(h);
            let ir = Rect::new(r.x + pad, r.y + (h - s) * 0.5, s, s);
            paint_icon(
                scene,
                icon_of(row.severity),
                ir,
                pal.severity(row.severity),
                StrokeToken::Default.px(),
            );
            let tx = ir.x + ir.w + pad;
            let line_text = tr_with("code.problems.line", &[("line", &row.line)]);
            let lw = chrome::label_w(ts, &line_text);
            let room = (r.x + r.w - pad - lw - gap - tx).max(0.0);
            let tw = chrome::label_w(ts, row.text).min(room);
            let ty = r.y + (h - font) * 0.5;
            paint_text_elided(ts, scene, row.text, tx, ty, font, room, ui.text1);
            chrome::label_in(
                scene,
                ts,
                &line_text,
                Rect::new(tx + tw + gap, r.y, lw, h),
                ui.text3,
            );
            self.problems.push(PaintedProblem {
                index: i,
                rect: r,
                severity: row.severity,
                text: row.text.to_owned(),
                line: row.line,
                selected,
                label: problem_label(row.severity, row.text, row.line),
            });
        }
    }

    /// Com a lista à mostra, a `List` «Problems: N» (raiz em `roots`, filha do `Document` `doc`) com
    /// um `ListItem` por linha pintada (a realçada pelo F8 seleccionada).
    pub(super) fn problems_a11y(
        &self,
        doc: NodeId,
        roots: &mut Vec<NodeId>,
        out: &mut Vec<(NodeId, Node)>,
    ) {
        if !(self.open && self.tab == BandTab::Problems) {
            return;
        }
        let list = NodeId(doc.0 + A11Y_PROBLEMS);
        let item = |i: usize| NodeId(list.0 + 1 + i as u64);
        let body = Rect::new(
            self.area.x,
            self.header.y + self.header.h,
            self.area.w,
            (self.area.h - self.header.h).max(0.0),
        );
        let (x, y, w, h) = bounds(body);
        out.push((
            list,
            NodeBuilder::new(Role::List)
                .label(tr_with(
                    "code.problems.a11y_list",
                    &[("count", &self.problem_total)],
                ))
                .bounds(x, y, w, h)
                .children(self.problems.iter().map(|p| item(p.index)))
                .build(),
        ));
        for p in &self.problems {
            let (x, y, w, h) = bounds(p.rect);
            let mut n = NodeBuilder::new(Role::ListItem)
                .label(p.label.clone())
                .bounds(x, y, w, h)
                .action(Action::Click)
                .build();
            n.set_selected(p.selected);
            out.push((item(p.index), n));
        }
        roots.push(list);
    }
}
