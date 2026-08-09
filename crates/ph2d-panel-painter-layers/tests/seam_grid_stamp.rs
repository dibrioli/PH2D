//! **O Grid Stamp tem controles, e eles são do Grid Stamp** (Enio, 2026-08-09).
//!
//! O método carimba a forma do pincel no centro da célula de uma grade própria, esticada para caber
//! nela. Isso pede quatro números (o tamanho da célula e o deslocamento da grade, um par por eixo) e
//! um interruptor que a desenha — e pede que eles **não existam** nos outros métodos, porque uma row
//! que não governa nada é exatamente o controle morto que a lei de visibilidade por-método deste
//! painel existe para impedir.
//!
//! Os gates são dirigidos por PONTEIRO pelo motivo que o irmão `seam_impasto_tool.rs` enuncia: um
//! widget que pinta, registra hit rect e é encaminhado pelo `event.rs` continua **morto sob o mouse**
//! sem um `InteractiveState` vindo do `populate`. **Um widget não está pronto quando PINTA. Está
//! pronto quando um teste CLICA nele.**

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{PainterTool, StrokeMethod};
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// As SEIS rows da grade, pelos ids que o painel de fato pinta.
///
/// ⚠️ A contagem se CONTA: o `Cell Fit` (Enio, 2026-08-09) entrou aqui no mesmo commit em que nasceu,
/// senão as duas metades deste arquivo — *existem no Grid Stamp* e *não existem fora dele* — passariam
/// a falar de uma lista menor que a que a tela desenha, e a row nova nasceria sem as duas.
fn grid_rows() -> [NodeId; 6] {
    [
        core_ids::PAINTER_BRUSH_GRID_CELL[0],
        core_ids::PAINTER_BRUSH_GRID_CELL[1],
        core_ids::PAINTER_BRUSH_GRID_OFFSET[0],
        core_ids::PAINTER_BRUSH_GRID_OFFSET[1],
        core_ids::PAINTER_BRUSH_GRID_FIT,
        core_ids::PAINTER_BRUSH_GRID_SHOW,
    ]
}

/// Um painter com `method` escolhido, com o snapshot publicado exatamente como o shell faz por frame.
fn tool_with(method: StrokeMethod) -> PainterTool {
    let mut tool = PainterTool::default();
    tool.set_brush_stroke_method(method.to_u8());
    set_current_brush(Some(tool.brush_settings()));
    tool
}

/// Pinta a vista do Brush e devolve o host (para clicar através), o estado e a lista id→rect.
fn painted(tool: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

/// O rect em que um widget foi pintado — `None` quando não foi pintado, que para esta seção é o ponto
/// inteiro. Área zero conta como ausente (um widget que o layout colapsou também não está na tela).
fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// Roda um clique REAL até o fim: despachante → painel → barramento → tool.
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

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// **O gate da PRESENÇA: as cinco rows existem no Grid Stamp.**
///
/// Mutação que tem de sangrar: tirar a chamada de `paint_grid_stamp_card` do `paint_stroke_section`.
/// Sem ela o método fica com uma grade que só o código conhece — o artista escolhe Grid Stamp e não
/// tem como dizer de que tamanho é a célula.
#[test]
fn the_grid_stamp_paints_its_five_rows() {
    let tool = tool_with(StrokeMethod::GridStamp);
    let (_host, _st, rects) = painted(&tool);
    for id in grid_rows() {
        assert!(
            rect_of(&rects, id).is_some(),
            "row da grade {id:?} não foi pintada no Grid Stamp — o método não tem controles"
        );
    }
}

/// **O gate da AUSÊNCIA: nenhum outro método as pinta.**
///
/// A grade é do Grid Stamp; oferecê-la sob Space ou Line seria um controle que não governa nada, que
/// é a lei que este painel honra em toda row por-método (Rate no Airbrush, Edge to Edge no Anchored).
/// Presença sozinha passaria com as rows pintadas sempre.
#[test]
fn no_other_method_paints_the_grid_rows() {
    for method in [
        StrokeMethod::Space,
        StrokeMethod::Dots,
        StrokeMethod::Airbrush,
        StrokeMethod::DragDot,
        StrokeMethod::Anchored,
        StrokeMethod::Line,
        StrokeMethod::Arc,
        StrokeMethod::Ellipse,
        StrokeMethod::Polygon,
        StrokeMethod::FreeHand,
    ] {
        let tool = tool_with(method);
        let (_host, _st, rects) = painted(&tool);
        for id in grid_rows() {
            assert!(
                rect_of(&rects, id).is_none(),
                "row da grade {id:?} pintada em {method:?} — controle morto: aquele método não tem \
                 grade nenhuma"
            );
        }
    }
}

/// **Show Grid responde ao mouse, e move só o desenho.**
///
/// As duas metades juntas de propósito: o clique tem de CHEGAR (populate → event → tool) e não pode
/// mexer no pincel. Mutação: tirar o id do `populate` — o retângulo continua pintado, o `event.rs`
/// continua encaminhando, e o clique morre no store por falta de `InteractiveState`.
#[test]
fn clicking_show_grid_flips_the_lattice_and_leaves_the_brush_alone() {
    let mut tool = tool_with(StrokeMethod::GridStamp);
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_BRUSH_GRID_SHOW)
        .expect("Show Grid não foi pintado no Grid Stamp");
    let before = tool.brush_settings();
    let was = tool.grid_show();
    let (x, y) = centre(r);
    click_through(&mut host, &mut st, &mut tool, x, y);
    assert_eq!(
        tool.grid_show(),
        !was,
        "o clique em Show Grid não chegou ao tool — a costura está morta"
    );
    let after = tool.brush_settings();
    assert_eq!(
        (after.grid_cell, after.grid_offset),
        (before.grid_cell, before.grid_offset),
        "Show Grid mexeu na grade — ele é DESENHO, não um parâmetro escondido do carimbo"
    );
}

