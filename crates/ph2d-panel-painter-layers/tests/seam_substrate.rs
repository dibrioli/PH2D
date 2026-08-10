//! **O SUBSTRATO é alcançável, em TODO meio — e as rows da aguada não vazaram para os outros.**
//!
//! As quatro condições que esta casa faz uma wave provar, uma gate cada: o controle EXISTE, é pintado
//! E registrado, o clique chega ao barramento, e a SEQUÊNCIA pousa no tool. Elas são independentes —
//! uma row pode pintar, registrar retângulo de hit e estar morta sob o mouse porque o `populate` nunca
//! lhe deu um `InteractiveState`, e pode estar viva e **reverter no quadro seguinte** porque o
//! `is_param_field` não a reclama (o defeito que o Enio reportou no Taper em 2026-08-08).
//!
//! ⚠️ **A quinta pergunta é só desta wave: a seção Paper mudou de dono.** Ela era `watercolor ||
//! wetpaint`; hoje é de todo meio, porque o Digital passou a LER o dente. Abrir uma seção inteira a
//! meios novos é o jeito mais barato de shipar controle morto em massa, então as três rows que **só a
//! aguada consome** (Color, Tooth, Mapping) têm gate de AUSÊNCIA no Digital com controle positivo na
//! aquarela — sem o positivo, um `paint` que falhasse por qualquer motivo faria a ausência passar por
//! vácuo.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::Tool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

/// Alta o bastante para a seção Paper caber sem rolagem — o painel é uma coluna e a Paper vive fundo
/// nela. Um viewport curto faria toda asserção de presença falhar por LAYOUT, não por fiação.
fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 6000.0) // LITERAL-PX-OK: fixture viewport
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

/// Drena o que o painel empurrou no barramento para dentro do tool — a última perna da costura.
fn pump(host: &mut MockPanelHost, tool: &mut PainterTool) {
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

fn tool_in(media: PaintMedia) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(media);
    t
}

/// **As duas rows do substrato existem, pintam e estão vivas nos QUATRO meios.**
///
/// O substrato é a superfície sob tudo, então o meio de pintura não pode decidir se ele é autorável —
/// e o Digital, que era exatamente o meio sem acesso a papel nenhum, é o motivo da wave.
///
/// **Mutação que tem de sangrar:** devolver o portão `if brush.watercolor || brush.wetpaint` à volta
/// de `paint_paper_section` — Digital e Impasto perdem as duas rows e o relevo fica inalcançável.
#[test]
fn the_substrate_rows_are_alive_in_every_medium() {
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let tool = tool_in(media);
        let (host, _st, rects) = painted(&tool);
        for (id, name) in [
            (core_ids::PAINTER_SUBSTRATE_RELIEF, "Relief"),
            (core_ids::PAINTER_SUBSTRATE_ROUGHNESS, "Roughness"),
        ] {
            assert!(
                rect_of(&rects, id).is_some(),
                "{media:?}: a seção Paper nunca pintou um retângulo de hit para `{name}`"
            );
            assert!(
                host.store().get(id).is_some(),
                "{media:?}: `{name}` pinta e registra hit mas não tem InteractiveState — morta sob o \
                 mouse"
            );
        }
    }
}

/// **As duas rows POUSAM no tool** — a condição que as outras não enxergam.
///
/// Uma row numérica é pintada por `paint_num_row`, que ESPELHA o valor do tool de volta a cada quadro:
/// uma row que o `is_param_field` não reclama fica pintada, viva, editável — e **reverte no instante em
/// que o artista solta**, porque o quadro seguinte reescreve por cima o valor que o tool nunca recebeu.
///
/// **Mutação que tem de sangrar:** tirar `PAINTER_SUBSTRATE_FIELDS` do `number_field::is_param_field`.
#[test]
fn the_two_substrate_rows_land_on_the_tool() {
    for (id, name, set, read) in [
        (
            core_ids::PAINTER_SUBSTRATE_RELIEF,
            "Relief",
            0.62_f64,
            (|t: &PainterTool| t.substrate_depth()) as fn(&PainterTool) -> f32,
        ),
        (
            core_ids::PAINTER_SUBSTRATE_ROUGHNESS,
            "Roughness",
            0.19,
            (|t: &PainterTool| t.substrate_roughness()) as fn(&PainterTool) -> f32,
        ),
    ] {
        let mut tool = tool_in(PaintMedia::Digital);
        let (mut host, mut st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, id).is_some(),
            "a row `{name}` não foi pintada no Digital"
        );
        host.store_mut().set_number_value(id, set);
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
        pump(&mut host, &mut tool);
        let got = read(&tool);
        assert!(
            (f64::from(got) - set).abs() < 1e-3,
            "`{name}` nunca chegou ao tool: pedi {set:.3} e o tool ainda lê {got:.3} — a row está \
             pintada e muda, então ela reverte no quadro seguinte"
        );
    }
}

