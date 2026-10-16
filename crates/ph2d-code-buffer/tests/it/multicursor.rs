//! Multicursor: Ctrl+D, Alt+clique, setas com «goal column», escrever em N cursores.

use ph2d_code_buffer::{Buffer, Motion, Range, Selection};

#[test]
fn ctrl_d_tres_vezes_e_escrever_e_um_passo_de_undo() {
    let src = "local vida = 1\nvida = vida + 1\n";
    let mut b = Buffer::new(src);
    b.set_selection(Selection::single(Range::cursor(8)));
    assert!(
        b.select_next_occurrence(),
        "1.º Ctrl+D selecciona a palavra"
    );
    assert!(b.select_next_occurrence());
    assert!(b.select_next_occurrence());
    assert_eq!(b.selection().len(), 3);
    assert!(!b.select_next_occurrence(), "não há 4.ª ocorrência");
    for c in "hp".chars() {
        b.insert(&c.to_string());
    }
    assert_eq!(b.to_string(), "local hp = 1\nhp = hp + 1\n");
    b.undo();
    assert_eq!(
        b.to_string(),
        src,
        "uma edição multicursor desfaz-se de uma vez"
    );
}

#[test]
fn backspace_em_cursores_vizinhos_nao_apaga_duas_vezes_o_mesmo() {
    let mut b = Buffer::new("abcd");
    b.set_selection(Selection::new(vec![Range::cursor(2), Range::cursor(3)], 0));
    b.delete_backward();
    assert_eq!(b.to_string(), "ad");
    assert_eq!(
        b.selection().len(),
        1,
        "os dois cursores fundiram-se no mesmo sítio"
    );
}

#[test]
fn alt_clique_poe_e_tira_um_cursor() {
    let mut b = Buffer::new("a\nb\nc");
    b.toggle_cursor(2);
    b.toggle_cursor(4);
    assert_eq!(b.selection().len(), 3);
    b.toggle_cursor(2);
    assert_eq!(b.selection().len(), 2);
    b.insert("-");
    assert_eq!(b.to_string(), "-a\nb\n-c");
    b.single_cursor();
    assert_eq!(b.selection().len(), 1);
}

#[test]
fn seta_para_baixo_lembra_a_coluna_numa_linha_curta() {
    let mut b = Buffer::new("abcdef\nab\nabcdef");
    b.set_selection(Selection::single(Range::cursor(5)));
    b.move_cursors(Motion::Down, false, 4);
    assert_eq!(b.selection().primary().head, 9, "fim da linha curta");
    b.move_cursors(Motion::Down, false, 4);
    assert_eq!(b.selection().primary().head, 15, "volta à coluna 5");
}

#[test]
fn home_esperto_alterna_entre_o_primeiro_nao_espaco_e_a_coluna_zero() {
    let mut b = Buffer::new("\t\tend");
    b.set_selection(Selection::single(Range::cursor(5)));
    b.move_cursors(Motion::LineStart, false, 4);
    assert_eq!(b.selection().primary().head, 2);
    b.move_cursors(Motion::LineStart, false, 4);
    assert_eq!(b.selection().primary().head, 0);
}

#[test]
fn ctrl_seta_salta_palavras_e_pontuacao_por_blocos() {
    let mut b = Buffer::new("self.vida  += dt");
    let mut stops = Vec::new();
    for _ in 0..5 {
        b.move_cursors(Motion::WordRight, false, 4);
        stops.push(b.selection().primary().head);
    }
    assert_eq!(stops, vec![4, 5, 9, 13, 16]);
}

#[test]
fn shift_seta_estende_e_seta_sem_shift_cai_na_ponta() {
    let mut b = Buffer::new("abcdef");
    b.set_selection(Selection::single(Range::cursor(2)));
    b.move_cursors(Motion::Right, true, 4);
    b.move_cursors(Motion::Right, true, 4);
    assert_eq!(b.selection().primary(), Range::new(2, 4));
    b.move_cursors(Motion::Left, false, 4);
    assert_eq!(b.selection().primary(), Range::cursor(2));
}
