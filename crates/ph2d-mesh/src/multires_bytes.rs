//! **QUANTO a pilha de níveis pesa** — a contabilidade de bytes.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados. Irmão do
//! `multires_reverse.rs`, e o corte é o mesmo do `mesh_memory.rs`: o pai
//! responde *o que uma pilha É*, aqui mora *quanto dela o alocador segura* —
//! pergunta que só existe porque a fila de desfazer do escultor guarda níveis
//! inteiros e precisa de um teto em BYTES.

use super::{DetachedLevel, Details, Mesh, Multires, SharedBefore, Stamped};

impl Details {
    /// Bytes segurados — somado por quem guarda um nível numa fila de desfazer.
    fn bytes(&self) -> usize {
        self.xyz.capacity() * size_of::<[f32; 3]>()
            + self
                .colors
                .as_ref()
                .map_or(0, |c| c.capacity() * size_of::<[f32; 3]>())
            + self
                .masks
                .as_ref()
                .map_or(0, |m| m.capacity() * size_of::<f32>())
    }
}

impl DetachedLevel {
    /// Bytes segurados — a malha do nível mais o detalhe dele.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.mesh.footprint_bytes() + self.details.bytes()
    }
}

impl SharedBefore {
    /// Bytes segurados — ver [`Stamped::bytes`].
    fn bytes(&self) -> usize {
        self.positions.capacity() * size_of::<[f32; 3]>()
            + self
                .colors
                .as_ref()
                .map_or(0, |c| c.capacity() * size_of::<[f32; 3]>())
            + self
                .masks
                .as_ref()
                .map_or(0, |m| m.capacity() * size_of::<f32>())
    }
}

impl Multires {
    /// **Quantos bytes esta pilha segura** — todos os níveis e todos os
    /// detalhes.
    ///
    /// Ver [`Mesh::footprint_bytes`]: quem GUARDA uma pilha (a fila de desfazer
    /// do escultor, quando uma peça é removida ou fundida) precisa de um teto em
    /// bytes, e um teto por CONTAGEM de entradas é multiplicador, não limite.
    #[must_use]
    pub fn footprint_bytes(&self) -> usize {
        self.levels.iter().map(Mesh::footprint_bytes).sum::<usize>()
            + self.details.iter().map(Details::bytes).sum::<usize>()
    }
}

impl Stamped {
    /// Bytes segurados — o que a fila de desfazer paga por esta entrada, irmã
    /// do [`super::Reversal::bytes`].
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.before.as_ref().map_or(0, SharedBefore::bytes)
            + self.details.as_ref().map_or(0, Details::bytes)
    }
}
