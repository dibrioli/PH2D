//! ⭐⭐ **UM `Collide` QUE A SIMULAÇÃO NÃO OUVE** (doc 125 §2) — a régua do
//! [`Deficit::ColisorMudo`](crate::Deficit::ColisorMudo).
//!
//! O colisor tem quem o DECLARA (o `Collide` da `source.shape` e do `source.object`) e quem o LÊ:
//! um simulador dentro do passo (o corpo mole, a corda, o `sim.step`) ou o passe do fim do
//! cozimento, que desde o doc 115 §33 se arma SOZINHO em todo sink. ⇒ uma declaração que chega a
//! um sink nunca é muda; o que sobra é a forma que VESTE uma simulação (o corpo está a montante da
//! junção) sem que nenhum leitor a receba: depois do corpo o `Collide` só arruma o desenho (doc 115
//! §15.3). A cura (`"shape"`) é ligar a forma à porta `shape` do corpo.

use std::collections::BTreeSet;

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;

use crate::param_reader;

/// A porta pela qual um corpo recebe as PEÇAS que simula.
const PORTA_DAS_PECAS: &str = "shape";

pub(crate) fn colisor_mudo(
    graph: &Graph,
    reg: &NodeRegistry,
    node: NodeId,
    ty: NodeTypeId,
) -> Option<&'static str> {
    let chave = reg.collider_switch(ty)?;
    if param_reader(graph, reg, node, ty)(chave) < 0.5 {
        return None;
    }
    let jusante = fecho(graph, node, true);
    if jusante
        .iter()
        .any(|n| reg.reads_declared_collider(tipo(graph, *n)))
    {
        return None;
    }
    let corpo_a_montante = graph.nodes().iter().any(|b| {
        let t = NodeTypeId::of(&b.type_name);
        reg.reads_declared_collider(t)
            && tem_porta_das_pecas(reg, t)
            && fecho(graph, b.id, true).iter().any(|n| jusante.contains(n))
    });
    corpo_a_montante.then_some("shape")
}

fn tipo(graph: &Graph, n: NodeId) -> NodeTypeId {
    graph
        .nodes()
        .iter()
        .find(|i| i.id == n)
        .map_or(NodeTypeId::of(""), |i| NodeTypeId::of(&i.type_name))
}

fn tem_porta_das_pecas(reg: &NodeRegistry, t: NodeTypeId) -> bool {
    reg.manifests()
        .find(|m| m.id == t)
        .is_some_and(|m| m.inputs.iter().any(|p| p.name == PORTA_DAS_PECAS))
}

/// O fecho pelas arestas, para a frente ou para trás, sem `de`.
///
/// ⚠️ **As arestas de ESTADO (`delayed`) contam**, e foi o censo das cenas que o mediu: numa zona de
/// simulação as peças entram pela porta `init` e chegam ao `sim.step` pela aresta de estado
/// (`zone --pre--> força → step`) — sem ela as cenas `=114`/`=115` acusavam um `Collide` que o
/// `sim.step` lê a cada tique.
fn fecho(graph: &Graph, de: NodeId, frente: bool) -> BTreeSet<NodeId> {
    let mut visto = BTreeSet::new();
    let mut pilha = vec![de];
    while let Some(n) = pilha.pop() {
        for e in graph.edges() {
            let (a, b) = if frente {
                (e.from.0, e.to.0)
            } else {
                (e.to.0, e.from.0)
            };
            if a == n && b != de && visto.insert(b) {
                pilha.push(b);
            }
        }
    }
    visto
}
