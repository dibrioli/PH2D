//! ⭐ **O editor de nós mostra o grafo do objecto em EDIÇÃO** (doc ObjectsAndComponents 01 §13, F5)
//! — e só ele: os nós, os fios, as molduras e os cartões de grupo dos outros objectos Motion ficam
//! fora da vista. É o mesmo corte que o nível do subgrafo faz, um degrau acima: a vista dobra-se
//! primeiro pelo dono e depois pelo nível (`fold`).
//!
//! Um nó sem dono (o instante entre ser criado e a manutenção o adoptar) é mostrado: escondê-lo
//! faria um nó acabado de criar desaparecer.

use ph2d_motion_doc::subgraph as model;
use ph2d_nodegraph::graph::NodeId;
use ph2d_panel_motion_graph::GraphViewSnapshot;

use crate::motion_state::MotionState;

/// O nó `n` é do objecto em edição (ou ainda não tem dono)?
#[must_use]
pub fn mostra_no(motion: &MotionState, n: NodeId) -> bool {
    match (motion.objeto.editing, motion.doc.owner_of(n)) {
        (Some(e), Some(o)) => o == e,
        _ => true,
    }
}

/// O cartão do grupo `sid` é do objecto em edição? (Um grupo é de quem tem os nós dele.)
#[must_use]
pub fn mostra_grupo(motion: &MotionState, sid: u32) -> bool {
    model::member_nodes_deep(&motion.doc.subgraphs, &motion.doc.members, sid)
        .first()
        .is_none_or(|n| mostra_no(motion, *n))
}

/// Corta o snapshot ao grafo do objecto em edição.
pub fn so_do_dono(motion: &MotionState, snap: &mut GraphViewSnapshot) {
    let Some(editing) = motion.objeto.editing else {
        return;
    };
    snap.nodes.retain(|n| mostra_no(motion, NodeId(n.id)));
    snap.edges
        .retain(|e| mostra_no(motion, NodeId(e.from_node)) && mostra_no(motion, NodeId(e.to_node)));
    snap.backdrops.retain(|b| {
        motion
            .doc
            .backdrop_owners
            .get(&b.id)
            .is_none_or(|o| *o == editing)
    });
}
