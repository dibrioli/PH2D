//! **Um fio solto no CORPO de um nó que não o aceita** (smoke do dono da `=141`, 13/10: *«o aviso
//! não some só e o modal de nós se abre»*) — a frase que diz PORQUÊ, e a única porta para a frase
//! da corrente por-elemento.
//!
//! O painel não tem toasts: ele emite `GraphIntent::DropRefused` quando a largada caiu num corpo
//! sem entrada livre compatível e sem fileira no menu, e esta metade decide a frase. ⛔ Antes, a
//! largada caía no ramo do canvas VAZIO e abria a paleta — e a recusa com a cura (o
//! `DriveParam` de uma corrente) era inalcançável por um gesto real.

use super::{MotionState, fold, subgraph};
use ph2d_nodegraph::graph::NodeId;

/// **A recusa da corrente por-elemento, com a CURA** (estudo do Mini Cavalry, doc 99 §10e: ele
/// converte em silêncio e inventa um resultado que ninguém autorou; aqui o artista escolhe, e o nó
/// que converte fica à vista e ajustável). Dita pelo `DriveParam` e pela largada no corpo.
pub(super) fn aviso_da_corrente(motion: &MotionState, from: (NodeId, u16)) -> String {
    let cura = subgraph::converter_from(motion, from).map_or_else(String::new, |ty| {
        ph2d_i18n::tr_with(
            "app.motion.motion_bridge_subgraph.insert_a_to_read_one_number_from_it",
            &[("ty", &subgraph::nome_no_menu(motion, ty))],
        )
    });
    ph2d_i18n::tr_with(
        "app.motion.motion_bridge_subgraph.can_t_drive_that_output_is_a_per_element_stream",
        &[("cura", &cura)],
    )
}

/// A frase da largada recusada sobre o corpo `to_view` (um nó ou um cartão de grupo). Uma fonte
/// que não conduz um VALOR sobre um corpo com params conduzíveis é a corrente por-elemento — o
/// artista queria um param, e a cura é o nó que lê um número; o resto não tem onde aterrar.
pub(super) fn no_corpo(motion: &MotionState, from: (NodeId, u16), to_view: u32) -> String {
    let (nome, conduz) = match subgraph::target(to_view) {
        subgraph::Target::Node(n) => (
            nome_do_cartao(motion, n),
            fold::tem_param_conduzivel(motion, n),
        ),
        subgraph::Target::Card(sid) => {
            let titulo = ph2d_motion_doc::subgraph::find(&motion.doc.subgraphs, sid)
                .map(|s| s.title.clone())
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| fold::DEFAULT_TITLE.tr().to_string());
            let conduz = ph2d_motion_doc::subgraph::member_nodes_deep(
                &motion.doc.subgraphs,
                &motion.doc.members,
                sid,
            )
            .into_iter()
            .any(|n| fold::tem_param_conduzivel(motion, n));
            (titulo, conduz)
        }
    };
    if conduz && !subgraph::source_can_drive(motion, from) {
        return aviso_da_corrente(motion, from);
    }
    ph2d_i18n::tr_with(
        "app.motion.motion_bridge_recusa.nothing_on_takes_this_wire",
        &[("no", &nome)],
    )
}

/// O nome que o CARTÃO mostra — o do artista, ou o do tipo no menu (`card_title`, a porta do
/// retrato do painel).
fn nome_do_cartao(motion: &MotionState, n: NodeId) -> String {
    let g = &motion.doc.graph;
    let Some(inst) = g.node(n) else {
        return String::new();
    };
    ph2d_node_registry::card_title(
        g.label(n),
        motion
            .registry
            .ui_manifest(inst.type_id())
            .map(|u| ph2d_i18n::tr(u.display_key)),
        &inst.type_name,
    )
}
