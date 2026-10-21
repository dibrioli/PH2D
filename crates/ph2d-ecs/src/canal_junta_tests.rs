use super::*;
use crate::canal::{FieldKey, familia};

fn k(campo: u16, i: u16) -> FieldKey {
    FieldKey::new(familia::JUNTA, campo, i, 0)
}

/// ⛔ Os números são gravados no documento da timeline: renumerar reprova.
#[test]
fn o_vocabulario_da_junta_nao_renumera() {
    assert_eq!(familia::JUNTA, 3);
    let fixos = [
        (MOTOR_LIGADO, 1),
        (MOTOR_MODO, 2),
        (MOTOR_FORCA, 3),
        (MOTOR_EIXO, 4),
        (QUEBRA_LIGADA, 10),
        (QUEBRA_FORCA, 11),
        (QUEBRA_TORQUE, 12),
        (LIMITES_LIGADOS, 20),
        (LIMITE_MIN, 21),
        (LIMITE_MAX, 22),
        (SOLDA_MOLE, 30),
        (RIGIDEZ, 31),
        (AMORTECIMENTO, 32),
        (EIXO_MODO, 40),
        (EIXO_MIN, 41),
        (EIXO_MAX, 42),
    ];
    for (n, (atual, gravado)) in fixos.iter().enumerate() {
        assert_eq!(atual, gravado, "a constante nº {n} da junta foi renumerada");
    }
}

/// Interruptores e escolhas são discretos; cada linha é o seu grupo; os eixos partem-se por `i`.
#[test]
fn cada_campo_da_junta_diz_o_que_e() {
    for d in [
        MOTOR_LIGADO,
        MOTOR_MODO,
        MOTOR_EIXO,
        QUEBRA_LIGADA,
        LIMITES_LIGADOS,
        SOLDA_MOLE,
    ] {
        assert!(k(d, 0).info().is_some_and(|i| i.discreto), "campo {d}");
    }
    for c in [
        MOTOR_FORCA,
        QUEBRA_FORCA,
        QUEBRA_TORQUE,
        LIMITE_MIN,
        LIMITE_MAX,
        RIGIDEZ,
        AMORTECIMENTO,
    ] {
        assert!(k(c, 0).info().is_some_and(|i| !i.discreto), "campo {c}");
    }
    assert!(
        !k(LIMITE_MIN, 0).mesmo_grupo(k(LIMITE_MAX, 0)),
        "duas linhas"
    );
    assert_eq!(EIXO_PASSO, 3);
    for a in 0..N_EIXOS {
        for base in [EIXO_MODO, EIXO_MIN, EIXO_MAX] {
            let campo = do_eixo(base, a);
            assert_eq!(eixo_de(campo), Some((a, base)));
            let i = k(campo, 0).info().expect("campo de eixo");
            assert_eq!(i.discreto, base == EIXO_MODO);
        }
    }
    assert!(
        !k(do_eixo(EIXO_MODO, 0), 0).mesmo_grupo(k(do_eixo(EIXO_MODO, 2), 0)),
        "um eixo por grupo"
    );
    assert_ne!(
        k(do_eixo(EIXO_MIN, 0), 0).membro(),
        k(do_eixo(EIXO_MIN, 1), 0).membro(),
        "o dope-sheet diz de que eixo é a linha"
    );
    assert!(eixo_de(do_eixo(EIXO_MODO, N_EIXOS)).is_none());
    assert_eq!(k(RIGIDEZ, 0).membro(), Some("canal.junta.rigidez"));
    assert!(k(5, 0).info().is_none(), "buraco no vocabulário");
}
