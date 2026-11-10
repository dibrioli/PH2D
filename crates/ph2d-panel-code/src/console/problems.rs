//! ⭐ **O separador Problems da faixa de baixo** (acabamento, item 4 de `docs/CodeEditor/03_onda2.md`):
//! os MESMOS problemas da calha, um por linha — a pintura das linhas, o que o leitor de ecrã diz de
//! cada uma e os nós da lista. A faixa (o cabeçalho, os separadores, o console e o clique) mora em
//! [`super`].

use super::{BandTab, ConsoleHit, ConsoleView, bounds};
use crate::chrome::{self, Ui};
use crate::frame::Severity;
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing};
use ph2d_ui_table::{Cell, Column, Hit, Ink, Layout, Metrics, Row, StableWidths, TableState};
use ph2d_vector::VectorScene;

/// Uma linha da lista de problemas, já composta: o texto (a explicação do I8 ou a mensagem crua), a
/// linha do script (1-based) e em quem aconteceu (vazio = «—»).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProblemRow<'a> {
    pub severity: Severity,
    pub text: &'a str,
    pub line: usize,
    pub on: &'a str,
}

/// Uma linha da lista pintada no último quadro (a sonda dos testes e o nó do leitor de ecrã).
#[derive(Clone, Debug, PartialEq)]
pub struct PaintedProblem {
    pub index: usize,
    pub rect: Rect,
    pub severity: Severity,
    pub text: String,
    /// O «On» como pintado («—» sem objectos).
    pub on: String,
    /// 1-based.
    pub line: usize,
    pub selected: bool,
    /// O que o leitor de ecrã diz dela.
    pub label: String,
}

/// A tabela da lista (plano 14 Q4 C): a rolagem, as larguras e o que o rato acende e prime.
#[derive(Default)]
pub(super) struct Table {
    state: TableState,
    steady: StableWidths,
    last_current: Option<usize>,
    pressed: Option<usize>,
    mouse: Option<(f32, f32)>,
    pub(super) painted: Option<(Layout, Vec<Row>)>,
}

impl Table {
    /// A roda: `dy` px (positivo = para cima), em linhas inteiras (o `layout` prende ao fim).
    pub(super) fn scroll_by(&mut self, dy: f32) {
        let rh = Metrics::standard().row_h;
        let n = (dy / rh).round();
        let n = if n == 0.0 && dy != 0.0 {
            dy.signum()
        } else {
            n
        };
        self.state.scroll = (self.state.scroll - n * rh).max(0.0);
    }

    fn on_bar(&self, x: f32, y: f32) -> bool {
        self.painted
            .as_ref()
            .is_some_and(|(lay, rows)| lay.hit(rows, x, y) == Some(Hit::Bar))
    }
}

const SEV: &str = "sev";
const PROBLEM: &str = "problem";
const ON: &str = "on";
const LINE: &str = "line";

fn columns() -> Vec<Column> {
    vec![
        Column::new(SEV, String::new()).snug(),
        Column::new(PROBLEM, tr("code.problems.col.problem")).flex(),
        Column::new(ON, tr("code.problems.col.on")),
        Column::new(LINE, tr("code.problems.col.line")).end().mono(),
    ]
}

fn dot_of(s: Severity) -> ColorToken {
    match s {
        Severity::Error => ColorToken::Danger,
        Severity::Warning => ColorToken::Warn,
        Severity::Info => ColorToken::Info,
    }
}

/// O «On» de uma linha: os objectos, ou «—».
fn on_text(row: &ProblemRow<'_>) -> String {
    if row.on.is_empty() {
        tr("code.problems.on_none").to_owned()
    } else {
        row.on.to_owned()
    }
}

