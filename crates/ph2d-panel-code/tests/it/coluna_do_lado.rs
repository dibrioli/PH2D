//! Smoke do dono 10/11: na Sheet, a coluna do lado (Call stack · Waiting here · N) com muitos à espera
//! passava por cima do pé. Agora rola por si: linhas inteiras, a barra da casa só se transborda, e
//! nada pinta nem recebe clique fora da caixa dela.

use ph2d_a11y::NodeId;
use ph2d_code_debugcard::{CardState, DebugCard, StackEntry, WhoTag};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::widget::{scrollbar_thumb_rect, scrollbar_track_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::CodeView;
use ph2d_panel_code::debug_card::{CardBox, CardClick, DebugCardView};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{DrawOp, Shape, VectorScene};

const ABA: Rect = Rect::new(0.0, 0.0, 900.0, 900.0);

fn parado(frames: usize, waiting: usize) -> DebugCard {
    DebugCard {
        state: CardState::Paused,
        line: 7,
        function: "update".into(),
        who: WhoTag {
            name: "Runner".into(),
            color: Some(1),
        },
        stack: (0..frames)
            .map(|i| StackEntry {
                function: format!("f{i}"),
                line: 10 + i as u32,
            })
            .collect(),
        waiting: (0..waiting)
            .map(|i| WhoTag {
                name: format!("Coin {}", i + 2),
                color: Some(i % 4),
            })
            .collect(),
        can_step: true,
        ..DebugCard::default()
    }
}

/// Pinta `c` encostado ao fundo da aba e devolve o que se gravou.
fn pinta(v: &mut DebugCardView, c: &DebugCard) -> Vec<DrawOp> {
    let h = v.height(c, ABA);
    let r = Rect::new(ABA.x, ABA.y + ABA.h - h, ABA.w, h);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::new());
    scene.record();
    v.paint(
        c,
        [NodeId(1), NodeId(2)],
        r,
        &mut CodeView::new(),
        &mut scene,
        &mut ts,
        Theme::default(),
    );
    scene.take_record()
}

fn lado(v: &DebugCardView) -> CardBox {
    v.painted.side.expect("a coluna do lado")
}

fn dentro(r: Rect, of: Rect) -> bool {
    let e = 0.5;
    r.y >= of.y - e && r.y + r.h <= of.y + of.h + e
}

