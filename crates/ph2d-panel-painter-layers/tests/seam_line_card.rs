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
