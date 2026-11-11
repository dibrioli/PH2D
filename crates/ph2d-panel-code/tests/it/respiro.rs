//! O respiro acima da linha 1 (dono 10/11): `Metrics::top_pad` é conteúdo que rola com o texto — a
//! pintura, o clique, o cursor, a calha, o «mostrar a linha» e a virtualização lêem-no do mesmo sítio.

use crate::{Deco, area_para, pinta, texto, ts};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{CodeView, Hit, Problem, Severity};

const VISTA: usize = 20;

fn pintada(
    n: usize,
) -> (
    CodeView,
    Buffer,
    ph2d_text::TextSystem,
    ph2d_editor_core::zones::Rect,
) {
    let buf = Buffer::new(&texto(n, 40));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, VISTA);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    (v, buf, ts, area)
}

/// O topo da fila da linha `line` no ecrã, lido das colunas que a pintura pôs.
fn topo(v: &CodeView, buf: &Buffer, line: usize) -> Option<f32> {
    let h = v.metrics().line_h;
    v.probe_columns()
        .iter()
        .find(|(c, _)| buf.text().line_of_byte(c.byte) == line)
        .map(|(_, mid)| mid - h * 0.5)
}

/// (b) A linha 1 começa no topo do texto MAIS o respiro, e o respiro é um token não nulo.
#[test]
fn a_linha_1_comeca_depois_do_respiro() {
    let (v, buf, _, area) = pintada(100);
    let pad = v.metrics().top_pad;
    assert!(pad > 0.0, "o respiro existe");
    assert_eq!(pad, ph2d_tokens::Spacing::Md.px());
    let t0 = topo(&v, &buf, 0).expect("a linha 1 pintada");
    assert!(
        (t0 - (area.y + pad)).abs() < 0.01,
        "{t0} ≠ {} + {pad}",
        area.y
    );
    let t1 = topo(&v, &buf, 1).expect("a linha 2 pintada");
    assert!((t1 - t0 - v.metrics().line_h).abs() < 0.01);
}

/// (c) Um clique no respiro cai na linha 1, na coluna do x (VS Code, Zed).
#[test]
fn um_clique_no_respiro_cai_na_linha_1() {
    let (v, buf, _, area) = pintada(100);
    let (c, _) = v
        .probe_columns()
        .into_iter()
        .find(|(c, _)| c.byte == 3)
        .expect("a coluna do byte 3");
    let x = c.x0 + (c.x1 - c.x0) * 0.25;
    assert_eq!(v.hit(&buf, x, area.y + 1.0), Hit::Text(3));
    assert_eq!(
        v.hit(&buf, x, area.y + v.metrics().top_pad - 0.5),
        Hit::Text(3)
    );
    // Controlo: logo abaixo do respiro é ainda a linha 1, uma linha abaixo já é a 2.
    let l2 = buf.text().byte_of_line(1) + 3;
    let y2 = area.y + v.metrics().top_pad + v.metrics().line_h * 1.5;
    assert_eq!(v.hit(&buf, x, y2), Hit::Text(l2));
}

/// (d) O cursor e o ponto da calha da linha 1 ficam na fila dela, não no respiro.
#[test]
fn o_cursor_e_a_calha_da_linha_1_ficam_alinhados_com_o_texto() {
    let buf = Buffer::new(&texto(100, 40));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, VISTA);
    let d = Deco {
        problems: vec![Problem {
            line: 0,
            severity: Severity::Error,
            culprit: None,
            message: String::new(),
            on: String::new(),
            explanation: None,
        }],
        ..Deco::default()
    };
    pinta(&mut v, &buf, &mut ts, area, &d);
    let t0 = topo(&v, &buf, 0).expect("a linha 1");
    let caret = v.caret_rect(&buf, 0).expect("o cursor à vista");
    assert!(
        (caret.y - t0).abs() < 0.01,
        "cursor {} ≠ texto {t0}",
        caret.y
    );
    let (_, dy) = v.problem_dot(0).expect("o ponto da calha");
    let mid = t0 + v.metrics().line_h * 0.5;
    assert!((dy - mid).abs() < 0.01, "ponto {dy} ≠ meio da linha {mid}");
    // O clique na calha, no respiro, é a calha da linha 1.
    assert!(matches!(
        v.hit(&buf, area.x + v.metrics().gutter_w * 0.5, area.y + 1.0),
        Hit::Mark(0) | Hit::Gutter(0)
    ));
}

/// (e) Mostrar uma linha deixa-a INTEIRA à vista: a subir, com o respiro por cima; a descer, rente ao
/// fundo; de volta à linha 1, a rolagem é 0.
#[test]
fn mostrar_uma_linha_deixa_a_inteira_abaixo_do_respiro() {
    let (mut v, mut buf, mut ts, area) = pintada(400);
    let (pad, lh) = (v.metrics().top_pad, v.metrics().line_h);
    let tr = v.text_rect();
    let fundo = buf.text().byte_of_line(300);
    buf.set_selection(Selection::single(Range::cursor(fundo)));
    v.ensure_visible(&buf, fundo);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let c = v.caret_rect(&buf, fundo).expect("à vista");
    assert!(
        (c.y + lh - (tr.y + tr.h)).abs() < 1.0,
        "rente ao fundo: {}",
        c.y
    );
    let cima = buf.text().byte_of_line(250);
    v.ensure_visible(&buf, cima);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let c = v.caret_rect(&buf, cima).expect("à vista");
    assert!(
        (c.y - (tr.y + pad)).abs() < 1.0,
        "abaixo do respiro: {} {}",
        c.y,
        tr.y
    );
    v.ensure_visible(&buf, 0);
    assert_eq!(v.scroll().1, 0.0);
}

/// (f) A virtualização pinta só as filas que tocam a vista: a contagem é a da vista com o respiro; no
/// fim da rolagem a última linha acaba no fundo do texto.
#[test]
fn a_virtualizacao_conta_o_respiro() {
    let (mut v, buf, mut ts, area) = pintada(400);
    let (pad, lh) = (v.metrics().top_pad, v.metrics().line_h);
    let h = v.text_rect().h;
    let n = ((h - pad) / lh).ceil() as usize;
    assert_eq!(v.visible_lines(), 0..n);
    assert_eq!(v.cache().len(), n);
    v.scroll_by(1.0e9);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let ultimo = topo(&v, &buf, 399).expect("a última linha pintada");
    let tr = v.text_rect();
    assert!((ultimo + lh - (tr.y + tr.h)).abs() < 1.0, "{ultimo}");
    assert_eq!(v.visible_lines().end, 400);
}
