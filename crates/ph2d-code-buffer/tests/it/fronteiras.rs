//! ⛔ **As contas do `Text` são totais** (auditoria do fecho da Onda 1): uma coluna, uma unidade
//! UTF-16 ou uma largura de tab vindas de fora (o LSP, o rato, uma preferência) nunca derrubam o
//! editor nem devolvem meio carácter; e o texto colado num cursor de cada vez nunca traz um `\r`.

use ph2d_code_buffer::{Buffer, EditKind, Point, Range, Selection, Text, display_width};

#[test]
fn byte_of_point_no_meio_de_um_caracter_desce_ao_inicio_dele() {
    let t = Text::new("é😀x\nb");
    assert_eq!(t.byte_of_point(Point { row: 0, col: 1 }), 0, "dentro do é");
    assert_eq!(t.byte_of_point(Point { row: 0, col: 4 }), 2, "dentro do 😀");
    assert_eq!(t.byte_of_point(Point { row: 0, col: 6 }), 6, "o x");
    assert_eq!(
        t.byte_of_point(Point {
            row: 9,
            col: usize::MAX
        }),
        t.len(),
        "fora do texto"
    );
}

#[test]
fn byte_of_utf16_fora_da_faixa_ou_no_meio_de_um_par_nao_entra_em_panico() {
    let t = Text::new("a😀b");
    assert_eq!(t.byte_of_utf16(999), t.len(), "fora da faixa = o fim");
    assert_eq!(
        t.byte_of_utf16(2),
        1,
        "no meio do par substituto = o início do 😀"
    );
    assert_eq!(t.byte_of_utf16(3), 5);
    assert_eq!(
        t.utf16_of_byte(3),
        1,
        "um byte no meio do 😀 conta como o início dele"
    );
}

#[test]
fn uma_tab_de_largura_zero_nao_divide_por_zero() {
    assert_eq!(display_width("\tx", 0), 2);
    let t = Text::new("\tx\ty");
    assert_eq!(t.byte_at_display_col(0, 1, 0), 1);
    assert_eq!(t.display_col(3, 0), 3);
}

#[test]
fn colar_um_texto_por_cursor_normaliza_o_cr_solto() {
    let mut b = Buffer::new("1\n2\n");
    b.set_selection(Selection::new(vec![Range::cursor(1), Range::cursor(3)], 0));
    let mut i = 0;
    b.insert_each(
        |_| {
            i += 1;
            ["a\rb", "c\r\nd"][i - 1].to_owned()
        },
        EditKind::Other,
    );
    let t = b.text().to_string();
    assert!(!t.contains('\r'), "um `\\r` entrou no buffer: {t:?}");
    assert_eq!(t, "1a\nb\n2c\nd\n");
}
