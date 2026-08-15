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
            || (before.thread_width_px - after.thread_width_px).abs() > 1e-4
            || (before.thread_opacity - after.thread_opacity).abs() > 1e-4;
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

/// Os controles que o `Wire` acrescenta ao card (plano 38 W4).
const WIRE_SLIDERS: [(NodeId, &str); 3] = [
    (core_ids::PAINTER_LINE_WIRE_HISTORY, "History"),
    (core_ids::PAINTER_LINE_SKETCHY_WIDTH, "Line Width"),
    (core_ids::PAINTER_LINE_SKETCHY_OPACITY, "Opacity"),
];

/// **AS ROWS DO `Wire` SÓ EXISTEM COM ELE ESCOLHIDO — e as DUAS compartilhadas ficam no LUGAR.**
///
/// ⚠️ A `Line Width` e a `Opacity` são as mesmas rows nos dois tipos porque são o mesmo fato (a tinta
/// de um fio; ver `set_thread_width_norm`), e este gate é quem prova que trocar de tipo **não
/// embaralha o card**: o que muda é a lei, não onde os controles foram parar.
#[test]
fn the_wire_rows_exist_only_under_the_wire_type_and_share_the_ink_rows() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(3); // Wire
    let (_, _, on) = painted(&tool);
    for (id, name) in WIRE_SLIDERS {
        assert!(
            rect_of(&on, id).is_some(),
            "o slider {name} do Wire não é pintado"
        );
    }
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_WIRE_CONNECTION).is_some(),
        "o checkbox Connection Line não é pintado"
    );
    // O que é do OUTRO tipo não vaza para este.
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_SKETCHY_REACH).is_none(),
        "o Reach do Sketchy sobrevive no Wire — é um controle que não faz nada"
    );
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_SKETCHY_MAGNETIFY).is_none(),
        "o Magnetify sobrevive no Wire"
    );

    tool.set_line_kind(0); // None
    let (_, _, off) = painted(&tool);
    assert!(
        rect_of(&off, core_ids::PAINTER_LINE_WIRE_HISTORY).is_none(),
        "o History sobrevive ao tipo None"
    );
    assert!(
        rect_of(&off, core_ids::PAINTER_LINE_WIRE_CONNECTION).is_none(),
        "o Connection Line sobrevive ao tipo None"
    );
}

/// **O `History` ESTÁ VIVO SOB O MOUSE** — arrastado por um ponteiro REAL, do despachante ao tool.
#[test]
fn the_wire_history_slider_is_alive_under_the_pointer() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(3);
    let before = tool.brush_settings().wire_history;
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_LINE_WIRE_HISTORY).expect("o History não é pintado");
    let (x, y) = (r.x + r.w - 1.0, r.y + r.h * 0.5);
    for ev in host.drag_at(r.x + r.w * 0.5, y, x, y) {
        host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert!(
        (tool.brush_settings().wire_history - before).abs() > 1e-4,
        "arrastar o History não autorou nada no pincel"
    );
}

/// Os SEIS controles que a `Ribbon` acrescenta ao card (plano 38 W6).
///
/// ⚠️ **As duas últimas são as [`THREAD_INK_ROWS`] compartilhadas, e é POR ISSO que elas estão
/// aqui:** o trilho de fora e TODA travessa de uma faixa são FIOS, então `thread_width_px` e
/// `thread_opacity` decidem como a fita aparece. Elas shiparam alcançáveis só no Sketchy e no Wire
/// — um controle que governa o que se vê e vive noutro modo é um controle que o artista não tem.
const RIBBON_SLIDERS: [(NodeId, &str); 6] = [
    (core_ids::PAINTER_LINE_RIBBON_WEIGHT, "Weight"),
    (core_ids::PAINTER_LINE_RIBBON_FRICTION, "Friction"),
    (core_ids::PAINTER_LINE_RIBBON_GRAVITY, "Gravity"),
    (core_ids::PAINTER_LINE_RIBBON_RUNGS, "Rungs"),
    (core_ids::PAINTER_LINE_SKETCHY_WIDTH, "Line Width"),
    (core_ids::PAINTER_LINE_SKETCHY_OPACITY, "Opacity"),
];

