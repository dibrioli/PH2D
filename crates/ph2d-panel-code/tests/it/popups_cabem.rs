//! Auditoria de layout 11/11 dos popups do editor: a lista que não cabe nas sete linhas leva a barra
//! da casa, e o balão que não cabe inteiro acaba em «…» em vez de perder linhas caladas.

use crate::{Deco, frame};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{COMPLETION_ROWS, CodeView, CompletionRow, CompletionView, Ink, ListKind};
use ph2d_panel_code::{HintProbe, SignatureView};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn linhas(n: usize) -> Vec<CompletionRow> {
    (0..n)
        .map(|i| CompletionRow {
            label: format!("member_{i}"),
            matched: Vec::new(),
            ink: Ink::Function,
            detail: Some("(number, string) -> boolean".into()),
        })
        .collect()
}

/// A lista de `total` linhas rolada a `top`: o polegar da barra, e as linhas pintadas.
fn lista(total: usize, top: usize) -> (Option<Rect>, Vec<ph2d_panel_code::CompletionPaintedRow>) {
    let (buf, d) = (ph2d_code_buffer::Buffer::new("ph2d.\n"), Deco::default());
    let todas = linhas(total);
    let mut ts = TextSystem::without_system_fonts();
    let mut v = CodeView::default();
    let mut spans = |r| crate::spans_de(&buf, r);
    let mut marcas = crate::marcas(&d);
    let mut f = frame(&buf, &d, &mut spans, &mut marcas);
    f.completion = Some(CompletionView {
        anchor: 5,
        rows: &todas[top..(top + COMPLETION_ROWS).min(total)],
        total,
        top,
        selected: top,
        width_chars: 40,
        doc: None,
        preview: &[],
        kind: ListKind::Completion,
    });
    let area = Rect::new(0.0, 0.0, 900.0, 500.0);
    v.paint(f, area, &mut VectorScene::new(), &mut ts, &Theme::default());
    (
        v.completion_bar(),
        v.completion_painted().unwrap_or_default(),
    )
}

/// *Mutação: `bar_w` sempre `0` em `paint_completion` ⇒ a lista longa fica sem barra.*
#[test]
fn a_lista_longa_tem_barra_e_a_curta_nao() {
    let (bar, p) = lista(30, 0);
    let bar = bar.expect("a lista longa sem barra");
    assert_eq!(p.len(), COMPLETION_ROWS);
    let (topo, fundo) = (p[0].3.1, p[p.len() - 1].3.1);
    assert!(bar.h < fundo - topo, "o polegar é a lista inteira: {bar:?}");
    assert!(
        p.iter().all(|r| r.3.0 < bar.x),
        "a barra por cima das linhas: {bar:?}"
    );
    let (desceu, _) = lista(30, 30 - COMPLETION_ROWS);
    let desceu = desceu.expect("a barra sumiu no fim");
    assert!(
        desceu.y > bar.y,
        "o polegar não desceu: {bar:?} → {desceu:?}"
    );
    assert_eq!(lista(COMPLETION_ROWS, 0).0, None, "a lista curta com barra");
}

fn assinatura(area: Rect) -> Option<HintProbe> {
    let (buf, d) = (
        ph2d_code_buffer::Buffer::new("ph2d.set(\n"),
        Deco::default(),
    );
    let mut ts = TextSystem::without_system_fonts();
    let mut v = CodeView::default();
    let mut spans = |r| crate::spans_de(&buf, r);
    let mut marcas = crate::marcas(&d);
    let mut f = frame(&buf, &d, &mut spans, &mut marcas);
    let label = "function ph2d.set(entity: number, field: \"rotation\" | \"x\", value: number): ()";
    f.signature = Some(SignatureView {
        label,
        active: Some(18..32),
        doc: Some("The entity doc. Writes one pose field of the entity, right now, in this frame."),
    });
    v.paint(f, area, &mut VectorScene::new(), &mut ts, &Theme::default());
    v.hints_painted().1
}

/// *Mutação: tirar o `elide` da última linha em `paint_box` ⇒ as linhas cortadas somem caladas.*
#[test]
fn o_balao_que_nao_cabe_acaba_em_reticencias_e_fica_na_area() {
    let inteira = assinatura(Rect::new(0.0, 0.0, 420.0, 700.0))
        .expect("a assinatura não se pintou")
        .lines
        .len();
    let mut cortadas = 0;
    for h in (80..=400).step_by(4) {
        let area = Rect::new(0.0, 0.0, 420.0, h as f32);
        let Some(s) = assinatura(area) else { continue };
        assert!(
            s.rect.y >= area.y && s.rect.y + s.rect.h <= area.y + area.h,
            "{h}: {s:?}"
        );
        if s.lines.len() < inteira {
            cortadas += 1;
            let ultima = s.lines.last().unwrap();
            assert!(ultima.ends_with('…'), "{h}: cortada sem «…»: {:?}", s.lines);
        }
    }
    assert!(
        cortadas > 0,
        "controlo: nenhuma altura cortou as {inteira} linhas"
    );
}
