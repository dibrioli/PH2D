//! ⛔ **O Ctrl+Z não é a mão** (smoke do dono, F41): o desfazer devolve um campo SEM curva ao valor de
//! antes; contra a base do quadro anterior isso lia-se «mudou» e o Auto Key recriava a chave que o
//! mesmo Ctrl+Z tirara — e a chave nova esvaziava o refazer.

use super::test_helpers::{E, frame, pose};
use super::*;
use ph2d_ecs::FieldKey;

fn quadro_depois_do_desfazer(esquece: bool) -> bool {
    let k = FieldKey::new(ph2d_ecs::canal::familia::VETOR, 1, 0, 0);
    let mut st = TimelineState::new();
    st.keys_mode = true;
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.seek(0.5);
    ph.pause();
    let mut ak = AutokeyState::default();
    let amostra = [(E, pose(&[]))];
    ak.campos.insert(E, vec![(k, 2.0)]);
    frame(&mut st, &ph, &amostra, false, true, &mut ak);
    assert!(st.doc.binding_for(E, PropKind::Field(k)).is_none());
    // O desfazer devolveu o campo a 1,0.
    ak.campos.insert(E, vec![(k, 1.0)]);
    if esquece {
        ak.esquece_o_quadro();
    }
    frame(&mut st, &ph, &amostra, false, true, &mut ak);
    st.doc.binding_for(E, PropKind::Field(k)).is_some()
}

/// O arrasto com Auto Key e o `Esc`: sem curva nasce nada, com curva ela fica como estava; e a mão seguinte grava.
#[test]
fn esc_a_meio_do_arrasto_devolve_chaves_e_base() {
    let k = FieldKey::new(ph2d_ecs::canal::familia::FISICA, 1, 0, 0);
    let mut st = TimelineState::new();
    st.keys_mode = true;
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.seek(0.5);
    ph.pause();
    let mut ak = AutokeyState::default();
    let amostra = [(E, pose(&[]))];
    ak.campos.insert(E, vec![(k, 1.0)]);
    frame(&mut st, &ph, &amostra, false, true, &mut ak);
    let antes = st.doc.clone();
    for v in [1.2, 1.5] {
        ak.campos.insert(E, vec![(k, v)]);
        frame(&mut st, &ph, &amostra, true, true, &mut ak);
    }
    assert!(st.doc != antes, "controlo: o arrasto grava");
    ak.desiste_do_gesto(&mut st);
    ak.campos.insert(E, vec![(k, 1.0)]);
    for _ in 0..3 {
        frame(&mut st, &ph, &amostra, false, true, &mut ak);
        assert!(st.doc == antes, "o Esc devolve as chaves");
        assert!(!st.history.can_undo() && !st.history.is_open(), "sem passo");
    }
    ak.campos.insert(E, vec![(k, 1.7)]);
    frame(&mut st, &ph, &amostra, true, true, &mut ak);
    assert!(st.doc != antes, "a mão seguinte grava");
}

#[test]
fn esc_a_meio_do_arrasto_devolve_a_curva_que_ja_havia() {
    let (mut st, ph) = super::test_helpers::state_with_tx_track();
    st.keys_mode = true;
    let mut ak = AutokeyState::default();
    let curva = [(E, pose(&[(super::test_helpers::TX, 5.0)]))];
    let fora = [(E, pose(&[(super::test_helpers::TX, 9.0)]))];
    frame(&mut st, &ph, &curva, false, true, &mut ak);
    let antes = st.doc.clone();
    frame(&mut st, &ph, &fora, true, true, &mut ak);
    assert!(st.doc != antes, "controlo: o arrasto grava");
    ak.desiste_do_gesto(&mut st);
    frame(&mut st, &ph, &curva, false, true, &mut ak);
    assert!(st.doc == antes, "a curva volta como estava");
    assert!(ak.displaced.is_empty());
}

#[test]
fn o_quadro_depois_do_desfazer_nao_recria_a_chave() {
    assert!(
        quadro_depois_do_desfazer(false),
        "controlo: sem esquecer, a mudança cunha a chave"
    );
    assert!(
        !quadro_depois_do_desfazer(true),
        "o Ctrl+Z não recria a chave que tirou"
    );
}

/// ⛔ Ctrl+Z e Stop do jogo: no quadro do restauro o mundo está fora da curva, o apply ainda não o
/// alcançou. Não é a mão: nem chave (armado) nem pino (desarmado), e o quadro seguinte volta a valer.
#[test]
fn o_quadro_do_restauro_nao_cunha_nem_prende() {
    for armado in [true, false] {
        let (mut st, ph) = super::test_helpers::state_with_tx_track();
        st.keys_mode = true;
        let mut ak = AutokeyState::default();
        let curva = [(E, pose(&[(super::test_helpers::TX, 5.0)]))];
        let fora = [(E, pose(&[(super::test_helpers::TX, 9.0)]))];
        frame(&mut st, &ph, &curva, false, armado, &mut ak);
        let antes = st.doc.clone();
        ak.esquece_o_quadro();
        frame(&mut st, &ph, &fora, false, armado, &mut ak);
        assert!(st.doc == antes, "armado={armado}: cunhou no restauro");
        assert!(ak.displaced.is_empty(), "armado={armado}: prendeu a pose");
        frame(&mut st, &ph, &fora, false, armado, &mut ak);
        assert!(
            st.doc != antes || !ak.displaced.is_empty(),
            "armado={armado}: o aviso tem de durar um quadro só"
        );
    }
}