/// **AS ROWS DA `Ribbon` SÓ EXISTEM COM ELA ESCOLHIDA — e a tinta de FIO está entre elas.**
///
/// ⚠️ **A metade da AUSÊNCIA carrega o peso aqui:** os quatro knobs da mola não podem vazar para os
/// outros tipos (nenhum deles tem uma mola), e o que é do Sketchy/Wire não pode aparecer sob a fita.
///
/// **Mutação que sangra:** tirar as `THREAD_INK_ROWS` do `RIBBON_SLIDERS` (as duas rows somem do
/// card e a fita volta a ser a única família de fio sem controlo sobre a própria tinta).
#[test]
fn the_ribbon_rows_exist_only_under_the_ribbon_type_and_carry_the_thread_ink() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(4); // Ribbon
    let (_, _, on) = painted(&tool);
    for (id, name) in RIBBON_SLIDERS {
        assert!(
            rect_of(&on, id).is_some(),
            "o slider {name} da Ribbon não é pintado"
        );
    }
    // O que é de OUTRO tipo não vaza para este.
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_SKETCHY_REACH).is_none(),
        "o Reach do Sketchy sobrevive na Ribbon — é um controle que não faz nada"
    );
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_WIRE_HISTORY).is_none(),
        "o History do Wire sobrevive na Ribbon"
    );

    // E os quatro knobs da mola não vazam para quem não tem mola.
    tool.set_line_kind(2); // Sketchy
    let (_, _, off) = painted(&tool);
    for (id, name) in RIBBON_SLIDERS.iter().take(4) {
        assert!(
            rect_of(&off, *id).is_none(),
            "o slider {name} da Ribbon sobrevive no Sketchy"
        );
    }
}

/// **CADA SLIDER DA `Ribbon` ESTÁ VIVO SOB O MOUSE, E AUTORA O SEU CAMPO** — arrastado por um
/// ponteiro REAL, do despachante ao tool.
///
/// ⚠️ **A fita shipou SEIS sliders sem nenhum gate os clicar**, que é literalmente a cicatriz que o
/// plano 38 registou uma wave antes (*"o card Line não tinha seam nenhum — id, row, `populate`,
/// encaminhamento e setter, e nenhum gate os exercitava"*): o seam nasceu, a wave seguinte
/// acrescentou quatro rows, e a lista do gate não as seguiu. Um gate por FAMÍLIA que não é estendido
/// com a família apodrece no mesmo lugar em que nasceu.
///
/// ⚠️ **Mutações que sangram:** tirar um id do `populate` (o slider pinta, registra hit e não recebe
/// arrasto) · tirá-lo da whitelist do `event_brush_forward` (o arrasto acontece e não sai do painel)
/// · tirar o braço do `trait_impls` (o valor chega e não escreve em nada).
#[test]
fn every_ribbon_slider_is_alive_under_the_pointer() {
    for (id, name) in RIBBON_SLIDERS {
        let mut tool = PainterTool::default();
        tool.set_line_kind(4);
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
        let moved = (before.ribbon_weight - after.ribbon_weight).abs() > 1e-4
            || (before.ribbon_friction - after.ribbon_friction).abs() > 1e-4
            || (before.ribbon_gravity - after.ribbon_gravity).abs() > 1e-4
            || (before.ribbon_rungs - after.ribbon_rungs).abs() > 1e-4
            || (before.thread_width_px - after.thread_width_px).abs() > 1e-4
            || (before.thread_opacity - after.thread_opacity).abs() > 1e-4;
        assert!(moved, "arrastar o slider {name} não autorou nada no pincel");
    }
}

/// Os TRÊS controles que o `Rough` acrescenta ao card (plano 38 W6).
const ROUGH_SLIDERS: [(NodeId, &str); 3] = [
    (core_ids::PAINTER_LINE_ROUGH_AMOUNT, "Roughness"),
    (core_ids::PAINTER_LINE_ROUGH_BOWING, "Bowing"),
    (core_ids::PAINTER_LINE_ROUGH_PASSES, "Passes"),
];

