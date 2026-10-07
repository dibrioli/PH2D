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
        && eligible(graph, ops, kernels, src, claimed, forbidden, (driven, true)))
    .then_some(src)
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
