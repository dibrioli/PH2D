//! Gates da disposição do diagrama booleano — a forma CANONICA da lista, que e' o que o undo ve'.

use super::{VecBoolGraphPos, VecBoolNodePos};

/// **A MESMA DISPOSIÇÃO ESCRITA EM QUALQUER ORDEM DÁ OS MESMOS BYTES.**
///
/// ⚠️ É o gate do undo: o passo é registado por diff, e sem esta lei arrastar dois círculos em
/// ordens diferentes daria dois estados que desenham igual.
#[test]
fn a_mesma_disposicao_em_qualquer_ordem_da_os_mesmos_bytes() {
    let mut a = VecBoolGraphPos::default();
    a.set(7, [10.0, 20.0]);
    a.set(2, [30.0, 40.0]);
    let mut b = VecBoolGraphPos::default();
    b.set(2, [30.0, 40.0]);
    b.set(7, [10.0, 20.0]);
    assert_eq!(a, b, "a ordem de escrita sobreviveu à canonização");
    assert_eq!(a.nodes[0].id, 2, "a lista não ficou ordenada por id");
}

/// **MOVER SUBSTITUI, NÃO EMPILHA.**
#[test]
fn mover_o_mesmo_circulo_substitui() {
    let mut p = VecBoolGraphPos::default();
    p.set(1, [0.0, 0.0]);
    p.set(1, [50.0, 60.0]);
    assert_eq!(p.nodes.len(), 1);
    assert_eq!(p.get(1), Some([50.0, 60.0]));
}

/// **APAGAR UMA FORMA LEVA A POSIÇÃO DELA.**
#[test]
fn esquecer_uma_forma_tira_a_posicao() {
    let mut p = VecBoolGraphPos::default();
    p.set(1, [1.0, 1.0]);
    p.set(2, [2.0, 2.0]);
    p.forget(1);
    assert_eq!(
        p.nodes,
        vec![VecBoolNodePos {
            id: 2,
            at: [2.0, 2.0]
        }]
    );
}

/// **UMA FORMA SEM POSIÇÃO DIZ `None`** — e é isso que faz o diagrama arrumar sozinho em vez de a
/// empilhar na origem.
#[test]
fn uma_forma_sem_posicao_diz_none() {
    let p = VecBoolGraphPos::default();
    assert_eq!(p.get(9), None);
}
