//! ⭐⭐⭐ **O PAINEL EM MODO PEÇA 3D** (`docs/3D/30` §4, W3): o que a peça ainda não oferece é
//! pintado APAGADO e não fica registado — não há clique mudo —, o menu de ajustes apaga os que leem a
//! vizinhança, e fora da peça tudo volta (o CONTROLO de cada gate).
//!
//! ⚠️ Lê o que a PINTURA registou (`MockPanelHost::paint`), que é o que o rato pode alcançar.

use ph2d_a11y::NodeId;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::state::PainterLayersPanelState;
use ph2d_panel_painter_layers::{
    PainterLayersPanel, set_current_dock_shows_layers, set_current_layers,
    set_current_layers_on_piece, set_current_selection,
};
use ph2d_tool_painter::ids::{self as ids, painter_adjustment_kind_option_id};
use ph2d_tool_painter::{AdjustmentKind, LayerId, LayerStack};
use ph2d_ui_testkit::MockPanelHost;

/// A pilha de uma peça: a base e uma camada de cima; `activa` escolhe qual.
fn pilha(activa_base: bool) -> (LayerStack, LayerId) {
    let mut s = LayerStack::new();
    let base = s.add_raster("base", 1024, 3).expect("base");
    let cima = s.add_raster("cima", 1024, 3).expect("cima");
    s.set_active(if activa_base { base } else { cima });
    (s, base)
}

fn registados(na_peca: bool, s: LayerStack, menu_aberto: bool) -> Vec<NodeId> {
    set_current_dock_shows_layers(true);
    set_current_selection(s.active().into_iter().collect());
    set_current_layers(Some(s));
    set_current_layers_on_piece(na_peca);
    let mut host = MockPanelHost::with_panel_and_shared_chrome::<PainterLayersPanel>();
    if menu_aberto {
        host.set_dropdown_open(ids::PAINTER_LAYERS_ADD_ADJUSTMENT, true);
    }
    let regs = host.paint::<PainterLayersPanel>(
        &mut PainterLayersPanelState,
        Rect::new(0.0, 0.0, 1600.0, 900.0),
    );
    set_current_layers_on_piece(false);
    regs.into_iter()
        .filter(|(_, r)| r.w > 0.0 && r.h > 0.0)
        .map(|(id, _)| id)
        .collect()
}

/// ⭐⭐⭐ **GATE — na peça, o que ela não oferece não é clicável**: grupo, textura, Lock, Ref e o
/// Apply ficam fora; apagar a BASE também. CONTROLO: fora da peça estão todos.
#[test]
fn na_peca_o_que_ela_nao_oferece_nao_e_clicavel() {
    let oferecidos = [
        ids::PAINTER_LAYERS_ADD,
        ids::PAINTER_LAYERS_DUPLICATE,
        ids::PAINTER_LAYERS_MASK,
        ids::PAINTER_LAYERS_CLIP,
        ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
    ];
    let desligados = [
        ids::PAINTER_LAYERS_GROUP,
        ids::PAINTER_LAYERS_ADD_TEXTURE,
        ids::PAINTER_LAYERS_ALPHA_LOCK,
        ids::PAINTER_LAYERS_REFERENCE,
        ids::PAINTER_APPLY,
        ids::PAINTER_LAYERS_DELETE,
    ];
    let na_base = registados(true, pilha(true).0, false);
    for id in oferecidos {
        assert!(
            na_base.contains(&id),
            "{id:?} serve na peça e não está clicável"
        );
    }
    for id in desligados {
        assert!(
            !na_base.contains(&id),
            "{id:?} não serve na peça e está clicável"
        );
    }
    // Apagar serve numa camada que não é a base.
    assert!(registados(true, pilha(false).0, false).contains(&ids::PAINTER_LAYERS_DELETE));

    // ⛔ CONTROLO: fora da peça, tudo.
    let doc = registados(false, pilha(true).0, false);
    for id in oferecidos.into_iter().chain(desligados) {
        assert!(doc.contains(&id), "o CONTROLO: no 2D {id:?} está clicável");
    }
}

/// ⭐⭐⭐ **GATE — na peça o menu de ajustes APAGA os que leem a vizinhança** (o desfoque não se
/// clica; o Inverter sim). CONTROLO: fora da peça o desfoque clica-se.
#[test]
fn na_peca_o_menu_de_ajustes_apaga_os_que_leem_a_vizinhanca() {
    let opcao = |k: AdjustmentKind| {
        let i = AdjustmentKind::ALL
            .iter()
            .position(|x| *x == k)
            .expect("o tipo está no menu");
        painter_adjustment_kind_option_id(i as u8)
    };
    let (blur, invert) = (
        opcao(AdjustmentKind::GaussianBlur),
        opcao(AdjustmentKind::Invert),
    );
    let peca = registados(true, pilha(true).0, true);
    assert!(peca.contains(&invert), "o Inverter serve na peça");
    assert!(!peca.contains(&blur), "o desfoque está apagado na peça");
    let doc = registados(false, pilha(true).0, true);
    assert!(doc.contains(&blur), "o CONTROLO: no 2D o desfoque clica-se");
}
