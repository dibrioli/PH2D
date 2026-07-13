//! [`FlipDrawing`] — um desenho (conjunto de traços), refcontado pelos frames.
//!
//! Um mesmo desenho pode ser reusado por vários frames (ciclo / "duplicate as
//! instance") — o `users` conta quantos frames o referenciam. Quando cai a zero,
//! [`crate::FlipObject::remove_unused_drawings`] o reclama e remapeia os índices.
//! Espelha o `DrawingRuntime::user_count` do GP (`02_referencia §1`).
//!
//! **O `users` é um cache mantido pelas ops de frame do objeto** (`add_user`/
//! `remove_user`), com a verdade canônica reconstruível por
//! `FlipObject::recompute_users` (contagem direta a partir dos frames). A
//! compactação sempre recomputa antes de reclamar, então drift eventual do
//! cache se auto-corrige.

use crate::stroke::FlipStroke;
use serde::{Deserialize, Serialize};

/// Um desenho: os traços + a contagem de frames que o usam. `Default` = vazio,
/// `users = 0` (o refcount é acertado quando um frame o referencia).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlipDrawing {
    /// Os traços deste desenho, em ordem de z (fundo → topo).
    pub strokes: Vec<FlipStroke>,
    /// Quantos frames referenciam este desenho (refcount). `0` = reclamável.
    users: u32,
}

impl FlipDrawing {
    /// Desenho vazio, sem usuários (o `users` é acertado quando um frame o
    /// referencia — via as ops de `FlipObject`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Contagem atual de usuários.
    #[must_use]
    pub fn users(&self) -> u32 {
        self.users
    }

    /// Tem ao menos um frame apontando (não é lixo).
    #[must_use]
    pub fn has_users(&self) -> bool {
        self.users > 0
    }

    /// É instanciado por mais de um frame (ciclo / reuse). Editar um desenho
    /// instanciado propaga para todos os frames que o compartilham — o valor do
    /// "duplicate as instance".
    #[must_use]
    pub fn is_instanced(&self) -> bool {
        self.users > 1
    }

    /// +1 usuário (um frame passou a referenciar).
    pub fn add_user(&mut self) {
        self.users = self.users.saturating_add(1);
    }

    /// −1 usuário (um frame deixou de referenciar). Satura em `0`.
    pub fn remove_user(&mut self) {
        self.users = self.users.saturating_sub(1);
    }

    /// Define a contagem diretamente — usado por `FlipObject::recompute_users`
    /// (a reconstrução canônica a partir dos frames).
    pub(crate) fn set_users(&mut self, n: u32) {
        self.users = n;
    }

    // ── Seleção (W6 — o Edit Mode; `FlipStroke::selected`, domínio Curve) ──
    //
    // A seleção é **derivada dos traços**, nunca guardada em paralelo: perguntar
    // "quem está selecionado?" é uma varredura, e é assim que ela não pode
    // dessincronizar de uma inserção/remoção (o balde insere no meio da lista).

    /// Os índices dos traços selecionados, em ordem de z.
    #[must_use]
    pub fn selected_indices(&self) -> Vec<usize> {
        self.strokes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.selected)
            .map(|(i, _)| i)
            .collect()
    }

    /// Há algum traço selecionado?
    #[must_use]
    pub fn any_selected(&self) -> bool {
        self.strokes.iter().any(|s| s.selected)
    }

    /// Desmarca tudo. Devolve `true` se algo mudou (o chamador decide se isso é um
    /// passo de undo).
    pub fn clear_selection(&mut self) -> bool {
        let mut changed = false;
        for s in &mut self.strokes {
            changed |= std::mem::replace(&mut s.selected, false);
        }
        changed
    }

    /// Remove os traços selecionados. Devolve quantos saíram.
    pub fn delete_selected(&mut self) -> usize {
        let before = self.strokes.len();
        self.strokes.retain(|s| !s.selected);
        before - self.strokes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refcount_transitions() {
        let mut d = FlipDrawing::new();
        assert!(!d.has_users() && !d.is_instanced());
        d.add_user();
        assert!(d.has_users() && !d.is_instanced());
        d.add_user();
        assert!(d.is_instanced());
        d.remove_user();
        assert!(d.has_users() && !d.is_instanced());
        d.remove_user();
        assert!(!d.has_users());
        // Satura em 0 (não faz underflow).
        d.remove_user();
        assert_eq!(d.users(), 0);
    }
}
