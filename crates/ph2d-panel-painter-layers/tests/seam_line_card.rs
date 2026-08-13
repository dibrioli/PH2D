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

/// **A row `Amount` existe SÓ com o `Speed` escolhido** — presença E ausência, porque um controle sob
/// um tipo que não o consome é um knob morto.
#[test]
fn the_amount_row_is_painted_only_for_the_speed_type() {
    let mut tool = PainterTool::default();
    let (_, _, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_LINE_SPEED_AMOUNT).is_none(),
        "o tipo None pintou a row Amount, que ele não consome"
    );

    tool.set_line_kind(1);
    let (_, _, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_LINE_SPEED_AMOUNT).is_some(),
        "o tipo Speed NÃO pintou a row Amount"
    );
}

/// **E o slider `Amount` é vivo sob o mouse** — arrastar a pista chega ao tool com o valor da FAIXA
/// (a fronteira de display: a pista anda `0..1` e o motor guarda QUADROS de antecipação).
#[test]
fn dragging_the_amount_slider_reaches_the_tool_in_frames() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(1);
    let (mut host, mut st, rects) = painted(&tool);
    let sl = rect_of(&rects, core_ids::PAINTER_LINE_SPEED_AMOUNT)
        .expect("o slider Amount não é pintado com o Speed escolhido");
    // Clica no meio da pista: `0.5` da faixa ⇒ metade do teto.
    let (cx, cy) = centre(sl);
    click_through(&mut host, &mut st, &mut tool, cx, cy);
    let a = tool.brush_settings().line_speed_amount;
    assert!(
        a > 0.1,
        "o clique na pista não chegou ao tool (Amount ficou em {a})"
    );
    assert!(
        a <= ph2d_tool_painter::MAX_SPEED_AMOUNT,
        "o Amount passou do teto medido: {a}"
    );
}
