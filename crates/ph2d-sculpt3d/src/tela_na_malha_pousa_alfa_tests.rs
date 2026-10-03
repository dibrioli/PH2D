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
