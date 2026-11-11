//! E4 — o clique devolve o byte que a pintura pôs lá: tabs, acentos, emoji e uma dobra antes.

use crate::{Deco, TAB, area_para, pinta, ts};
use ph2d_code_buffer::Buffer;
use ph2d_panel_code::{CodeView, FoldMark, Hit};

pub const AMOSTRA: &str = "local a = 1\n\tif x then\t-- é ação\ncabeça da dobra\nescondida 1\nescondida 2\nescondida 3\n\t\tprint(\"olá 😀 mundo\")\nx\ty\t\tz\n";

pub fn dobra() -> Deco {
    Deco {
        folds: vec![FoldMark {
            start: 2,
            end: 5,
            folded: true,
        }],
        ..Deco::default()
    }
}

#[test]
fn o_clique_devolve_o_byte_que_a_pintura_pos_la() {
    let buf = Buffer::new(AMOSTRA);
    let t = buf.text();
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 20);
    pinta(&mut v, &buf, &mut ts, area, &dobra());
    let cols = v.probe_columns();
    for needle in ["\t", "é", "😀", "ç"] {
        let b = AMOSTRA.find(needle).unwrap_or_else(|| panic!("{needle}"));
        assert!(
            cols.iter().any(|(c, _)| c.byte == b),
            "a amostra tem uma coluna em {needle:?}"
        );
    }
    let adv = v.metrics().advance;
    for (c, mid) in &cols {
        let line = t.line_of_byte(c.byte);
        assert!(!(3..=5).contains(&line), "linha escondida pintada: {line}");
        let w = c.x1 - c.x0;
        assert!(w > 0.0, "coluna sem largura em {}", c.byte);
        assert_eq!(
            v.hit(&buf, c.x0 + w * 0.25, *mid),
            Hit::Text(c.byte),
            "1/4 do glifo do byte {}",
            c.byte
        );
        let next = t.next_grapheme(c.byte);
        assert_eq!(
            v.hit(&buf, c.x0 + w * 0.75, *mid),
            Hit::Text(next),
            "3/4 do glifo do byte {}",
            c.byte
        );
        let caret = v
            .caret_rect(&buf, c.byte)
            .expect("o byte pintado está à vista");
        assert!(
            (caret.x - c.x0).abs() < 0.01,
            "caret_rect {} ≠ pintura {} no byte {}",
            caret.x,
            c.x0,
            c.byte
        );
        // A grelha da mono: a coluna de ecrã (tabs até à paragem) × o avanço medido.
        if !t.line_string(line).contains('😀') {
            let x_col0 = cols
                .iter()
                .find(|(k, _)| k.byte == t.byte_of_line(line))
                .map(|(k, _)| k.x0)
                .unwrap_or(c.x0);
            let esperado = t.display_col(c.byte, TAB) as f32 * adv;
            assert!(
                (c.x0 - x_col0 - esperado).abs() < 0.05,
                "byte {}: x {} ≠ coluna × avanço {}",
                c.byte,
                c.x0 - x_col0,
                esperado
            );
        }
    }
}

#[test]
fn calha_dobra_estado_e_fora() {
    let buf = Buffer::new(AMOSTRA);
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 20);
    pinta(&mut v, &buf, &mut ts, area, &dobra());
    let m = *v.metrics();
    let y = |row: usize| row as f32 * m.line_h + m.line_h * 0.5;
    assert_eq!(v.hit(&buf, m.mark_w, y(0)), Hit::Gutter(0));
    assert_eq!(v.hit(&buf, m.gutter_w - m.fold_w * 0.5, y(2)), Hit::Fold(2));
    assert_eq!(
        v.hit(&buf, m.gutter_w - m.fold_w * 0.5, y(1)),
        Hit::Gutter(1),
        "sem dobra, o marcador é calha"
    );
    assert_eq!(
        v.hit(&buf, m.gutter_w + m.mark_w, y(3)),
        Hit::Text(buf.text().byte_of_line(6)),
        "a fila 3 é a linha 6"
    );
    assert_eq!(
        v.hit(&buf, area.w - 1.0, y(0)),
        Hit::Text(buf.text().line_end(0)),
        "à direita do fim: o fim da linha"
    );
    assert_eq!(v.hit(&buf, area.w * 0.5, area.h - 1.0), Hit::Status);
    assert_eq!(v.hit(&buf, area.w + 1.0, y(0)), Hit::Outside);
}

