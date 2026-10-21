use super::*;
use crate::canal::{FieldKey, familia};

fn k(campo: u16) -> FieldKey {
    FieldKey::new(familia::FISICA, campo, 0, 0)
}

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_da_fisica_nao_renumera() {
    assert_eq!(familia::FISICA, 5);
    let fixos = [
        (TIPO, 1),
        (CCD, 2),
        (TRAVA_ROTACAO, 3),
        (TRAVA_X, 4),
        (TRAVA_Y, 5),
        (ESCALA_GRAVIDADE, 6),
        (DOMINANCIA, 7),
        (VEL_X, 10),
        (VEL_Y, 11),
        (VEL_ANGULAR, 12),
        (AMORT_LINEAR, 13),
        (AMORT_ANGULAR, 14),
        (AMORT_MODO, 15),
        (MASSA_MANUAL, 16),
        (MASSA, 17),
        (FORMA, 20),
        (RAIO, 21),
        (MEIO_X, 22),
        (MEIO_Y, 23),
        (CAPSULA, 24),
        (DESLOC_X, 25),
        (DESLOC_Y, 26),
        (DENSIDADE, 27),
        (RESTITUICAO, 28),
        (ATRITO, 29),
        (COMBINA_RESTITUICAO, 30),
        (COMBINA_ATRITO, 31),
        (CAMADA, 32),
        (SENSOR, 33),
        (UMA_VIA, 34),
        (SEM_PAREDE, 35),
        (ADERENCIA, 36),
        (ESTEIRA, 37),
        (FORCA_X, 40),
        (FORCA_Y, 41),
        (FORCA_NO_MUNDO, 42),
        (ZONA_ARRASTO, 43),
        (ZONA_DENSIDADE, 44),
        (ZONA_ARRASTO_FORMA, 45),
        (ZONA_TORQUE, 46),
        (ZONA_QUEDA, 47),
    ];
    assert_eq!(fixos.len(), CAMPOS.len(), "um campo sem número fixado");
    for (n, (atual, gravado)) in fixos.iter().enumerate() {
        assert_eq!(
            atual, gravado,
            "a constante nº {n} da física foi renumerada"
        );
    }
}

/// Interruptores e escolhas são discretos (Hold); os pares de eixos são UM grupo; nenhum campo
/// do vocabulário fica sem `info` nem sem nome.
#[test]
fn cada_campo_do_corpo_diz_o_que_e() {
    for campo in CAMPOS {
        assert!(k(campo).info().is_some(), "campo {campo} sem info");
        assert!(k(campo).membro().is_some(), "campo {campo} sem nome");
    }
    for discreto in [
        TIPO,
        FORMA,
        CCD,
        TRAVA_ROTACAO,
        TRAVA_X,
        TRAVA_Y,
        DOMINANCIA,
        AMORT_MODO,
        MASSA_MANUAL,
        COMBINA_RESTITUICAO,
        COMBINA_ATRITO,
        CAMADA,
        SENSOR,
        UMA_VIA,
        SEM_PAREDE,
        FORCA_NO_MUNDO,
    ] {
        assert!(k(discreto).info().is_some_and(|i| i.discreto), "{discreto}");
    }
    for continuo in [RAIO, DENSIDADE, ESCALA_GRAVIDADE, VEL_ANGULAR, ZONA_QUEDA] {
        assert!(
            k(continuo).info().is_some_and(|i| !i.discreto),
            "{continuo}"
        );
    }
    for (a, b) in [
        (VEL_X, VEL_Y),
        (MEIO_X, MEIO_Y),
        (DESLOC_X, DESLOC_Y),
        (FORCA_X, FORCA_Y),
    ] {
        assert!(k(a).mesmo_grupo(k(b)), "o par {a}/{b} é um grupo");
        assert_ne!(k(a).membro(), k(b).membro(), "o dope-sheet diz o eixo");
    }
    assert!(!k(RESTITUICAO).mesmo_grupo(k(ATRITO)), "duas linhas");
    assert!(k(8).info().is_none(), "buraco no vocabulário");
}
