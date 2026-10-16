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

/// ⛔ **O histórico tem tecto em BYTES** (auditoria do fecho, item 11): passado o tecto, os passos
/// mais ANTIGOS saem; os recentes continuam a desfazer-se, e o estado salvo que saiu deixa o texto
/// sujo (nunca «limpo» por engano). Um tecto pequeno aqui; o do app é [`HISTORY_BYTES_MAX`].
#[test]
fn o_historico_tem_tecto_em_bytes_e_os_passos_antigos_saem_primeiro() {
    use ph2d_code_buffer::EditKind;
    let mut b = Buffer::new("");
    b.set_history_cap(16 << 10);
    b.mark_saved();
    for i in 0..2_000 {
        b.insert_each(|_| format!("linha {i}\n"), EditKind::Other);
    }
    assert!(b.history().bytes() <= 16 << 10, "o histórico guarda {} bytes", b.history().bytes());
    let n = b.history().undo_len();
    assert!(n > 10 && n < 2_000, "{n} passos guardados");
    b.undo();
    assert!(!b.to_string().contains("linha 1999"), "o passo mais recente desfaz-se");
    while b.history().can_undo() {
        b.undo();
    }
    assert!(b.to_string().starts_with("linha 0\n"), "os antigos saíram (não se desfazem)");
    assert!(b.is_dirty(), "o estado salvo saiu do histórico: o texto não está «limpo»");
}

/// A conta dos bytes segue o desfazer e o refazer (nada fica contado duas vezes nem esquecido).
#[test]
fn a_conta_dos_bytes_do_historico_segue_o_desfazer_e_o_refazer() {
    let mut b = Buffer::new("");
    typed(&mut b, "hello world");
    let cheio = b.history().bytes();
    assert!(cheio > 0);
    b.undo();
    b.redo();
    assert_eq!(b.history().bytes(), cheio);
    b.undo();
    typed(&mut b, "!");
    assert!(b.history().bytes() < cheio + 200, "o refazer descartado saiu da conta");
}

/// 📏 **A medição do tecto do histórico** (corre à mão, `--ignored --nocapture`): bytes por passo para
/// cada espécie de gesto, contados pela MESMA conta que o tecto lê (`History::bytes`).
#[test]
#[ignore = "medição: corre à mão com --nocapture"]
fn mede_os_bytes_por_passo_do_historico() {
    use ph2d_code_buffer::EditKind;
    let mut b = Buffer::new("");
    let frase = "local vida = vida - dano * 2 -- comentário ";
    let teclas: String = frase.repeat(1_000 / frase.len() + 1).chars().take(1_000).collect();
    let antes = b.history().bytes();
    typed(&mut b, &teclas);
    let por_tecla = (b.history().bytes() - antes) as f64 / 1_000.0;
    println!("escrever: {por_tecla:.1} B/tecla ({} grupos em 1 000 teclas)", b.history().undo_len());
    let antes = b.history().bytes();
    for _ in 0..1_000 {
        b.insert_each(|_| "\n".into(), EditKind::Other);
    }
    println!("Enter: {:.1} B/passo", (b.history().bytes() - antes) as f64 / 1_000.0);
    for n in [1usize << 10, 64 << 10, 1 << 20] {
        let antes = b.history().bytes();
        b.insert(&"x".repeat(n));
        println!("colar {n} B: {} B/passo", b.history().bytes() - antes);
    }
    let dia = 8.0 * 60.0 * 300.0;
    println!("um dia de escrita (8 h × 300 teclas/min = {dia:.0} teclas): {:.1} MB", dia * por_tecla / 1e6);
}
