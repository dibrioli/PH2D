//! ⭐⭐⭐ **ARRUMAR O DOCUMENTO — a porta única, e a única que sabe medir um cartão.**
//!
//! A arrumação em camadas vive no `ph2d-nodegraph` e a versão consciente de grupos no
//! `ph2d-motion-doc`; desde 2026-09-20 nenhuma das duas escolhe o ESPAÇAMENTO sozinha, porque
//! ele depende de quanto cada cartão desenha — e isso é o nome dele (texto medido) e a contagem
//! de fileiras dele (registo + params do cartão). Esta é a porta em que essas duas pontas se
//! encontram.
//!
//! ⚠️ **Ela existe porque há DOIS chamadores** — a tecla de arrumar do painel do grafo
//! (`GraphIntent::ArrangeLayout`) e as cenas de smoke que se arrumam ao nascer
//! ([`crate::smoke_layout`]) — e *duas chamadas à mesma lei divergem no dia em que uma delas
//! ganhar um argumento*. O da direita já tinha divergido antes: ele arrumava sem medir.
//!
//! ⛔ **E o ramo sem painel não é um esquecimento:** a `ph2d-app-motion` compila sem a feature
//! `panel-motion-graph`, e ali não há medidor de texto nenhum — o que se pode fazer é o cartão
//! histórico, que é exactamente a arrumação que este repo teve até hoje.

use crate::motion_state::MotionState;
use ph2d_motion_doc::subgraph::member_nodes_deep;
use ph2d_nodegraph::graph::NodeId;

/// Arruma a raiz e o interior de cada grupo, com cada cartão medido.
pub(crate) fn arrumar(motion: &mut MotionState) {
    #[cfg(feature = "panel-motion-graph")]
    {
        crate::motion_bridge::medida::arrumar(motion);
    }
    #[cfg(not(feature = "panel-motion-graph"))]
    {
        ph2d_motion_doc::layout::arrange(
            &mut motion.doc,
            &ph2d_motion_doc::layout::CartaoDeFabrica,
        );
    }
}

/// ⭐ **Arrumar ao ABRIR** (pedido do dono, 15/10: *«dar o auto arrange… só não se o usuário mover
/// os nós e salvar»*): a MESMA [`arrumar`], e depois os nós dos donos arrumados à mão
/// (`MotionDoc::arrumados_a_mao`) voltam exactamente ao lugar gravado — e os cartões de grupo
/// cujos nós são todos deles também. Abrir não é edição: sem undo.
pub(crate) fn arrumar_ao_abrir(m: &mut MotionState) {
    let doc = &m.doc;
    let a_mao = |n: &NodeId| {
        doc.owner_of(*n)
            .is_some_and(|o| doc.arrumados_a_mao.contains(&o))
    };
    let nos: Vec<_> = doc
        .owners
        .keys()
        .filter(|n| a_mao(n))
        .filter_map(|n| doc.graph.pos(*n).map(|p| (*n, p)))
        .collect();
    let cartoes: Vec<_> = doc
        .subgraphs
        .iter()
        .filter(|s| {
            let deep = member_nodes_deep(&doc.subgraphs, &doc.members, s.id);
            !deep.is_empty() && deep.iter().all(a_mao)
        })
        .map(|s| (s.id, s.x, s.y))
        .collect();
    arrumar(m);
    for (n, p) in nos {
        m.doc.graph.set_pos(n, p);
    }
    for (id, x, y) in cartoes {
        if let Some(s) = m.doc.subgraphs.iter_mut().find(|s| s.id == id) {
            (s.x, s.y) = (x, y);
        }
    }
}

#[cfg(all(test, feature = "panel-motion-graph"))]
#[path = "motion_arrumar_tests.rs"]
mod tests;
