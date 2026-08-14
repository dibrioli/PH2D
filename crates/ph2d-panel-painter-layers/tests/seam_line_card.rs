//! **O card Line responde ao ponteiro** — o dropdown `Type` e o slider `Amount` do `Speed`
//! (plano 38 W2).
//!
//! Os gates são **dirigidos por PONTEIRO** pelo motivo que este diretório já pagou várias vezes: um
//! widget que pinta, registra retângulo de hit e é encaminhado pelo `event.rs` continua **morto sob o
//! mouse** se o `populate` não lhe deu um `InteractiveState`. Um dropdown tem DOIS desses widgets — o
//! chip e, uma vez aberto, cada opção —, então os dois são clicados aqui em vez de sintetizados.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// Um clique REAL, do despachante ao tool.
fn click_through(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    x: f32,
    y: f32,
) {
    for ev in host.click_at(x, y) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// **O ponteiro escolhe cada tipo de linha** — o chip abre, a opção pinta, e clicá-la chega ao tool.
///
/// **Mutações que sangram:** tirar o passe diferido de popover do `paint_brush_popovers` (as opções
/// nunca chegam à tela) · tirar a row do `option_route` (a opção é clicada e decodifica para nada) ·
/// tirar o braço `SelectOption` do tool (o clique chega e não muda nada).
#[test]
fn a_pointer_can_pick_every_line_type() {
    for want in [1u8, 0u8] {
        let mut tool = PainterTool::default();
        // Começa de um tipo que NÃO é o alvo, para *"já estava escolhido"* não passar este gate.
        tool.set_line_kind(if want == 0 { 1 } else { 0 });
        let (mut host, mut st, rects) = painted(&tool);
        let chip = rect_of(&rects, core_ids::PAINTER_LINE_TYPE)
            .expect("o chip Type do card Line não é pintado");
        let (cx, cy) = centre(chip);
        click_through(&mut host, &mut st, &mut tool, cx, cy);

        // Re-pinta com o chip ABERTO: é este passe que põe as opções na tela.
        set_current_brush(Some(tool.brush_settings()));
        let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
        let opt =
            rect_of(&rects, core_ids::painter_line_type_option_id(want)).unwrap_or_else(|| {
                panic!("clicar o chip Type não o abriu — a opção {want} nunca chegou à tela")
            });
        let (ox, oy) = centre(opt);
        click_through(&mut host, &mut st, &mut tool, ox, oy);

        assert_eq!(
            tool.brush_settings().line_kind,
            want,
            "clicar a opção {want} não trocou o tipo de linha"
        );
    }
}

/// **E NÃO HÁ MAIS NADA A AJUSTAR NO `Speed`** — o card oferece o TIPO e mais nada.
///
/// ⚠️ O Alchemy não dá controle nenhum sobre o arremesso (Enio 2026-08-13: *"em alchemy o slider não
/// é necessário"*), então a antecipação é UMA constante de produto. Este gate é o que impede o
/// slider de voltar por hábito: um controle a mais aqui seria uma segunda resposta a *"quão longe a
/// tinta voa?"*, e a primeira é a lei `velocidade × antecipação`.
#[test]
fn the_speed_type_has_nothing_left_to_tune() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(1);
    let (_, _, rects) = painted(&tool);
    let rows = rects.iter().filter(|(_, r)| r.w > 0.0 && r.h > 0.0).count();
    assert!(rows > 0, "controle: o card Line tem de pintar alguma coisa");
    // O chip do tipo é o ÚNICO widget que o `Speed` acrescenta ao card.
    assert!(
        rect_of(&rects, core_ids::PAINTER_LINE_TYPE).is_some(),
        "o chip Type sumiu do card"
    );
    assert!(
        rect_of(&rects, core_ids::PAINTER_LINE_SOLID).is_some(),
        "o checkbox Solid sumiu do card"
    );
}

/// Os cinco controles que o `Sketchy` acrescenta ao card, e o que cada um autora no tool.
const SKETCHY_SLIDERS: [(NodeId, &str); 4] = [
    (core_ids::PAINTER_LINE_SKETCHY_REACH, "Reach"),
    (core_ids::PAINTER_LINE_SKETCHY_DENSITY, "Density"),
    (core_ids::PAINTER_LINE_SKETCHY_WIDTH, "Line Width"),
    (core_ids::PAINTER_LINE_SKETCHY_OPACITY, "Opacity"),
];

