//! **O CONDUTOR NA PLACA** (doc 110 §14.1 (2)) — quando o número de um param dirigido pode ir do
//! condutor ao uniform do consumidor SEM passar pela CPU.
//!
//! ⚠️ **Só quando o param ENTRA SÓ no uniform.** Uma lei do hospedeiro que o lesse (a variante, a
//! aplicabilidade, a contagem, as varreduras de uma grelha, uma operação estrutural, um uniform
//! derivado, uma redução) precisaria dele na CPU — e leria o valor de omissão enquanto a placa lia
//! o fio: as duas rotas a desenhar documentos diferentes, a falha que o §6 existe para impedir.
//! ⇒ o consumidor tem de ser um **kernel de MAPA puro**, e o param um dos `params` dele.

use super::{DrivenParams, eligible};
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::gpu::{GpuKernel, KernelResolver};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use std::collections::BTreeSet;

/// Um kernel de MAPA puro: nenhuma lei do hospedeiro lê um param dele.
fn mapa_puro(kernels: &dyn KernelResolver, ty: NodeTypeId, k: &GpuKernel) -> bool {
    k.variant_by_param.is_none()
        && k.applicable.is_none()
        && k.count_law.is_none()
        && kernels.grid(ty).is_none()
        && kernels.stream_op(ty).is_none()
        && kernels.algorithm(ty).is_none()
        && kernels.state_select(ty).is_none()
        && kernels.derived_uniforms(ty).is_empty()
        && kernels.reduces(ty).is_empty()
}

/// O condutor de `node.param`, se a placa o pode cozer e entregar o número: a saída 0 dele, um
/// consumidor de mapa puro com o param no uniform, e o condutor ELEGÍVEL (com a mesma caminhada).
#[allow(clippy::too_many_arguments)] // a pergunta do `eligible`, com os mesmos conjuntos
pub(super) fn na_placa(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    node: NodeId,
    param: &str,
    claimed: &BTreeSet<NodeId>,
    forbidden: &BTreeSet<NodeId>,
    driven: &DrivenParams,
) -> Option<NodeId> {
    let (src, port) = *graph.param_sources(node)?.get(param)?;
    let ty = graph.node(node)?.type_id();
    let k = kernels.gpu_kernel(ty)?;
    let entra_no_uniform = k.params.contains(&param);
    (port == 0
        && entra_no_uniform
        && mapa_puro(kernels, ty, k)
        && eligible(graph, ops, kernels, src, claimed, forbidden, (driven, true))
        && !deixa_fronteira_temporal(graph, ops, kernels, src))
    .then_some(src)
}

/// ⛔ **A cadeia do condutor, encenada na placa, deixaria uma fronteira TEMPORAL?** Essa fronteira
/// faz o recuo (`plan_uniao`) expulsar TODO laço do plano — na `=139` o `Integrate` da galáxia
/// saía por causa do gatilho do envelope que acelera o redemoinho (doc 120 §8.3.3). Cozido na CPU,
/// o condutor dá o mesmo número por tique, sem fronteira — e, com estado, a marcha avança-o
/// ([`condutores_com_estado`]).
///
/// ⚠️ Pergunta ESTÁTICA (mapa vazio, sem a caminhada): a mesma resposta para o
/// [`device_driven_params`] e para o planeador, que é o que as faz concordar.
fn deixa_fronteira_temporal(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
    condutor: NodeId,
) -> bool {
    let (vazio, sem_mapa) = (BTreeSet::new(), DrivenParams::new());
    let mut encenados = BTreeSet::new();
    let mut pilha = vec![condutor];
    while let Some(n) = pilha.pop() {
        if encenados.contains(&n) {
            continue;
        }
        if !eligible(
            graph,
            ops,
            kernels,
            n,
            &encenados,
            &vazio,
            (&sem_mapa, true),
        ) {
            if super::uniao::boundary_is_static(graph, ops, n) {
                continue;
            }
            return true;
        }
        encenados.insert(n);
        let Some(op) = graph.node(n).and_then(|i| ops.resolve(i.type_id())) else {
            continue;
        };
        for porta in 0..op.manifest().inputs.len() {
            if let Some((src, _, false)) = graph.input_edge(n, porta) {
                pilha.push(src);
            }
        }
        for p in graph.param_sources(n).into_iter().flat_map(|f| f.keys()) {
            if let Some(src) = na_placa(graph, ops, kernels, n, p, &encenados, &vazio, &sem_mapa) {
                pilha.push(src);
            }
        }
    }
    false
}

/// **Os fios que a placa dirige** — o que a ponte NÃO coze na CPU (`valores_dirigidos`). Uma
/// pergunta estática, sem a caminhada: ela erra só para o lado seguro — se a caminhada recusar
/// um destes condutores (um recuo), o consumidor fica sem número e recua com ele, inteiro.
pub fn device_driven_params(
    graph: &Graph,
    ops: &dyn OpResolver,
    kernels: &dyn KernelResolver,
) -> BTreeSet<(NodeId, String)> {
    let vazio = BTreeSet::new();
    let sem_mapa = DrivenParams::new();
    graph
        .all_param_sources()
        .iter()
        .flat_map(|(n, ps)| ps.keys().map(move |p| (*n, p)))
        .filter(|(n, p)| na_placa(graph, ops, kernels, *n, p, &vazio, &vazio, &sem_mapa).is_some())
        .map(|(n, p)| (n, p.clone()))
        .collect()
}

/// ⛔ **[`GpuPlan::condutores_na_marcha`](super::GpuPlan::condutores_na_marcha)**: por estágio, a
/// fonte de cada param cujo número vem do mapa da CPU (não é condutor na placa) e cuja cadeia tem
/// estado. Uma vez cada, na ordem dos estágios.
pub(super) fn condutores_com_estado(
    graph: &Graph,
    ops: &dyn OpResolver,
    stages: &[super::GpuStage],
    device_drivers: &std::collections::BTreeMap<NodeId, Vec<(String, NodeId)>>,
) -> Vec<NodeId> {
    let mut fora: Vec<NodeId> = Vec::new();
    for s in stages {
        let na_placa = device_drivers.get(&s.node);
        for (p, (src, _)) in graph
            .param_sources(s.node)
            .into_iter()
            .flat_map(|f| f.iter())
        {
            if na_placa.is_some_and(|v| v.iter().any(|(q, _)| q == p)) || fora.contains(src) {
                continue;
            }
            if cadeia_com_estado(graph, ops, *src) {
                fora.push(*src);
            }
        }
    }
    fora
}

/// A cadeia de `n` (fios de entrada e condutores) tem um `pre` ou um efeito `Stateful`? Um nó sem
/// op responde que sim: na dúvida, a marcha avança-o.
fn cadeia_com_estado(graph: &Graph, ops: &dyn OpResolver, n: NodeId) -> bool {
    let mut vistos = BTreeSet::new();
    let mut pilha = vec![n];
    while let Some(k) = pilha.pop() {
        if !vistos.insert(k) {
            continue;
        }
        let Some(op) = graph.node(k).and_then(|i| ops.resolve(i.type_id())) else {
            return true;
        };
        let man = op.manifest();
        if man.effect == ph2d_nodegraph::effect::Effect::Stateful {
            return true;
        }
        for porta in 0..man.inputs.len() {
            match graph.input_edge(k, porta) {
                Some((_, _, true)) => return true,
                Some((src, _, false)) => pilha.push(src),
                None => {}
            }
        }
        pilha.extend(
            graph
                .param_sources(k)
                .into_iter()
                .flat_map(|f| f.values().map(|(s, _)| *s)),
        );
    }
    false
}
