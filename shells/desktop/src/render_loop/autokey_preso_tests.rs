//! ⭐ **C3 — o Auto Key de um campo PRESO** (`docs/CartoesUI/09`): A é editado por um cartão solto com B
//! escolhido. Com A fora da população, a mudança perdia-se sem chave e a curva repunha o valor.

use super::test_helpers::frame;
use super::*;
use ph2d_editor_core::ids::INSP_PANEL;
use ph2d_editor_core::{NodeId, screens::hero::HeroScreen};

const A: u64 = 7;
const B: u64 = 9;

fn com_a_preso_e_b_escolhido() -> HeroScreen {
    const SEC: NodeId = NodeId(0xC3);
    let mut h = HeroScreen::new(NodeId(1));
    h.store.detach_card(SEC, INSP_PANEL, (0.0, 0.0), 100.0);
    h.store
        .set_card_subject(INSP_PANEL, Some((A, "A".to_owned())));
    h.store.toggle_loose_pin(SEC);
    h.gizmo.replace_selection(Some(B));
    h
}

/// *Mutação: os pinos fora da `populacao` ⇒ A não entra, nenhuma chave nasce.*
#[test]
fn an_edit_on_a_pinned_card_keys_its_object_and_one_undo_takes_it_back() {
    let (sim, drive) = (
        ph2d_ecs::SimWorld::new(),
        ph2d_preview_drive::PreviewDrive::default(),
    );
    let h = com_a_preso_e_b_escolhido();
    let pop = populacao(
        &h,
        &ph2d_app_skeleton::state::SkeletonState::default(),
        &sim,
        &drive,
    );
    assert!(pop.contains(&A) && pop.contains(&B), "{pop:?}");

    let mut st = TimelineState::new();
    st.keys_mode = true;
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    let mut ak = AutokeyState::default();
    let pose = |v: f32| {
        let mut p: PoseSample = [None; PropKind::AUTOKEYED.len()];
        p[super::test_helpers::ROT] = Some(v);
        p
    };
    let quadro = |v: f32, st: &mut TimelineState, ak: &mut AutokeyState| {
        let amostra: Vec<(u64, PoseSample)> = pop
            .iter()
            .map(|&e| (e, pose(if e == A { v } else { 0.0 })))
            .collect();
        frame(st, &ph, &amostra, false, true, ak);
    };
    quadro(0.0, &mut st, &mut ak);
    assert!(st.doc.bindings().is_empty(), "o quadro parado gravou");
    quadro(3.0, &mut st, &mut ak);
    let b = st.doc.bindings();
    assert!(
        !b.is_empty() && b.iter().all(|x| x.entity == A),
        "a chave não é de A"
    );
    assert!(st.undo(), "sem passo de desfazer");
    assert!(
        st.doc.bindings().is_empty(),
        "o desfazer não tirou a chave de A"
    );
}
