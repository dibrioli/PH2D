//! ⭐ **Um documento aberto** — o texto de UM script do projeto, com tudo o que a aba precisa: a
//! gramática que o segue, a vista, as dobras, a busca, o diagnóstico do último salvar e o selo.

use crate::docs::SaveKind;
use crate::find::FindState;
use crate::folds::ClosedFolds;
use crate::problems::{ink_of, problems};
use ph2d_code_buffer::{Buffer, TextEdit};
use ph2d_code_syntax::{AutoEdit, IndentStyle, Syntax, SyntaxError};
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
    /// As dobras FECHADAS (seguem o texto pelas edições).
    pub(crate) folds: ClosedFolds,
    pub find: FindState,
    pub find_view: FindBarView,
    pub console_view: ConsoleView,
    /// O diagnóstico do Luau para o texto SALVO (`None` = compilou, ou ainda não foi lido).
    pub saved_diag: Option<ScriptDiagnostic>,
    /// Onde ele mora no buffer: o byte do início da linha (segue as edições) e se era um erro de
    /// SINTAXE (esse o tree-sitter mostra vivo; os outros — uma declaração `ph2d.property` — ficam
    /// na calha enquanto há ●).
    diag_anchor: Option<(usize, bool)>,
    /// O relógio da última tecla/clique nesta aba (o cursor não pisca enquanto se escreve).
    pub last_input_ns: u128,
    pub badge: Option<Badge>,
    /// O último Ctrl+S aplicou o texto, mas o projecto ainda não tem ficheiro (o selo di-lo).
    pub project_unsaved: bool,
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
            folds: ClosedFolds::default(),
            find: FindState::default(),
            find_view: FindBarView::default(),
            console_view: ConsoleView::default(),
            saved_diag: None,
            diag_anchor: None,
            last_input_ns: 0,
            badge: None,
            project_unsaved: false,
            preedit: None,
            save: None,
            drag: None,
            last_click: None,
            derived: Derived::default(),
        }
    }

    /// Fecha com alterações por salvar: fica só o texto e o histórico ([`crate::docs::Parked`]).
    pub fn park(self) -> crate::docs::Parked {
        crate::docs::Parked {
            title: self.title,
            buffer: self.buffer,
            synced: self.synced,
        }
    }

    /// Reabre um estacionado: a gramática e a vista refazem-se do texto.
    pub fn unpark(script: ScriptId, p: crate::docs::Parked) -> Self {
        let mut d = Self::new(script, &p.title, "");
        d.syntax = Syntax::new(p.buffer.text());
        d.buffer = p.buffer;
        d.synced = p.synced;
        d
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
        if let Some(a) = &mut self.diag_anchor {
            for e in edits {
                a.0 = map_byte(a.0, e);
            }
        }
        self.folds.edited(edits, self.buffer.text());
        self.find.follow(&self.buffer, edits);
    }

    fn derive(&mut self) {
        let rev = self.buffer.revision();
        if self.derived.revision == Some(rev) {
            return;
        }
        self.derived.errors = self.syntax.errors();
        self.derived.revision = Some(rev);
        self.folds.check(&mut self.syntax, self.buffer.text());
    }

    /// Os problemas desta revisão (E7). O diagnóstico do texto salvo mostra-se na linha onde o seu
    /// texto foi parar; um erro de sintaxe sai com o ● (o tree-sitter mostra-o vivo).
    pub fn problems(&mut self) -> Vec<Problem> {
        self.derive();
        let dirty = self.buffer.is_dirty();
        let t = self.buffer.text();
        let saved = match (&self.saved_diag, self.diag_anchor) {
            (Some(d), Some((byte, syntax))) if !(dirty && syntax) => {
                Some((d, t.line_of_byte(byte.min(t.len()))))
            }
            _ => None,
        };
        problems(t, &self.derived.errors, saved)
    }

    /// O diagnóstico do Luau para o texto salvo (lido da cena em todo quadro): ancora-o quando muda.
    pub fn set_saved_diag(&mut self, diag: Option<ScriptDiagnostic>) {
        if diag == self.saved_diag {
            return;
        }
        self.derive();
        let t = self.buffer.text();
        self.diag_anchor = diag.as_ref().and_then(|d| d.line).map(|n| {
            let line = (n as usize)
                .saturating_sub(1)
                .min(t.line_count().saturating_sub(1));
            let syntax = crate::problems::culprit_on_line(t, &self.derived.errors, line).is_some();
            (t.byte_of_line(line), syntax)
        });
        self.saved_diag = diag;
    }

    /// Dobra (ou desdobra) todas as dobras do texto (a única pergunta pelo ficheiro inteiro).
    pub fn fold_all(&mut self, fold: bool) {
        self.derive();
        let t = self.buffer.text();
        let all = if fold {
            self.syntax.folds(t)
        } else {
            Vec::new()
        };
        self.folds.set_all(&all, t);
        self.folds.arm(&mut self.syntax);
        // O cursor escondido sobe para o cabeçalho da dobra de fora (o idioma do VS Code).
        let line = t.line_of_byte(self.buffer.selection().primary().head);
        let outer = all
            .iter()
            .filter(|f| fold && f.start_line < line && line <= f.end_line)
            .map(|f| f.start_line)
            .min();
        if let Some(start) = outer {
            let b = t.byte_of_line(start);
            self.buffer
                .set_selection(ph2d_code_buffer::Selection::single(
                    ph2d_code_buffer::Range::cursor(b),
                ));
        }
    }

    /// As dobras fechadas, em linhas (os testes lêem-nas).
    pub fn closed_folds(&self) -> Vec<(usize, usize)> {
        self.folds
            .marks(self.buffer.text())
            .iter()
            .map(|f| (f.start, f.end))
            .collect()
    }

    /// Abre/fecha a dobra que começa na linha `line`.
    pub fn toggle_fold(&mut self, line: usize) {
        self.derive();
        let t = self.buffer.text();
        if let Some(f) = self.syntax.fold_at(t, line) {
            self.folds.toggle(f, t);
            self.folds.arm(&mut self.syntax);
        }
    }

    /// Abre as dobras fechadas que escondem a linha do cursor primário.
    pub fn unfold_cursor(&mut self) {
        self.derive();
        let t = self.buffer.text();
        let line = t.line_of_byte(self.buffer.selection().primary().head);
        self.folds.unfold_line(line, t);
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
        [Some(h), (h > 0).then(|| t.prev_grapheme(h))]
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
        blink_on: bool,
    ) {
        let (find_r, code_r, console_r) = self.layout(area, console.open);
        let problems = self.problems();
        let closed = self.folds.marks(self.buffer.text());
        let bracket = self.bracket();
        if self.find.open {
            self.find.refresh(&self.buffer);
        }
        let preedit = self.preedit.as_ref().map(|(s, _)| s.as_str());
        let Self {
            buffer,
            syntax,
            view,
            find,
            badge,
            project_unsaved,
            folds: closed_set,
            ..
        } = self;
        let mut spans = |r: std::ops::Range<usize>| -> Vec<(std::ops::Range<usize>, Ink)> {
            syntax
                .highlights(buffer.text(), r)
                .into_iter()
                .map(|h| (h.range, ink_of(h.kind)))
                .collect()
        };
        // Só as dobras das linhas que a vista pinta (a calha, o «…», o clique).
        let mut marks = |lines: std::ops::Range<usize>| -> Vec<FoldMark> {
            let t = buffer.text();
            syntax
                .folds_in(t, lines)
                .into_iter()
                .map(|f| FoldMark {
                    start: f.start_line,
                    end: f.end_line,
                    folded: closed_set.is_closed(t.byte_of_line(f.start_line)),
                })
                .collect()
        };
        let frame = CodeFrame {
            buffer,
            spans: &mut spans,
            problems: &problems,
            folds: &mut marks,
            closed: &closed,
            bracket,
            matches: if find.open { &find.matches } else { &[] },
            current_match: if find.open { find.current } else { None },
            preedit,
            caret_on: find.focus.is_none() && blink_on,
            tab: TAB,
            badge: badge.as_ref(),
            project_unsaved: *project_unsaved,
        };
        view.paint(frame, code_r, scene, ts, theme);
        if let Some(r) = find_r {
            let bar = self.find.bar();
            self.find_view.paint(&bar, r, scene, ts, theme);
        }
        self.console_view.paint(
            console,
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
pub(crate) fn map_byte(b: usize, e: &TextEdit) -> usize {
    if b < e.start_byte {
        b
    } else if b >= e.old_end_byte {
        b - e.old_end_byte + e.new_end_byte
    } else {
        e.start_byte
    }
}
