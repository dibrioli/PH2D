//! **O `+`, o menu e o `x` da pilha são pintados, registados e ALCANÇÁVEIS** — a costura inteira.
//!
//! Ordem do dono, 2026-09-21: *«a seção nasce sem nenhuma camada. Teremos um botão + para criar
//! camadas (as possibilidades aparecem no dropdown ao lado do +) … cada camada passa a ter um x
//! para ser retirada»*.
//!
//! ⛔⛔ **As condições são INDEPENDENTES** — o controlo existir, ser pintado, o clique chegar ao
//! barramento, e a SEQUÊNCIA levar a algum lado. Esta casa já pagou a diferença sete vezes na
//! escultura: *um controlo nunca pintado e um pintado e morto sob o dedo dão ao artista o MESMO
//! report*. Este gate manda os cliques pela porta do painel REAL.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// Pinta o painel com o estado desta ferramenta e manda um `Click` por ele. Devolve se o clique
/// **chegou ao barramento** como um `PanelEvent` para a ferramenta.
fn clica(t: &PainterTool, id: ph2d_a11y::NodeId) -> bool {
    set_current_brush(Some(t.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let outcome = host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(id));
    let chegou = host
        .drained_actions()
        .iter()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if *i == id));
    outcome == EventOutcome::Consumed && chegou
}

/// ⭐⭐⭐ **O clique no `+` atravessa o painel e CRIA uma camada.**
///
/// ⚠️ A sequência inteira: o painel encaminha, a ferramenta cria, e a pilha fica com mais uma —
/// *um clique que chega ao barramento e não muda nada é a metade que falta*.
#[test]
fn o_clique_no_mais_cria_uma_camada() {
    let mut t = PainterTool::default();
    t.toggle_composite();
    assert_eq!(t.composite_len(), 0, "CONTROLO: a pilha nasce vazia");

    let id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD;
    assert!(clica(&t, id), "o clique no `+` morre dentro do painel");
    // Pela porta do CONTRATO, nunca pela rota interna: ela é um dos braços do despacho, e entrar
    // abaixo dele afirmaria que a lei existe sem afirmar que ela é ALCANÇADA.
    t.handle_panel_event(PanelEvent::Click(id));
    assert_eq!(t.composite_len(), 1, "o `+` não criou a camada");
}

/// ⭐⭐⭐ **O clique no `x` atravessa o painel e RETIRA a camada.**
#[test]
fn o_clique_no_x_retira_a_camada() {
    let mut t = PainterTool::default();
    t.toggle_composite();
    t.handle_panel_event(PanelEvent::Click(
        ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD,
    ));
    assert_eq!(t.composite_len(), 1, "CONTROLO: há uma camada para tirar");

    let id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_REMOVE[0];
    assert!(clica(&t, id), "o clique no `x` morre dentro do painel");
    t.handle_panel_event(PanelEvent::Click(id));
    assert_eq!(t.composite_len(), 0, "o `x` não retirou a camada");
}

/// ⭐⭐ **Com a pilha CHEIA o `+` deixa de ser ALCANÇÁVEL, e a rota RECUSA** — *«ao usar todas
/// inativa-se o dropdown e o botão +»*.
///
/// ⚠️⚠️ **As duas metades medem coisas diferentes e a 1.ª redacção deste gate confundiu-as:** ela
/// mandava um `Click` sintético e esperava que ele morresse — e um `Click` sintético **não passa
/// pelo hit-test**, logo ele chega mesmo com o botão inerte. *Inércia prova-se pelo RECT que o
/// cartão regista; recusa prova-se pelo que a ferramenta FAZ.*
#[test]
fn com_a_pilha_cheia_o_mais_fica_inerte_e_a_rota_recusa() {
    let id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD;
    let registado = |t: &PainterTool| {
        set_current_brush(Some(t.brush_settings()));
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        host.paint::<PainterLayersPanel>(&mut st, viewport())
            .iter()
            .any(|(i, r)| *i == id && r.w > 0.0 && r.h > 0.0)
    };

    let mut t = PainterTool::default();
    t.toggle_composite();
    assert!(
        registado(&t),
        "CONTROLO: com a pilha vazia o `+` é alcançável"
    );

    // Encher pela quota: 3 Brush · 2 Erase · 1 Blur · 1 Smear.
    for op in [0u8, 0, 0, 3, 3, 2, 1] {
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
            op.to_string(),
        ));
        t.handle_panel_event(PanelEvent::Click(id));
    }
    assert_eq!(
        t.composite_len(),
        ph2d_tool_painter::N_COMPOSITE_LAYERS,
        "a fixtura não encheu a pilha, logo ela não contém o fenómeno"
    );
    assert!(
        !registado(&t),
        "com a pilha CHEIA o `+` continua a registar um rect — ele fica alcançável"
    );
    // E a rota recusa, mesmo que alguém lhe chegue por outro caminho.
    t.handle_panel_event(PanelEvent::Click(id));
    assert_eq!(
        t.composite_len(),
        ph2d_tool_painter::N_COMPOSITE_LAYERS,
        "a rota criou uma camada além da quota"
    );
}