fn meio(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

#[test]
fn muitos_a_espera_ficam_dentro_da_coluna_e_a_barra_aparece() {
    let mut v = DebugCardView::default();
    let ops = pinta(&mut v, &parado(1, 30));
    let b = lado(&v);
    let view = b.track.body;
    assert!(b.bar.is_some(), "transborda: a barra");
    assert!(b.max > 0.0);
    let n = v.painted.waiting.len();
    assert!(n > 0 && n < 30, "só os que cabem: {n}");
    assert!(v.painted.waiting.iter().all(|(_, r)| dentro(*r, view)));
    assert!(v.painted.stack.iter().all(|(_, r)| dentro(*r, view)));
    // Nada do que a coluna pinta (a linha acesa, a barra) passa da caixa: o pé.
    let (x0, x1) = (f64::from(b.rect.x), f64::from(b.rect.x + b.rect.w));
    let bottom = f64::from(view.y + view.h) + 0.5;
    let fills: Vec<f64> = ops
        .iter()
        .filter_map(|op| match op {
            DrawOp::Fill {
                path, transform, ..
            } => Some((*transform * path.clone()).bounding_box()),
            _ => None,
        })
        .filter(|bb| bb.x0 >= x0 - 0.5 && bb.x1 <= x1 + 0.5)
        .map(|bb| bb.y1)
        .collect();
    assert!(fills.len() >= 3, "a linha acesa e a barra: {fills:?}");
    assert!(fills.iter().all(|&y| y <= bottom), "{fills:?} > {bottom}");
    // Poucos: tudo à vista, sem barra.
    let mut v = DebugCardView::default();
    pinta(&mut v, &parado(1, 2));
    assert!(lado(&v).bar.is_none() && lado(&v).track.hit().is_none());
    assert_eq!(v.painted.waiting.len(), 2);
}

#[test]
fn a_roda_sobre_a_coluna_anda_linhas_inteiras_e_para_no_fim() {
    let (c, mut v) = (parado(1, 30), DebugCardView::default());
    pinta(&mut v, &c);
    let rh = ph2d_tokens::row_pitch_px();
    let (x, y) = meio(lado(&v).track.body);
    let first = v.painted.waiting[0].1.y;
    let table = v.table.scroll;
    assert!(v.wheel(x, y, -1.0));
    assert!((v.side_scroll - rh).abs() < 0.01, "um toque = uma linha");
    assert!((v.table.scroll - table).abs() < 0.01, "a tabela não rola");
    pinta(&mut v, &c);
    assert!(
        (first - v.painted.waiting[0].1.y - rh).abs() < 0.01,
        "subiu uma linha"
    );
    assert!(v.wheel(x, y, -1e6));
    pinta(&mut v, &c);
    let b = lado(&v);
    assert!((v.side_scroll - b.max).abs() < 0.01, "preso ao fim");
    assert!(((v.side_scroll / rh).round() * rh - v.side_scroll).abs() < 0.01);
    assert_eq!(
        v.painted.waiting.last().unwrap().0,
        "Coin 31",
        "o último à vista"
    );
    assert!(v.wheel(x, y, 1e6));
    assert!(v.side_scroll.abs() < 0.01, "e volta ao topo");
}

#[test]
fn so_as_chamadas_a_vista_recebem_o_clique() {
    let (c, mut v) = (parado(30, 0), DebugCardView::default());
    pinta(&mut v, &c);
    let view = lado(&v).track.body;
    let (line, r) = v.painted.stack[0];
    assert_eq!(
        v.press(r.x + 2.0, r.y + r.h * 0.5),
        Some(CardClick::Jump(line))
    );
    v.release();
    let shown = v.painted.stack.len();
    assert!(shown < 30);
    // Logo abaixo da caixa (o pé): a chamada que estaria lá não vai à linha.
    let below = (r.x + 2.0, view.y + view.h + 2.0);
    assert_ne!(
        v.press(below.0, below.1),
        Some(CardClick::Jump(10 + shown as u32))
    );
    v.release();
    // Rolada até ao fim, a última vai à sua linha.
    let (x, y) = meio(view);
    assert!(v.wheel(x, y, -1e6));
    pinta(&mut v, &c);
    let &(last, lr) = v.painted.stack.last().unwrap();
    assert_eq!(last, 39);
    assert_eq!(
        v.press(lr.x + 2.0, lr.y + lr.h * 0.5),
        Some(CardClick::Jump(39))
    );
}

#[test]
fn a_barra_da_coluna_arrasta_pela_casa_e_pede_o_ponteiro_vertical() {
    let (c, mut v) = (parado(1, 30), DebugCardView::default());
    pinta(&mut v, &c);
    let b = lado(&v);
    let hit = b.track.hit().expect("transborda");
    let track = scrollbar_track_rect(b.track.body);
    let thumb = scrollbar_thumb_rect(track, 0.0, b.track.content_h, b.track.visible_h);
    let (x, y0) = (hit.x + hit.w * 0.5, thumb.y + thumb.h * 0.5);
    assert_eq!(v.look(x, y0), Some(PointerLook::ResizeNs));
    assert_eq!(v.press(x, y0), Some(CardClick::Taken));
    assert!(v.bar_dragging());
    assert!(v.drag_to(x, y0 + (track.h - thumb.h) * 10.0));
    assert_eq!(v.look(0.0, 0.0), Some(PointerLook::ResizeNs), "a arrastar");
    pinta(&mut v, &c);
    assert!((v.side_scroll - b.max).abs() < 0.01, "o fim");
    assert!(v.release() && !v.bar_dragging());
    // Carregar no alto da trilha salta o polegar para lá.
    assert!(v.press(x, track.y + 1.0).is_some());
    v.release();
    pinta(&mut v, &c);
    assert!(v.side_scroll < b.max * 0.5);
}

#[test]
fn quando_a_lista_encolhe_a_rolagem_prende_e_outra_paragem_volta_ao_topo() {
    let mut v = DebugCardView::default();
    pinta(&mut v, &parado(1, 30));
    let (x, y) = meio(lado(&v).track.body);
    assert!(v.wheel(x, y, -1e6));
    let fim = v.side_scroll;
    pinta(&mut v, &parado(1, 20));
    let b = lado(&v);
    assert!(
        b.max < fim && (v.side_scroll - b.max).abs() < 0.01,
        "preso ao novo fim"
    );
    assert_eq!(v.painted.waiting.last().unwrap().0, "Coin 21");
    pinta(&mut v, &parado(1, 2));

    assert!(v.side_scroll.abs() < 0.01 && v.painted.waiting.len() == 2);
    // Rolada, e a paragem muda (outra linha): volta ao topo.
    pinta(&mut v, &parado(1, 30));
    assert!(v.wheel(x, y, -1e6) && v.side_scroll > 0.0);
    let mut outra = parado(1, 30);
    outra.line = 8;
    pinta(&mut v, &outra);
    assert!(v.side_scroll.abs() < 0.01, "paragem nova");
}
