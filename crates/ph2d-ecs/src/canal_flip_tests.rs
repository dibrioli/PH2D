use super::*;
use crate::canal::{FieldKey, familia};

fn k(campo: u16, camada: u16) -> FieldKey {
    FieldKey::new(familia::FLIP, campo, camada, 0)
}

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_do_flip_nao_renumera() {
    assert_eq!(familia::FLIP, 4);
    let fixos = [
        (VISIVEL, 1),
        (OPACIDADE, 2),
        (MISTURA, 3),
        (PROFUNDIDADE, 4),
    ];
    for (n, (atual, gravado)) in fixos.iter().enumerate() {
        assert_eq!(atual, gravado, "a constante nº {n} do Flip foi renumerada");
    }
}

/// O olho e a mistura são discretos; cada linha é o seu grupo, e o grupo parte-se por camada.
#[test]
fn cada_campo_da_camada_diz_o_que_e() {
    for d in [VISIVEL, MISTURA] {
        assert!(k(d, 0).info().is_some_and(|i| i.discreto), "campo {d}");
    }
    for c in [OPACIDADE, PROFUNDIDADE] {
        assert!(k(c, 0).info().is_some_and(|i| !i.discreto), "campo {c}");
    }
    for campo in CAMPOS {
        assert!(k(campo, 0).info().is_some_and(|i| i.por_i && !i.texto));
    }
    assert!(k(OPACIDADE, 3).mesmo_grupo(k(OPACIDADE, 3)));
    assert!(
        !k(OPACIDADE, 3).mesmo_grupo(k(OPACIDADE, 4)),
        "cada camada tem a sua curva"
    );
    assert!(
        !k(OPACIDADE, 3).mesmo_grupo(k(PROFUNDIDADE, 3)),
        "duas linhas"
    );
    assert_eq!(k(MISTURA, 0).membro(), Some("canal.flip.mistura"));
    assert!(k(5, 0).info().is_none(), "buraco no vocabulário");
}
