//! Gates do [`super::VecBoolEdges`] — a forma CANONICA da lista, que e' o que o undo ve'.

use super::{VecBoolEdge, VecBoolEdges};

/// **A LISTA É CANÓNICA** — a mesma relação escrita em qualquer ordem dá os mesmos bytes.
///
/// ⚠️ É o gate do undo: o passo é registado por diff de bytes, e sem esta lei reordenar a lista
/// (um painel, um merge) viraria um passo que não mudou nada na tela.
#[test]
fn a_mesma_relacao_escrita_em_qualquer_ordem_da_os_mesmos_bytes() {
    let a = VecBoolEdges::new(vec![
        VecBoolEdge {
            from: 3,
            to: 1,
            op: 1,
        },
        VecBoolEdge {
            from: 2,
            to: 1,
            op: 0,
        },
    ]);
    let b = VecBoolEdges::new(vec![
        VecBoolEdge {
            from: 2,
            to: 1,
            op: 0,
        },
        VecBoolEdge {
            from: 3,
            to: 1,
            op: 1,
        },
    ]);
    assert_eq!(a, b, "a ordem de escrita sobreviveu à canonização");
}

/// **UMA LIGAÇÃO POR PAR ORDENADO** — e a direção continua a ser dado.
#[test]
fn um_par_repetido_colapsa_mas_a_direcao_oposta_sobrevive() {
    let v = VecBoolEdges::new(vec![
        VecBoolEdge {
            from: 2,
            to: 1,
            op: 0,
        },
        VecBoolEdge {
            from: 2,
            to: 1,
            op: 3,
        },
        VecBoolEdge {
            from: 1,
            to: 2,
            op: 1,
        },
    ]);
    assert_eq!(
        v.edges.len(),
        2,
        "o par repetido não colapsou, ou a direção oposta morreu"
    );
    assert_eq!(
        v.get(2, 1),
        Some(0),
        "sobreviveu o código maior, não o menor"
    );
    assert_eq!(v.get(1, 2), Some(1), "a ligação de volta é outra ligação");
}

/// **`set` SUBSTITUI e mantém a canonicidade** — o mesmo estado, escrito por dois caminhos.
#[test]
fn set_substitui_e_o_resultado_e_o_mesmo_da_construcao() {
    let mut v = VecBoolEdges::default();
    v.set(3, 1, 9);
    v.set(2, 1, 0);
    v.set(3, 1, 1);
    assert_eq!(
        v,
        VecBoolEdges::new(vec![
            VecBoolEdge {
                from: 2,
                to: 1,
                op: 0
            },
            VecBoolEdge {
                from: 3,
                to: 1,
                op: 1
            },
        ])
    );
}

/// **APAGAR UMA FORMA LEVA AS LIGAÇÕES DELA** — nos dois sentidos.
#[test]
fn esquecer_uma_forma_corta_as_ligacoes_dos_dois_lados() {
    let mut v = VecBoolEdges::new(vec![
        VecBoolEdge {
            from: 2,
            to: 1,
            op: 0,
        },
        VecBoolEdge {
            from: 1,
            to: 3,
            op: 0,
        },
        VecBoolEdge {
            from: 2,
            to: 3,
            op: 0,
        },
    ]);
    v.forget(1);
    assert_eq!(
        v.edges,
        vec![VecBoolEdge {
            from: 2,
            to: 3,
            op: 0
        }]
    );
}
