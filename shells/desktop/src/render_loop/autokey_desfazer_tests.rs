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
