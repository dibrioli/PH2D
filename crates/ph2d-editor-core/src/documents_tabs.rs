//! ⭐ B5(4) — **o desfazer das operações de ABA** (doc 15): criar, duplicar, importar, restaurar uma
//! versão, renomear, arrastar e apagar um quadro.
//!
//! As abas são a barra partilhada por todos os documentos, então têm UMA pilha só delas; cada documento
//! (a cena, cada quadro) continua com a sua. O `Ctrl+Z` de um documento desfaz o MAIS RECENTE entre o
//! último passo dele e a última operação de abas — as horas são as do
//! [`ph2d_board_model::undo_tick`]. Um quadro apagado guarda-se com o desfazer dele: volta inteiro.

use ph2d_board_model::{Board, BoardId, BoardOp, Element, History, MAX_STEPS, undo_tick};
use ph2d_kanban_model::{Kanban, KanbanId};

use super::Documents;

/// Um passo: o que o desfaz.
#[derive(Debug)]
pub(crate) enum TabStep {
    /// Põe este nome de volta.
    Rename(BoardId, String),
    /// Leva a aba de volta a esta posição.
    Move(BoardId, usize),
    /// Tira o quadro (o inverso de criar, duplicar, importar, restaurar uma versão); se era o activo,
    /// abre `back` (a aba de antes), senão a vizinha.
    Remove {
        board: BoardId,
        back: Option<BoardId>,
    },
    /// Põe de volta, na posição `at`, um quadro apagado e o desfazer dele; ele fica activo se o era
    /// quando saiu (`focus` — o tldraw, `paginas_P5`/`P6`).
    Restore {
        board: Box<Board>,
        at: usize,
        history: History,
        focus: bool,
    },
    /// D17 — põe de volta um Kanban apagado e as referências dele em cada quadro (mesmas chaves).
    KanbanBack {
        kanban: Box<Kanban>,
        refs: Vec<(BoardId, Element)>,
    },
    /// D17 — apaga outra vez o Kanban (o refazer de [`TabStep::KanbanBack`]).
    KanbanGone(KanbanId),
}

/// Uma pilha das abas: cada passo com a sua hora.
type TabStack = Vec<(u64, TabStep)>;

/// As duas pilhas das abas, com a hora de cada passo.
#[derive(Debug, Default)]
pub(crate) struct TabHistory {
    undo: TabStack,
    redo: TabStack,
    /// A hora do último passo NOVO da cena (a shell avisa): um refazer de abas mais velho já não vale.
    scene_push: u64,
}

/// O que um passo de abas mudou na barra — quem o pede regista ou esquece a aba.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabChange {
    /// O quadro voltou (registe a aba dele).
    Back(BoardId),
    /// O quadro saiu (esqueça a aba dele).
    Gone(BoardId),
    /// Nome ou posição.
    Same,
}

impl TabHistory {
    fn push(&mut self, step: TabStep) {
        self.undo.push((undo_tick(), step));
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
    }
}

impl Documents {
    /// Guarda o inverso de uma operação de abas que acabou de se fazer.
    pub(crate) fn record_tab(&mut self, step: TabStep) {
        self.tabs.push(step);
    }

    /// A hora da operação de abas que o próximo `Ctrl+Z` desfaria (`None` = nenhuma).
    #[must_use]
    pub fn tab_undo_at(&self) -> Option<u64> {
        self.tabs.undo.last().map(|s| s.0)
    }

    /// A hora da operação de abas que o próximo refazer levaria — `None` também quando o artista fez
    /// outra coisa (num quadro ou na cena) depois de a desfazer: refazer um «apagar» por cima de um
    /// quadro que já mudou apagava trabalho novo.
    #[must_use]
    pub fn tab_redo_at(&self) -> Option<u64> {
        let at = self.tabs.redo.last()?.0;
        let newest = self
            .live
            .histories
            .values()
            .map(History::last_push)
            .fold(self.tabs.scene_push, u64::max);
        (at > newest).then_some(at)
    }

    /// A shell empurrou um passo novo da CENA: os refazer de abas anteriores deixam de valer.
    pub fn scene_stepped(&mut self) {
        self.tabs.scene_push = undo_tick();
    }

    /// Desfaz (ou refaz) a última operação de abas. `None` = nada a fazer nessa direcção.
    pub fn undo_tab(&mut self, redo: bool) -> Option<TabChange> {
        if redo && self.tab_redo_at().is_none() {
            self.tabs.redo.clear();
            return None;
        }
        // Um passo cujo quadro já não está (saiu por outra via) gasta-se e o seguinte responde: um
        // `Ctrl+Z` nunca se gasta a desfazer o nada.
        loop {
            let (from, _) = self.stacks(redo);
            let (_, step) = from.pop()?;
            if let Some((inverse, change)) = self.apply_tab(step) {
                let (_, to) = self.stacks(redo);
                to.push((undo_tick(), inverse));
                return Some(change);
            }
        }
    }

