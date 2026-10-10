//! ⭐⭐ **O menu «+ Adjustment» ROLA de ponta a ponta** — a prova partilhada sobre a função real que
//! todos os menus deste painel usam.

use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_tool_painter::AdjustmentKind;
use ph2d_tool_painter::ids::{PAINTER_LAYERS_ADD_ADJUSTMENT, painter_adjustment_kind_option_id};
use ph2d_ui_testkit::{MockPanelHost, prova_que_a_lista_longa_rola};

/// Uma janela BAIXA: o painel clampa a lista à janela, e as 24 espécies não cabem.
const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1280.0,
    h: 360.0,
};
const CHIP: Rect = Rect {
    x: 40.0,
    y: 60.0,
    w: 240.0,
    h: 24.0,
};

#[test]
fn o_menu_de_ajustes_rola_de_ponta_a_ponta() {
    let layout = HeroLayout::for_viewport(JANELA);
    let opcoes: Vec<_> = (0..AdjustmentKind::ALL.len())
        .map(|i| painter_adjustment_kind_option_id(i as u8))
        .collect();
    let mut host = MockPanelHost::new();
    prova_que_a_lista_longa_rola(
        &mut host,
        PAINTER_LAYERS_ADD_ADJUSTMENT,
        &opcoes,
        &mut |h| {
            h.paint_fn(&layout, |ctx| {
                crate::adjust_menu::paint_adjustment_menu_popover(
                    ctx,
                    ph2d_tokens::Theme::Forge,
                    CHIP,
                );
            })
        },
    );
}
