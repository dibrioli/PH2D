//! ⭐ **As CERCAS DE FONTE da rota da placa** — o que a fonte de uma cadeia traz decide se o
//! dispositivo a pode desenhar (irmão por responsabilidade do `motion_bridge_gpu`, partido pelo
//! tecto de LOC no fecho da linha de 2026-09-24).
//!
//! Duas fontes, e cada uma pergunta de maneira diferente: o **vector vivo** (`source.shape`) pelo
//! TIPO do nó — a placa não tem rota para `geometry_id` —, e o **objecto** (`source.object`) pelo
//! CONTEÚDO do que a membrana publicou: um objecto que resolve para vector recusa, um todo no átlas
//! passa mesmo quando o sufixo na placa muda a contagem (doc 120 §8.2).

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::NodeTypeId;

/// Does this document bring in a live vector SHAPE (`source.shape`)? (ADR-0154)
///
/// A live vector is drawn by the vector pass (`geometry_id`), which the
/// GPU-resident cook has NO route for — so a document carrying one draws as
/// blank atlas quads the moment a GPU stage runs (`source → duplicator → … `
/// is Hybrid). Recuse it to the CPU render (which draws it) at PLAN time, so the
/// CPU pump owns the tick from scratch and no sequential prefix is marched
/// twice. The signal is a registry flag `source.shape` sets
/// (`is_live_vector_source`), not a node-name match.
///
/// ⚠️ An OBJECT source (`source.object`, `texture_id`) is NOT here: the GPU cook
/// now draws it (the lowering carries the id, the renderer binds the texture per
/// run). It recuses only when its GPU suffix reorders / changes count — see
/// [`graph_has_object_source`] + [`ph2d_gpu_cook::GpuPlan::suffix_changes_count`].
pub(crate) fn graph_has_live_vector_source(graph: &Graph, reg: &NodeRegistry) -> bool {
    graph
        .nodes()
        .iter()
        .any(|n| reg.is_live_vector_source(NodeTypeId::of(n.type_name.as_str())))
}

/// Does this document bring in an engine OBJECT (`source.object`, `texture_id`)?
/// Read together with [`ph2d_gpu_cook::GpuPlan::suffix_changes_count`] for the
/// count-changing cerca: an object graph whose GPU suffix reorders / changes
/// count would mis-bind the texture-run partition (the boundary `texture_id`
/// column no longer aligns with the device buffer), so it recuses to the CPU
/// render. The signal is the registry flag `source.object` sets.
pub(crate) fn graph_has_object_source(graph: &Graph, reg: &NodeRegistry) -> bool {
    graph
        .nodes()
        .iter()
        .any(|n| reg.is_object_source(NodeTypeId::of(n.type_name.as_str())))
}

/// Does the cook's external table carry a LIVE VECTOR (`geometry_id > 0`)? — the
/// CONTENT-aware half of the object recusal (ADR-0154 reused for objects).
///
/// Whether a `source.object` resolves to a vector depends on what the artist NAMED
/// (a sprite → `texture_id`, a vector → `geometry_id`), which the node-type registry
/// cannot see. The membrane publishes the externals BEFORE the cook runs (post-drain,
/// pre-cook), so this per-frame scan answers the real question and lets a pure-sprite
/// object graph stay on the GPU stamp while a vector-bearing one recuses. Cheap: a
/// handful of externals, one scalar-column probe each.
pub(crate) fn cook_publishes_live_geometry(cook: &ph2d_nodegraph::cook::Cook) -> bool {
    use ph2d_nodegraph::attr::Column;
    cook.externals().values().any(|e| {
        matches!(e.value.get("geometry_id"), Some(Column::Scalar(v)) if v.iter().any(|&g| g > 0.5))
    })
}

/// **Todo objecto publicado vive no ÁTLAS partilhado?** (`texture_id` todo `0`, ou nenhum publicado)
/// — a metade de CONTEÚDO da cerca da contagem (doc 120 §8.2).
///
/// ⚠️ A comparação é **`v as u32 == 0`, à letra do `texture_runs_from_boundary`** (e do
/// `scalar_at(..) as u32` da CPU): é essa a pergunta que decide se a partição é vazia, e uma
/// segunda redacção dela (um `v < 0.5`, por exemplo) discordaria num `0,7` que as duas lêem
/// diferente. Sem nenhum objecto publicado não há textura a partir — `true`.
pub(crate) fn cook_publishes_only_atlas_objects(cook: &ph2d_nodegraph::cook::Cook) -> bool {
    use ph2d_nodegraph::attr::Column;
    cook.externals()
        .values()
        .all(|e| match e.value.get("texture_id") {
            Some(Column::Scalar(v)) => v.iter().all(|&t| t as u32 == 0),
            _ => true,
        })
}