/// **AS ROWS DO `Sketchy` SÓ EXISTEM COM ELE ESCOLHIDO** — presença E ausência, que é o par que
/// prova que o card pinta a tabela do escopo e não a união de todas.
#[test]
fn the_sketchy_rows_exist_only_under_the_sketchy_type() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(2); // Sketchy
    let (_, _, on) = painted(&tool);
    for (id, name) in SKETCHY_SLIDERS {
        assert!(
            rect_of(&on, id).is_some(),
            "o slider {name} do Sketchy não é pintado"
        );
    }
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_SKETCHY_MAGNETIFY).is_some(),
        "o checkbox Magnetify não é pintado"
    );

    tool.set_line_kind(0); // None
    let (_, _, off) = painted(&tool);
    for (id, name) in SKETCHY_SLIDERS {
        assert!(
            rect_of(&off, id).is_none(),
            "o slider {name} sobrevive ao tipo None — é um controle que não faz nada"
        );
    }
    assert!(
        rect_of(&off, core_ids::PAINTER_LINE_SKETCHY_MAGNETIFY).is_none(),
        "o Magnetify sobrevive ao tipo None"
    );
}

/// **CADA SLIDER DO `Sketchy` ESTÁ VIVO SOB O MOUSE, E AUTORA O SEU CAMPO** — arrastado por um
/// ponteiro REAL, do despachante ao tool.
///
/// ⚠️ **Mutações que sangram:** tirar um id do `populate` (o slider pinta, registra hit e não recebe
/// arrasto) · tirá-lo da whitelist do `event_brush_forward` (o arrasto acontece e não sai do painel)
/// · tirar o braço do `trait_impls` (o valor chega e não escreve em nada).
#[test]
fn every_sketchy_slider_is_alive_under_the_pointer() {
    for (id, name) in SKETCHY_SLIDERS {
        let mut tool = PainterTool::default();
        tool.set_line_kind(2);
        let before = tool.brush_settings();
        let (mut host, mut st, rects) = painted(&tool);
        let r = rect_of(&rects, id).unwrap_or_else(|| panic!("o slider {name} não é pintado"));
        // Arrasta para o EXTREMO DIREITO da pista: o valor tem de subir para o topo da faixa.
        let (x, y) = (r.x + r.w - 1.0, r.y + r.h * 0.5);
        for ev in host.drag_at(r.x + r.w * 0.5, y, x, y) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                tool.handle_panel_event(pe);
            }
        }
        let after = tool.brush_settings();
        let moved = (before.sketchy_reach - after.sketchy_reach).abs() > 1e-4
            || (before.sketchy_density - after.sketchy_density).abs() > 1e-4
            || (before.sketchy_width_px - after.sketchy_width_px).abs() > 1e-4
            || (before.sketchy_opacity - after.sketchy_opacity).abs() > 1e-4;
        assert!(moved, "arrastar o slider {name} não autorou nada no pincel");
    }
}

/// **O `Magnetify` ALTERNA sob um clique REAL** — e volta, que é a metade que prova que o clique é
/// um toggle e não uma escrita fixa.
#[test]
fn the_magnetify_checkbox_toggles_under_a_real_click() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(2);
    let start = tool.brush_settings().sketchy_magnetify;
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_LINE_SKETCHY_MAGNETIFY)
        .expect("o Magnetify não é pintado");
    let (cx, cy) = centre(r);
    click_through(&mut host, &mut st, &mut tool, cx, cy);
    assert_eq!(
        tool.brush_settings().sketchy_magnetify,
        !start,
        "o clique não alternou o Magnetify"
    );
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_LINE_SKETCHY_MAGNETIFY).expect("sumiu ao alternar");
    let (cx, cy) = centre(r);
    click_through(&mut host, &mut st, &mut tool, cx, cy);
    assert_eq!(
        tool.brush_settings().sketchy_magnetify,
        start,
        "o segundo clique não voltou"
    );
}
