//! ⭐ **Um documento aberto** — o texto de UM script do projeto, com tudo o que a aba precisa: a
//! gramática que o segue, a vista, as dobras, a busca, o diagnóstico do último salvar e o selo.

use crate::docs::SaveKind;
use crate::find::FindState;
use crate::problems::{ink_of, problems};
use ph2d_code_buffer::{Buffer, TextEdit};
use ph2d_code_syntax::{AutoEdit, Fold, IndentStyle, Syntax, SyntaxError};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{
    Badge, CodeFrame, CodeView, ConsoleView, FindBarView, FoldMark, Ink, Problem,
};
use ph2d_script::{ScriptDiagnostic, ScriptId};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// A largura de uma tabulação literal no ecrã, em colunas (o Luau do projeto indenta com 2 espaços;
/// uma tab colada de fora mostra-se na paragem de 4, a do Luau/Roblox Studio).
pub(crate) const TAB: usize = 4;
/// A indentação que o editor escreve: a do `bob.luau` e de todo script do projeto.
pub(crate) const INDENT: IndentStyle = IndentStyle::Spaces(2);

/// O que a gramática disse sobre UMA revisão do texto.
#[derive(Default)]
struct Derived {
    revision: Option<u64>,
    folds: Vec<Fold>,
    errors: Vec<SyntaxError>,
}

pub(crate) struct CodeDoc {
    pub script: ScriptId,
    pub title: String,
    pub buffer: Buffer,
    /// O texto da biblioteca da última vez que a aba e ela concordaram (abrir, salvar).
    pub synced: String,
    pub syntax: Syntax,
    pub auto: AutoEdit,
    pub view: CodeView,
    /// O byte do início de cada linha com a dobra FECHADA (segue o texto pelas edições).
    pub folded: Vec<usize>,
    pub find: FindState,
    pub find_view: FindBarView,
    pub console_view: ConsoleView,
    /// O diagnóstico do Luau para o texto SALVO (`None` = compilou, ou ainda não foi lido).
    pub saved_diag: Option<ScriptDiagnostic>,
    pub badge: Option<Badge>,
    /// A composição em curso; `true` = nasceu de uma tecla morta (a próxima tecla desfá-la).
    pub preedit: Option<(String, bool)>,
    /// Um salvar pedido pelo teclado, à espera do quadro da família.
    pub save: Option<SaveKind>,
    /// O byte onde o arrasto da seleção começou.
    pub drag: Option<usize>,
    pub last_click: Option<(u128, usize)>,
    derived: Derived,
}

impl CodeDoc {
    pub fn new(script: ScriptId, title: &str, text: &str) -> Self {
        let buffer = Buffer::new(text);
        let syntax = Syntax::new(buffer.text());
        Self {
            script,
            title: title.to_owned(),
            buffer,
            synced: text.to_owned(),
            syntax,
            auto: AutoEdit::new(INDENT),
            view: CodeView::new(),
            folded: Vec::new(),
            find: FindState::default(),
            find_view: FindBarView::default(),
            console_view: ConsoleView::default(),
            saved_diag: None,
            badge: None,
            preedit: None,
            save: None,
            drag: None,
            last_click: None,
            derived: Derived::default(),
        }
    }

    /// A gramática segue `edits` (as do `Buffer` — o `AutoEdit` já a avança sozinho) e as dobras
    /// fechadas andam com o texto.
    pub fn edited(&mut self, edits: &[TextEdit], syntax_too: bool) {
        if edits.is_empty() {
            return;
        }
        if syntax_too {
            self.syntax.edit(edits, self.buffer.text());
        }
        for b in &mut self.folded {
            for e in edits {
                *b = map_byte(*b, e);
            }
        }
        let t = self.buffer.text();
        for b in &mut self.folded {
            *b = t.byte_of_line(t.line_of_byte(*b));
        }
        self.folded.sort_unstable();
        self.folded.dedup();
    }

    fn derive(&mut self) {
        let rev = self.buffer.revision();
        if self.derived.revision == Some(rev) {
            return;
        }
        let t = self.buffer.text();
        self.derived.folds = self.syntax.folds(t);
        self.derived.errors = self.syntax.errors();
        self.derived.revision = Some(rev);
        let starts: Vec<usize> = self
            .derived
            .folds
            .iter()
            .map(|f| t.byte_of_line(f.start_line))
            .collect();
        self.folded.retain(|b| starts.contains(b));
    }

    /// Os problemas desta revisão (E7).
    pub fn problems(&mut self) -> Vec<Problem> {
        self.derive();
        let dirty = self.buffer.is_dirty();
        problems(
            self.buffer.text(),
            &self.derived.errors,
            self.saved_diag.as_ref(),
            dirty,
        )
    }

