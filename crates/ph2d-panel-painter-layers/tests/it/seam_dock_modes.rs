//! ⭐⭐⭐ **OS DOIS MODOS DO PAINTER SÃO UM GRUPO SEGMENTADO, E OS DOIS ESTÃO VIVOS SOB O DEDO.**
//!
//! > *«a aba ficou com nome de Layers em vez de algo como painter. Abaixo os modos Brush e Layers,
//! > as sub abas estão mal formatadas. Talvez fique melhor como Um toggle Button Group»* — Enio,
//! > 2026-09-09, com foto.
//!
//! ⛔ O que havia era **um botão** de `52 px` no cabeçalho, rotulado com o nome do **OUTRO** modo e
//! elidido a `"Lay…"` — ele pedia ao artista que lesse um rótulo cortado *e* que soubesse que o
//! rótulo era o destino, não o estado.
//!
//! ⚠️ **O clique é REAL** (`click_at` → despacho → painel → barramento → ferramenta): um
//! `Click(id)` sintético passaria com o segmento **morto sob o rato**, que é exactamente como a
//! caixa *Enable* do Wet Paint chegou a shipar inclicável.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_dock_shows_layers};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn painted(shows_layers: bool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_dock_shows_layers(shows_layers);
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

/// ⭐⭐ **A FILEIRA: dois segmentos, lado a lado, da mesma altura, por baixo do cabeçalho.**
#[test]
fn the_two_modes_are_a_segmented_row_under_the_header() {
    let (_h, _s, rects) = painted(true);
    let brush = rect_of(&rects, core_ids::PAINTER_SIDEBAR_TOGGLE_DOCK)
        .expect("o segmento «Brush» tem de ser pintado e registado");
    let layers = rect_of(&rects, core_ids::PAINTER_LAYERS_TOGGLE_DOCK)
        .expect("o segmento «Layers» tem de ser pintado e registado");

    assert!(
        brush.x < layers.x,
        "a ordem da fileira é Brush → Layers: {brush:?} contra {layers:?}"
    );
    assert!(
        (brush.y - layers.y).abs() < 0.5 && (brush.h - layers.h).abs() < 0.5,
        "os dois segmentos têm de partilhar a linha: {brush:?} contra {layers:?}"
    );
    assert!(
        (brush.w - layers.w).abs() < 1.5,
        "um grupo segmentado reparte a largura por igual — {:.1} contra {:.1}. ⛔ Era isto que o \
         botão antigo não fazia: ele tinha 52 px fixos e elidia o nome",
        brush.w,
        layers.w
    );

    // ⚠️ E o CLOSE continua no cabeçalho, ACIMA da fileira — a prova de que ela desceu.
    let close = rect_of(&rects, core_ids::PAINTER_LAYERS_CLOSE).expect("o fecho é pintado");
    assert!(
        close.y + close.h <= brush.y + 0.5,
        "a fileira de modos tinha de ficar ABAIXO do cabeçalho: fecho {close:?}, segmento {brush:?}"
    );
}

/// ⭐⭐⭐ **UM SEGMENTO ESCOLHE — tocar no que já está aceso não faz nada.**
///
/// ⛔ Era a lei que faltava: com um botão só, clicar era sempre INVERTER. Num grupo segmentado isso
/// faria a fileira piscar ao ser tocada duas vezes no mesmo sítio.
#[test]
fn a_segment_chooses_its_side_and_never_toggles() {
    let mut tool = PainterTool::default();

    // Está em Layers; tocar em «Layers» outra vez tem de ser um no-op.
    let (mut host, mut st, rects) = painted(true);
    let layers = rect_of(&rects, core_ids::PAINTER_LAYERS_TOGGLE_DOCK).expect("segmento Layers");
    tool.set_dock_shows_layers(true);
    click_through(
        &mut host,
        &mut st,
        &mut tool,
        layers.x + layers.w * 0.5,
        layers.y + layers.h * 0.5,
    );
    assert!(
        tool.dock_shows_layers(),
        "tocar no segmento JÁ ESCOLHIDO inverteu o modo — é um alternador, não um selector"
    );

    // E tocar no outro TROCA.
    let brush = rect_of(&rects, core_ids::PAINTER_SIDEBAR_TOGGLE_DOCK).expect("segmento Brush");
    click_through(
        &mut host,
        &mut st,
        &mut tool,
        brush.x + brush.w * 0.5,
        brush.y + brush.h * 0.5,
    );
    assert!(
        !tool.dock_shows_layers(),
        "controlo partido: o clique no outro segmento não trocou nada, logo o teste de cima não \
         mede um no-op — mede um caminho morto"
    );
}
