//! Os gates de [`super`] — a LEI do realce, medida por VALOR. O censo textual que afirma que os
//! pintores a CHAMAM vive em `interact_node_drop_tests`, e as duas metades são obrigatórias: uma
//! lei que ninguém chama não desenha, e uma chamada a uma lei errada desenha a coisa errada.

use super::*;
use crate::snapshot::Piscada;

fn estado() -> MotionGraphPanelState {
    MotionGraphPanelState::default()
}

/// ⭐⭐⭐ **O ALVO ACENDE A FORÇA CHEIA ENQUANTO A MÃO PAIRA** — e só ele.
#[test]
fn o_alvo_da_largada_acende_a_forca_cheia() {
    let mut st = estado();
    st.largada_viva = Some(Largada::Troca(7));
    assert_eq!(realce_do_cartao(&st, 7), Some(1.0));
    assert_eq!(realce_do_cartao(&st, 8), None, "so' o ALVO acende");
    assert_eq!(
        realce_do_fio(&st, (7, 0)),
        None,
        "uma troca nao acende fios"
    );

    let mut st = estado();
    st.largada_viva = Some(Largada::Fio(3, 1));
    assert_eq!(realce_do_fio(&st, (3, 1)), Some(1.0));
    assert_eq!(
        realce_do_fio(&st, (3, 0)),
        None,
        "a PORTA faz parte do nome"
    );
    assert_eq!(realce_do_cartao(&st, 3), None, "um fio nao acende cartas");
}

/// ⭐⭐ **E O ECO ACENDE COM A FORÇA QUE A SHELL RESOLVEU** — a mesma cor, a mesma largura, só a
/// força a descer. *É isso que liga, para o olho, a promessa e o que aconteceu.*
#[test]
fn o_eco_acende_com_a_forca_que_a_shell_resolveu() {
    let mut st = estado();
    st.piscada_viva = Some(Piscada {
        nos: vec![4],
        fios: vec![(5, 2)],
        t: 0.25,
    });
    assert_eq!(realce_do_cartao(&st, 4), Some(0.25));
    assert_eq!(realce_do_fio(&st, (5, 2)), Some(0.25));
    assert_eq!(realce_do_cartao(&st, 9), None);
    assert_eq!(realce_do_fio(&st, (5, 0)), None);
}

/// ⚠️ **A PROMESSA ganha do ECO** — se as duas coincidirem (a mão volta a pairar sobre o que
/// acabou de acontecer), o que o artista precisa de ler é o que vai acontecer A SEGUIR.
#[test]
fn a_promessa_ganha_do_eco() {
    let mut st = estado();
    st.largada_viva = Some(Largada::Troca(4));
    st.piscada_viva = Some(Piscada {
        nos: vec![4],
        fios: Vec::new(),
        t: 0.1,
    });
    assert_eq!(realce_do_cartao(&st, 4), Some(1.0));
}

/// ⛔ **Sem gesto nenhum, nada acende** — o caminho de omissão é a tela de sempre.
#[test]
fn em_repouso_nada_acende() {
    let st = estado();
    assert_eq!(realce_do_cartao(&st, 1), None);
    assert_eq!(realce_do_fio(&st, (1, 0)), None);
}
