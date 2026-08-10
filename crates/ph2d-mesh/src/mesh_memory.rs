//! **QUANTO uma malha pesa** — a contabilidade de bytes.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os planos privados. O corte é de
//! responsabilidade: o pai responde *o que uma malha É*, e este arquivo *quanto
//! dela o alocador está segurando* — pergunta que nasceu quando alguém passou a
//! GUARDAR malhas (o histórico do escultor) e precisou de um teto.

use super::{Face, Mesh, QueryScratch, RegionScratch};

impl Mesh {
    /// **Quantos bytes esta malha segura** — todos os planos, mais as duas
    /// estruturas derivadas que ninguém vê e que pesam metade do total.
    ///
    /// ⚠️ **Ela existe porque quem GUARDA uma malha precisa de um teto em
    /// BYTES**, e a lição é do ADR-0117: um teto por CONTAGEM é multiplicador,
    /// não limite — a pilha de undo do escultor guarda a malha inteira a cada
    /// remesh, e a 512 isso são **146 MB por entrada** (medido em
    /// `ph2d-sdf/tests/probe_repeat_remesh.rs`, pela residência do processo).
    ///
    /// ⚠️ **E a soma dos planos PÚBLICOS não serve**, que é por que este método
    /// mora aqui e não no chamador: posições, normais e faces são ~74 MB dos
    /// 146 — a adjacência e o octree são a outra metade, e um teto que os
    /// ignorasse deixaria passar o dobro do que declara.
    ///
    /// A conta é de CAPACIDADE, não de comprimento: é o que o alocador de fato
    /// está segurando.
    #[must_use]
    pub fn footprint_bytes(&self) -> usize {
        let opt = |v: &Option<Vec<f32>>| v.as_ref().map_or(0, |x| x.capacity() * size_of::<f32>());
        self.positions.capacity() * size_of::<[f32; 3]>()
            + self.normals.capacity() * size_of::<[f32; 3]>()
            + self.curvatures.capacity() * size_of::<f32>()
            + self.curv_world.capacity() * size_of::<f32>()
            + self
                .colors
                .as_ref()
                .map_or(0, |c| c.capacity() * size_of::<[f32; 3]>())
            + opt(&self.masks)
            + opt(&self.ao)
            + opt(&self.thickness)
            + self.faces.capacity() * size_of::<Face>()
            + self.face_normals.capacity() * size_of::<[f32; 3]>()
            + self.adjacency.memory_bytes()
            + self.octree.memory_bytes()
    }
}

impl QueryScratch {
    /// Bytes que este scratch segura — a sonda de memória o soma para que o
    /// custo do gesto não fique fora da conta.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        (self.faces.capacity() + self.seen.capacity()) * size_of::<u32>()
    }
}

impl RegionScratch {
    /// Bytes que este scratch segura.
    #[must_use]
    pub fn capacity_bytes(&self) -> usize {
        (self.faces.capacity() + self.verts.capacity()) * size_of::<u32>()
            + self.face_seen.capacity()
            + self.vert_seen.capacity()
            + self.tmp.capacity() * size_of::<[f32; 3]>()
            + self.refit.capacity_bytes()
    }
}
