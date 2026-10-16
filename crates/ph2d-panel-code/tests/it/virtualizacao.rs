//! E4 — só as linhas VISÍVEIS se moldam, e a cache segue a vista, nunca o ficheiro.

use crate::{Deco, area_para, pinta, texto, ts};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{CodeView, FoldMark};

const LINHAS: usize = 7_500;
const VISTA: usize = 40;

#[test]
fn rolar_uma_linha_molda_no_maximo_uma() {
    let buf = Buffer::new(&texto(LINHAS, 60));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, VISTA);
    let d = Deco::default();
    pinta(&mut v, &buf, &mut ts, area, &d);
    let s0 = v.cache().shapes();
    assert_eq!(
        v.cache().len(),
        VISTA,
        "a cache tem as linhas visíveis, não o ficheiro"
    );
    v.scroll_by(v.metrics().line_h);
    pinta(&mut v, &buf, &mut ts, area, &d);
    assert!(
        v.cache().shapes() - s0 <= 1,
        "rolar uma linha moldou {}",
        v.cache().shapes() - s0
    );
    assert_eq!(v.cache().len(), VISTA);
    assert_eq!(v.visible_lines(), 1..VISTA + 1);
    // Controlo: rolar uma página inteira molda a página (o contador conta).
    let s1 = v.cache().shapes();
    v.scroll_by(v.page_rows() as f32 * v.metrics().line_h);
    pinta(&mut v, &buf, &mut ts, area, &d);
    assert_eq!(v.cache().shapes() - s1, VISTA as u64);
}

#[test]
fn editar_uma_linha_molda_so_ela() {
    let mut buf = Buffer::new(&texto(LINHAS, 60));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, VISTA);
    let d = Deco::default();
    pinta(&mut v, &buf, &mut ts, area, &d);
    let s0 = v.cache().shapes();
    let at = buf.text().byte_of_line(10) + 3;
    buf.set_selection(Selection::single(Range::cursor(at)));
    buf.insert("x");
    pinta(&mut v, &buf, &mut ts, area, &d);
    assert_eq!(v.cache().shapes() - s0, 1, "editar a linha 10 molda só ela");
    assert_eq!(v.cache().len(), VISTA);
    // Controlo: repintar sem mudar nada não molda.
    let s1 = v.cache().shapes();
    pinta(&mut v, &buf, &mut ts, area, &d);
    assert_eq!(v.cache().shapes(), s1);
}

#[test]
fn ensure_visible_mostra_a_ultima_linha() {
    let mut buf = Buffer::new(&texto(LINHAS, 60));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, VISTA);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let end = buf.text().len();
    assert!(
        !v.visible_lines().contains(&(LINHAS - 1)),
        "controlo: a última linha começa fora da vista"
    );
    buf.set_selection(Selection::single(Range::cursor(end)));
    v.ensure_visible(&buf, end);
    assert!(v.visible_lines().contains(&(LINHAS - 1)));
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    assert!(
        v.caret_rect(&buf, end).is_some(),
        "o cursor está à vista depois da pintura"
    );
    // E de volta ao topo.
    buf.set_selection(Selection::single(Range::cursor(0)));
    v.ensure_visible(&buf, 0);
    assert_eq!(v.visible_lines().start, 0);
}

#[test]
fn linhas_dobradas_nao_se_pintam_nem_ocupam_filas() {
    let buf = Buffer::new(&texto(100, 30));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 20);
    let d = Deco {
        folds: vec![FoldMark {
            start: 2,
            end: 50,
            folded: true,
        }],
        ..Deco::default()
    };
    pinta(&mut v, &buf, &mut ts, area, &d);
    let linhas: Vec<usize> = v
        .probe_columns()
        .iter()
        .map(|(c, _)| buf.text().line_of_byte(c.byte))
        .collect();
    assert!(
        linhas.iter().all(|l| !(3..=50).contains(l)),
        "nenhuma linha escondida foi pintada"
    );
    assert!(
        linhas.contains(&51) && linhas.contains(&2),
        "a linha depois da dobra sobe para a fila seguinte"
    );
    assert_eq!(v.cache().len(), 20);
    assert_eq!(v.visible_lines(), 0..20 + 48);
}
