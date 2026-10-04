//! ⭐⭐ Gates da lei de pousar sobre uma CAMADA (`docs/3D/30` §11): a
//! amostra como o plano a guarda, cor pré-multiplicada + opacidade.

use super::{Mistura, pousa};

/// **`Sobre` uma amostra TRANSPARENTE pousa a tela: cor `pm·k`, opacidade
/// `a·k`** — e sobre uma opaca a opacidade fica `1`.
#[test]
fn sobre_uma_amostra_transparente_pousa_a_tela_com_a_opacidade_dela() {
    let tela = Mistura::Sobre {
        pm: [0.4, 0.2, 0.1],
        a: 0.5,
    };
    let (c, a) = pousa([0.0; 3], 0.0, tela, 0.8);
    assert_eq!((c, a), ([0.4 * 0.8, 0.2 * 0.8, 0.1 * 0.8], 0.5 * 0.8));
    let (_, a) = pousa([0.3; 3], 1.0, tela, 0.8);
    assert!((a - 1.0).abs() < 1e-6, "sobre opaco fica opaco ({a})");
    let (_, a) = pousa([0.1; 3], 0.25, tela, 1.0);
    assert!(
        (a - (0.25 * 0.5 + 0.5)).abs() < 1e-6,
        "a soma «por cima» ({a})"
    );
}

/// **A `Diferenca` soma-se à cor DIREITA e não mexe na opacidade** — numa
/// amostra a meia opacidade a cor pré-multiplicada move `d·k·alfa`.
#[test]
fn a_diferenca_mexe_na_cor_direita_e_nao_na_opacidade() {
    let (c, a) = pousa(
        [0.25, 0.1, 0.0],
        0.5,
        Mistura::Diferenca([0.2, 0.0, 0.0]),
        1.0,
    );
    assert_eq!(a, 0.5);
    assert!(
        (c[0] - 0.35).abs() < 1e-6 && (c[1] - 0.1).abs() < 1e-6,
        "{c:?}"
    );
    assert_eq!(
        pousa([0.3; 3], 0.0, Mistura::Diferenca([0.5; 3]), 1.0),
        ([0.3; 3], 0.0),
        "num transparente nada"
    );
}

/// ⭐⭐⭐ **GATE (`docs/3D/30` §16) — a tela semeada com uma CAMADA: a diferença em cor
/// pré-multiplicada E em opacidade.** Numa amostra transparente, tinta nova entra com a opacidade da
/// tela; a borracha tira opacidade; e numa camada opaca é a `Diferenca` de antes, ao bit.
#[test]
fn a_tela_semeada_com_uma_camada_poe_e_tira_opacidade() {
    let tinta = Mistura::Camada {
        dpm: [0.6, 0.1, 0.05],
        da: 0.75,
    };
    let (c, a) = pousa([0.0; 3], 0.0, tinta, 1.0);
    assert_eq!(
        (c, a),
        ([0.6, 0.1, 0.05], 0.75),
        "num transparente a tinta entra"
    );
    let (c, a) = pousa([0.0; 3], 0.0, tinta, 0.5);
    assert_eq!((c, a), ([0.3, 0.05, 0.025], 0.375), "pesada pela máscara");
    let borracha = Mistura::Camada {
        dpm: [-0.2, -0.2, -0.2],
        da: -0.5,
    };
    let (c, a) = pousa([0.4, 0.4, 0.4], 0.8, borracha, 1.0);
    assert!(
        (a - 0.3).abs() < 1e-6 && c.iter().all(|&x| x <= a + 1e-6),
        "a borracha tira: {c:?} {a}"
    );
    let d = [0.12, -0.3, 0.05];
    assert_eq!(
        pousa(
            [0.5, 0.6, 0.2],
            1.0,
            Mistura::Camada { dpm: d, da: 0.0 },
            0.7
        ),
        pousa([0.5, 0.6, 0.2], 1.0, Mistura::Diferenca(d), 0.7),
        "numa camada opaca é a Diferenca, ao bit"
    );
}
