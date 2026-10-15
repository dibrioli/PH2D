//! E7 (a parte da pintura) — o símbolo culpado sublinhado no SÍTIO exacto.

use crate::clique::{AMOSTRA, dobra};
use crate::{Deco, area_para, pinta, ts};
use ph2d_code_buffer::Buffer;
use ph2d_panel_code::{CodeView, Problem, Severity};

fn problema(line: usize, culprit: std::ops::Range<usize>, severity: Severity) -> Problem {
    Problem { line, severity, culprit: Some(culprit), message: String::new() }
}

#[test]
fn o_culpado_e_sublinhado_nas_colunas_dele() {
    let buf = Buffer::new(AMOSTRA);
    let t = buf.text();
    let igual = AMOSTRA.find('=').expect("=");
    let if_ = AMOSTRA.find("if").expect("if");
    let esc = AMOSTRA.find("escondida 2").expect("escondida");
    let traco = AMOSTRA.find("--").expect("--");
    let d = Deco {
        problems: vec![
            problema(0, igual..igual + 1, Severity::Error),
            problema(1, if_..if_ + 2, Severity::Warning),
            problema(1, traco..traco + 2, Severity::Error),
            problema(4, esc..esc + 3, Severity::Error),
        ],
        ..dobra()
    };
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 20);
    pinta(&mut v, &buf, &mut ts, area, &d);
    let adv = v.metrics().advance;
    let cols = v.probe_columns();
    let x_of = |b: usize| cols.iter().find(|(c, _)| c.byte == b).map(|(c, _)| c.x0).expect("coluna");
    let u = v.last_underlines();
    assert_eq!(u.len(), 3, "o culpado numa linha escondida não se pinta: {u:?}");
    // `=` na coluna 8 da linha 0; `if` depois de UMA tab = coluna 4; `--` depois da tab que começa na
    // coluna 13 e pára na 16.
    for (k, (b, n, col)) in [(igual, 1, 8), (if_, 2, 4), (traco, 2, 16)].into_iter().enumerate() {
        let line_x0 = x_of(t.byte_of_line(t.line_of_byte(b)));
        assert!((u[k].x0 - x_of(b)).abs() < 0.01, "início do sublinhado {k}");
        assert!((u[k].x0 - line_x0 - col as f32 * adv).abs() < 0.05, "coluna do sublinhado {k}");
        assert!((u[k].x1 - u[k].x0 - n as f32 * adv).abs() < 0.05, "largura do sublinhado {k}");
        let row_top = t.line_of_byte(b).min(1) as f32 * v.metrics().line_h;
        assert!(u[k].y0 >= row_top && u[k].y1 <= row_top + v.metrics().line_h, "dentro da linha {k}");
    }
    assert_eq!((u[0].severity, u[1].severity), (Severity::Error, Severity::Warning));
}

#[test]
fn culpado_sem_largura_sublinha_uma_coluna() {
    let buf = Buffer::new("if x then\n");
    let fim = buf.text().line_end(0);
    let d = Deco { problems: vec![problema(0, fim..fim, Severity::Error)], ..Deco::default() };
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 5);
    pinta(&mut v, &buf, &mut ts, area, &d);
    let u = v.last_underlines();
    assert_eq!(u.len(), 1);
    assert!((u[0].x1 - u[0].x0 - v.metrics().advance).abs() < 0.01);
}
