//! **Uma camada também tem cor** (Enio, 2026-08-09).
//!
//! O checkbox que liga as cores da textura era gateado em `count <= 1`, então um sprite de UMA camada
//! não tinha como pintar com as próprias cores — a capacidade existia no motor (o modo liga um bit, e
//! `color_on` desligado já significa *"a camada pinta as cores que capturou"*) e a **UI dela era a
//! única coisa que faltava**. O bug era um controle ausente, não um mecanismo.
//!
//! ⚠️ **O rótulo muda com a contagem, o ESTADO não.** Um id novo seria uma segunda porta para o mesmo
//! bit; *"Per-Layer Color"* sobre uma camada nomeia uma divisão que não existe. Uma pergunta, um
//! estado, duas maneiras honestas de a fazer.
//!
//! Dirigido por PONTEIRO pelo motivo de sempre: **um widget não está pronto quando PINTA. Está pronto
//! quando um teste CLICA nele.**

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

/// Um painter com uma Shape de `layers` camadas instalada, publicada como o shell publica por frame.
fn tool_with_shape_layers(layers: usize) -> PainterTool {
    let (w, h) = (4u32, 4u32);
    let n = (w * h) as usize;
    let mut tool = PainterTool::default();
    if layers == 1 {
        // A rota do SPRITE plano — a que o "Use as Brush Shape" da hierarquia toma para um sprite que
        // não é o documento aberto, e a que o report nomeia.
        let mut px = vec![255u8; n * 4];
        for (i, p) in px.chunks_exact_mut(4).enumerate() {
            p[0] = (i * 11) as u8;
        }
        tool.set_brush_shape_image_rgba(&px, w, h, Some(7));
    } else {
        tool.set_brush_shape_layers(vec![(vec![200u8; n], w, h); layers]);
    }
    set_current_brush(Some(tool.brush_settings()));
    tool
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

fn click_through(
    host: &mut MockPanelHost,
    st: &mut PainterLayersPanelState,
    tool: &mut PainterTool,
    r: Rect,
) {
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<PainterLayersPanel>(st, ev);
    }
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
}

/// **O report: com UMA camada o checkbox existe e o clique chega ao motor.**
///
/// Mutação que tem de sangrar: o gate `count == 0` de volta a `count <= 1`.
#[test]
fn a_single_layer_shape_offers_the_texture_colours_and_the_click_lands() {
    let mut tool = tool_with_shape_layers(1);
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_SHAPE_PER_LAYER_COLOR)
        .expect("uma camada tambem tem cor — o checkbox tem de ser pintado");
    assert!(
        !tool.brush_settings().shape_per_layer_color,
        "controle: o modo nasce DESLIGADO, senao o clique abaixo nao prova nada"
    );
    click_through(&mut host, &mut st, &mut tool, r);
    assert!(
        tool.brush_settings().shape_per_layer_color,
        "o checkbox foi pintado e o clique nao chegou ao motor — controle morto sob o mouse"
    );
}

/// **As rows por-camada NÃO são oferecidas com uma camada só.**
///
/// *"Layer 1 Color"* seria uma segunda forma de dizer *"pinte com a cor do pincel"*, que é exatamente
/// o que desmarcar o checkbox acima já faz — o item de menu morto que este codebase evita mantendo
/// uma tabela por menu. Presença sozinha passaria com as rows pintadas sempre.
#[test]
fn a_single_layer_shape_does_not_offer_a_per_layer_row() {
    let mut tool = tool_with_shape_layers(1);
    tool.toggle_brush_shape_per_layer_color();
    let (_host, _st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::painter_shape_layer_color_check_id(0)).is_none(),
        "a row 'Layer 1 Color' foi pintada — ela duplica a cor do pincel numa camada so"
    );
}

/// **E com DUAS camadas nada regride:** o checkbox continua lá e as rows por-camada voltam.
///
/// ⚠️ Este é o CONTROLE da mudança inteira. Sem ele, "mostrar com uma camada" poderia ter sido
/// escrito trocando o gate por `count == 1` e o modo multi-camada teria morrido em silêncio.
#[test]
fn two_layers_still_get_the_per_layer_rows() {
    let tool = tool_with_shape_layers(2);
    let (_host, _st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_SHAPE_PER_LAYER_COLOR).is_some(),
        "o checkbox sumiu do caso multi-camada"
    );
    let mut on = tool;
    on.toggle_brush_shape_per_layer_color();
    let (_host, _st, rects) = painted(&on);
    assert!(
        rect_of(&rects, core_ids::painter_shape_layer_color_check_id(0)).is_some(),
        "as rows por-camada sumiram — o modo multi-camada perdeu os controles dele"
    );
}

/// **Sem Shape capturada, nenhum checkbox** — a lei do controle morto, na outra ponta.
#[test]
fn no_shape_no_texture_colour_checkbox() {
    let tool = PainterTool::default();
    let (_host, _st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, core_ids::PAINTER_SHAPE_PER_LAYER_COLOR).is_none(),
        "sem Shape capturada nao ha cor de textura para ligar, e o checkbox nao pode existir"
    );
}

/// **De onde a transparência vem** — o checkbox do report de 2026-08-09, e ele é oferecido só quando
/// há uma segunda lei para escolher.
///
/// Mutação que tem de sangrar: o gate `brush.shape_has_alpha_choice` no painel.
#[test]
fn the_alpha_source_checkbox_is_offered_with_a_choice_and_the_click_lands() {
    // Uma máscara CRUA (sem cor capturada): existe silhueta, mas não existe a outra lei.
    let mut bare = PainterTool::default();
    bare.set_brush_shape_layers(vec![(vec![200u8; 16], 4, 4)]);
    let (_h, _s, rects) = painted(&bare);
    assert!(
        rect_of(&rects, core_ids::PAINTER_SHAPE_ALPHA_FROM_IMAGE).is_none(),
        "sem RGB capturado a luminância não existe — o checkbox seria um controle morto"
    );

    let mut tool = tool_with_shape_layers(1);
    let (mut host, mut st, rects) = painted(&tool);
    let r = rect_of(&rects, core_ids::PAINTER_SHAPE_ALPHA_FROM_IMAGE)
        .expect("com cor capturada há duas leis — o checkbox tem de ser pintado");
    let before = tool.brush_shape_alpha_from_image();
    click_through(&mut host, &mut st, &mut tool, r);
    assert_eq!(
        tool.brush_shape_alpha_from_image(),
        !before,
        "o checkbox foi pintado e o clique não chegou ao motor"
    );
}

/// **Ele fica ACIMA do de cor** (Enio: *"um novo checkbox acima de use texture color"*), e a ordem
/// não é decoração: um decide o que a forma É, o outro com que tinta ela pinta.
#[test]
fn the_alpha_source_sits_above_the_texture_colour_checkbox() {
    let tool = tool_with_shape_layers(1);
    let (_h, _s, rects) = painted(&tool);
    let alpha = rect_of(&rects, core_ids::PAINTER_SHAPE_ALPHA_FROM_IMAGE).expect("alpha");
    let colour = rect_of(&rects, core_ids::PAINTER_SHAPE_PER_LAYER_COLOR).expect("cor");
    assert!(
        alpha.y < colour.y,
        "o checkbox da silhueta ({}) tem de vir antes do de cor ({})",
        alpha.y,
        colour.y
    );
}