/// **Cada slider escreve no seu próprio número.**
///
/// Um clique no MEIO de uma barra pede o meio do curso, então depois dele aquele valor tem de ter
/// mudado e **os outros três não**. É o defeito que um `idx()` trocado produz — e ele é mudo: a
/// célula sai quadrada quando devia ser retangular, ou o deslocamento anda no eixo errado.
///
/// Mutação: rotear `PAINTER_BRUSH_GRID_OFFSET` para `set_grid_cell_norm`. O laço sangra na 3ª volta.
#[test]
fn each_grid_slider_writes_only_its_own_number() {
    // (id, o que ele governa) — `None` = nenhum dos quatro, o que nenhum caso usa.
    let sliders = [
        (core_ids::PAINTER_BRUSH_GRID_CELL[0], 0usize),
        (core_ids::PAINTER_BRUSH_GRID_CELL[1], 1),
        (core_ids::PAINTER_BRUSH_GRID_OFFSET[0], 2),
        (core_ids::PAINTER_BRUSH_GRID_OFFSET[1], 3),
        (core_ids::PAINTER_BRUSH_GRID_FIT, 4),
    ];
    for (id, slot) in sliders {
        let mut tool = tool_with(StrokeMethod::GridStamp);
        let (mut host, mut st, rects) = painted(&tool);
        let r = rect_of(&rects, id).expect("slider da grade não foi pintado");
        let before = tool.brush_settings();
        let (_, y) = centre(r);
        // Bem à esquerda do trilho: um pedido inequívoco e longe de todo valor default.
        click_through(&mut host, &mut st, &mut tool, r.x + r.w * 0.1, y);
        let after = tool.brush_settings();
        let b = [
            before.grid_cell[0],
            before.grid_cell[1],
            before.grid_offset[0],
            before.grid_offset[1],
            before.grid_fit,
        ];
        let a = [
            after.grid_cell[0],
            after.grid_cell[1],
            after.grid_offset[0],
            after.grid_offset[1],
            after.grid_fit,
        ];
        assert_ne!(
            a[slot], b[slot],
            "arrastar {id:?} não moveu o número que ele nomeia — a costura está morta"
        );
        for other in 0..5 {
            if other != slot {
                assert!(
                    (a[other] - b[other]).abs() < 1e-6,
                    "arrastar {id:?} moveu o número {other} também — eixo/knob trocado no roteador"
                );
            }
        }
    }
}
