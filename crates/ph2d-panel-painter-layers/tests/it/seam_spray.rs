//! **A row `Count` responde ao ponteiro** — o SPRAY do plano 38 W5.
//!
//! ⚠️ **Dirigido por PONTEIRO, não por evento sintético**, pelo motivo que este diretório já pagou:
//! um `WidgetEvent` construído à mão pula a checagem de FOCABILIDADE do store, então um slider que
//! pinta, publica retângulo de hit e é encaminhado pelo `event.rs` passa num gate sintético **e fica
//! morto sob o mouse** se o `populate` não lhe deu `InteractiveState` (as 36 células da matriz de
//! camadas da física, 2026-07-18).
//!
//! As quatro condições, uma por gate: o controle **existe** · é **pintado e registrado** · o gesto
//! **chega ao barramento** · e a **sequência leva a algum lugar** (o `spray_count` do tool muda).

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{PainterTool, SPRAY_COUNT_MAX};
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

/// Escoa os eventos que um gesto produziu até o tool, como o shell faz.
fn forward(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    events: Vec<ph2d_editor_core::interaction::WidgetEvent>,
) {
    for ev in events {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// **O card Jitter pinta a row `Count`, e ela é a PRIMEIRA dele.**
///
/// ⚠️ A posição é a feature, não decoração: o Count é o número que transforma as três rows abaixo
/// dele — Position, Scale, Rotation — de *tremer* em *espalhar*. Escondê-lo no fim faria o artista
/// achar o espalhamento antes de achar o que o multiplica.
#[test]
fn the_jitter_card_paints_the_count_row_first() {
    let tool = PainterTool::default();
    let (_host, _st, rects) = painted(&tool);
    let count = rect_of(&rects, core_ids::PAINTER_BRUSH_SPRAY_COUNT)
        .expect("a row Count não foi pintada — o card Jitter não a oferece");
    let position = rect_of(&rects, core_ids::PAINTER_BRUSH_JITTER)
        .expect("controle: a row Position tem de estar pintada no mesmo card");
    assert!(
        count.y < position.y,
        "o Count tem de abrir o card (Count y={}, Position y={})",
        count.y,
        position.y
    );
    // …e o chip numérico ao lado, que é onde o artista LÊ a contagem.
    assert!(
        rect_of(&rects, core_ids::PAINTER_BRUSH_SPRAY_COUNT_CHIP).is_some(),
        "o chip do Count não foi pintado — o número fica ilegível"
    );
}

/// **O gesto chega ao tool e a contagem SOBE.** Arrastar o slider até o fim da pista tem de deixar o
/// pincel no teto medido — as quatro condições fechadas de uma vez.
#[test]
fn dragging_the_count_slider_reaches_the_brush() {
    let mut tool = PainterTool::default();
    assert_eq!(
        tool.brush_settings().spray_count,
        1,
        "controle: o pincel nasce com uma marca por ponto do caminho"
    );
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_BRUSH_SPRAY_COUNT).expect("a row Count não pintou");
    let y = r.y + r.h * 0.5;
    let events = host.drag_at(r.x + 1.0, y, r.x + r.w - 1.0, y);
    assert!(
        !events.is_empty(),
        "o arrasto não produziu evento nenhum — o slider está morto sob o mouse (populate)"
    );
    forward(&mut host, &mut st, &mut tool, events);
    assert_eq!(
        tool.brush_settings().spray_count,
        SPRAY_COUNT_MAX,
        "arrastar até o fim da pista tem de pousar no teto medido"
    );
}

/// **O CHIP lê a contagem que o pincel guardou.** O chip não é um segundo número: ele é o link
/// mapped-integer do store projetado sobre a mesma pista, e é ele que o artista LÊ.
///
/// ⚠️ O gate fecha o ciclo pelo GESTO — arrasta, o tool guarda, e o chip mostra. Comparar as duas
/// fórmulas lado a lado seria o oráculo que usa a função sob teste para computar o que espera.
#[test]
fn the_chip_reads_the_count_the_brush_holds() {
    let mut tool = PainterTool::default();
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_BRUSH_SPRAY_COUNT).expect("a row Count não pintou");
    let y = r.y + r.h * 0.5;
    let events = host.drag_at(r.x + 1.0, y, r.x + r.w - 1.0, y);
    forward(&mut host, &mut st, &mut tool, events);
    let chip = host.store().get(core_ids::PAINTER_BRUSH_SPRAY_COUNT_CHIP);
    let shown = match chip {
        Some(ph2d_editor_core::interaction::InteractiveState::NumberInput { value, .. }) => *value,
        other => panic!("o chip do Count não é um NumberInput no store: {other:?}"),
    };
    #[allow(clippy::cast_precision_loss)]
    let want = f64::from(tool.brush_settings().spray_count);
    assert!(
        (shown - want).abs() < 0.5,
        "o chip escreve {shown} e o pincel guarda {want} — as duas metades do mapeamento divergiram"
    );
}
