//! O desfazer por GRUPOS: o que o artista sente como um gesto é um passo.

use ph2d_code_buffer::{Buffer, Motion, Range, Selection};

fn typed(b: &mut Buffer, s: &str) {
    for g in s.chars() {
        b.insert(&g.to_string());
    }
}

#[test]
fn uma_palavra_escrita_e_um_passo_e_o_espaco_abre_outro() {
    let mut b = Buffer::new("");
    typed(&mut b, "hello world");
    assert_eq!(b.history().undo_len(), 2, "«hello» e « world»");
    b.undo();
    assert_eq!(b.to_string(), "hello");
    b.undo();
    assert_eq!(b.to_string(), "");
}

#[test]
fn o_cursor_que_salta_fecha_o_grupo() {
    let mut b = Buffer::new("xy");
    b.set_selection(Selection::single(Range::cursor(1)));
    typed(&mut b, "ab");
    b.move_cursors(Motion::DocEnd, false, 4);
    typed(&mut b, "cd");
    assert_eq!(b.to_string(), "xaby\u{0}".replace('\u{0}', "cd"));
    b.undo();
    assert_eq!(b.to_string(), "xaby");
}

#[test]
fn backspace_seguido_e_um_passo_e_nao_se_junta_a_escrita() {
    let mut b = Buffer::new("");
    typed(&mut b, "abc");
    for _ in 0..2 {
        b.delete_backward();
    }
    assert_eq!(b.to_string(), "a");
    b.undo();
    assert_eq!(b.to_string(), "abc");
    b.undo();
    assert_eq!(b.to_string(), "");
}

#[test]
fn enter_e_colar_sao_sempre_um_passo_proprio() {
    let mut b = Buffer::new("");
    typed(&mut b, "ab");
    b.insert("\n");
    b.insert("cola isto");
    assert_eq!(b.history().undo_len(), 3);
}

#[test]
fn salvar_fecha_o_grupo_e_limpa_o_sujo() {
    let mut b = Buffer::new("");
    typed(&mut b, "ab");
    assert!(b.is_dirty());
    b.mark_saved();
    assert!(!b.is_dirty());
    typed(&mut b, "cd");
    b.undo();
    assert_eq!(b.to_string(), "ab", "o grupo depois de salvar não se juntou ao de antes");
    assert!(!b.is_dirty(), "desfazer até ao texto salvo volta a «limpo»");
    b.redo();
    assert!(b.is_dirty());
    b.undo();
    b.undo();
    typed(&mut b, "z");
    b.undo();
    assert!(b.is_dirty(), "o estado salvo ficou no redo apagado: nunca mais é «limpo» sem salvar");
}

#[test]
fn desfazer_devolve_a_selecao_de_antes() {
    let mut b = Buffer::new("abc");
    b.set_selection(Selection::single(Range::new(0, 3)));
    b.insert("x");
    b.undo();
    assert_eq!(b.selection().primary(), Range::new(0, 3));
    b.redo();
    assert_eq!(b.selection().primary(), Range::cursor(1));
}
