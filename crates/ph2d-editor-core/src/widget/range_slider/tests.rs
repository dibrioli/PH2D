//! O gate da barra de duas pegas: a pega que o premir agarra, o arrasto que nunca as cruza, e a
//! pintura que põe cada pega onde o valor diz.

use super::*;

const TRILHA: Rect = Rect {
    x: 100.0,
    y: 10.0,
    w: 200.0,
    h: 8.0,
};

#[test]
fn o_premir_agarra_a_pega_mais_perto() {
    assert_eq!(range_handle_at(TRILHA, 110.0, 0.1, 0.9), RangeHandle::Lo);
    assert_eq!(range_handle_at(TRILHA, 270.0, 0.1, 0.9), RangeHandle::Hi);
    assert_eq!(range_handle_at(TRILHA, 180.0, 0.1, 0.9), RangeHandle::Lo);
    assert_eq!(range_handle_at(TRILHA, 220.0, 0.1, 0.9), RangeHandle::Hi);
}

#[test]
fn juntas_separam_se_para_o_lado_do_ponteiro() {
    assert_eq!(range_handle_at(TRILHA, 150.0, 0.5, 0.5), RangeHandle::Lo);
    assert_eq!(range_handle_at(TRILHA, 250.0, 0.5, 0.5), RangeHandle::Hi);
    // Juntas no fim: só a de baixo se pode mexer, e é ela que o premir agarra.
    assert_eq!(range_handle_at(TRILHA, 300.0, 1.0, 1.0), RangeHandle::Lo);
}

#[test]
fn arrastar_segue_o_ponteiro_e_prende_as_pontas() {
    assert_eq!(
        range_drag(TRILHA, 150.0, RangeHandle::Lo, 0.1, 0.9),
        (0.25, 0.9)
    );
    assert_eq!(
        range_drag(TRILHA, 250.0, RangeHandle::Hi, 0.1, 0.9),
        (0.1, 0.75)
    );
    assert_eq!(
        range_drag(TRILHA, -50.0, RangeHandle::Lo, 0.1, 0.9),
        (0.0, 0.9)
    );
    assert_eq!(
        range_drag(TRILHA, 900.0, RangeHandle::Hi, 0.1, 0.9),
        (0.1, 1.0)
    );
}

#[test]
fn as_pegas_nunca_se_cruzam_encostam() {
    assert_eq!(
        range_drag(TRILHA, 290.0, RangeHandle::Lo, 0.1, 0.6),
        (0.6, 0.6)
    );
    assert_eq!(
        range_drag(TRILHA, 110.0, RangeHandle::Hi, 0.4, 0.9),
        (0.4, 0.4)
    );
}

#[test]
fn a_pega_pinta_se_no_valor_dela_e_dentro_da_trilha() {
    let r = range_handle_rect(TRILHA, 0.3);
    let centro = r.x + r.w * 0.5;
    assert!((centro - (TRILHA.x + TRILHA.w * 0.3)).abs() < 1e-4);
    for v in [0.0, 0.3, 1.0] {
        let r = range_handle_rect(TRILHA, v);
        assert!(r.w > 0.0 && r.h == TRILHA.h);
        assert!(
            r.x >= TRILHA.x && r.x + r.w <= TRILHA.x + TRILHA.w + 1e-4,
            "{v}: {r:?}"
        );
    }
    assert_eq!(
        range_handle_rect(TRILHA, 0.0).x,
        TRILHA.x,
        "a do zero encosta à ponta"
    );
}

#[test]
fn trilha_sem_largura_ou_ponteiro_estragado_le_zero() {
    let nula = Rect::new(0.0, 0.0, 0.0, 8.0);
    assert_eq!(range_value_at(nula, 10.0), 0.0);
    assert_eq!(range_value_at(TRILHA, f32::NAN), 0.0);
}
