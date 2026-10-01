//! ⭐⭐ **As notas de uma secção do Vector pintam-se no FIM dela** (2026-10-01, ordem do dono: *«a
//! possibilidade de criar notas deve existir em quaisquer painéis de qualquer tipo»*). O Vector tem
//! o laço de secções PRÓPRIO (`paint_body_plan`), logo é um quarto leitor da porta partilhada
//! (`notes_chrome::fecha_seccao`) — e o que este gate mede é que ele a chama.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::VectorPanel;
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 600.0,
    h: 2400.0,
};

fn a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = rects
        .iter()
        .filter(|(id, _)| ph2d_panel_vector::ids::VECTOR_SECTIONS.contains(id))
        .copied()
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v.dedup_by_key(|(id, _)| *id);
    v
}

/// ⭐⭐ **A nota fica entre o título da secção dela e o da seguinte.** *Mutação: o laço do Vector
/// sem o `cromo` ⇒ a nota cai no fim do corpo, abaixo de todas as secções.*
#[test]
fn a_nota_pinta_se_no_fim_da_sua_seccao_no_vector() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let antes = a_vista(&host.paint::<VectorPanel>(&mut st, VIEWPORT));
    assert!(
        antes.len() >= 2,
        "fixtura: duas secções à vista ({antes:?})"
    );
    let (alvo, _) = antes[0];
    let p = core_ids::VECTOR_PANEL;
    assert_eq!(host.add_note(p, Some(alvo)), Some(0));
    let rects = host.paint::<VectorPanel>(&mut st, VIEWPORT);
    let depois = a_vista(&rects);
    let cab = depois
        .iter()
        .find(|(id, _)| *id == alvo)
        .expect("a secção")
        .1;
    let seguinte = depois
        .iter()
        .find(|(_, r)| r.y > cab.y)
        .expect("uma secção a seguir")
        .1;
    let nota = rects
        .iter()
        .rev()
        .find(|(id, r)| *id == core_ids::note_ids(p).slot[0] && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("a nota foi pintada");
    assert!(
        nota.y > cab.y && nota.y + nota.h <= seguinte.y,
        "a nota ({nota:?}) não ficou entre a secção ({cab:?}) e a seguinte ({seguinte:?})"
    );
}
