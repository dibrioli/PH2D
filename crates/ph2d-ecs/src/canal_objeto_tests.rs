use super::*;
use crate::canal::{FieldKey, familia};

fn k(campo: u16, j: u16) -> FieldKey {
    FieldKey::new(familia::OBJETO, campo, 0, j)
}

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_do_objeto_nao_renumera() {
    assert_eq!(familia::OBJETO, 2);
    let fixos = [
        (SKEW_X, 1),
        (SKEW_Y, 2),
        (EMISSAO, 10),
        (OPACIDADE, 11),
        (TINTA, 20),
        (AUTO_TINTA, 30),
        (CANTOS, 40),
        (SILHUETA, 50),
    ];
    for (n, (atual, gravado)) in fixos.iter().enumerate() {
        assert_eq!(
            atual, gravado,
            "a constante nº {n} do objecto foi renumerada"
        );
    }
}

/// Cada campo tem o rótulo e o grupo da linha do Inspector que o mostra.
#[test]
fn cada_campo_do_objeto_diz_o_que_e() {
    let skew = k(SKEW_X, 0).info().expect("skew");
    assert!(!skew.discreto && !skew.por_i);
    assert!(
        k(SKEW_X, 0).mesmo_grupo(k(SKEW_Y, 0)),
        "X e Y são uma linha"
    );
    for off in 0..3 {
        assert!(k(TINTA, 0).mesmo_grupo(k(TINTA + off, 0)));
    }
    assert!(
        k(TINTA + 3, 0).info().is_none(),
        "o alfa do tint é o `Opacity`"
    );
    for off in 0..4 {
        assert!(k(AUTO_TINTA, 0).mesmo_grupo(k(AUTO_TINTA + off, 0)));
        for canto in 0..N_CANTOS {
            assert!(
                k(CANTOS, 0).mesmo_grupo(k(CANTOS + off, canto)),
                "o degradê inteiro é um grupo"
            );
        }
    }
    assert!(k(SILHUETA, 0).info().is_some_and(|i| i.discreto));
    assert!(!k(EMISSAO, 0).mesmo_grupo(k(OPACIDADE, 0)));
    assert!(!k(TINTA, 0).mesmo_grupo(k(AUTO_TINTA, 0)));
    assert_eq!(k(TINTA + 1, 0).membro(), Some("canal.m.g"));
    assert_eq!(k(SKEW_Y, 0).membro(), Some("canal.m.y"));
    assert_eq!(k(EMISSAO, 0).membro(), Some("canal.objeto.emissao"));
    assert!(k(3, 0).info().is_none(), "buraco no vocabulário");
}
