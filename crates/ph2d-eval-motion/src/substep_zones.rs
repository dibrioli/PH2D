//! **O achador de SUBSTEPS** — quem o pump pergunta antes do cook de cada quadro.
//!
//! Filho de [`super`]: ele mexe no `cook` privado do pump.

use super::MotionCookPump;
use ph2d_nodegraph::cook::{OpResolver, substep_islands};
use ph2d_nodegraph::graph::Graph;

impl MotionCookPump {
    /// **Roda os SUBSTEPS declarados** antes do cook do quadro (folha 13, o último P1).
    ///
    /// ⚠️ **Substepa ILHAS, nunca declarantes soltos** — e isso é correção, não afinação:
    /// substepar cada zona por conta OVER-STEPA qualquer uma que viva no cone de outra, porque o
    /// laço de baixo re-cozinha o cone inteiro. Medido num par acoplado, a de cima ia de `4,876`
    /// para `15,094`. Quem parte em ilhas é [`substep_islands`], a MESMA porta que o sequenciador
    /// de device pergunta — duas cópias divergiriam justamente aqui.
    ///
    /// ⚠️ **`frame_start` sai do relógio do PRÓPRIO cook**, e `None` (o primeiro tique de todos)
    /// **pula**: ali a zona ainda emite o `init` e não há span a subdividir. Adivinhar um começo
    /// aqui é o defeito que [`ph2d_nodegraph::cook::Cook::substep`] documenta — o 1º quadro
    /// rodaria grosso e a defasagem nunca mais sairia.
    pub(super) fn substep_declared_zones(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        playhead: f64,
    ) {
        let Some(frame_start) = self.cook.prev_playhead() else {
            return;
        };
        for island in substep_islands(graph, ops) {
            let _ = self.cook.substep(
                graph,
                ops,
                island.root,
                frame_start,
                playhead,
                island.substeps,
            );
        }
    }
}
