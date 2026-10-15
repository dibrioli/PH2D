//! Histórico por GRUPOS: um passo de undo junta as transações que o artista sente como uma só.
//!
//! Lei do agrupamento (determinística, sem relógio):
//! - só se agrupam `Typing` com `Typing` e `Deleting` com `Deleting`;
//! - a transação nova tem de partir da seleção em que o grupo acabou (o cursor não saltou);
//! - escrever um espaço depois de algo que não é espaço abre grupo novo («palavra contínua»);
//! - `seal()` (salvar, clique, foco) fecha o grupo.

use crate::change::ChangeSet;
use crate::selection::Selection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    /// Um grafema escrito à mão, sem quebra de linha.
    Typing,
    /// Backspace / Delete.
    Deleting,
    /// Tudo o resto (colar, Enter, substituir, indentar): sempre um passo próprio.
    Other,
}

#[derive(Debug, Clone)]
struct Step {
    forward: ChangeSet,
    inverse: ChangeSet,
}

#[derive(Debug, Clone)]
pub(crate) struct Group {
    steps: Vec<Step>,
    pub(crate) before: Selection,
    pub(crate) after: Selection,
    kind: EditKind,
    last_char_space: bool,
    sealed: bool,
}

#[derive(Debug, Clone)]
pub struct History {
    undo: Vec<Group>,
    redo: Vec<Group>,
    /// Profundidade do undo em que o texto foi salvo; `None` = esse estado já não é alcançável.
    saved: Option<usize>,
}

impl Default for History {
    fn default() -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), saved: Some(0) }
    }
}

impl History {
    pub(crate) fn record(
        &mut self,
        forward: ChangeSet,
        inverse: ChangeSet,
        before: Selection,
        after: Selection,
        kind: EditKind,
        inserted: &str,
    ) {
        if !self.redo.is_empty() && self.saved.is_some_and(|s| s > self.undo.len()) {
            self.saved = None;
        }
        self.redo.clear();
        let space = !inserted.is_empty() && inserted.chars().all(char::is_whitespace);
        let step = Step { forward, inverse };
        if let Some(g) = self.undo.last_mut() {
            let joins = !g.sealed
                && kind != EditKind::Other
                && g.kind == kind
                && g.after == before
                && !(kind == EditKind::Typing && space && !g.last_char_space);
            if joins {
                g.steps.push(step);
                g.after = after;
                g.last_char_space = space;
                return;
            }
        }
        self.undo.push(Group { steps: vec![step], before, after, kind, last_char_space: space, sealed: false });
    }

    pub(crate) fn mark_saved(&mut self) {
        self.seal();
        self.saved = Some(self.undo.len());
    }

    /// O texto é o que foi salvo (desfazer até lá volta a «limpo»).
    pub fn at_saved(&self) -> bool {
        self.saved == Some(self.undo.len())
    }

    pub fn seal(&mut self) {
        if let Some(g) = self.undo.last_mut() {
            g.sealed = true;
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// As trocas a aplicar, em ordem, para desfazer o último grupo, e a seleção de antes.
    pub(crate) fn pop_undo(&mut self) -> Option<(Vec<ChangeSet>, Selection)> {
        let mut g = self.undo.pop()?;
        g.sealed = true;
        let cs = g.steps.iter().rev().map(|s| s.inverse.clone()).collect();
        let sel = g.before.clone();
        self.redo.push(g);
        self.seal();
        Some((cs, sel))
    }

    pub(crate) fn pop_redo(&mut self) -> Option<(Vec<ChangeSet>, Selection)> {
        let g = self.redo.pop()?;
        let cs = g.steps.iter().map(|s| s.forward.clone()).collect();
        let sel = g.after.clone();
        self.undo.push(g);
        Some((cs, sel))
    }
}
