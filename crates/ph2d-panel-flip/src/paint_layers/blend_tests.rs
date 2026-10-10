//! ⭐⭐ **A lista de BLEND de uma camada do Flip ROLA de ponta a ponta** — a prova partilhada sobre
//! a função real do passe diferido.

use super::*;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_ui_testkit::{MockPanelHost, prova_que_a_lista_longa_rola};

/// Uma janela BAIXA: o painel clampa a lista à janela, e os 22 modos não cabem.
const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1280.0,
    h: 360.0,
};
const LAYER: u64 = 7;

#[test]
fn a_lista_de_blend_rola_de_ponta_a_ponta() {
    let layout = HeroLayout::for_viewport(JANELA);
    let pending = PendingBlend {
        layer: LAYER,
        chip_rect: Rect::new(40.0, 60.0, 200.0, 24.0),
        cur: 0,
    };
    let opcoes: Vec<_> = (0..MAX_BLEND_MODES)
        .map(|m| ids::flip_layer_blend_option_id(LAYER, m))
        .collect();
    let dd = ids::flip_layer_widget_id(LAYER, FlipLayerWidget::Blend);
    let mut host = MockPanelHost::new();
    prova_que_a_lista_longa_rola(&mut host, dd, &opcoes, &mut |h| {
        h.paint_fn(&layout, |ctx| {
            paint_blend_popover(ctx, Theme::Forge, &pending)
        })
    });
}
