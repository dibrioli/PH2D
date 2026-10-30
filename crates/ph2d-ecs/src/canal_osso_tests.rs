use super::*;
use crate::canal::{FieldKey, familia};

fn k(campo: u16) -> FieldKey {
    FieldKey::new(familia::OSSO, campo, 0, 0)
}

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_do_osso_nao_renumera() {
    assert_eq!(familia::OSSO, 13);
    let fixos = [(COMPRIMENTO, 1), (FORCA, 2), (SEGMENTOS, 3)];
    assert_eq!(fixos.len(), CAMPOS.len(), "um campo sem número fixado");
    for (atual, gravado) in fixos {
        assert_eq!(atual, gravado, "o campo {gravado} do osso foi renumerado");
    }
}

/// Os segmentos são discretos; cada campo é o seu grupo; o osso é de um objecto, não da Cena.
#[test]
fn cada_campo_do_osso_diz_o_que_e() {
    for campo in CAMPOS {
        assert!(k(campo).info().is_some(), "campo {campo} sem info");
        assert!(k(campo).membro().is_some(), "campo {campo} sem nome");
        assert!(!k(campo).da_cena(), "campo {campo} é do osso");
    }
    assert!(k(SEGMENTOS).info().is_some_and(|i| i.discreto()));
    assert!(k(COMPRIMENTO).info().is_some_and(|i| !i.discreto()));
    assert!(!k(COMPRIMENTO).mesmo_grupo(k(FORCA)));
    assert!(k(4).info().is_none(), "buraco no vocabulário");
}
