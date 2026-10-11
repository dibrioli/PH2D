//! ⭐⭐⭐ **UM TIQUE QUE NÃO VAI SER DESENHADO COZE SÓ O ESTADO** (handoff COLLIDE_VIZINHOS, 11/10).
//!
//! A shell recupera um tique em dívida por cozimento (`ticks_owed`) e o scrub re-simula do
//! checkpoint até ao alvo; nos dois, só o ÚLTIMO tique chega ao ecrã. Compor os intermédios era
//! pagar a cadeia `Pure` inteira para o lixo: sem placa, a cena `PH2D_GPU_COOK_DEMO=8` cozinhava
//! o `motion.collide` (42 ms) em cada tique recuperado — `1 fps · 520 ms` no HUD.
//!
//! ⚠️ **O corte é pelo EFEITO do nó, nunca pela cena.** O estado entre tiques vive em dois sítios:
//! na saída de uma FONTE de `pre` — que a marcha (`marcha.rs`) coze em todo tique, logo um
//! consumidor de `pre` (`strobe`, `Pure` no manifesto) lê o tique certo sem ter sido cozido nos
//! intermédios — e no `memo` de um nó, que só anda quando ele coze (o `sim.step`, `Temporal`).
//! ⇒ todo nó não-`Pure` do cone dos sinks, e o cone dele, coze em todo tique; as tomadas também,
//! porque o livro-razão delas é por tique.

use crate::MotionCookPump;
use ph2d_nodegraph::cook::{OpResolver, TimeScopes};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// Os nós do cone de `sinks` que carregam estado entre tiques — ver o cabeçalho.
#[must_use]
pub fn nos_com_estado(graph: &Graph, ops: &dyn OpResolver, sinks: &[NodeId]) -> Vec<NodeId> {
    ph2d_nodegraph::cone_a_montante(graph, sinks)
        .into_iter()
        .filter(|&n| {
            // Um tipo que não resolve não coze; no tique desenhado ele erra como sempre.
            graph
                .node(n)
                .and_then(|i| ops.resolve(i.type_id()))
                .is_some_and(|op| op.manifest().effect != Effect::Pure)
        })
        .collect()
}

impl MotionCookPump {
    /// O braço `Sinks` de um tique que NÃO vai ser desenhado: coze só [`nos_com_estado`]. O
    /// `instances` e as `faixas` ficam com a última composição (o tique desenhado sobrescreve-os).
    pub(crate) fn cozinha_so_o_estado(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        sinks: &[NodeId],
        playhead: f64,
        scopes: &TimeScopes,
    ) {
        // Quem limpa as tomadas é o braço que as publica (ver `cozinha_as_tomadas`).
        self.tap_streams.clear();
        self.cozimentos_so_do_estado += 1;
        for n in nos_com_estado(graph, ops, sinks) {
            // O erro é do tique desenhado, que coze o mesmo nó e o guarda em `last_error`.
            let _ = self
                .cook
                .cook_scoped_fanned(graph, ops, n, playhead, scopes, &self.fans);
        }
    }

    /// Quantos tiques coseram só o estado — o readout que torna a economia OBSERVÁVEL (o desenho
    /// final é o mesmo ao bit, logo nenhuma régua de valor a vê).
    #[must_use]
    pub fn cozimentos_so_do_estado(&self) -> u64 {
        self.cozimentos_so_do_estado
    }
}

#[cfg(test)]
#[path = "so_o_estado_tests.rs"]
mod tests;