/// **AS ROWS DO `Rough` SÓ EXISTEM COM ELE ESCOLHIDO — e ele NÃO carrega a tinta de fio.**
///
/// ⚠️ **A metade da AUSÊNCIA é a que diz o desenho:** ao contrário do Sketchy, do Wire e da Ribbon,
/// este tipo **não costura nada** — ele desenha o TRAÇO outra vez, com os dabs do próprio pincel.
/// Um `Line Width` sob ele seriam duas rows que não fazem nada, e é exactamente o inverso do buraco
/// que a fita tinha.
#[test]
fn the_rough_rows_exist_only_under_the_rough_type_and_carry_no_thread_ink() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(5); // Rough
    let (_, _, on) = painted(&tool);
    for (id, name) in ROUGH_SLIDERS {
        assert!(
            rect_of(&on, id).is_some(),
            "o slider {name} do Rough não é pintado"
        );
    }
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_SKETCHY_WIDTH).is_none(),
        "o Rough pinta Line Width -- ele nao costura fio nenhum, e' uma row que nao faz nada"
    );
    assert!(
        rect_of(&on, core_ids::PAINTER_LINE_RIBBON_WEIGHT).is_none(),
        "o Weight da fita sobrevive no Rough"
    );

    tool.set_line_kind(0); // None
    let (_, _, off) = painted(&tool);
    for (id, name) in ROUGH_SLIDERS {
        assert!(
            rect_of(&off, id).is_none(),
            "o slider {name} sobrevive ao tipo None"
        );
    }
}

/// **CADA SLIDER DO `Rough` ESTÁ VIVO SOB O MOUSE, E AUTORA O SEU CAMPO.**
#[test]
fn every_rough_slider_is_alive_under_the_pointer() {
    for (id, name) in ROUGH_SLIDERS {
        let mut tool = PainterTool::default();
        tool.set_line_kind(5);
        let before = tool.brush_settings();
        let (mut host, mut st, rects) = painted(&tool);
        let r = rect_of(&rects, id).unwrap_or_else(|| panic!("o slider {name} não é pintado"));
        // Arrasta para o EXTREMO ESQUERDO: as `Passes` nascem em 2 e o topo da pista já é o teto,
        // então um arrasto para a direita não moveria o slider que o gate existe para exercitar.
        let (x, y) = (r.x + 1.0, r.y + r.h * 0.5);
        for ev in host.drag_at(r.x + r.w * 0.5, y, x, y) {
            host.apply_panel_event::<PainterLayersPanel>(&mut st, ev);
        }
        for action in host.drained_actions() {
            if let EditorAction::ToolPanelEvent(pe) = action {
                tool.handle_panel_event(pe);
            }
        }
        let after = tool.brush_settings();
        let moved = (before.rough_amount - after.rough_amount).abs() > 1e-4
            || (before.rough_bowing - after.rough_bowing).abs() > 1e-4
            || before.rough_passes != after.rough_passes;
        assert!(moved, "arrastar o slider {name} não autorou nada no pincel");
    }
}

/// **O `Connection Line` ALTERNA sob um clique REAL** — e volta.
#[test]
fn the_connection_line_checkbox_toggles_under_a_real_click() {
    let mut tool = PainterTool::default();
    tool.set_line_kind(3);
    let start = tool.brush_settings().wire_connection_line;
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_LINE_WIRE_CONNECTION)
        .expect("o Connection Line não é pintado");
    let (cx, cy) = centre(r);
    click_through(&mut host, &mut st, &mut tool, cx, cy);
    assert_eq!(
        tool.brush_settings().wire_connection_line,
        !start,
        "o clique não alternou o Connection Line"
    );
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_LINE_WIRE_CONNECTION).expect("sumiu ao alternar");
    let (cx, cy) = centre(r);
    click_through(&mut host, &mut st, &mut tool, cx, cy);
    assert_eq!(
        tool.brush_settings().wire_connection_line,
        start,
        "o segundo clique não voltou"
    );
}
