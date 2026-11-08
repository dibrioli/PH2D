//! **O Size de cada camada do Composite em px de diâmetro** (dono 08/11: *«Dentro de composite os sizes devem ser em
//! px»*). O motor guarda o multiplicador do raio; o chip mostra `round(2 · raio · efectivo)` e o que se escreve volta
//! como `px / (2 · raio)`. Pela porta real do painel e do tool, nos dois painéis.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::ids as p;
use ph2d_ui_testkit::MockPanelHost;

fn ferramenta(tamanho: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.toggle_composite();
    t.handle_panel_event(PanelEvent::Click(p::PAINTER_BRUSH_COMPOSITE_ADD));
    assert_eq!(t.composite_len(), 1, "CONTROLO: a camada nasceu");
    t.set_brush_size_px(20.0);
    t.set_brush_spacing(0.1);
    t.set_composite_layer_size(0, tamanho);
    t
}

fn painel(t: &PainterTool, cartoes: bool) -> (MockPanelHost, PainterLayersPanelState) {
    ph2d_panel_painter_layers::cards::force_cards_on_this_thread(Some(cartoes));
    set_current_brush(Some(t.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let j = Rect::new(0.0, 0.0, 1600.0, 6000.0);
    let _ = host.paint::<PainterLayersPanel>(&mut st, j);
    host.open_all_sections();
    for _ in 0..3 {
        let _ = host.paint::<PainterLayersPanel>(&mut st, j);
    }
    (host, st)
}

#[test]
fn o_size_da_camada_mostra_e_aceita_px_de_diametro() {
    let chip = p::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[0];
    for cartoes in [true, false] {
        // 1,5 × raio 20 ⇒ 60 px de diâmetro; a faixa vai do piso (2·20·0,1 = 4) ao tecto (2·20·4 = 160).
        let mut t = ferramenta(1.5);
        let (mut host, mut st) = painel(&t, cartoes);
        let (min, max, passo) = host.store().number_range(chip).expect("a faixa do chip");
        assert!(
            (min - 4.0).abs() < 1e-3 && (max - 160.0).abs() < 1e-3 && (passo - 1.0).abs() < 1e-9,
            "cartões={cartoes}: faixa {min}..{max} passo {passo}"
        );
        if cartoes {
            assert_eq!(
                host.store().number_value(chip),
                Some(60.0),
                "o chip mostra px"
            );
        }
        for ev in host.type_into_number(chip, "30") {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        let mut chegou = false;
        for a in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = a {
                chegou |= matches!(pe, PanelEvent::SetValue(id, _) if id == p::PAINTER_BRUSH_COMPOSITE_SIZE[0]);
                t.handle_panel_event(pe);
            }
        }
        assert!(
            chegou,
            "cartões={cartoes}: o chip escrito não chegou ao tool"
        );
        let s = t.brush_settings().composite_size[0];
        assert!(
            (s - 0.75).abs() < 1e-5,
            "cartões={cartoes}: 30 px ⇒ {s}, não 0,75"
        );
    }
}

/// O número é o que se PINTA: abaixo do piso do Spacing a camada pinta o piso, e o chip diz o piso.
#[test]
fn o_size_abaixo_do_piso_mostra_o_piso_em_px() {
    let t = ferramenta(0.0);
    let (host, _) = painel(&t, true);
    let chip = p::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[0];
    assert_eq!(host.store().number_value(chip), Some(4.0));
}
