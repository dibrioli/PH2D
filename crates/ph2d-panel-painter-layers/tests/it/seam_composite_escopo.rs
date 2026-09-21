//! **O chip do ESCOPO da borracha é pintado, registado e ALCANÇÁVEL** — a costura inteira.
//!
//! Ordem do dono, 2026-09-20: *«uma opção em erase: se a borracha atua só no próprio traço do Brush
//! ou se ela apaga também a camada da imagem abaixo»*.
//!
//! ⛔⛔ **As três condições são INDEPENDENTES e esta casa já pagou a diferença sete vezes na
//! escultura:** um controlo *nunca pintado* e um *pintado e morto sob o dedo* dão ao artista o
//! MESMO report (*«o botão não funciona»*), e um clique sintético num id fixo passa com o chip
//! morto — só o gesto que percorre o painel REAL separa os dois. Este gate é o do meio: ele pinta
//! o cartão, procura o rect VIVO do chip e manda o `Click` pela porta do painel.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// Uma ferramenta com o composite ligado e a posição `0` na operação `op` (wire: `3` = Erase).
fn com_operacao(op: u8) -> PainterTool {
    let mut t = PainterTool::default();
    t.toggle_composite();
    // ⛔ A pilha nasce VAZIA desde 2026-09-21: a camada CRIA-SE, e a operação dela é escolhida na
    // criação (o chip que a ciclava saiu — com as quotas do dono, um ciclo livre torná-las-ia
    // mentira).
    t.acrescenta_camada(op);
    t.set_composite_layer_strength(0, 1.0);
    t
}

fn pinta(t: &PainterTool) -> (MockPanelHost, PainterLayersPanelState, Vec<(NodeId, Rect)>) {
    set_current_brush(Some(t.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    (host, st, rects)
}

fn vezes_pintado(t: &PainterTool) -> usize {
    let alvo = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE[0];
    let (_, _, rects) = pinta(t);
    rects
        .iter()
        .filter(|(id, r)| *id == alvo && r.w > 0.0 && r.h > 0.0)
        .count()
}

/// ⭐⭐⭐ **O chip aparece SÓ na borracha, e o CLIQUE nele chega à ferramenta.**
///
/// ⚠️ **As duas metades, e cada uma sozinha mente:** a positiva sozinha deixaria o chip pintado nas
/// quatro operações (um controlo morto em três delas, porque ali a pergunta não tem sujeito); a
/// negativa sozinha ficaria verde sobre um cartão que nunca o pinta — que é a ordem do dono
/// inalcançável.
#[test]
fn o_chip_do_escopo_e_pintado_so_na_borracha() {
    let mut erros: Vec<String> = Vec::new();
    for op in 0..ph2d_tool_painter::N_COMPOSITE_OPS as u8 {
        let n = vezes_pintado(&com_operacao(op));
        let e_borracha = op == 3; // o discriminante de `CompositeOp::Erase` no instantâneo
        if e_borracha && n != 1 {
            erros.push(format!("a borracha pinta o chip {n}× (tem de ser 1)"));
        }
        if !e_borracha && n != 0 {
            erros.push(format!(
                "a operação {op} pinta o chip {n}× — controlo morto"
            ));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join(" · "));
}

/// ⭐⭐ **O `Click` no chip atravessa o painel e chega à ferramenta, e ela CICLA o escopo.**
///
/// ⚠️ O gate fecha a corrente inteira: o painel encaminha (`PanelEvent::Click`), a ferramenta
/// roda o escopo, e a volta ao princípio prova que o ciclo é `% N_COMPOSITE_ERASE_SCOPES` e não um
/// `+1` que sai da faixa.
#[test]
fn o_clique_no_chip_cicla_o_escopo_da_borracha() {
    let id = ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE[0];
    let mut t = com_operacao(3);
    assert_eq!(
        t.brush_settings().composite_erase_scope[0],
        0,
        "CONTROLO: de fábrica a borracha come a imagem por baixo (o comportamento de sempre)"
    );
    let n = ph2d_tool_painter::N_COMPOSITE_ERASE_SCOPES;
    for volta in 1..=n {
        let (mut host, mut st, _) = pinta(&t);
        let outcome = host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(id));
        let chegou = host.drained_actions().iter().any(|a| {
            matches!(
                a,
                EditorAction::ToolPanelEvent(ph2d_editor_core::tool::PanelEvent::Click(i))
                    if *i == id
            )
        });
        assert!(
            outcome == EventOutcome::Consumed && chegou,
            "volta {volta}: o clique no chip morre dentro do painel (outcome {outcome:?})"
        );
        // ⚠️ Pela porta do contrato (`Tool::handle_panel_event`), nunca pelo `route_composite_event`
        // directo: a rota é UM dos catorze braços do despacho, e entrar abaixo dele afirmaria que a
        // lei existe sem afirmar que ela é ALCANÇADA.
        ph2d_editor_core::tool::Tool::handle_panel_event(
            &mut t,
            ph2d_editor_core::tool::PanelEvent::Click(id),
        );
        let esperado = (volta % n) as u8;
        assert_eq!(
            t.brush_settings().composite_erase_scope[0],
            esperado,
            "volta {volta}: o escopo não ciclou para {esperado}"
        );
    }
}
