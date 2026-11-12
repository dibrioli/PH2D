//! As barras da lista de completação e da página de ajuda arrastam-se como a do texto (a lei do
//! `OwnBar` da casa): no polegar agarra, na calha salta para debaixo do dedo, o arrasto segue o rato.
//! Sobre a barra da lista a linha por baixo não é escolhida nem aceite.

use crate::pagina::{LARGA, pagina, pinta, quadro};
use crate::{Deco, frame, ts};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::{
    COMPLETION_ROWS, CodeView, CompletionHit, CompletionRow, CompletionView, HelpView, Ink,
    ListKind,
};
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const TOTAL: usize = 30;

/// Pinta a lista de [`TOTAL`] linhas rolada a `top` e devolve o polegar.
fn lista(v: &mut CodeView, top: usize) -> Rect {
    let (buf, d) = (ph2d_code_buffer::Buffer::new("ph2d.\n"), Deco::default());
    let todas: Vec<_> = (0..TOTAL)
        .map(|i| CompletionRow {
            label: format!("member_{i}"),
            matched: Vec::new(),
            ink: Ink::Function,
            detail: None,
        })
        .collect();
    let mut spans = |r| crate::spans_de(&buf, r);
    let mut marcas = crate::marcas(&d);
    let mut f = frame(&buf, &d, &mut spans, &mut marcas);
    f.completion = Some(CompletionView {
        anchor: 5,
        rows: &todas[top..(top + COMPLETION_ROWS).min(TOTAL)],
        total: TOTAL,
        top,
        selected: top,
        width_chars: 40,
        doc: None,
        preview: &[],
        kind: ListKind::Completion,
    });
    let area = Rect::new(0.0, 0.0, 900.0, 500.0);
    v.paint(
        f,
        area,
        &mut VectorScene::new(),
        &mut ts(),
        &Theme::default(),
    );
    v.completion_bar().expect("a lista longa sem barra")
}

fn meio(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// *Mutações: `list_bar_press` devolve sempre `None` · `list_bar_drag` não actualiza a última linha
/// pedida · `completion_hit` ignora a barra.*
#[test]
fn a_barra_da_lista_arrasta_salta_na_calha_e_nao_escolhe_a_linha() {
    let mut v = CodeView::default();
    let mut top = 0;
    let bar = lista(&mut v, top);
    let (x, y) = meio(bar);
    assert_eq!(
        v.completion_hit(x, y),
        Some(CompletionHit::Body),
        "sobre a barra a linha por baixo seria aceite"
    );
    assert_eq!(v.list_bar_look(x, y), Some(PointerLook::ResizeNs));
    assert_eq!(v.list_bar_press(x, y), Some(0), "no polegar não salta");
    // Arrastar em passos pequenos soma o mesmo que um só: cada passo pede a diferença.
    let mut soma = 0;
    for k in 1..=8 {
        soma += v
            .list_bar_drag(y + k as f32 * 4.0)
            .expect("o arrasto perdeu-se");
    }
    assert!(soma > 0, "o arrasto não rolou a lista");
    let mut outro = CodeView::default();
    let _ = lista(&mut outro, 0);
    let _ = outro.list_bar_press(x, y);
    assert_eq!(outro.list_bar_drag(y + 32.0), Some(soma), "passos ≠ um só");
    top = (top as isize + soma) as usize;
    let desceu = lista(&mut v, top);
    assert!(
        desceu.y > bar.y,
        "o polegar não seguiu: {bar:?} → {desceu:?}"
    );
    // Até ao fim e além: a lista pára na última página.
    let fim = v.list_bar_drag(y + 10_000.0).expect("o arrasto perdeu-se");
    assert_eq!(top as isize + fim, (TOTAL - COMPLETION_ROWS) as isize);
    assert!(v.list_bar_release());
    assert_eq!(v.list_bar_drag(y), None, "largou e continua a arrastar");
    assert!(!v.list_bar_release());
    // Na calha por baixo do polegar: salta para debaixo do dedo.
    let mut w = CodeView::default();
    let bar = lista(&mut w, 0);
    let fundo = bar.y + bar.h * 3.0;
    let salto = w.list_bar_press(x, fundo).expect("a calha não agarra");
    assert!(salto > 0, "a calha não saltou");
    // Fora da barra a lista não é tomada.
    let mut z = CodeView::default();
    let bar = lista(&mut z, 0);
    assert_eq!(z.list_bar_press(bar.x - 20.0, y), None);
}

/// *Mutações: `bar_press` devolve sempre `false` · `bar_drag_to` não mexe na rolagem.*
#[test]
fn a_barra_da_pagina_arrasta_e_salta_na_calha() {
    let doc = pagina();
    let (mut v, mut ts) = (HelpView::new(), ts());
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    let bar = v.probe().bar.expect("a página longa sem barra");
    let (x, y) = meio(bar);
    assert_eq!(v.pointer_look(x, y), Some(PointerLook::ResizeNs));
    assert!(v.bar_press(x, y), "o polegar não agarra");
    assert_eq!(v.scroll(), 0.0, "no polegar não salta");
    assert!(v.bar_drag_to(y + 40.0));
    let rolou = v.scroll();
    assert!(rolou > 40.0, "o arrasto não seguiu o polegar: {rolou}");
    pinta(&mut v, &quadro(&doc), &mut ts, LARGA);
    let desceu = v.probe().bar.expect("a barra sumiu");
    assert!(
        desceu.y > bar.y,
        "o polegar não seguiu: {bar:?} → {desceu:?}"
    );
    assert!(v.bar_release());
    assert!(!v.bar_drag_to(y + 80.0), "largou e continua a arrastar");
    assert_eq!(v.scroll(), rolou);
    // Na calha, perto do fundo: salta para debaixo do dedo.
    let mut w = HelpView::new();
    pinta(&mut w, &quadro(&doc), &mut ts, LARGA);
    assert!(
        w.bar_press(x, LARGA.y + LARGA.h - 4.0),
        "a calha não agarra"
    );
    assert!(w.scroll() > rolou, "a calha não saltou: {}", w.scroll());
    assert!(!HelpView::new().bar_press(x, y), "sem página não há barra");
}
