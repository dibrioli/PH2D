//! B39 (UI que mente): a barra, a cor e a prévia dizem o que o motor tem.

use ph2d_editor_core::interaction::WidgetStore;

/// B39.4 (DF-77): a bare slider (layer Opacity, Impasto depth, adjustment bars) kept the value it
/// was first registered with — after an undo the widget still held the OLD value, so the a11y
/// reading, the keys and a relative drag started from what the layer no longer has.
#[test]
fn a_barra_nua_rele_o_valor_da_camada_depois_do_undo() {
    let mut store = WidgetStore::default();
    let id = ph2d_a11y::NodeId(0xB394);
    crate::paint_rows::register_bare_slider(&mut store, id, 0.3);
    // Undo: the layer is back at 0.7 and the panel republishes it.
    crate::paint_rows::register_bare_slider(&mut store, id, 0.7);
    let v = store.slider(id).map(|(_, v)| v);
    assert_eq!(v, Some(0.7), "the bar holds the value the layer HAS");
}

/// B39.4 (DF-78): the colour read-back compared the pick with the PUBLISHED colour, which arrives a
/// frame after the tool applies it — one pick went out twice (two undo steps for one gesture).
#[test]
fn uma_cor_escolhida_sai_uma_vez_enquanto_o_motor_nao_a_aplica() {
    use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
    use ph2d_ui_testkit::MockPanelHost;
    let sw = ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB;
    let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    host.store_mut().set_picker_target(Some(sw));
    host.store_mut().set_widget_color(sw, [10, 20, 30, 255]);
    let mut brush = crate::paint_brush::FALLBACK_BRUSH;
    brush.paper_color = [1.0, 1.0, 1.0];
    let quadro = |host: &mut MockPanelHost, brush| {
        let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 360.0, 800.0);
        let layout = ph2d_editor_core::screens::HeroLayout::for_viewport(viewport);
        let mut scene = ph2d_vector::VectorScene::new();
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        let mut ctx = PaintCtx {
            host,
            layout: &layout,
            slot: viewport,
            viewport,
            scene: &mut scene,
            text_system: &mut text,
        };
        crate::paint_watercolor_paper::paper_color_readback(&mut ctx, brush);
        ctx.host.bus_mut().drain().count()
    };
    // Two frames before the tool's colour comes back: ONE event.
    let n = quadro(&mut host, brush) + quadro(&mut host, brush);
    assert_eq!(n, 1, "one pick, one event");
    // Applied: nothing; a NEW pick goes out again.
    brush.paper_color = [10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0];
    assert_eq!(quadro(&mut host, brush), 0);
    host.store_mut().set_widget_color(sw, [40, 50, 60, 255]);
    assert_eq!(quadro(&mut host, brush), 1);
}

/// B39.4 sweep: the «sent once» memory outlived its gesture — the same colour picked again in a new
/// picker session (another document, after an undo) never went out. Opening the picker forgets it.
#[test]
fn abrir_o_seletor_de_novo_envia_a_mesma_cor_outra_vez() {
    let mut store = WidgetStore::default();
    let id = ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB;
    assert!(crate::cor_enviada::deve_enviar(id, "1,2,3", false));
    assert!(!crate::cor_enviada::deve_enviar(id, "1,2,3", false));
    crate::event::picker::abre(&mut store, id);
    assert!(
        crate::cor_enviada::deve_enviar(id, "1,2,3", false),
        "a new gesture sends its pick"
    );
}

/// B39.5 (DF-84): the layer row's grab rect used the height of the PREVIOUS frame — on the frame the
/// relief turned on, the relief band below the row was nobody's (the click fell through). The row is
/// measured in the same frame and registered beneath its children.
#[test]
fn ligar_o_relevo_mede_a_linha_da_camada_no_mesmo_quadro() {
    use ph2d_editor_core::zones::Rect;
    use ph2d_tool_painter::LayerStack;
    use ph2d_tool_painter::ids::{PainterLayerWidget as W, painter_layer_widget_id};
    crate::cards::force_cards_on_this_thread(Some(true));
    crate::state::set_current_dock_shows_layers(true);
    let mut stack = LayerStack::new();
    let ids = [
        stack.add_group("A").expect("layer"),
        stack.add_group("B").expect("layer"),
    ];
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::PainterLayersPanel>();
    let mut st = crate::PainterLayersPanelState;
    let vp = Rect::new(0.0, 0.0, 400.0, 4000.0);
    let mut quadro = |host: &mut ph2d_ui_testkit::MockPanelHost, stack: &LayerStack| {
        crate::state::set_current_layers(Some(stack.clone()));
        host.paint::<crate::PainterLayersPanel>(&mut st, vp)
    };
    // The card heights ride a spring stepped by the frame's `dt`: from the wall clock, 400 frames
    // settled at opt-level 0 and left the section 4 px tall at opt-level 1 (`ci-test`, no relief
    // bar painted). Settle at the real frame's ceiling, then measure ONE 60 Hz frame.
    use ph2d_editor_core::aquecimento::{PASSO_DE_QUADRO_S, com_passo_fixo};
    let _ = quadro(&mut host, &stack);
    host.open_all_sections();
    com_passo_fixo(0.1, || {
        for _ in 0..60 {
            let _ = quadro(&mut host, &stack);
        }
    });
    for id in ids {
        stack.get_mut(id).expect("layer").has_relief = true;
    }
    let regs = com_passo_fixo(PASSO_DE_QUADRO_S, || quadro(&mut host, &stack));
    crate::state::set_current_layers(None);
    crate::cards::force_cards_on_this_thread(None);
    let ultimo = |id| regs.iter().rev().find(|(i, _)| *i == id).map(|(_, r)| *r);
    let mut vistos = 0;
    for id in ids {
        let w = |k| painter_layer_widget_id(id.0, k);
        let (Some(linha), Some(relevo)) = (ultimo(w(W::Row)), ultimo(w(W::ImpastoDepth))) else {
            continue;
        };
        vistos += 1;
        assert!(
            linha.y + linha.h >= relevo.y + relevo.h,
            "the row's grab rect reaches the relief bar on the same frame ({linha:?} × {relevo:?})"
        );
    }
    assert!(
        vistos > 0,
        "a relief bar is painted on the frame it turns on"
    );
}
