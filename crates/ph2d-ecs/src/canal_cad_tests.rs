use super::*;
use crate::canal::familia;

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_do_cad_nao_renumera() {
    assert_eq!(familia::CAD, 9);
    assert_eq!(MEDIDA, 1);
}

/// O id da regra atravessa a chave inteiro, cada regra tem a sua curva, e a medida interpola.
#[test]
fn a_chave_leva_o_id_da_regra_inteiro() {
    for id in [1, 0xFFFF, 0x1_0000, 0xDEAD_BEEF, u32::MAX] {
        assert_eq!(regra_de(chave(MEDIDA, id)), id);
    }
    assert!(!chave(MEDIDA, 3).mesmo_grupo(chave(MEDIDA, 4)));
    assert!(chave(MEDIDA, 3).info().is_some_and(|i| !i.discreto()));
    assert!(chave(MEDIDA, 3).membro().is_some());
    assert!(chave(99, 3).info().is_none());
}
