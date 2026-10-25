use super::*;
use crate::canal::familia;

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_das_constraints_nao_renumera() {
    assert_eq!(familia::CONSTRAINT, 8);
    assert_eq!((FORCA, LIGADA), (1, 2));
    assert_eq!((LIMITE_MIN, LIMITE_MAX, DISTANCIA, DESVIO), (3, 4, 5, 6));
    assert_eq!((MODO, ESPACO, MISTURA), (7, 8, 9));
}

/// O id do item atravessa a chave inteiro (32 bits), e cada item tem a sua curva.
#[test]
fn a_chave_leva_o_id_do_item_inteiro() {
    for id in [0, 1, 0xFFFF, 0x1_0000, 0xDEAD_BEEF, u32::MAX] {
        assert_eq!(item_de(chave(FORCA, id)), id);
    }
    assert!(!chave(FORCA, 3).mesmo_grupo(chave(FORCA, 4)));
    assert!(chave(FORCA, 3).info().is_some_and(|i| !i.discreto()));
    assert!(chave(LIGADA, 3).info().is_some_and(|i| i.discreto()));
    assert!(chave(99, 3).info().is_none());
    for campo in CAMPOS {
        assert!(info(campo).is_some(), "o campo {campo} não diz o que é");
    }
    assert!(chave(MODO, 3).info().is_some_and(|i| i.discreto()));
}
