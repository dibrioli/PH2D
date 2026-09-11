//! **O RELEVO DO DEPÓSITO é alcançável na seção SHAPE, e o Shine só existe quando significa algo.**
//!
//! As quatro condições que esta casa faz uma wave provar, uma gate cada: o controle EXISTE, é pintado
//! E registrado, o clique chega ao barramento, e a SEQUÊNCIA pousa no tool. Elas são independentes —
//! uma row pode pintar, registrar retângulo de hit e estar morta sob o mouse porque o `populate` nunca
//! lhe deu um `InteractiveState`, e pode estar viva e **reverter no quadro seguinte** porque o
//! `is_param_field` não a reclama (o defeito que o Enio reportou no Taper em 2026-08-08).
//!
//! ⚠️ **A quinta pergunta é desta wave: o Shine é CONDICIONAL.** Ele só é oferecido com o Relief acima
//! de zero, porque um realce especular precisa de uma normal fora do plano — medido, sobre o papel nu
//! o Shine move **0,00** nível (o `⛔` do `substrate_relief.rs`) e sobre o relevo do depósito move até
//! 3,36. Uma row de ausência sem **controle positivo** passaria por vácuo (um `paint` que falhasse por
//! qualquer motivo satisfaz "não achei"), então os dois lados são afirmados na mesma gate.

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

/// Alta o bastante para a seção Shape caber sem rolagem — um viewport curto faria toda asserção de
/// presença falhar por LAYOUT, não por fiação.
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

fn digital() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    t
}

/// **O Relief está vivo onde o depósito deixa CORPO, e ausente onde não deixa — as duas metades.**
///
/// ⚠️ **A lista dos dois lados foi MEDIDA, não escolhida** (`film_probe::probe_the_film_across_the_media`),
/// e a primeira versão desta gate pedia os quatro meios: o que o relevo acrescenta é **Digital 14,46 ·
/// Impasto 1,21 · Watercolor 0,00 · Wet Paint 0,00**. A aguada e o fluido têm render próprio e nunca
/// cruzam o `derive_height` — o doc do `impasto_applies` já dizia isso (*"the wash short-circuits before
/// the height pass ever runs"*), que é por que o painel pergunta a ELE em vez de re-derivar a
/// disjunção. Os dois ausentes são o que impede esta wave de shipar dois controles mortos.
#[test]
fn the_deposit_rows_live_where_the_deposit_lays_body() {
    let id = core_ids::PAINTER_SHAPE_RELIEF;
    for media in [PaintMedia::Digital, PaintMedia::Impasto] {
        let mut tool = PainterTool::default();
        tool.set_paint_media(media);
        let (host, _st, rects) = painted(&tool);
        assert!(
            rect_of(&rects, id).is_some(),
            "{media:?}: a seção Shape nunca pintou um retângulo de hit para `Relief`"
        );
        assert!(
            host.store().get(id).is_some(),
            "{media:?}: `Relief` pinta e registra hit mas não tem InteractiveState — morta sob o mouse"
        );
    }
    for media in [PaintMedia::Watercolor, PaintMedia::WetPaint] {
        let mut tool = PainterTool::default();
        tool.set_paint_media(media);
        let (_h, _s, rects) = painted(&tool);
        assert!(
            rect_of(&rects, id).is_none(),
            "{media:?}: o relevo do depósito mede 0,00 aqui — a row seria controle morto"
        );
    }
}

/// **O Relief POUSA no tool** — a condição que as outras não enxergam.
///
/// Uma row numérica é pintada por `paint_num_row`, que ESPELHA o valor do tool de volta a cada quadro:
/// uma row que o `is_param_field` não reclama fica pintada, viva, editável — e **reverte no instante em
/// que o artista solta**.
///
/// **Mutação que tem de sangrar:** tirar `PAINTER_SHAPE_DEPOSIT_FIELDS` do `is_param_field`.
#[test]
fn the_deposit_relief_row_lands_on_the_tool() {
    let mut tool = digital();
    let id = core_ids::PAINTER_SHAPE_RELIEF;
    let (mut host, mut st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, id).is_some(),
        "a row `Relief` não foi pintada no Digital"
    );
    host.store_mut().set_number_value(id, 0.42);
    host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    pump(&mut host, &mut tool);
    assert!(
        (tool.shape_relief() - 0.42).abs() < 1e-3,
        "`Relief` não pousou no tool; leu {}",
        tool.shape_relief()
    );
}

/// **O Shine aparece com o relevo e SOME sem ele** — as duas metades, e a segunda com controle
/// positivo (a primeira É o positivo da segunda).
///
/// ⚠️ Sem relevo o Shine é knob morto por MEDIÇÃO, não por gosto: sobre o dente do papel um realce
/// move `0,00` nível, o que o `⛔` do `substrate_relief.rs` já mediu ao reprovar um realce próprio para
/// o papel. Oferecê-lo ali seria a row que desenha e não faz nada.
///
/// **Mutação que tem de sangrar:** pintar o Shine incondicionalmente.
#[test]
fn the_shine_row_appears_with_the_relief_and_not_without_it() {
    let shine = core_ids::PAINTER_SHAPE_SHINE;

    let bare = digital();
    let (_h, _s, rects) = painted(&bare);
    assert_eq!(
        bare.shape_relief(),
        0.0,
        "a fixture tem de nascer sem relevo"
    );
    assert!(
        rect_of(&rects, shine).is_none(),
        "sem relevo o Shine nao tem o que iluminar e nao pode ser oferecido"
    );

    let mut lit = digital();
    lit.set_shape_relief(0.5);
    let (host, _s, rects) = painted(&lit);
    assert!(
        rect_of(&rects, shine).is_some(),
        "com relevo o Shine tem de ser oferecido (o CONTROLE POSITIVO da metade acima)"
    );
    assert!(
        host.store().get(shine).is_some(),
        "`Shine` pinta e registra hit mas não tem InteractiveState — morta sob o mouse"
    );
}

/// **O Shine POUSA no MESMO campo que o card Material edita** — duas VISTAS, um valor.
///
/// **Mutação que tem de sangrar:** rotear o `PAINTER_SHAPE_SHINE` para um campo próprio.
#[test]
fn the_shine_row_lands_on_the_paints_material() {
    let mut tool = digital();
    tool.set_shape_relief(0.5);
    let id = core_ids::PAINTER_SHAPE_SHINE;
    let (mut host, mut st, rects) = painted(&tool);
    assert!(
        rect_of(&rects, id).is_some(),
        "a row `Shine` não foi pintada"
    );
    host.store_mut().set_number_value(id, 0.23);
    host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    pump(&mut host, &mut tool);
    assert!(
        (tool.brush_settings().impasto_shine - 0.23).abs() < 1e-3,
        "`Shine` tem de escrever o material da tinta; leu {}",
        tool.brush_settings().impasto_shine
    );
}
