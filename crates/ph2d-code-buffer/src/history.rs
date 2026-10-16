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
    /// Os bytes deste grupo (a conta do tecto).
    bytes: usize,
}

fn change_set_bytes(cs: &ChangeSet) -> usize {
    std::mem::size_of_val(cs.changes()) + cs.changes().iter().map(|c| c.insert.capacity()).sum::<usize>()
}

fn step_bytes(s: &Step) -> usize {
    change_set_bytes(&s.forward) + change_set_bytes(&s.inverse)
}

fn selection_bytes(s: &Selection) -> usize {
    std::mem::size_of_val(s.ranges())
}

/// ⛔ **Os bytes que o histórico de UM documento guarda.** De que recurso é: da MEMÓRIA. Medido
/// (`docs/CodeEditor/02_onda1.md` §Medições do fecho, D): 194 B por tecla escrita, 297 por Enter, o
/// texto colado por colar; um dia de escrita contínua (144 000 teclas) = 27.9 MB, e cabe.
pub const HISTORY_BYTES_MAX: usize = 32 << 20;

#[derive(Debug, Clone)]
pub struct History {
    undo: Vec<Group>,
    redo: Vec<Group>,
    /// Profundidade do undo em que o texto foi salvo; `None` = esse estado já não é alcançável.
    saved: Option<usize>,
    cap: usize,
    /// Os bytes dos grupos dos dois lados.
    bytes: usize,
}

impl Default for History {
    fn default() -> Self {
        Self { undo: Vec::new(), redo: Vec::new(), saved: Some(0), cap: HISTORY_BYTES_MAX, bytes: 0 }
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
        self.bytes -= self.redo.iter().map(|g| g.bytes).sum::<usize>();
        self.redo.clear();
        let space = !inserted.is_empty() && inserted.chars().all(char::is_whitespace);
        let step = Step { forward, inverse };
        let joins = self.undo.last().is_some_and(|g| {
            !g.sealed
                && kind != EditKind::Other
                && g.kind == kind
                && g.after == before
                && !(kind == EditKind::Typing && space && !g.last_char_space)
        });
        match self.undo.last_mut() {
            Some(g) if joins => {
                let was = g.bytes;
                let cap0 = g.steps.capacity();
                g.bytes += step_bytes(&step) + selection_bytes(&after);
                g.bytes -= selection_bytes(&g.after);
                g.steps.push(step);
                g.bytes += (g.steps.capacity() - cap0) * std::mem::size_of::<Step>();
                g.after = after;
                g.last_char_space = space;
                self.bytes += g.bytes - was;
            }
            _ => {
                let bytes = std::mem::size_of::<Group>()
                    + std::mem::size_of::<Step>()
                    + step_bytes(&step)
                    + selection_bytes(&before)
                    + selection_bytes(&after);
                self.bytes += bytes;
                self.undo.push(Group { steps: vec![step], before, after, kind, last_char_space: space, sealed: false, bytes });
            }
        }
        self.enforce_cap();
    }

    /// Passado o tecto, os grupos mais ANTIGOS saem (o último fica sempre); o estado salvo que saia
    /// deixa de ser alcançável.
    fn enforce_cap(&mut self) {
        let mut k = 0;
        while self.bytes > self.cap && k + 1 < self.undo.len() {
            self.bytes -= self.undo[k].bytes;
            k += 1;
        }
        if k > 0 {
            self.undo.drain(..k);
            self.saved = self.saved.and_then(|s| s.checked_sub(k));
        }
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

    /// Os bytes guardados (os passos dos dois lados e as seleções de cada grupo).
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    pub(crate) fn set_cap(&mut self, bytes: usize) {
        self.cap = bytes;
        self.enforce_cap();
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