    fn stacks(&mut self, redo: bool) -> (&mut TabStack, &mut TabStack) {
        let t = &mut self.tabs;
        if redo {
            (&mut t.redo, &mut t.undo)
        } else {
            (&mut t.undo, &mut t.redo)
        }
    }

    fn apply_tab(&mut self, step: TabStep) -> Option<(TabStep, TabChange)> {
        let before = self.active;
        Some(match step {
            TabStep::Rename(id, name) => {
                let old = self.boards.get(id)?.name.clone();
                self.boards.rename(id, name);
                (TabStep::Rename(id, old), TabChange::Same)
            }
            TabStep::Move(id, to) => {
                let from = self.position(id)?;
                self.boards.move_tab(id, to);
                (TabStep::Move(id, from), TabChange::Same)
            }
            TabStep::Remove { board, back } => {
                let (gone, at, history) = self.take_board(board)?;
                if before == Some(board)
                    && let Some(back) = back.filter(|b| self.boards.get(*b).is_some())
                {
                    self.activate(Some(back));
                }
                let restore = TabStep::Restore {
                    board: Box::new(gone),
                    at,
                    history,
                    focus: before == Some(board),
                };
                (restore, TabChange::Gone(board))
            }
            TabStep::Restore {
                board,
                at,
                history,
                focus,
            } => {
                let id = board.id;
                if !self.boards.restore(at, *board) {
                    return None;
                }
                self.live.histories.insert(id, history);
                if focus {
                    self.activate(Some(id));
                }
                (
                    TabStep::Remove {
                        board: id,
                        back: before,
                    },
                    TabChange::Back(id),
                )
            }
            TabStep::KanbanBack { kanban, refs } => {
                let id = kanban.id;
                if self.kanbans.borrow().get(id).is_some() {
                    return None;
                }
                self.kanbans.borrow_mut().insert(*kanban);
                for (b, el) in refs {
                    if let Some(board) = self.boards.get_mut(b) {
                        let _ = BoardOp::Put(el).apply(&mut board.doc);
                    }
                }
                (TabStep::KanbanGone(id), TabChange::Same)
            }
            TabStep::KanbanGone(id) => {
                let (kanban, refs) = self.take_kanban(id)?;
                let kanban = Box::new(kanban);
                (TabStep::KanbanBack { kanban, refs }, TabChange::Same)
            }
        })
    }

    /// ⭐ D17 (dono 30/10) — apaga o Kanban `id` do projecto (o menu do cartão nos Assets). As
    /// referências dele em cada quadro saem no MESMO passo (um elemento sem documento não teria o que
    /// mostrar) e a aba dele fecha; o `Ctrl+Z` devolve o documento e cada referência, com as mesmas
    /// chaves. `false` = não existe (ou é a agenda, que não se apaga).
    pub fn remove_kanban(&mut self, id: KanbanId) -> bool {
        let Some((kanban, refs)) = self.take_kanban(id) else {
            return false;
        };
        let kanban = Box::new(kanban);
        self.record_tab(TabStep::KanbanBack { kanban, refs });
        true
    }

    fn take_kanban(&mut self, id: KanbanId) -> Option<(Kanban, Vec<(BoardId, Element)>)> {
        if id == KanbanId::AGENDA {
            return None;
        }
        let kanban = self.kanbans.borrow_mut().remove(id)?;
        let ids: Vec<BoardId> = self.boards.boards().iter().map(|b| b.id).collect();
        let mut refs = Vec::new();
        for b in ids {
            if let Some(h) = self.live.histories.get_mut(&b) {
                h.forget_redo_putting(|e| e.kanban().is_some_and(|r| r.kanban == id));
            }
            let Some(board) = self.boards.get_mut(b) else {
                continue;
            };
            let gone: Vec<Element> = board
                .doc
                .live()
                .filter(|e| e.kanban().is_some_and(|k| k.kanban == id))
                .cloned()
                .collect();
            for el in &gone {
                let _ = BoardOp::Delete(el.id).apply(&mut board.doc);
            }
            if self.active == Some(b)
                && let Some(ed) = self.live.editor.as_mut()
            {
                let keep: Vec<_> = ed.selection().iter().copied().collect();
                ed.select(
                    &board.doc,
                    keep.into_iter().filter(|e| gone.iter().all(|g| g.id != *e)),
                );
            }
            refs.extend(gone.into_iter().map(|el| (b, el)));
        }
        crate::screens::hero::board_video::kanban::link::request_close(id);
        Some((kanban, refs))
    }

    pub(crate) fn position(&self, id: BoardId) -> Option<usize> {
        self.boards.boards().iter().position(|b| b.id == id)
    }

    /// Esquece as operações de abas (um projecto novo: os ids recomeçam).
    pub(crate) fn forget_tabs(&mut self) {
        self.tabs = TabHistory::default();
    }
}

#[cfg(test)]
#[path = "documents_tabs_tests.rs"]
mod tests;
