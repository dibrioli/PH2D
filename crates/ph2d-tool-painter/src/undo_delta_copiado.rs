//! Os bytes de uma entrada de undo: os que ela segura e os que CLONÁ-LA aloca (o que um instantâneo
//! do tocador declara).

use super::{ImageEntry, StoredEntry, StoredImages, StoredMap, StoredPlane};

impl<T: Copy + PartialEq + Send + Sync> StoredPlane<T> {
    /// Só a janela (`Patch`); os buffers inteiros (`Whole`) são `Arc` partilhados com o original, que
    /// os segura de qualquer forma.
    pub(crate) fn copiado_bytes(&self) -> usize {
        match self {
            Self::Patch { before, after, .. } => (before.len() + after.len()) * size_of::<T>(),
            Self::Unchanged | Self::Whole { .. } => 0,
        }
    }
}

impl<T: Copy + PartialEq + Send + Sync> StoredEntry<T> {
    pub(super) fn heap_bytes(&self) -> usize {
        match self {
            Self::Both(p) => p.heap_bytes(),
            Self::OnlyBefore(v) | Self::OnlyAfter(v) => v.len() * size_of::<T>(),
        }
    }

    fn copiado_bytes(&self) -> usize {
        match self {
            Self::Both(p) => p.copiado_bytes(),
            Self::OnlyBefore(_) | Self::OnlyAfter(_) => 0,
        }
    }
}

impl<T: Copy + PartialEq + Send + Sync> StoredMap<T> {
    /// Ver [`StoredPlane::copiado_bytes`].
    pub(crate) fn copiado_bytes(&self) -> usize {
        self.entries.values().map(StoredEntry::copiado_bytes).sum()
    }
}

impl StoredImages {
    /// Ver [`StoredPlane::copiado_bytes`].
    pub(crate) fn copiado_bytes(&self) -> usize {
        self.entries
            .values()
            .map(|e| match e {
                ImageEntry::Patch { before, after, .. } => before.len() + after.len(),
                _ => 0,
            })
            .sum()
    }
}
