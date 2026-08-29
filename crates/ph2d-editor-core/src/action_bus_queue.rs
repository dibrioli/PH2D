//! ⭐ **A FILA** do [`super::action_bus`] — o irmão por ASSUNTO do ficheiro que a nomeia.
//!
//! # ⚠️ Porque a fila saiu, e não o vocabulário
//!
//! O `action_bus.rs` são duas coisas: o **vocabulário** (`EditorAction`, 560 das 708 linhas que ele
//! tinha) e a **fila** que o transporta. Ele bateu no tecto de 700 LOC em 2026-08-27, e a lei da
//! casa é *partir por assunto, nunca subir a tolerância*.
//!
//! ⛔ **O corte óbvio seria tirar o `EditorAction`** — é ele que cresce. Mas ele cresce por
//! **acrescento no meio**, e este repo corre linhas paralelas em worktrees: mover 560 linhas põe
//! toda linha que acrescenta uma acção em conflito textual com esta. ⇒ sai a **fila**, que são 54
//! linhas no FIM do ficheiro, onde ninguém escreve. *Ao criar foundational, projecte-o para
//! isolamento* (CLAUDE.md §0.2).

use super::action_bus::EditorAction;

/// FIFO queue of [`EditorAction`]s. Held on `HeroScreen` as a single
/// `bus: ActionBus` field replacing the 20 scattered `pending_X`
/// `Option`s. Cleared by the shell once per frame after drain.
#[derive(Debug, Default)]
pub struct ActionBus {
    queue: Vec<EditorAction>,
}

impl ActionBus {
    /// Construct an empty bus. Equivalent to `Default::default()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append `action` to the back of the queue.
    pub fn push(&mut self, action: EditorAction) {
        self.queue.push(action);
    }

    /// Drain every pending action. The bus is empty after this call.
    /// Returns an iterator the shell consumes via a single `match`.
    pub fn drain(&mut self) -> std::vec::Drain<'_, EditorAction> {
        self.queue.drain(..)
    }

    /// Non-consuming iterator over queued actions. Used by editor-
    /// side guards that need to ask "does the bus already carry a
    /// variant of this kind?" without dispatching it — e.g.
    /// `inspector_sync` skips re-seeding the Inspector's name /
    /// visibility widgets when an unsent edit is already in flight,
    /// so a frame between push + drain doesn't clobber the user's
    /// in-progress UI state.
    pub fn iter(&self) -> std::slice::Iter<'_, EditorAction> {
        self.queue.iter()
    }

    /// True iff no actions are queued.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Number of pending actions.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Discard any queued actions without dispatching. Used by tests
    /// + reset paths; production code should always `drain`.
    #[cfg(any(test, debug_assertions))]
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}
