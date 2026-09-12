//! **O espelho da última sincronia de seleção canvas ↔ Hierarquia** (ADR-0110) — o TIPO. A lei
//! (quem mudou neste quadro, e portanto quem manda) é o `sync_selection` da shell
//! (`vec_selection.rs`).
//!
//! ⚠️ **Desceu da shell em 2026-09-12** (`line/render-loop`, A9 da auditoria de arquitectura): o
//! campo `vec_sel` da `App` tinha o TIPO na shell, e um campo assim não pode juntar-se ao
//! [`crate::state::VecState`]. Os dois campos, que eram privados ao módulo da shell, passaram a
//! `pub`: quem os lê e escreve é o mesmo `sync_selection`, agora do outro lado da fronteira.

use ph2d_vec_scene::VecPathId;

/// O espelho da última sincronia de seleção, dos DOIS lados. Ter os dois é o que
/// permite saber **quem** mudou neste frame — e portanto quem manda.
#[derive(Default)]
pub struct VecSelSync {
    /// Bits **vetoriais** que o gizmo tinha (ordenados). Só os nossos: um sprite
    /// selecionado nunca entra aqui, e por isso nunca é confundido com uma
    /// mudança da árvore.
    pub bits: Vec<u64>,
    /// Seleção de objeto do pen no mesmo instante.
    pub paths: Vec<VecPathId>,
}

impl VecSelSync {
    /// Força o próximo `sync_selection` da shell a RE-RODAR a promoção filho→container. O sync só
    /// a reroda quando o pen MUDA (branch 1) ou o conjunto vetorial do gizmo muda (branch 2) — e
    /// criar um **Envelope** re-parenteia o filho SEM mexer em nenhum dos dois. Sem esta
    /// invalidação, o gizmo fica no filho e a gaiola do envelope recém-criado nunca acende (alças
    /// de nó em vez da gaiola). Chamado pelo `create` do render_loop, logo antes do `sync` do mesmo
    /// frame.
    pub fn invalidate(&mut self) {
        self.paths.clear();
    }
}
