//! **O achador de SUBSTEPS** — quem o pump pergunta antes do cook de cada quadro.
//!
//! Filho de [`super`]: ele mexe no `cook` privado do pump.

use super::MotionCookPump;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::graph::Graph;

/// O nome do param que declara *"o meu interior sub-tica"* — a convenção que o pump lê
/// (ver [`MotionPump::substep_declared_zones`]).
const SUBSTEPS_PARAM: &str = "substeps";

impl MotionCookPump {
    /// **Roda os SUBSTEPS declarados** antes do cook do quadro (folha 13, o último P1).
    ///
    /// ⚠️ **A declaração é uma CONVENÇÃO DE MANIFESTO, não um canal novo:** um nó cujo manifesto
    /// declara um param chamado `substeps` está a dizer *"o meu interior sub-tica"*, e é o mesmo
    /// param que o artista edita — um fato, um lugar. É a forma das convenções de stream que este
    /// módulo já usa (`texture_id`, `geometry_id`), e evita uma tabela paralela que o próximo nó
    /// nasceria fora.
    ///
    /// ⚠️ **`frame_start` sai do relógio do PRÓPRIO cook**, e `None` (o primeiro tique de todos)
    /// **pula**: ali a zona ainda emite o `init` e não há span a subdividir. Adivinhar um começo
    /// aqui é o defeito que [`Cook::substep`] documenta — o 1º quadro rodaria grosso e a
    /// defasagem nunca mais sairia.
    pub(super) fn substep_declared_zones(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        playhead: f64,
    ) {
        let Some(frame_start) = self.cook.prev_playhead() else {
            return;
        };
        for inst in graph.nodes() {
            let Some(op) = ops.resolve(inst.type_id()) else {
                continue;
            };
            let manifest = op.manifest();
            if manifest.param_default(SUBSTEPS_PARAM).is_none() {
                continue;
            }
            let n = graph
                .node_param_overrides(inst.id)
                .and_then(|m| m.get(SUBSTEPS_PARAM).copied())
                .or_else(|| manifest.param_default(SUBSTEPS_PARAM))
                .unwrap_or(1.0);
            // Um substep é uma CONTAGEM: arredonda, e o piso do laço já trata `<= 1`.
            let n = n.round().clamp(1.0, u32::from(u16::MAX) as f32) as u32;
            let _ = self
                .cook
                .substep(graph, ops, inst.id, frame_start, playhead, n);
        }
    }
}
