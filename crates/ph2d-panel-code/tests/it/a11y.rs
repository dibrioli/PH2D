//! E12 — um `TextRun` por linha visível, com o texto exacto; rolar troca os nós.

use crate::{Deco, area_para, pinta, texto, ts};
use ph2d_a11y::{AkNodeId, NodeId, Role};
use ph2d_code_buffer::{Buffer, Range, Selection};
use ph2d_panel_code::{CodeView, RUN_CHARS};

const BASE: NodeId = NodeId(1_000);
const VISTA: usize = 30;

#[test]
fn um_no_por_linha_visivel_e_rolar_troca_os_nos() {
    let mut buf = Buffer::new(&texto(500, 40));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, VISTA);
    let cursor = buf.text().byte_of_line(5) + 2;
    buf.set_selection(Selection::single(Range::cursor(cursor)));
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let nos = v.a11y_nodes(&buf, BASE);
    assert_eq!(
        nos.len(),
        VISTA + 2,
        "o Document, as linhas e o contador da barra"
    );
    let (id, doc) = &nos[0];
    assert_eq!((*id, doc.role()), (BASE, Role::Document));
    let (contador, botao) = nos.last().unwrap();
    assert_eq!(botao.role(), Role::Button);
    assert_eq!(botao.label(), Some("Errors: 0, Warnings: 0"));
    let filhos: Vec<AkNodeId> = (0..VISTA)
        .map(|l| CodeView::a11y_run_id(BASE, l).into())
        .chain([AkNodeId::from(*contador)])
        .collect();
    assert_eq!(doc.children(), filhos.as_slice());
    let sel = doc.text_selection().expect("seleção");
    assert_eq!(
        sel.focus.node,
        AkNodeId::from(CodeView::a11y_run_id(BASE, 5))
    );
    assert_eq!(sel.focus.character_index, 2);
    for (k, (id, run)) in nos[1..=VISTA].iter().enumerate() {
        assert_eq!(*id, CodeView::a11y_run_id(BASE, k));
        assert_eq!(run.role(), Role::TextRun);
        assert_eq!(
            run.value(),
            Some(buf.text().line_string(k).as_str()),
            "o texto exacto da linha {k}"
        );
        assert_eq!(
            run.character_lengths().len(),
            buf.text().line_string(k).chars().count()
        );
    }
    // Controlo: rolar dez linhas troca os nós.
    v.scroll_by(10.0 * v.metrics().line_h);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let nos = v.a11y_nodes(&buf, BASE);
    assert_eq!(nos.len(), VISTA + 2);
    assert_eq!(nos[1].0, CodeView::a11y_run_id(BASE, 10));
    assert_eq!(nos[1].1.value(), Some(buf.text().line_string(10).as_str()));
    assert_eq!(
        nos[0].1.text_selection().map(|s| s.focus.node),
        Some(CodeView::a11y_run_id(BASE, 10).into()),
        "o cursor fora da vista prende-se à primeira linha"
    );
}

#[test]
fn linha_longa_corta_em_255() {
    let longa = "a".repeat(RUN_CHARS + 40);
    let buf = Buffer::new(&longa);
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 5);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let nos = v.a11y_nodes(&buf, BASE);
    assert_eq!(nos[1].1.value().map(str::len), Some(RUN_CHARS));
    assert_eq!(nos[1].1.character_lengths().len(), RUN_CHARS);
}

/// ⛔ Um grafema com mais de 255 bytes (uma letra com 200 acentos combinantes) não cabe num `u8`: a
/// soma dos `character_lengths` tem de dar o comprimento do `value` (o contrato do AccessKit), e a
/// posição do cursor conta as mesmas unidades.
#[test]
fn um_grafema_maior_que_255_bytes_nao_desalinha_os_comprimentos() {
    let gigante = format!("a{}", "\u{301}".repeat(200));
    let linha = format!("x{gigante}y");
    let mut buf = Buffer::new(&linha);
    buf.set_selection(Selection::single(Range::cursor(linha.len())));
    let (mut v, mut ts) = (CodeView::new(), ts());
    let area = area_para(&mut v, &buf, &mut ts, 5);
    pinta(&mut v, &buf, &mut ts, area, &Deco::default());
    let nos = v.a11y_nodes(&buf, BASE);
    let run = &nos[1].1;
    let valor = run.value().expect("texto");
    let soma: usize = run
        .character_lengths()
        .iter()
        .map(|&l| usize::from(l))
        .sum();
    assert_eq!(
        soma,
        valor.len(),
        "a soma dos comprimentos = os bytes do texto"
    );
    assert_eq!(valor, linha, "o texto inteiro está no nó");
    let foco = nos[0].1.text_selection().expect("seleção").focus;
    assert_eq!(
        foco.character_index,
        run.character_lengths().len(),
        "o cursor no fim = a última unidade"
    );
}
