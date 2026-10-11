//! ⭐⭐⭐ **O COLISOR DE UM OBJECTO DA CENA** — doc 115 §33 (decisão do dono de 2026-10-11:
//! *«automático é melhor»*).
//!
//! Até à §33 a membrana declarava a caixa de TODO objecto publicado (doc 115 W4) e o `Collide` do
//! `motion.output` armava o passe. Hoje a declaração é do CARTÃO: o `source.object` tem o próprio
//! `Collide` (o espelho do da `source.shape`), a lei da caixa mora com ele
//! ([`ph2d_node_source_object::colisor`]) e o passe do fim arma-se sozinho quando alguma peça
//! declara. ⇒ a aparência que a membrana publica não leva colisor nenhum.
//!
//! O que aqui fica é a **MIGRAÇÃO** de um documento gravado antes da §33
//! ([`migra_o_collide_da_saida`]): um `Output` com o `Collide` ligado separava os objectos que lhe
//! chegavam, e esse documento tem de abrir a separar as MESMAS peças.

use std::collections::BTreeSet;

use ph2d_nodegraph::graph::{Graph, NodeId};

/// O nome do interruptor que o `motion.output` tinha antes da §33 — só a migração o lê.
const COLLIDE_LEGADO_DA_SAIDA: &str = "collide";

/// ⭐⭐⭐ **MIGRA o `Collide` do `Output` (pré-§33) para os OBJECTOS a montante dele.**
///
/// Para cada `motion.output` com o override legado `collide ≥ 0,5`, liga o `Collide` de todo
/// `source.object` a montante (as formas já tinham o botão delas, e era ele que as fazia declarar);
/// depois apaga o override legado de TODA saída — ligado ou não, ele é um param que o nó já não
/// declara (`Violation::UnknownParam`). Devolve quantos objectos mudaram. Idempotente: a segunda
/// corrida não acha override nenhum e devolve `0`.
///
/// ⚠️ O fecho a montante segue TODAS as arestas, as de estado (`delayed`) incluídas — a mesma lei
/// do diagnóstico do colisor: numa zona de simulação as peças chegam pela aresta de estado.
pub fn migra_o_collide_da_saida(graph: &mut Graph) -> usize {
    let legado = |g: &Graph, n: NodeId| {
        g.node_param_overrides(n)
            .and_then(|p| p.get(COLLIDE_LEGADO_DA_SAIDA))
            .copied()
    };
    let saidas: Vec<(NodeId, bool)> = graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ph2d_node_motion_output::TYPE_NAME)
        .filter_map(|n| legado(graph, n.id).map(|v| (n.id, v >= 0.5)))
        .collect();
    let mut objectos = BTreeSet::new();
    for &(saida, ligada) in &saidas {
        if ligada {
            objectos.extend(
                a_montante(graph, saida)
                    .into_iter()
                    .filter(|n| e_um_objecto(graph, *n)),
            );
        }
    }
    for &(saida, _) in &saidas {
        graph.clear_param(saida, COLLIDE_LEGADO_DA_SAIDA);
    }
    let chave = ph2d_node_source_object::COLLIDE_PARAM;
    let mut mudaram = 0;
    for o in objectos {
        let ja = graph
            .node_param_overrides(o)
            .and_then(|p| p.get(chave))
            .is_some_and(|v| *v >= 0.5);
        if !ja {
            graph.set_param(o, chave, 1.0);
            mudaram += 1;
        }
    }
    mudaram
}

fn e_um_objecto(graph: &Graph, n: NodeId) -> bool {
    graph
        .nodes()
        .iter()
        .any(|i| i.id == n && i.type_name == ph2d_node_source_object::MANIFEST.name)
}

/// O fecho para trás pelas arestas, sem `de`.
fn a_montante(graph: &Graph, de: NodeId) -> BTreeSet<NodeId> {
    let mut visto = BTreeSet::new();
    let mut pilha = vec![de];
    while let Some(n) = pilha.pop() {
        for e in graph.edges() {
            let a = e.from.0;
            if e.to.0 == n && a != de && visto.insert(a) {
                pilha.push(a);
            }
        }
    }
    visto
}

#[cfg(test)]
#[path = "motion_bridge_objects_collider_tests.rs"]
mod tests;
