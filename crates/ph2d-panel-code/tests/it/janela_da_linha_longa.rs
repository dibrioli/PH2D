//! ⛔ **Uma linha maior que o tecto pinta a janela À VISTA, não os primeiros bytes** (o tecto
//! `LINE_RENDER_BYTES_MAX` é do quadro): com a vista no fim de uma linha colada de 64 KB, o
//! ecrã ficava em branco — o pedido de vista levava a rolagem ao troço que não se pintava.

use crate::{Deco, TAB, area_para, pinta, ts};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{CodeView, Hit};

/// Duas linhas curtas e uma de ~64 KB com tabs, `é` e espaços no meio.
fn longa() -> (Buffer, usize) {
    let l = format!("local x = 1 --{}", "a\tbc é ".repeat(64 << 10 >> 3));
    let buf = Buffer::new(&format!("local a = 1\n{l}\nlocal b = 2"));
    (buf, 1)
}

/// A coluna de ecrã que a conta da mono põe no x `x` (a origem e a rolagem da vista).
fn coluna_em(v: &CodeView, x: f32) -> usize {
    let m = v.metrics();
    ((x - v.text_rect().x - m.text_pad + v.scroll().0) / m.advance).floor() as usize
}

#[test]
fn a_vista_no_fim_da_linha_longa_pinta_o_fim_dela() {
    let (mut buf, line) = longa();
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 10);
    let fim = buf.text().line_end(line);
    buf.set_selection(Selection::single(Range::cursor(fim)));
    v.ensure_visible(&buf, fim);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let tr = v.text_rect();
    let cols: Vec<_> = v
        .probe_columns()
        .into_iter()
        .filter(|(c, _)| buf.text().line_of_byte(c.byte) == line)
        .collect();
    let ultimo = buf.text().prev_grapheme(fim);
    let c = cols
        .iter()
        .find(|(c, _)| c.byte == ultimo)
        .map(|p| p.0)
        .unwrap_or_else(|| {
            panic!(
                "o último grafema não se pintou: bytes {:?}",
                cols.first()
                    .zip(cols.last())
                    .map(|(a, b)| (a.0.byte, b.0.byte))
            )
        });
    assert!(
        c.x0 >= tr.x && c.x1 <= tr.x + tr.w,
        "o fim pinta-se fora da vista: {c:?} em {tr:?}"
    );
    assert_eq!(
        coluna_em(&v, (c.x0 + c.x1) * 0.5),
        buf.text().display_col(ultimo, TAB),
        "o x do fim é o da coluna dele na linha"
    );
    let caret = v.caret_rect(&buf, fim).expect("o cursor do fim à vista");
    assert!(caret.x >= tr.x && caret.x <= tr.x + tr.w, "{caret:?}");
    assert!(
        (caret.x - c.x1).abs() <= 1.0,
        "cursor {caret:?} × fim {c:?}"
    );
    // Controlo: a janela é a do tecto, não a linha inteira.
    assert!(cols.len() <= ph2d_panel_code::LINE_RENDER_BYTES_MAX);
}

#[test]
fn o_clique_no_meio_da_janela_deslocada_cai_no_byte_certo() {
    let (mut buf, line) = longa();
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 10);
    let t = buf.text();
    let meio = t.floor_char((t.byte_of_line(line) + t.line_end(line)) / 2);
    buf.set_selection(Selection::single(Range::cursor(meio)));
    v.ensure_visible(&buf, meio);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let tr = v.text_rect();
    let t = buf.text();
    let vistas: Vec<_> = v
        .probe_columns()
        .into_iter()
        .filter(|(c, _)| t.line_of_byte(c.byte) == line && c.x0 >= tr.x && c.x1 <= tr.x + tr.w)
        .collect();
    assert!(
        vistas.len() > 20,
        "a vista mostra a linha: {}",
        vistas.len()
    );
    assert!(
        vistas.iter().any(|(c, _)| c.byte == meio),
        "o cursor do meio pinta-se"
    );
    for (c, mid) in vistas.iter().step_by(7) {
        let x = c.x0 + (c.x1 - c.x0) * 0.25;
        assert_eq!(v.hit(&buf, x, *mid), Hit::Text(c.byte), "{c:?}");
        assert_eq!(
            coluna_em(&v, x),
            t.display_col(c.byte, TAB),
            "o byte {} pinta-se na coluna dele",
            c.byte
        );
    }
}

#[test]
fn uma_linha_curta_pinta_se_inteira_a_partir_da_coluna_0() {
    let buf = Buffer::new("local a = 1\n\tlocal é = 2");
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 10);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let t = buf.text();
    let cols = v.probe_columns();
    let x0 = v.text_rect().x + v.metrics().text_pad;
    for line in 0..2 {
        let (ls, le) = (t.byte_of_line(line), t.line_end(line));
        let mine: Vec<_> = cols
            .iter()
            .filter(|(c, _)| t.line_of_byte(c.byte) == line)
            .collect();
        assert_eq!(mine.first().map(|p| p.0.byte), Some(ls));
        assert!((mine[0].0.x0 - x0).abs() <= 1.0, "{:?}", mine[0]);
        assert_eq!(mine.last().map(|p| t.next_grapheme(p.0.byte)), Some(le));
        for (c, mid) in mine {
            assert_eq!(
                v.hit(&buf, c.x0 + (c.x1 - c.x0) * 0.25, *mid),
                Hit::Text(c.byte)
            );
        }
    }
}