#[test]
fn a_pre_edicao_empurra_o_texto_e_o_cursor_fica_no_fim_dela() {
    let mut buf = Buffer::new("ab\n");
    buf.set_selection(ph2d_code_buffer::Selection::single(
        ph2d_code_buffer::Range::cursor(1),
    ));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 5);
    let sem = {
        pinta(&mut v, &buf, &mut ts, area, &Deco::default());
        v.probe_columns()
    };
    let d = Deco {
        preedit: Some("é".into()),
        ..Deco::default()
    };
    pinta(&mut v, &buf, &mut ts, area, &d);
    let com = v.probe_columns();
    let b_sem = sem.iter().find(|(c, _)| c.byte == 1).map(|(c, _)| c.x0);
    let b_com = com.iter().find(|(c, _)| c.byte == 1).map(|(c, _)| c.x1);
    let adv = v.metrics().advance;
    assert!(
        (b_com.unwrap_or(0.0) - b_sem.unwrap_or(0.0) - adv * 2.0).abs() < 0.05,
        "o «b» anda uma coluna para a direita"
    );
}

/// O separador da grelha e da Sheet carrega por [`Splitter::press`], sem relógio: dois cliques
/// sem arrasto nunca fazem o duplo que esquece a largura do artista.
#[test]
#[allow(clippy::field_reassign_with_default)] // o resto dos campos é privado
fn o_separador_sem_relogio_nunca_faz_duplo_clique() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_panel_code::splitter::{Split, Splitter};
    let mut s = Splitter::default();
    s.side_w = Some(80.0);
    s.painted = Some(Split {
        hit: Rect::new(0.0, 0.0, 10.0, 100.0),
        side_w: 80.0,
        min: 0.0,
        max: 200.0,
    });
    for _ in 0..3 {
        assert!(s.press(5.0, 50.0), "controlo: agarra");
        assert!(s.release());
    }
    assert_eq!(s.side_w, Some(80.0), "a largura do artista fica");
}

/// ⭐ Dono 10/11: com o encaixe do View ▸ Text rendering, o cursor, o clique e a pintura leem o MESMO
/// x encaixado: cada coluna começa num glifo pintado, o cursor fica nela e o clique devolve-a.
#[test]
fn o_cursor_e_o_clique_seguem_o_x_encaixado_de_cada_modo() {
    use ph2d_panel_code::code_text::set_mode;
    use ph2d_tokens::TextRendering;
    let buf = Buffer::new("local alive = x1 + y2\n");
    for mode in TextRendering::ALL {
        set_mode(mode);
        let (mut v, mut ts) = (CodeView::new(), ts());
        let area = area_para(&mut v, &buf, &mut ts, 4);
        let s = pinta(&mut v, &buf, &mut ts, area, &Deco::default());
        let pintados: Vec<f32> = s
            .inner()
            .encoding()
            .resources
            .glyphs
            .iter()
            .map(|g| g.x)
            .collect();
        for (c, mid) in v.probe_columns() {
            assert!(
                pintados.contains(&c.x0),
                "{mode:?}: coluna {} em {} sem glifo",
                c.byte,
                c.x0
            );
            let caret = v.caret_rect(&buf, c.byte).expect("à vista").x;
            assert_eq!(caret, c.x0, "{mode:?}: o cursor do byte {}", c.byte);
            let w = c.x1 - c.x0;
            assert_eq!(
                v.hit(&buf, c.x0 + w * 0.25, mid),
                Hit::Text(c.byte),
                "{mode:?}"
            );
        }
        set_mode(TextRendering::Default);
    }
}
