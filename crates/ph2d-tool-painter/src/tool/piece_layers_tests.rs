//! Os gates da [`super`] — o painel de camadas sobre a pilha da peça 3D (`docs/3D/30` §4).
//!
//! ⚠️ Eles entram pela porta do painel (`handle_panel_event`, a mesma que o barramento chama) e
//! afirmam as DUAS metades: o pedido que sai para a escultura, e a pilha da TELA intacta.

use super::*;
use crate::ids::{self, PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::tool::{PanelEvent, Tool};

/// A pilha de uma peça: a base, uma de cima, e um ajuste HSB.
fn pilha_da_peca() -> (LayerStack, [RtLayerId; 3]) {
    let mut s = LayerStack::new();
    let base = s.add_raster("base", 1024, 3).expect("base");
    let cima = s.add_raster("cima", 1024, 3).expect("cima");
    let hsb = s
        .add_adjustment(AdjustmentKind::HueSaturationBrightness)
        .expect("hsb");
    s.set_active(cima);
    (s, [base, cima, hsb])
}

/// O Painter com a tela da vista presa e a pilha da peça espelhada.
fn na_peca() -> (PainterTool, [RtLayerId; 3]) {
    let mut t = PainterTool::default();
    assert!(t.bind_screen_canvas(64, 48));
    let (s, ids) = pilha_da_peca();
    t.sync_piece_layers(Some(&s));
    (t, ids)
}

/// ⭐⭐⭐ **GATE — com a tela presa o painel MOSTRA a pilha da peça, e cada gesto vira um pedido
/// para ela; a pilha da tela nunca muda.** CONTROLO: sem a tela, o mesmo «+» cresce a pilha do
/// documento e não pede nada.
#[test]
fn com_a_tela_presa_os_gestos_do_painel_vao_para_a_pilha_da_peca() {
    let (mut t, [base, cima, _]) = na_peca();
    let tela = t.layers().clone();
    assert!(t.panel_shows_the_piece());
    assert_eq!(
        t.panel_layers().map(LayerStack::len),
        Some(3),
        "o painel mostra a pilha da peça"
    );
    assert_eq!(t.panel_selection().into_iter().collect::<Vec<_>>(), [cima]);

    t.handle_panel_event(PanelEvent::Click(ids::PAINTER_LAYERS_ADD));
    t.handle_panel_event(PanelEvent::Click(ids::PAINTER_LAYERS_DUPLICATE));
    t.handle_panel_event(PanelEvent::Click(ids::PAINTER_LAYERS_DELETE));
    t.handle_panel_event(PanelEvent::Click(ids::PAINTER_LAYERS_MASK));
    t.handle_panel_event(PanelEvent::SelectOption(
        ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
        AdjustmentKind::ALL
            .iter()
            .position(|k| *k == AdjustmentKind::Invert)
            .expect("Invert")
            .to_string(),
    ));
    assert_eq!(
        t.take_piece_layer_ops(),
        [
            PieceLayerOp::NewLayer,
            PieceLayerOp::Duplicate(cima),
            PieceLayerOp::Delete(cima),
            PieceLayerOp::NewMask(cima),
            PieceLayerOp::NewAdjustment(AdjustmentKind::Invert),
        ]
    );
    assert_eq!(t.layers(), &tela, "a pilha da TELA não mexeu");

    // A linha da base: o metadado volta inteiro, com a activa nova e SEM arrasto.
    let row = painter_layer_widget_id(base.0, PainterLayerWidget::Row);
    t.handle_panel_event(PanelEvent::Click(row));
    match t.take_piece_layer_ops().as_slice() {
        [PieceLayerOp::Metadata { stack, gesture }] => {
            assert_eq!(stack.active(), Some(base));
            assert_eq!(*gesture, None, "um clique não é arrasto");
        }
        outro => panic!("o clique na linha pediu {outro:?}"),
    }
    assert_eq!(t.layers(), &tela);

    // ⛔ CONTROLO: sem a tela, o «+» é do documento.
    t.release_screen_canvas();
    assert!(!t.panel_shows_the_piece());
    assert!(
        t.take_piece_layer_ops().is_empty(),
        "soltar a tela larga os pedidos"
    );
    t.sync_piece_layers(Some(&pilha_da_peca().0));
    assert_eq!(
        t.panel_layers(),
        Some(t.layers()),
        "sem a tela, o painel mostra o documento"
    );
    t.set_source(vec![0; 16 * 16 * 4], 16, 16);
    let antes = t.layers().len();
    t.handle_panel_event(PanelEvent::Click(ids::PAINTER_LAYERS_ADD));
    assert_eq!(
        t.layers().len(),
        antes + 1,
        "o controlo: o «+» do 2D cresce o documento"
    );
    assert!(t.take_piece_layer_ops().is_empty());
}

/// ⭐⭐⭐ **GATE — um ARRASTO de opacidade é metadado com o id do controlo**: o espelho segue a cada
/// passo (o pedido seguinte do mesmo quadro parte dele) e o valor é o do 2D.
#[test]
fn arrastar_a_opacidade_pede_o_metadado_com_o_arrasto() {
    let (mut t, [_, cima, _]) = na_peca();
    let op = painter_layer_widget_id(cima.0, PainterLayerWidget::Opacity);
    t.handle_panel_event(PanelEvent::SetValue(op, 0.5));
    t.handle_panel_event(PanelEvent::SetValue(op, 0.25));
    let pedidos = t.take_piece_layer_ops();
    let ops: Vec<f32> = pedidos
        .iter()
        .map(|p| match p {
            PieceLayerOp::Metadata { stack, gesture } => {
                assert_eq!(*gesture, Some(op));
                stack.get(cima).expect("cima").opacity
            }
            outro => panic!("{outro:?}"),
        })
        .collect();
    assert_eq!(ops, [0.5, 0.25]);
    assert_eq!(
        t.panel_layers()
            .and_then(|s| s.get(cima))
            .map(|l| l.opacity),
        Some(0.25),
        "o espelho já mostra o valor"
    );
}

/// ⭐⭐ **GATE — os parâmetros de um ajuste na peça são os do 2D, ao bit**: o MESMO evento sobre a
/// pilha do documento e sobre a da peça dá os mesmos parâmetros.
#[test]
fn o_parametro_de_um_ajuste_na_peca_e_o_do_2d() {
    let (mut t, [_, _, hsb]) = na_peca();
    let slider = painter_layer_widget_id(hsb.0, PainterLayerWidget::AdjParam0);
    t.handle_panel_event(PanelEvent::SetValue(slider, 0.8));
    let pedidos: [PieceLayerOp; 1] = t.take_piece_layer_ops().try_into().expect("um pedido");
    let [PieceLayerOp::Metadata { stack, .. }] = pedidos else {
        panic!("não é metadado");
    };
    let peca = stack.get(hsb).map(|l| l.kind.clone());

    let mut doc = PainterTool::default();
    doc.set_source(vec![0; 16 * 16 * 4], 16, 16);
    let (mut s, _) = pilha_da_peca();
    std::mem::swap(&mut doc.layers, &mut s);
    let slider2 = painter_layer_widget_id(hsb.0, PainterLayerWidget::AdjParam0);
    doc.handle_panel_event(PanelEvent::SetValue(slider2, 0.8));
    assert_eq!(peca, doc.layers().get(hsb).map(|l| l.kind.clone()));
    assert_ne!(
        peca,
        pilha_da_peca().0.get(hsb).map(|l| l.kind.clone()),
        "o CONTROLO: o parâmetro mudou"
    );
}

/// ⭐ **GATE — o que a peça não oferece não pede nada** (o painel desliga-o; a ferramenta é a rede)
/// e não cai na pilha da tela.
#[test]
fn o_que_a_peca_nao_oferece_nao_pede_nada() {
    let (mut t, _) = na_peca();
    let tela = t.layers().clone();
    for ev in [
        PanelEvent::Click(ids::PAINTER_LAYERS_GROUP),
        PanelEvent::Click(ids::PAINTER_LAYERS_ALPHA_LOCK),
        PanelEvent::Click(ids::PAINTER_LAYERS_REFERENCE),
        PanelEvent::Click(ids::PAINTER_LAYERS_ADD_TEXTURE),
        PanelEvent::Click(ids::PAINTER_APPLY),
    ] {
        t.handle_panel_event(ev);
    }
    assert!(t.take_piece_layer_ops().is_empty());
    assert_eq!(t.layers(), &tela);
}

/// ⭐⭐ **GATE (W4) — a profundidade e o `Add`/`Level` de uma camada na peça são o metadado do 2D, ao
/// bit**: o arrasto pede-o com o id do gesto (um desfazer por arrasto), o chip sem; os valores são os
/// que o MESMO evento dá numa camada do documento 2D. CONTROLO: a pilha da TELA não muda.
#[test]
fn a_profundidade_e_o_level_na_peca_sao_os_do_2d() {
    let (mut t, [base, ..]) = na_peca();
    let tela = t.layers().clone();
    let fundo = painter_layer_widget_id(base.0, PainterLayerWidget::ImpastoDepth);
    let chip = painter_layer_widget_id(base.0, PainterLayerWidget::ImpastoLevel);
    t.handle_panel_event(PanelEvent::SetValue(fundo, 0.3));
    t.handle_panel_event(PanelEvent::Click(chip));
    let pedidos = t.take_piece_layer_ops();
    let [
        PieceLayerOp::Metadata {
            stack: a,
            gesture: ga,
        },
        PieceLayerOp::Metadata {
            stack: b,
            gesture: gb,
        },
    ] = pedidos.as_slice()
    else {
        panic!("dois pedidos de metadado: {pedidos:?}");
    };
    assert_eq!(
        (*ga, *gb),
        (Some(fundo), None),
        "o arrasto leva o gesto, o chip não"
    );
    assert_eq!(t.layers(), &tela, "a pilha da TELA não mexeu");

    let mut doc = PainterTool::default();
    doc.set_source(vec![0; 16 * 16 * 4], 16, 16);
    let l = doc.layers().active().expect("a camada do documento");
    let id2 = |w| painter_layer_widget_id(l.0, w);
    doc.handle_panel_event(PanelEvent::SetValue(
        id2(PainterLayerWidget::ImpastoDepth),
        0.3,
    ));
    let d2 = doc.layers().get(l).map(|c| c.impasto_depth);
    doc.handle_panel_event(PanelEvent::Click(id2(PainterLayerWidget::ImpastoLevel)));
    let m2 = doc.layers().get(l).map(|c| c.impasto_composite);
    let d = a.get(base).map(|c| c.impasto_depth);
    assert_eq!(
        d.map(f32::to_bits),
        d2.map(f32::to_bits),
        "a profundidade é a do 2D"
    );
    assert!(
        d.is_some_and(|d| (d + 0.4).abs() < 1e-6),
        "0,3 do curso é −0,4: {d:?}"
    );
    assert_eq!(
        b.get(base).map(|c| c.impasto_composite),
        m2,
        "o Level é o do 2D"
    );
    assert_eq!(m2, Some(crate::layers::ReliefComposite::Level));
}
