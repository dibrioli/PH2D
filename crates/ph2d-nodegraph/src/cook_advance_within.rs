//! ⭐⭐⭐ **AVANÇAR SÓ O QUE A FRONTEIRA PRECISA** — a marcha do `pre` restrita ao cone a montante
//! de um conjunto de nós (ciclo 12, [doc 120 §8.5](../../../docs/Motion%20Nodes/120_ciclo_12_os_tectos_confortaveis.md)).
//!
//! ## O defeito que isto cura, medido
//!
//! [`Cook::advance_tick_fanned`] cozinha **toda** fonte de `pre` do grafo a cada tique — é a lei
//! certa quando a CPU coze o grafo inteiro. Na rota HÍBRIDA a CPU coze só até às FRONTEIRAS, e um
//! laço que o dispositivo reclamou (o `pre` do `motion.integrate`) **continuava a ser simulado na
//! CPU** para o resultado ser deitado fora: na escada dos tectos, com o carimbo já na placa, o
//! prefixo — que devia ser só a `source.object` — custava **`4,0 ms`** por quadro na RTX a `32 768`
//! partículas, contra `0,25 ms` do cozimento inteiro no dispositivo.
//!
//! ⇒ a marcha passa a perguntar **quem a fronteira de facto lê**: o cone a montante dos nós pedidos,
//! seguindo TODAS as arestas de entrada — as de `pre` incluídas, porque um nó que lê o tique
//! anterior de uma fonte precisa que essa fonte tenha avançado. Uma fonte de `pre` fora do cone é do
//! dispositivo (ou de ninguém), e é o que a rota totalmente-na-placa já faz: lá o pump não marcha.
//!
//! ⚠️ **Aditiva:** o [`Cook::advance_tick_fanned`] e os irmãos ficam byte-idênticos (passam `None`
//! ao laço interno). Só quem pede um alvo restrito recebe a marcha restrita.

use crate::cook::{Cook, CookError, OpResolver, TimeFans, TimeScopes};
use crate::graph::{Graph, NodeId};
use std::collections::BTreeSet;

/// **O cone a montante** de `alvo`, inclusivo: todo nó de que algum deles depende, por aresta
/// directa, de `pre` **ou por um fio que conduz um PARAM** (doc 58).
///
/// ⛔⛔ **O fio de param é dependência, e a 1.ª redacção esquecia-o** (auditoria do fecho,
/// 2026-09-24): ele não vive nas `edges()` mas o cozimento segue-o como uma entrada
/// (`Cook::cook_node`, 1b), e um param conduzido é precisamente o que torna um nó FRONTEIRA. Sem
/// este braço, um laço de `pre` que só conduz o param da fronteira ficava fora do cone e era
/// re-semeado a cada tique — o param lia sempre o 1.º valor.
///
/// ⚠️ **NÃO é o `cook_substep::upstream_cone`, e a diferença é de propósito:** aquele salta as
/// arestas de `pre` porque responde *«o que se re-avalia DENTRO de um tique»*; este responde *«que
/// fontes de `pre` têm de AVANÇAR para a fronteira ler o tique anterior certo»*, e aí a aresta de
/// `pre` é exactamente a dependência que conta (gate `o_cone_segue_as_arestas_de_pre`).
#[must_use]
pub fn cone_a_montante(graph: &Graph, alvo: &[NodeId]) -> BTreeSet<NodeId> {
    let mut cone: BTreeSet<NodeId> = alvo.iter().copied().collect();
    let mut fila: Vec<NodeId> = alvo.to_vec();
    while let Some(n) = fila.pop() {
        let por_aresta = graph
            .edges()
            .iter()
            .filter(|e| e.to.0 == n)
            .map(|e| e.from.0);
        let por_param = graph
            .param_sources(n)
            .into_iter()
            .flat_map(|fontes| fontes.values().map(|&(no, _)| no));
        for de in por_aresta.chain(por_param).collect::<Vec<_>>() {
            if cone.insert(de) {
                fila.push(de);
            }
        }
    }
    cone
}

impl Cook {
    /// [`Self::advance_tick_fanned`], mas avançando **só** as fontes de `pre` no cone a montante de
    /// `alvo` — ver o cabeçalho do módulo.
    pub fn advance_tick_fanned_within(
        &mut self,
        graph: &Graph,
        ops: &dyn OpResolver,
        playhead: f64,
        scopes: &TimeScopes,
        fans: &TimeFans,
        alvo: &[NodeId],
    ) -> Result<(), CookError> {
        let cone = cone_a_montante(graph, alvo);
        self.advance_tick_inner(graph, ops, playhead, scopes, fans, Some(&cone))
    }
}

#[cfg(test)]
#[path = "cook_advance_within_tests.rs"]
mod tests;