/// **Subir o Relief sem papel escolhido ARMA um papel — pela ROW, não por uma chamada de teste.**
///
/// É o que torna a ordem das rows load-bearing: elas são pintadas ANTES do portão de `TextureKind::None`
/// justamente para existirem no estado em que não há papel. Se descessem para baixo do portão, o
/// armar-um-default viraria um guard que gesto nenhum alcança, e o interruptor acenderia sem mostrar
/// nada.
///
/// **Mutação que tem de sangrar:** mover as duas rows para depois do `if kind == TextureKind::None`.
#[test]
fn raising_the_relief_from_the_row_arms_a_paper() {
    let mut tool = tool_in(PaintMedia::Digital);
    assert_eq!(
        tool.brush_settings().paper_kind,
        0,
        "fixture: o pincel tem de começar SEM papel, senão este gate não testa nada"
    );
    let (mut host, mut st, rects) = painted(&tool);
    let id = core_ids::PAINTER_SUBSTRATE_RELIEF;
    assert!(
        rect_of(&rects, id).is_some(),
        "a row Relief não é alcançável quando não há papel — é exatamente aí que ela precisa estar"
    );
    host.store_mut().set_number_value(id, 1.0);
    host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    pump(&mut host, &mut tool);
    assert_ne!(
        tool.brush_settings().paper_kind,
        0,
        "ligar o relevo não armou papel nenhum — o interruptor acende e não mostra nada"
    );
}

/// **As rows que só a aguada consome NÃO vazaram para o Digital.**
///
/// `Color` (o fundo que a óptica da aquarela vê), `Tooth` (quanto o grão morde o wash) e `Mapping` (que
/// o substrato ignora por construção — ele força `Tiled`) não têm leitor nenhum fora da aguada. Abrir a
/// seção a todo meio sem gateá-las shiparia três controles mortos de uma vez.
///
/// O positivo é a metade que impede o vácuo: as MESMAS três têm de estar lá na aquarela.
///
/// **Mutação que tem de sangrar:** tirar o `if wash` de qualquer uma das três.
#[test]
fn the_wash_only_rows_do_not_leak_into_the_other_media() {
    let wash_only = [
        (core_ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB, "Color"),
        (core_ids::PAINTER_WATERCOLOR_PAPER_DEPTH, "Tooth"),
        (core_ids::PAINTER_WATERCOLOR_PAPER_MAPPING, "Mapping"),
    ];
    // O papel tem de estar ARMADO nos dois lados, senão o portão de `None` esconde as três por outro
    // motivo e a ausência no Digital seria verdadeira por acidente.
    let armed = |media| {
        let mut t = tool_in(media);
        t.set_substrate_depth(1.0);
        t
    };

    let (_h, _s, wc) = painted(&armed(PaintMedia::Watercolor));
    for (id, name) in wash_only {
        assert!(
            rect_of(&wc, id).is_some(),
            "fixture: a aquarela não pintou `{name}`, então a ausência no Digital não prova nada"
        );
    }
    let (_h, _s, digital) = painted(&armed(PaintMedia::Digital));
    for (id, name) in wash_only {
        assert!(
            rect_of(&digital, id).is_none(),
            "`{name}` foi pintada no Digital, onde nada a lê — controle morto"
        );
    }
    // E o controle positivo do outro lado: o que é do SUBSTRATO continua lá no Digital.
    assert!(
        rect_of(&digital, core_ids::PAINTER_SUBSTRATE_RELIEF).is_some(),
        "fixture: o painel do Digital não pintou a seção Paper, então as ausências acima são vácuo"
    );
}
