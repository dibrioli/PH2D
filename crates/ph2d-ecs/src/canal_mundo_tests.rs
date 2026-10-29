use super::*;
use crate::canal::{FieldKey, familia};

fn k(campo: u16) -> FieldKey {
    FieldKey::new(familia::MUNDO, campo, 0, 0)
}

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_do_mundo_nao_renumera() {
    assert_eq!(familia::MUNDO, 11);
    let fixos = [
        (GRAVIDADE_X, 1),
        (GRAVIDADE_Y, 2),
        (SUBPASSOS, 3),
        (ITERACOES, 4),
        (CONTACTO_HZ, 5),
        (AMORT_LINEAR, 6),
        (AMORT_ANGULAR, 7),
        (AR, 8),
        (SONO_VELOCIDADE, 9),
        (SONO_DEMORA, 10),
        (ADORMECE, 11),
    ];
    assert_eq!(fixos.len(), CAMPOS.len(), "um campo sem número fixado");
    for (atual, gravado) in fixos {
        assert_eq!(atual, gravado, "o campo {gravado} do mundo foi renumerado");
    }
}

/// Inteiros e o interruptor são discretos; a gravidade é UM grupo de dois eixos; todo campo é da
/// Cena; nenhum fica sem `info` nem nome.
#[test]
fn cada_campo_do_mundo_diz_o_que_e() {
    for campo in CAMPOS {
        assert!(k(campo).info().is_some(), "campo {campo} sem info");
        assert!(k(campo).membro().is_some(), "campo {campo} sem nome");
        assert!(k(campo).da_cena(), "campo {campo} é da Cena");
    }
    for discreto in [SUBPASSOS, ITERACOES, ADORMECE] {
        assert!(
            k(discreto).info().is_some_and(|i| i.discreto()),
            "{discreto}"
        );
    }
    for continuo in [GRAVIDADE_X, CONTACTO_HZ, AR, SONO_DEMORA] {
        assert!(
            k(continuo).info().is_some_and(|i| !i.discreto()),
            "{continuo}"
        );
    }
    assert!(k(GRAVIDADE_X).mesmo_grupo(k(GRAVIDADE_Y)));
    assert_ne!(k(GRAVIDADE_X).membro(), k(GRAVIDADE_Y).membro());
    assert!(!k(AMORT_LINEAR).mesmo_grupo(k(AMORT_ANGULAR)));
    assert!(k(12).info().is_none(), "buraco no vocabulário");
    assert!(
        !FieldKey::new(familia::FISICA, 6, 0, 0).da_cena(),
        "o corpo é de um objecto"
    );
}
