//! ⭐⭐ **UM `Collide` LIGADO QUE NINGUÉM OUVE** (doc 125 §2) — a régua do
//! [`Deficit::ColisorMudo`](crate::Deficit::ColisorMudo).
//!
//! O colisor tem duas chaves: quem o DECLARA (o `Collide` da `source.shape`) e quem o LÊ (um
//! simulador que o lê dentro do passo — o corpo mole, a corda, o `sim.step` — ou o passe do fim,
//! armado pelo `Collide` do Output). Uma chave sozinha não faz nada, e o botão que o artista ligou
//! fica mudo. As três perguntas, uma por cura:
//!
//! - `"shape"` — a forma VESTE uma simulação (o corpo está a montante da junção) e nenhum leitor a
//!   recebe: depois do corpo o `Collide` só arruma o desenho (doc 115 §15.3). A cura é ligar a
//!   forma à porta `shape` do corpo.
//! - `"output"` — a forma declara e nada a lê: ligar o `Collide` do Output.
//! - `"collide"` — o Output arma o passe e nada a montante declara: ligar o `Collide` de um Shape.

use std::collections::BTreeSet;

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN};
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
    let ligado = |n: NodeId, t: NodeTypeId, p: &str| param_reader(graph, reg, n, t)(p) >= 0.5;
    if let Some(chave) = reg.collider_switch(ty) {
        if !ligado(node, ty, chave) {
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
        if corpo_a_montante {
            return Some("shape");
        }
        let passe_armado = jusante.iter().any(|n| {
            let t = tipo(graph, *n);
            reg.contact_pass_switch(t).is_some_and(|c| ligado(*n, t, c))
        });
        return (!passe_armado).then_some("output");
    }
    if let Some(chave) = reg.contact_pass_switch(ty) {
        if !ligado(node, ty, chave) {
            return None;
        }
        let declara = fecho(graph, node, false).iter().any(|n| {
            let t = tipo(graph, *n);
            reg.is_object_source(t)
                || reg.collider_switch(t).is_some_and(|c| ligado(*n, t, c))
                || nomeia_uma_coluna_do_colisor(graph, *n)
        });
        return (!declara).then_some("collide");
    }
    None
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

/// Um param de TEXTO que escreve uma coluna do colisor (um `attr.*` que a declara à mão).
fn nomeia_uma_coluna_do_colisor(graph: &Graph, n: NodeId) -> bool {
    graph.node_text_params().get(&n).is_some_and(|ps| {
        ps.values().any(|v| {
            matches!(
                v.trim(),
                COLLIDER_COLUMN | COLLIDER_BOX_COLUMN | COLLIDER_OFFSET_COLUMN
            )
        })
    })
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
