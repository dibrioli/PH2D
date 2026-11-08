//! O estado da janela que oferece as cópias ao arrancar: uma linha por [`Orphan`] e três respostas.

use crate::Orphan;

/// A resposta do artista a uma cópia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Abre a cópia como o projecto actual, por gravar, com o caminho original lembrado.
    Restore,
    /// Apaga a cópia.
    Discard,
    /// Deixa-a no disco; volta a ser oferecida no próximo arranque (até fazer uma semana).
    Later,
}

/// A janela: as cópias por responder, a mais nova primeiro.
#[derive(Debug, Default)]
pub struct Prompt {
    /// As cópias por responder.
    pub entries: Vec<Orphan>,
}

impl Prompt {
    /// A janela aberta? (há o que oferecer)
    #[must_use]
    pub fn is_open(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Responde à linha `i`: tira-a da lista e devolve-a com a resposta. `Discard` já apaga; o
    /// `Restore` quem o executa é a shell (abrir o projecto é dela), pela [`crate::Session::adopt`].
    pub fn answer(&mut self, i: usize, choice: Choice) -> Option<(Orphan, Choice)> {
        if i >= self.entries.len() {
            return None;
        }
        let o = self.entries.remove(i);
        if choice == Choice::Discard {
            o.discard();
        }
        Some((o, choice))
    }

    /// `Esc` = `Later` em todas: nada se perde por fechar a janela.
    pub fn later_all(&mut self) {
        self.entries.clear();
    }
}
