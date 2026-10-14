//! **O alvo da trilha**: ela abre com o `StableId` do objecto Motion em edição (o dono dos nós
//! novos), e o palco que a toca coze só os sinks desse objecto — os outros objectos Motion da cena
//! continuam a ser do editor.

use super::tocar::{Modo, TocadorDoMotion};
use super::{FAMILIA, Recusa};
use crate::motion_state::MotionState;
use ph2d_macro::{MacroRecorder, RefRect};
use ph2d_nodegraph::graph::NodeId;

const A: u64 = 11;
const B: u64 = 22;

/// Dois objectos Motion, uma saída cada; o artista edita o `A`.
fn dois_objectos() -> (MotionState, NodeId, NodeId) {
    let mut m = super::estado_de_teste();
    let sa = m.doc.graph.add_node("motion.output");
    let sb = m.doc.graph.add_node("motion.output");
    m.doc.set_owner(sa, A);
    m.doc.set_owner(sb, B);
    m.objeto.editing = Some(A);
    (m, sa, sb)
}

fn gravador() -> MacroRecorder {
    MacroRecorder::new(RefRect { w: 1.0, h: 1.0 }, 0)
}

#[test]
fn a_trilha_abre_com_o_objecto_em_edicao() {
    let (mut m, _, _) = dois_objectos();
    let mut rec = gravador();
    let tr = m.gravacao_abre(&mut rec).expect("abre");
    assert!(m.gravacao_ativa());
    assert_eq!(
        m.gravacao_abre(&mut rec).map(|_| ()),
        Err(Recusa::JaGrava),
        "uma gravação de cada vez"
    );
    let doc = rec.finish();
    assert_eq!(doc.tracks[tr.0].family, FAMILIA);
    assert_eq!(doc.tracks[tr.0].target, A);

    let mut sem = super::estado_de_teste();
    let mut rec = gravador();
    let tr = sem.gravacao_abre(&mut rec).expect("abre");
    assert_eq!(rec.finish().tracks[tr.0].target, 0, "sem objecto em edição");
}

#[test]
fn o_palco_da_trilha_coze_so_o_objecto_dela() {
    let (mut m, sa, sb) = dois_objectos();
    let mut rec = gravador();
    let tr = m.gravacao_abre(&mut rec).expect("abre");
    let ph = ph2d_core::Playhead::new(1.0 / 60.0);
    crate::motion_bridge::quadro::o_quadro(
        &mut m,
        None,
        &ph,
        1.0 / 60.0,
        crate::motion_bridge::quadro::MaoDoEditor::default(),
    );
    assert_eq!(m.sinks, vec![sa, sb], "CONTROLO: o editor coze os dois");
    for c in m.gravacao_para() {
        rec.push(tr, 0, c).expect("tempo não decrescente");
    }
    let doc = rec.finish();
    let mut t = TocadorDoMotion::da_trilha(&doc.tracks[tr.0], Modo::Fiel).expect("o inicial lê-se");
    for ev in &doc.tracks[tr.0].events {
        t.aplica(&ev.payload, None).expect("o passo aplica-se");
    }
    assert_eq!(
        t.palco().sinks,
        vec![sa],
        "o palco coze só o objecto da trilha"
    );
}

/// Um nó que o artista CRIA durante a gravação é do objecto em edição — e no palco também: o palco
/// edita o objecto da trilha, senão o nó nasce sem dono e o filtro do dono larga-o.
#[cfg(feature = "panel-motion-graph")]
#[test]
fn o_no_criado_durante_a_gravacao_e_do_objecto_da_trilha_no_palco() {
    let (mut m, sa, _) = dois_objectos();
    let mut rec = gravador();
    let tr = m.gravacao_abre(&mut rec).expect("abre");
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    crate::motion_bridge::aplica_e_grava(
        &mut m,
        &mut ph,
        &mut ph2d_editor_core::ToastQueue::default(),
        &mut ph2d_editor_core::screens::layout::CenterSplit::None,
        vec![ph2d_panel_motion_graph::GraphIntent::AddNode {
            type_name: "motion.output",
            x: 0.0,
            y: 0.0,
        }],
    );
    let novo = *m.doc.graph.nodes().iter().map(|n| &n.id).max().expect("um nó");
    assert_eq!(m.doc.owner_of(novo), Some(A), "CONTROLO: no editor, o novo é do A");
    crate::motion_bridge::quadro::o_quadro(
        &mut m,
        None,
        &ph,
        1.0 / 60.0,
        crate::motion_bridge::quadro::MaoDoEditor::default(),
    );
    for c in m.gravacao_passos().into_iter().chain(m.gravacao_para()) {
        rec.push(tr, 0, c).expect("tempo não decrescente");
    }
    let doc = rec.finish();
    let mut t = TocadorDoMotion::da_trilha(&doc.tracks[tr.0], Modo::Fiel).expect("o inicial lê-se");
    for ev in &doc.tracks[tr.0].events {
        t.aplica(&ev.payload, None).expect("o passo aplica-se");
    }
    assert_eq!(t.palco().doc.owner_of(novo), Some(A));
    assert_eq!(t.palco().sinks, vec![sa, novo]);
}