/// Uma linha da tabela por problema (chave = a posição na lista).
fn table_rows(rows: &[ProblemRow<'_>]) -> Vec<Row> {
    let row = |(i, p): (usize, &ProblemRow<'_>)| Row {
        key: i.to_string(),
        cells: vec![
            Cell {
                dot: Some(dot_of(p.severity)),
                ..Cell::default()
            },
            Cell::new(p.text, Ink::Plain),
            Cell::new(on_text(p), Ink::Muted),
            Cell::new(p.line.to_string(), Ink::Number),
        ],
        ..Row::default()
    };
    rows.iter().enumerate().map(row).collect()
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

/// O que o leitor de ecrã diz de uma linha da lista (o molde do VS Code), com o «On» se há quem.
#[must_use]
pub fn problem_label(severity: Severity, text: &str, on: &str, line: usize) -> String {
    let key = match severity {
        Severity::Error => "code.problems.a11y_error",
        Severity::Warning => "code.problems.a11y_warning",
        Severity::Info => "code.problems.a11y_info",
    };
    let label = tr_with(key, &[("text", &text), ("line", &line)]);
    if on.is_empty() {
        return label;
    }
    tr_with("code.problems.a11y_on", &[("label", &label), ("who", &on)])
}

impl ConsoleView {
    /// A lista em tabela: o ponto da gravidade, o texto (cortado com «…»), em quem e a linha à
    /// direita. A linha `current` (o F8) é a seleccionada e rola à vista, em linhas inteiras.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_problems(
        &mut self,
        (rows, current): (&[ProblemRow<'_>], Option<usize>),
        body: Rect,
        view: &mut CodeView,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        ui: &Ui,
    ) {
        let t = &mut self.list;
        if rows.is_empty() {
            let pad = Spacing::Sm.px();
            let text = tr("code.problems.empty");
            let r = Rect::new(body.x + pad, body.y, body.w, super::row_h());
            chrome::label_in(scene, ts, text, r, ui.text3);
            self.empty = Some(text.to_owned());
            t.last_current = current;
            t.steady.reset();
            return;
        }
        let (cols, trows, m) = (columns(), table_rows(rows), Metrics::standard());
        if current != t.last_current {
            if let Some(c) = current {
                let fit = ((body.h - m.header_h) / m.row_h).floor().max(1.0);
                let (c, top) = (c as f32, t.state.scroll / m.row_h);
                if c < top {
                    t.state.scroll = c * m.row_h;
                } else if c >= top + fit {
                    t.state.scroll = (c + 1.0 - fit) * m.row_h;
                }
            }
            t.last_current = current;
        }
        let mut text = CodeCells {
            view,
            theme: ui.theme,
        };
        let natural = ph2d_ui_table::natural_widths(&cols, &trows, &m, ts, &mut text);
        let natural = t.steady.widths(&cols, &natural);
        t.state.selected = current.map(|c| c.to_string());
        let lay = ph2d_ui_table::layout(&cols, &trows, &natural, &mut t.state, body, &m);
        let over_bar = t
            .mouse
            .is_some_and(|(x, y)| lay.hit(&trows, x, y) == Some(Hit::Bar));
        let hover = t.mouse.filter(|_| !over_bar).and_then(|(x, y)| {
            lay.rows
                .iter()
                .find(|s| s.rect.contains(x, y) && lay.body.contains(x, y))
                .map(|s| s.index)
        });
        t.state.hover = hover.map(|h| h.to_string());
        let tones = (crate::cards::card_tone(), crate::cards::ground_tone());
        ph2d_ui_table::paint(
            &lay, &cols, &trows, &t.state, tones, scene, ts, &mut text, ui.theme,
        );
        let (h, p) = (
            hover.map(|h| h.to_string()),
            t.pressed.map(|p| p.to_string()),
        );
        ph2d_ui_table::paint_row_frames(&lay, &trows, h.as_deref(), p.as_deref(), scene, ui.theme);
        for span in &lay.rows {
            let (i, row) = (span.index, rows[span.index]);
            self.problems.push(PaintedProblem {
                index: i,
                rect: span.rect,
                severity: row.severity,
                text: row.text.to_owned(),
                on: on_text(&row),
                line: row.line,
                selected: current == Some(i),
                label: problem_label(row.severity, row.text, row.on, row.line),
            });
        }
        t.painted = Some((lay, trows));
    }

    /// A tabela da lista no último quadro: a geometria e as linhas (`None` = não pintada).
    #[must_use]
    pub fn problems_table(&self) -> Option<(&Layout, &[Row])> {
        let (lay, rows) = self.list.painted.as_ref()?;
        Some((lay, rows.as_slice()))
    }

    /// O botão em baixo em `(x, y)`: a barra ↕ começa o arrasto (fica no corpo); um problema fica
    /// premido (a moldura do clique) até largar. Devolve o alvo, como [`Self::hit`].
    pub fn press(&mut self, x: f32, y: f32) -> Option<ConsoleHit> {
        if let Some((lay, rows)) = self
            .list
            .painted
            .as_ref()
            .filter(|_| self.list.on_bar(x, y))
        {
            let (lay, rows) = (lay.clone(), rows.clone());
            let _ = self.list.state.press(&Hit::Bar, (x, y), &lay, &rows);
            return Some(ConsoleHit::Body);
        }
        let hit = self.hit(x, y);
        if let Some(ConsoleHit::Problem(k)) = hit {
            self.list.pressed = Some(k);
        }
        hit
    }

    /// O rato com o botão em baixo: arrasta a barra ↕ da lista. `true` = havia arrasto.
    pub fn drag_to(&mut self, y: f32) -> bool {
        self.list.state.bar_dragging() && {
            let _ = self.list.state.drag_to((0.0, y), &Metrics::standard());
            true
        }
    }

    /// Largar: acaba o arrasto da barra e a moldura do clique. `true` = havia arrasto.
    pub fn release(&mut self) -> bool {
        self.list.pressed = None;
        self.list.state.release()
    }

    /// O rato em `(x, y)` (a linha sob ele acende-se e ganha a moldura no próximo quadro).
    pub fn hover(&mut self, x: f32, y: f32) {
        self.list.mouse = self.area.contains(x, y).then_some((x, y));
    }

    /// O ponteiro na faixa: ↕ na barra da lista, a mão no que se clica, a seta no resto.
    #[must_use]
    pub fn look(&self, x: f32, y: f32) -> Option<PointerLook> {
        let hit = self.hit(x, y)?;
        Some(
            if self.list.state.bar_dragging() || self.list.on_bar(x, y) {
                PointerLook::ResizeNs
            } else if self.clickable(hit) {
                PointerLook::Hand
            } else {
                PointerLook::Arrow
            },
        )
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
