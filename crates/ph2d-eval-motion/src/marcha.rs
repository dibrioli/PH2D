//! ⭐⭐⭐ **A MARCHA do `pre` de um tique — UMA porta, para a reprodução e para o scrub** (ciclo 12,
//! doc 120 §8.5, e a auditoria do fecho de 2026-09-24).
//!
//! Na rota de SINKS (a CPU coze o grafo inteiro) toda fonte de `pre` avança. Na rota de FRONTEIRAS
//! (a híbrida) avança só o que a CPU de facto LÊ: o cone a montante das fronteiras **e das tomadas
//! armadas** — um laço que o dispositivo reclamou e que ninguém na CPU lê é do dispositivo, como na
//! rota totalmente-na-placa, onde a bomba não marcha (`4,0 ms` poupados por quadro na escada a
//! `32 768`).
//!
//! ⛔⛔ **As tomadas entram no cone, e a 1.ª redacção esquecia-as:** o `cozinha_as_tomadas` coze-as
//! nesta mesma bomba e no mesmo instante, logo um gizmo ou um `pulse.signal` a jusante de um laço
//! fora do cone lia um laço re-semeado a cada tique — uma simulação que nunca anda.
//!
//! ⛔ **E é UMA porta porque eram DUAS respostas:** o scrub marchava completo e a reprodução restrita,
//! logo as tomadas viam estados diferentes consoante se tocava ou se arrastava a régua, e o custo
//! que a cura tirou voltava em cada scrub.
//!
//! ⚠️ **Declarado:** o estado de um laço fora do cone NÃO se guarda na CPU (é do dispositivo). Se a
//! rota cair da placa para a CPU a meio, esse laço recomeça do tique em que caiu — a mesma
//! propriedade que a rota totalmente-na-placa sempre teve.

use crate::MotionCookPump;
use crate::cook_target::CookTarget;
use ph2d_nodegraph::cook::{OpResolver, TimeScopes};
use ph2d_nodegraph::graph::{Graph, NodeId};

impl MotionCookPump {
    /// Avança o `pre` de um tique para `target` — ver o cabeçalho do módulo.
    pub(crate) fn avanca_o_pre(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        playhead: f64,
        scopes: &TimeScopes,
        target: &CookTarget,
    ) {
        let _ = match target {
            CookTarget::Boundaries(nodes) => {
                let alvo: Vec<NodeId> = nodes.iter().chain(&self.taps).copied().collect();
                self.cook
                    .advance_tick_fanned_within(graph, ops, playhead, scopes, &self.fans, &alvo)
            }
            CookTarget::Sinks { .. } => self
                .cook
                .advance_tick_fanned(graph, ops, playhead, scopes, &self.fans),
        };
    }
}
