use super::*;
use crate::canal::familia;

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_do_cad_nao_renumera() {
    assert_eq!(familia::CAD, 9);
    assert_eq!(MEDIDA, 1);
    assert_eq!(ANGULO, 2);
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

/// A unidade de EXIBIÇÃO de um campo: o comprimento do CAD e as coordenadas do vetor são mundo; o ângulo do CAD
/// são graus; o resto é número nu.
#[test]
fn each_field_says_its_display_unit() {
    use crate::canal::{Unidade, unidade};
    assert_eq!(unidade(chave(MEDIDA, 3)), Unidade::Comprimento);
    assert_eq!(unidade(chave(ANGULO, 3)), Unidade::Angulo);
    assert!(
        chave(ANGULO, 3)
            .info()
            .is_some_and(|i| i.rotulo == "canal.cad.angulo")
    );
    let vetor = |campo| FieldKey::new(familia::VETOR, campo, 0, 0);
    assert_eq!(
        unidade(vetor(crate::canal_vetor::ANCORA_X)),
        Unidade::Comprimento
    );
    assert_eq!(
        unidade(vetor(crate::canal_vetor::FECHADO)),
        Unidade::Nenhuma
    );
    assert_eq!(unidade(chave(99, 3)), Unidade::Nenhuma);
}
