//! **O losango de cada campo da camada do Flip (plano 35 §A.8) é pintado e registado** — o olho, a
//! mistura, a opacidade e a profundidade, na coluna do decorator; o cadeado não tem (§8).
//!
//! A shell publica o mapa (`set_key_marks`, `ph2d_app_flip::marcas_camada`); a pintura regista o
//! losango pelo id derivado (`key_mark_id`) e o alvo diz os campos da linha
//! (`key_mark_target`) — o que o despacho entrega à shell no clique.

use std::collections::BTreeMap;

use ph2d_editor_core::widget::key_mark::{KeyMark, key_mark_id, key_mark_target, set_key_marks};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_flip::ids::{self, FlipLayerWidget};
use ph2d_panel_flip::state::FlipPanelState;
use ph2d_panel_flip::{FlipLayerRow, FlipLayersSnapshot, FlipPanel};
use ph2d_ui_testkit::MockPanelHost;

fn camada(id: u64, nome: &str) -> FlipLayerRow {
    FlipLayerRow {
        id,
        name: nome.to_owned(),
        blend: 0,
        opacity: 1.0,
        depth: 1.0,
        visible: true,
        locked: false,
    }
}

/// ⭐ Cada campo-canal da camada de cima tem losango sob o rato, com os campos da linha; o cadeado
/// e a camada que a shell não declarou não são alvo. (Mutação: tirar o `paint_decorator_for` do
/// olho ou do chip da mistura ⇒ o `expect` dispara.)
#[test]
fn cada_campo_da_camada_tem_o_seu_losango() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let (fundo, cima) = (3u64, 4u64);
    ph2d_panel_flip::set_current_flip_layers(FlipLayersSnapshot {
        rows: vec![camada(fundo, "Back"), camada(cima, "Top")],
        active: Some(cima),
    });
    let id = |k| ids::flip_layer_widget_id(cima, k);
    set_key_marks(BTreeMap::from([
        (id(FlipLayerWidget::Visibility), KeyMark::Chave),
        (id(FlipLayerWidget::Blend), KeyMark::Animado),
        (id(FlipLayerWidget::Opacity), KeyMark::Mudado),
        (id(FlipLayerWidget::OpacityNum), KeyMark::Mudado),
        (id(FlipLayerWidget::Depth), KeyMark::Livre),
        (id(FlipLayerWidget::DepthNum), KeyMark::Livre),
    ]));
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState::default();
    let pintado = host.paint::<FlipPanel>(
        &mut st,
        Rect::new(0.0, 0.0, 1600.0, 2400.0), // LITERAL-PX-OK: um viewport de teste alto
    );
    let alvo = |primeiro| {
        pintado
            .iter()
            .any(|(w, r)| *w == key_mark_id(primeiro) && r.w > 0.0)
    };
    for (k, o_que) in [
        (FlipLayerWidget::Visibility, "o olho"),
        (FlipLayerWidget::Blend, "a mistura"),
        (FlipLayerWidget::Opacity, "a opacidade"),
        (FlipLayerWidget::Depth, "a profundidade"),
    ] {
        assert!(alvo(id(k)), "o losango de {o_que} não foi registado");
    }
    assert_eq!(
        key_mark_target(key_mark_id(id(FlipLayerWidget::Opacity))).map(|c| c.fields),
        Some(vec![
            id(FlipLayerWidget::Opacity),
            id(FlipLayerWidget::OpacityNum)
        ]),
        "o slider e o chip são UMA linha, um losango"
    );
    assert_eq!(
        key_mark_target(key_mark_id(id(FlipLayerWidget::Blend))).map(|c| c.fields),
        Some(vec![id(FlipLayerWidget::Blend)])
    );
    assert!(!alvo(id(FlipLayerWidget::Lock)), "o cadeado não é canal");
    assert!(
        !alvo(ids::flip_layer_widget_id(
            fundo,
            FlipLayerWidget::Visibility
        )),
        "a camada que a shell não declarou não é alvo"
    );
    set_key_marks(BTreeMap::new());
}
