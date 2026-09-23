//! ⭐⭐ **A FORMA VIVA CONDICIONAL** — a cerca da placa para um nó que só desenha uma forma
//! vectorial em ALGUNS modos (o `source.lsystem` em `Branches`).
//!
//! A cerca irmã, [`super::graph_has_live_vector_source`], pergunta pelo TIPO: o `source.shape` e
//! o `source.text` desenham sempre uma forma, e a placa não tem rota para um `geometry_id`. O
//! L-System não se deixa perguntar assim — em `Lines` ele emite posições (e a placa desenha-as),
//! em `Branches` emite a fita que a shell construiu. ⇒ a pergunta é de INSTÂNCIA, com os params
//! resolvidos, e quem sabe responder é o NÓ
//! (`ph2d_node_registry::NodeRegistry::emits_live_vector`).
//!
//! ⛔⛔ **Achado pela varredura das cenas de várias saídas (doc 119 §7), e NÃO é um defeito do
//! multi-sink:** a `=108` corria na CPU só por ter várias saídas; com a cerca do multi-sink
//! levantada a placa desenhava **cinco quadrados onde a CPU desenha cinco plantas**. Um grafo do
//! artista com UM L-System seguido de um nó que a placa despacha (`lsystem → move → output`) já
//! caía no mesmo — nenhuma cena de demo tinha essa forma, e por isso nenhum gate o via.
//!
//! ⚠️ **Antes de planear, pela mesma razão da irmã:** a bomba da CPU tem de ser dona do tique
//! desde o início (nada de marchar um prefixo sequencial duas vezes).

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};

/// O motivo, dito em voz alta, de um nó que desenha uma forma NESTE modo ficar na CPU.
pub(crate) const RECUSA_FORMA_CONDICIONAL: &str =
    "CPU: um no desenha uma FORMA vectorial neste modo (ex.: L-System em Ramos)";

/// **Algum nó do documento desenha uma forma vectorial com os params DESTE quadro?**
///
/// Só os tipos que declaram um predicado pagam a resolução dos params (`O(params)` por nó
/// condicional; os outros saem pelo `has_live_vector_condition`, `O(log n)`).
pub(crate) fn desenha_forma_condicional(motion: &mut MotionState, seconds: f64) -> bool {
    let condicionais: Vec<(
        ph2d_nodegraph::graph::NodeId,
        NodeTypeId,
        &'static NodeManifest,
    )> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter_map(|n| {
            let ty = NodeTypeId::of(n.type_name.as_str());
            if !motion.registry.has_live_vector_condition(ty) {
                return None;
            }
            let manifest = motion.registry.resolve(ty)?.manifest();
            Some((n.id, ty, manifest))
        })
        .collect();
    condicionais.into_iter().any(|(id, ty, manifest)| {
        let resolved = crate::motion_externals::resolved_params(motion, id, seconds, manifest);
        let get = |name: &str| resolved.get(name).copied().unwrap_or(0.0);
        motion.registry.emits_live_vector(ty, &get)
    })
}

#[cfg(test)]
#[path = "motion_bridge_gpu_forma_tests.rs"]
mod tests;