    pub fn fold_marks(&mut self) -> Vec<FoldMark> {
        self.derive();
        let t = self.buffer.text();
        self.derived
            .folds
            .iter()
            .map(|f| FoldMark {
                start: f.start_line,
                end: f.end_line,
                folded: self.folded.contains(&t.byte_of_line(f.start_line)),
            })
            .collect()
    }

    /// Abre/fecha a dobra que começa na linha `line`.
    pub fn toggle_fold(&mut self, line: usize) {
        self.derive();
        if !self.derived.folds.iter().any(|f| f.start_line == line) {
            return;
        }
        let b = self.buffer.text().byte_of_line(line);
        match self.folded.iter().position(|&x| x == b) {
            Some(i) => {
                self.folded.remove(i);
            }
            None => self.folded.push(b),
        }
    }

    /// Abre as dobras que escondem a linha do cursor primário.
    pub fn unfold_cursor(&mut self) {
        self.derive();
        let t = self.buffer.text();
        let line = t.line_of_byte(self.buffer.selection().primary().head);
        let hide: Vec<usize> = self
            .derived
            .folds
            .iter()
            .filter(|f| f.start_line < line && line <= f.end_line)
            .map(|f| t.byte_of_line(f.start_line))
            .collect();
        self.folded.retain(|b| !hide.contains(b));
    }

    /// O cursor primário à vista (e fora de qualquer dobra).
    pub fn reveal(&mut self) {
        self.unfold_cursor();
        let head = self.buffer.selection().primary().head;
        self.view.ensure_visible(&self.buffer, head);
    }

    /// O par do parêntese junto ao cursor primário.
    fn bracket(&self) -> Option<(usize, usize)> {
        let t = self.buffer.text();
        let h = self.buffer.selection().primary().head;
        [Some(h), h.checked_sub(1)]
            .into_iter()
            .flatten()
            .find_map(|b| self.syntax.matching_bracket(t, b).map(|m| (b, m)))
    }

    /// As três faixas da aba: a barra de busca em cima, o código, o console em baixo.
    pub fn layout(&self, area: Rect, console_open: bool) -> (Option<Rect>, Rect, Rect) {
        let find_h = self.find.bar_height();
        let console_h = ConsoleView::height(console_open, area.h).min((area.h - find_h).max(0.0));
        let find = (find_h > 0.0).then(|| Rect::new(area.x, area.y, area.w, find_h));
        let code = Rect::new(
            area.x,
            area.y + find_h,
            area.w,
            (area.h - find_h - console_h).max(0.0),
        );
        let console = Rect::new(area.x, area.y + area.h - console_h, area.w, console_h);
        (find, code, console)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        area: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
        console: &crate::console::Store,
    ) {
        let (find_r, code_r, console_r) = self.layout(area, console.open);
        let problems = self.problems();
        let folds = self.fold_marks();
        let bracket = self.bracket();
        self.find.refresh(&self.buffer);
        let preedit = self.preedit.as_ref().map(|(s, _)| s.as_str());
        let Self {
            buffer,
            syntax,
            view,
            find,
            badge,
            ..
        } = self;
        let mut spans = |r: std::ops::Range<usize>| -> Vec<(std::ops::Range<usize>, Ink)> {
            syntax
                .highlights(buffer.text(), r)
                .into_iter()
                .map(|h| (h.range, ink_of(h.kind)))
                .collect()
        };
        let frame = CodeFrame {
            buffer,
            spans: &mut spans,
            problems: &problems,
            folds: &folds,
            bracket,
            matches: if find.open { &find.matches } else { &[] },
            current_match: if find.open { find.current } else { None },
            preedit,
            caret_on: find.focus.is_none(),
            tab: TAB,
            badge: badge.as_ref(),
        };
        view.paint(frame, code_r, scene, ts, theme);
        if let Some(r) = find_r {
            let bar = self.find.bar();
            self.find_view.paint(&bar, r, scene, ts, theme);
        }
        let rows = console.rows();
        self.console_view.paint(
            &rows,
            console.open,
            console.errors(),
            console_r,
            scene,
            ts,
            theme,
        );
    }
}

/// Um byte do texto antes da edição `e`, no texto depois dela.
fn map_byte(b: usize, e: &TextEdit) -> usize {
    if b < e.start_byte {
        b
    } else if b >= e.old_end_byte {
        b - e.old_end_byte + e.new_end_byte
    } else {
        e.start_byte
    }
}
