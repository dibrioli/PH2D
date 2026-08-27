//! **O CONSTRUTOR DO PLANO DE PREGUIÇA** (doc 89, folha 15) — quem decide, por quadro, que
//! roteadores podem saltar que entradas.
//!
//! ⚠️ **Ele mora no CRATE DO NÓ e não no escalonador**, pelo mesmo desenho que põe o
//! `time_fans` do `motion.clone` no crate dele: o `ph2d-nodegraph` não sabe o que é um
//! `value.switch`, e não é para saber. A shell chama isto uma vez por quadro e entrega o
//! resultado ao `Cook` ([`Cook::set_lazy_branches`]), como já entrega os externals.
//!
//! O que ele verifica, e por que ordem:
//!
//! ```text
//!   1. o no' e' um `value.switch`?
//!   2. o artista LIGOU o modo (`lazy = 1`)?          -> senao, nao entra no plano
//!   3. o `blend` escolhe a LEI (rotear / misturar)
//!   4. cada ramo: o cone dele e' saltavel?
//! ```
//!
//! ⚠️ **A terceira condição — a que este ficheiro possui — é a do ESTADO**, e ela é o que
//! separa uma optimização de um congelamento. Um ramo cuja sub-árvore acumula estado **não pode
//! ser saltado**: um tique que não o cozinhe deixa-o parado no passado, e o artista que volte a
//! ele encontra a simulação onde a largou. As outras duas (o `select` uniforme, a lei de quais
//! ramos são precisos) vivem no cook e no nó.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::cook::{LazyBranches, LazySelect, MAX_LAZY_CHOICES};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Graph, NodeId};
use std::collections::BTreeSet;

/// **O plano do quadro.** Vazio quando nenhum roteador tem o modo ligado — e vazio é o
/// comportamento de sempre, ao bit.
#[must_use]
pub fn plan(graph: &Graph, reg: &NodeRegistry) -> LazyBranches {
    let mut out = LazyBranches::new();
    for inst in graph.nodes() {
        if inst.type_name != crate::MANIFEST.name {
            continue;
        }
        let ov = graph.node_param_overrides(inst.id);
        let param =
            |name: &str, default: f32| ov.and_then(|m| m.get(name)).copied().unwrap_or(default);
        if param(crate::LAZY, 0.0) < 0.5 {
            continue; // o artista não ligou o modo
        }
        let blend = param("blend", 0.0) >= 0.5;
        let mut skippable = [false; MAX_LAZY_CHOICES];
        for (k, port) in crate::CHOICE_PORTS.iter().enumerate() {
            skippable[k] = branch_is_skippable(graph, reg, inst.id, *port);
        }
        out.insert(
            inst.id,
            LazySelect {
                select_port: crate::SELECT_PORT,
                select_column: crate::SELECT_COLUMN,
                choices: crate::CHOICE_PORTS,
                needed: if blend {
                    crate::needed_blend
                } else {
                    crate::needed_round
                },
                skippable,
            },
        );
    }
    out
}

/// **O CONE a montante desta porta é saltável?**
///
/// Saltável = nenhum nó do cone é [`Effect::Stateful`] **e** nenhuma aresta do cone é `pre`.
///
/// ⚠️ **As duas condições são uma só, e é por isso que um nó `Temporal` passa.** Um oscilador é
/// função pura do playhead — não cozinhá-lo num tique não perde nada, porque no tique seguinte
/// ele recalcula do relógio. O que acumula não é o `Temporal`: é a REALIMENTAÇÃO, e ela chega
/// sempre por uma aresta `pre` ou por um nó que declara `Stateful`. Recusar o `Temporal` também
/// seria conservador *e* errado sobre o mecanismo — e uma cerca que nomeia o mecanismo errado é
/// a que alguém remove por engano na wave seguinte.
///
/// ⚠️ **Os params CONDUZIDOS entram no cone** (doc 58): um fio que conduz um param é uma
/// dependência tão real quanto uma porta, e um driver com estado congelaria pelo mesmo motivo.
///
/// Uma porta sem aresta é saltável por vacuidade — não há nada para cozinhar.
fn branch_is_skippable(graph: &Graph, reg: &NodeRegistry, node: NodeId, port: u16) -> bool {
    let Some((src, _, delayed)) = graph.input_edge(node, port as usize) else {
        return true;
    };
    if delayed {
        return false; // a própria aresta do ramo é um `pre`
    }
    let mut seen = BTreeSet::new();
    let mut stack = vec![src];
    while let Some(n) = stack.pop() {
        if !seen.insert(n) {
            continue;
        }
        let Some(inst) = graph.node(n) else {
            return false; // um nó que não existe: não afirmamos nada sobre ele
        };
        let Some(op) = reg.resolve(inst.type_id()) else {
            return false; // um tipo que o registry não conhece
        };
        if op.manifest().effect == Effect::Stateful {
            return false;
        }
        for p in 0..op.manifest().inputs.len() {
            match graph.input_edge(n, p) {
                Some((_, _, true)) => return false, // realimentação dentro do cone
                Some((up, _, false)) => stack.push(up),
                None => {}
            }
        }
        if let Some(sources) = graph.param_sources(n) {
            for (up, _) in sources.values() {
                stack.push(*up);
            }
        }
    }
    true
}

#[cfg(test)]
#[path = "lazy_tests.rs"]
mod tests;
