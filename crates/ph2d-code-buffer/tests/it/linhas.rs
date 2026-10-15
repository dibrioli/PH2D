//! As métricas de linha, de ponto e de UTF-16 contra a contagem ingénua sobre a `String`.

use ph2d_code_buffer::{Point, Text};

fn naive_lines(s: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if c == '\n' {
            out.push((start, i));
            start = i + 1;
        }
    }
    out.push((start, s.len()));
    out
}

const AMOSTRAS: &[&str] = &[
    "",
    "a",
    "a\n",
    "\n",
    "\n\n",
    "local x = 1\nprint(x)\n",
    "ção\n😀 é\n\tfim",
    "a\n\nb\n\n",
];

#[test]
fn a_linha_vazia_depois_do_ultimo_enter_existe() {
    assert_eq!(Text::new("").line_count(), 1);
    assert_eq!(Text::new("a").line_count(), 1);
    assert_eq!(Text::new("a\n").line_count(), 2);
    assert_eq!(Text::new("\n\n").line_count(), 3);
}

#[test]
fn linhas_bytes_e_pontos_batem_com_a_contagem_ingenua() {
    for s in AMOSTRAS {
        let t = Text::new(s);
        let lines = naive_lines(s);
        assert_eq!(t.line_count(), lines.len(), "{s:?}");
        for (row, (start, end)) in lines.iter().copied().enumerate() {
            assert_eq!(t.byte_of_line(row), start, "{s:?} linha {row}");
            assert_eq!(t.line_end(row), end, "{s:?} linha {row}");
            assert_eq!(t.line_string(row), s[start..end], "{s:?} linha {row}");
            for b in start..=end {
                if !s.is_char_boundary(b) {
                    continue;
                }
                assert_eq!(t.line_of_byte(b), row, "{s:?} byte {b}");
                assert_eq!(t.point_of_byte(b), Point { row, col: b - start });
                assert_eq!(t.byte_of_point(Point { row, col: b - start }), b);
            }
        }
        assert_eq!(t.byte_of_line(t.line_count()), s.len());
    }
}

#[test]
fn utf16_bate_com_encode_utf16() {
    for s in AMOSTRAS {
        let t = Text::new(s);
        for (b, _) in s.char_indices().chain(std::iter::once((s.len(), ' '))) {
            let u = s[..b].encode_utf16().count();
            assert_eq!(t.utf16_of_byte(b), u, "{s:?} byte {b}");
            assert_eq!(t.byte_of_utf16(u), b, "{s:?} unidade {u}");
        }
    }
}

#[test]
fn crlf_e_cr_soltos_viram_lf() {
    assert_eq!(Text::new("a\r\nb\rc").to_string(), "a\nb\nc");
}

#[test]
fn grafemas_nunca_partem_um_emoji_nem_um_acento_combinado() {
    let s = "e\u{301}😀x";
    let t = Text::new(s);
    assert_eq!(t.next_grapheme(0), 3);
    assert_eq!(t.next_grapheme(3), 7);
    assert_eq!(t.prev_grapheme(7), 3);
    assert_eq!(t.prev_grapheme(3), 0);
}

#[test]
fn coluna_de_ecra_conta_o_tab_ate_a_paragem() {
    let t = Text::new("\tab\n  \tc");
    assert_eq!(t.display_col(1, 4), 4);
    assert_eq!(t.display_col(3, 4), 6);
    let l1 = t.byte_of_line(1);
    assert_eq!(t.display_col(l1 + 3, 4), 4);
    assert_eq!(t.byte_at_display_col(1, 4, 4), l1 + 3);
    assert_eq!(t.byte_at_display_col(0, 99, 4), 3);
}
