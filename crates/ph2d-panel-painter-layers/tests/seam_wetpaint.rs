//! **Wet Paint presence seams** — what the panel OFFERS in and around the wet mode.
//!
//! Two facts the 2026-07-21 smoke tripped on: the Enable checkbox has to be painted BOTH for the
//! plain brush (to arm) and inside the wet mode (to disarm — without it the artist cannot leave);
//! and the **Paper** section has to be offered in wet mode (W2.7 seeds the engine's tooth from the
//! Paper slot — with the section hidden there is no door to arm a paper, and the seam is
//! unreachable by hand). Presence AND absence are asserted: the plain brush shows no Paper (it
//! reads no substrate — Enio: "deve ser assim mesmo").

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::{PainterLayersPanelState, set_current_brush};
use ph2d_tool_painter::PainterTool;
use ph2d_ui_testkit::MockPanelHost;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 900.0)
}

fn painted(tool: &PainterTool) -> Vec<(NodeId, Rect)> {
    set_current_brush(Some(tool.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    host.paint::<PainterLayersPanel>(&mut st, viewport())
}

fn has(rects: &[(NodeId, Rect)], id: NodeId) -> bool {
    rects
        .iter()
        .any(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
}

/// The **Paint Mode** chip exists on BOTH sides of the arm — offered to enter wet (plain brush) and
/// to LEAVE it (hiding it there is "não consigo sair do modo wet" with different clothes). It replaced
/// the section's own Enable checkbox on 2026-07-22, and it inherits that checkbox's whole duty: the way
/// out has to be painted in the mode you need to get out of. Mutation that bleeds it: the chip painted
/// from inside a medium's section instead of above them all.
#[test]
fn the_paint_mode_chip_is_offered_to_arm_and_to_disarm() {
    let plain = PainterTool::default();
    assert!(
        has(&painted(&plain), core_ids::PAINTER_BRUSH_MEDIA),
        "the plain brush has no Paint Mode chip — no way to reach Wet Paint at all"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        has(&painted(&wet), core_ids::PAINTER_BRUSH_MEDIA),
        "the wet mode has no Paint Mode chip — the artist cannot leave"
    );
}

/// W2.7's door: the **Paper** section is offered in wet mode (the slot seeds
/// the engine's tooth) and stays hidden for the plain brush (no substrate to
/// read). Mutation that bleeds it: `|| brush.wetpaint` dropped from the
/// Paper gate in `paint_brush_sections`.
#[test]
fn the_paper_section_is_offered_in_wet_mode_and_hidden_for_the_plain_brush() {
    let plain = PainterTool::default();
    assert!(
        !has(&painted(&plain), core_ids::PAINTER_WATERCOLOR_PAPER_SECTION),
        "the plain brush must NOT offer Paper (deve ser assim mesmo — Enio)"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        has(&painted(&wet), core_ids::PAINTER_WATERCOLOR_PAPER_SECTION),
        "wet mode offers no Paper section — the W2.7 seam is unreachable by hand"
    );
}

/// W3: the seven curated knob rows are offered ONLY while armed — presence
/// (armed shows all seven) and absence (a knob for an engine that is not
/// running is a dead control wearing a live one's clothes). Mutation that
/// bleeds it: dropping the `if brush.wetpaint` gate around the rows (absence
/// half) or a row lost from the table (presence half).
#[test]
fn the_knob_rows_are_offered_only_while_armed() {
    let plain = painted(&PainterTool::default());
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let armed = painted(&wet);
    for id in core_ids::PAINTER_WETPAINT_FIELDS {
        assert!(
            has(&armed, id),
            "armed wet mode is missing a curated knob row ({id:?})"
        );
        assert!(
            !has(&plain, id),
            "the plain brush paints a wet knob for an engine that is not running ({id:?})"
        );
    }
}

/// W3 law #3: the **Watercolor section hides while Wet Paint is armed** — its optics reinterpret the
/// DIGITAL deposit and the fluid engine owns the wet one; two wet-media switches over one brush would
/// be two answers to "what does this stroke do".
///
/// ⚠️ Since 2026-07-22 this is enforced by the shape of the UI rather than by a special case: the media
/// are a four-way dropdown, so only the selected one's section is painted and the state this gate
/// defends against is unreachable. The gate stays because the LAW is what matters, not the mechanism —
/// and it now needs an explicit positive control (Watercolor selected), because the plain brush no
/// longer paints that section either. Mutation that bleeds it: the `match` in `paint_brush_sections`
/// falling through to paint every section.
#[test]
fn the_watercolor_section_hides_while_wet_is_armed() {
    let mut wc = PainterTool::default();
    wc.set_paint_media(ph2d_tool_painter::PaintMedia::Watercolor);
    assert!(
        has(&painted(&wc), core_ids::PAINTER_WATERCOLOR_SECTION),
        "positive control: selecting Watercolor must offer the Watercolor section"
    );
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    assert!(
        !has(&painted(&wet), core_ids::PAINTER_WATERCOLOR_SECTION),
        "wet mode still offers the Watercolor section — two wet-media switches over one brush"
    );
}

/// W3: a wet knob edit forwards through the panel's REAL event path (the
/// `is_param_field` ValueChanged arm) — the synthetic-event blind spot that
/// let the dead Enable checkbox ship is exactly what this refuses to repeat.
/// Mutation that bleeds it: `PAINTER_WETPAINT_FIELDS` dropped from
/// `number_field::is_param_field`.
#[test]
fn a_wet_knob_edit_forwards_through_the_panel() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::EventOutcome;
    use ph2d_editor_core::tool::PanelEvent;
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    set_current_brush(Some(wet.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport()); // registers the number chips
    let id = core_ids::PAINTER_WETPAINT_WATER;
    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "a wet knob edit died in the panel — the ValueChanged arm does not know the id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
        )),
        "the wet knob edit never forwarded as SetValue — dead under the mouse. drained = {actions:?}"
    );
}

/// Doc 21 (deposit-at-commit): the Method dropdown offers the FULL list
/// while Wet Paint is armed — every method authors a flat static preview
/// and the fluid receives the final dab list once at commit, so no method
/// is incompatible (the W3 narrowing rested on a refuted premise). Mutation
/// that bleeds it: re-adding the `b.wetpaint` narrowing arm to
/// `stroke_method_offer::offered_stroke_methods`.
#[test]
fn the_method_menu_offers_every_method_while_wet_is_armed() {
    use ph2d_panel_painter_layers::stroke_method_offer::offered_stroke_methods;
    let plain = PainterTool::default().brush_settings();
    let full = offered_stroke_methods(Some(&plain));
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let offered = offered_stroke_methods(Some(&wet.brush_settings()));
    assert_eq!(
        offered, full,
        "wet mode narrows the Method menu — deposit-at-commit made every method valid"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Doc 22 — the grown section: tools, tilt, canvas actions, Paper, Tuning.
// ─────────────────────────────────────────────────────────────────────────

/// Every doc-22 widget is offered while ARMED and absent for the plain brush
/// (presence AND absence — a control for an engine that is not running is a
/// dead control wearing a live one's clothes). Mutation that bleeds it: any
/// widget dropped from the armed branch, or painted outside it.
#[test]
fn every_doc22_wet_widget_is_offered_only_while_armed() {
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let on = painted(&wet);
    let off = painted(&PainterTool::default());
    let mut ids: Vec<NodeId> = core_ids::PAINTER_WETPAINT_TOOL_IDS.to_vec();
    ids.extend([
        core_ids::PAINTER_WETPAINT_TILT_TOGGLE,
        core_ids::PAINTER_WETPAINT_TILT_PAD,
        core_ids::PAINTER_WETPAINT_WETCANVAS,
        core_ids::PAINTER_WETPAINT_DRYCANVAS,
        core_ids::PAINTER_WETPAINT_FASTDRY,
        core_ids::PAINTER_WETPAINT_SHOWWET,
        core_ids::PAINTER_WETPAINT_PAPER_VISUAL,
        core_ids::PAINTER_WETPAINT_TUNING,
    ]);
    for id in ids {
        assert!(has(&on, id), "armed wet section is missing {id:?}");
        assert!(!has(&off, id), "{id:?} painted for the PLAIN brush");
    }
}

/// Every doc-22 CLICK forwards through the REAL panel seam (the Enable
/// checkbox shipped dead under the mouse once — a synthetic tool-side event
/// skips this forward, which is exactly how it shipped). Mutation: an id
/// missing from `PAINTER_WETPAINT_CLICKS` (allowlist + populate share it).
#[test]
fn every_doc22_wet_click_forwards_through_the_panel() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::EventOutcome;
    use ph2d_editor_core::tool::PanelEvent;
    for clicked in core_ids::PAINTER_WETPAINT_CLICKS {
        let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
        let mut st = PainterLayersPanelState;
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::Click(clicked));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "panel ignored the wet click {clicked:?} (allowlist arm missing)"
        );
        let actions = host.drained_actions();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
            )),
            "wet click {clicked:?} never reached the bus. drained = {actions:?}"
        );
    }
}

/// The TILT pad drag snaps to the dial's grid and forwards ring+spoke as two
/// `SetValue`s (the shared conversion — a drag to the right edge is ring 8,
/// spoke 0). Mutation: the ValueChanged arm missing from `event.rs`, or the
/// conversion diverging from the paint's placement law.
#[test]
fn the_tilt_pad_drag_forwards_ring_and_spoke() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
    use ph2d_editor_core::tool::PanelEvent;
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    host.store_mut().set_curve_point_drag(
        core_ids::PAINTER_WETPAINT_TILT_PAD,
        0,
        0,
        1.0, // right edge
        0.5, // vertical centre
    );
    let outcome = host.apply_panel_event::<PainterLayersPanel>(
        &mut st,
        WidgetEvent::ValueChanged(core_ids::PAINTER_WETPAINT_TILT_PAD),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the pad drain arm is missing"
    );
    let actions = host.drained_actions();
    let val = |target: NodeId| {
        actions.iter().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)) if *id == target => Some(*v),
            _ => None,
        })
    };
    assert_eq!(
        val(core_ids::PAINTER_WETPAINT_TILT_RING),
        Some(8.0),
        "a drag to the rim must snap to the outer ring"
    );
    assert_eq!(
        val(core_ids::PAINTER_WETPAINT_TILT_SPOKE),
        Some(0.0),
        "a drag straight right is spoke 0"
    );
}

/// **A GRADE é o primeiro widget da seção, acima do rádio de tools** — o pedido
/// literal do Enio (2026-07-29: *"quero esse slider como primeiro widget da
/// seção wet paint, acima das tools"*), e é gateável porque a posição é um
/// número: o `y` da row tem de ser menor que o `y` de TODOS os sete chips de
/// tool.
///
/// ⚠️ **A ordem não é estética aqui.** O custo do solver é linear nas células,
/// então esta é a decisão que governa a taxa VISUAL da água antes de qualquer
/// knob de física — e trocá-la ENCERRA a água viva (o bake), o que é uma decisão
/// de sessão, não de pincelada. Um controle desse peso enterrado entre os knobs
/// seria encontrado por acidente.
///
/// Mutação: mover a `card_row` da grade para depois do `seg_row` das tools faz o
/// `y` dela passar o dos chips e este gate nomeia qual.
#[test]
fn the_fluid_grid_row_is_the_first_widget_of_the_wet_section() {
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let rects = painted(&wet);
    let grid = rects
        .iter()
        .find(|(w, r)| *w == core_ids::PAINTER_WETPAINT_GRID && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
        .expect("a row da grade nao e pintada — o slider nao existe na tela");
    for (i, id) in core_ids::PAINTER_WETPAINT_TOOL_IDS.iter().enumerate() {
        let tool = rects
            .iter()
            .find(|(w, r)| w == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o chip de tool {i} nao e pintado"));
        assert!(
            grid.y < tool.y,
            "a grade (y={:.1}) tem de vir ACIMA do chip de tool {i} (y={:.1})",
            grid.y,
            tool.y
        );
    }
}

/// A row da grade está **viva sob o mouse** (pintada, registrada E focável) — a
/// diferença entre um widget que existe e um que responde.
///
/// ⚠️ Dirige o ponteiro REAL (`click_at`), não um `WidgetEvent` sintético: o
/// evento sintético pula a checagem de focabilidade no store, e foi assim que as
/// 36 células da matriz de colisão da física nasceram mortas com o gate verde.
#[test]
fn the_fluid_grid_row_is_alive_under_the_mouse() {
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    set_current_brush(Some(wet.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let r = rects
        .iter()
        .find(|(w, _)| *w == core_ids::PAINTER_WETPAINT_GRID)
        .map(|(_, r)| *r)
        .expect("a row da grade nao e pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    // O ponteiro REAL: o hit-index tem de resolver o id ali, e o clique tem de
    // produzir evento. Um id pintado e nao registrado engole o clique em
    // silencio — o modo de falha que este arquivo existe para pegar.
    assert_eq!(
        host.hit_at(cx, cy),
        Some(core_ids::PAINTER_WETPAINT_GRID),
        "o retangulo da grade nao responde ao ponteiro — pintado mas morto"
    );
    let evs = host.click_at(cx, cy);
    assert!(
        !evs.is_empty(),
        "clicar a row da grade nao produziu evento nenhum"
    );
    let _ = st;
}

// ── Plano 30 — a SEGUNDA razão (a grade de FLUXO) e o readout derivado. ──

/// O `Flow Grid` fica **logo abaixo** do `Grid Size` e **acima** dos tools: os
/// dois números de resolução são um GRUPO, e separá-los faria o artista
/// encontrar um sem o outro.
///
/// Mutação: mover a row para depois do `seg_row` das tools quebra a ordem e
/// este gate nomeia qual chip ela passou.
#[test]
fn the_flow_grid_row_sits_right_under_the_fluid_grid_row() {
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    let rects = painted(&wet);
    let find = |id: NodeId| {
        rects
            .iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    };
    let grid = find(core_ids::PAINTER_WETPAINT_GRID).expect("a row do Grid Size nao e pintada");
    let flow = find(core_ids::PAINTER_WETPAINT_FLOW)
        .expect("a row do Flow Grid nao e pintada — o slider nao existe na tela");
    assert!(
        grid.y < flow.y,
        "o Flow Grid (y={:.1}) tem de vir ABAIXO do Grid Size (y={:.1})",
        flow.y,
        grid.y
    );
    for (i, id) in core_ids::PAINTER_WETPAINT_TOOL_IDS.iter().enumerate() {
        let tool = find(*id).unwrap_or_else(|| panic!("o chip de tool {i} nao e pintado"));
        assert!(
            flow.y < tool.y,
            "o Flow Grid (y={:.1}) tem de vir ACIMA do chip de tool {i} (y={:.1})",
            flow.y,
            tool.y
        );
    }
}

/// A row do `Flow Grid` está **viva sob o mouse** — pintada, registrada E
/// focável, que são três coisas e não uma (a lição das 36 células da matriz de
/// colisão da física, que nasceram mortas com o gate verde porque ele mandava
/// `WidgetEvent` sintético em vez de clicar).
#[test]
fn the_flow_grid_row_is_alive_under_the_mouse() {
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    set_current_brush(Some(wet.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let rects = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let r = rects
        .iter()
        .find(|(w, _)| *w == core_ids::PAINTER_WETPAINT_FLOW)
        .map(|(_, r)| *r)
        .expect("a row do Flow Grid nao e pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(
        host.hit_at(cx, cy),
        Some(core_ids::PAINTER_WETPAINT_FLOW),
        "o retangulo do Flow Grid nao responde ao ponteiro — pintado mas morto"
    );
    assert!(
        !host.click_at(cx, cy).is_empty(),
        "clicar a row do Flow Grid nao produziu evento nenhum"
    );
    let _ = st;
}

/// **O readout diz a verdade** — e ele é o que impede que *Grid 2 + Flow 4* seja
/// uma grade de fluxo de 512² que ninguém sabe que existe.
///
/// ⚠️ O oráculo é o TOOL, não uma aritmética escrita aqui: as dimensões são
/// derivadas pelas MESMAS portas que o motor usa, e recomputá-las no gate
/// tornaria o gate um espelho do bug em vez de um oráculo dele.
#[test]
fn the_resolution_readout_is_derived_from_both_ratios() {
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    wet.set_source(vec![255u8; 2048 * 1024 * 4], 2048, 1024);
    // Sem as duas razões, o par tem de ser a própria tela.
    let b = wet.brush_settings();
    assert_eq!(
        b.wet_fluid_dims,
        (2048, 1024),
        "razoes 1: o fluido e a tela"
    );
    assert_eq!(
        b.wet_flow_dims,
        (2048, 1024),
        "razoes 1: o fluxo e o fluido"
    );
    wet.set_wet_grid_ratio(2.0);
    wet.set_wet_flow_ratio(4.0);
    let b = wet.brush_settings();
    assert_eq!(b.wet_fluid_dims, (1024, 512), "Grid 2 divide a TELA");
    assert_eq!(
        b.wet_flow_dims,
        (256, 128),
        "Flow 4 divide o FLUIDO, nao a tela"
    );
}

/// **O clique do `Flow Grid` CHEGA AO BARRAMENTO** — a terceira das quatro
/// condições, e ela **não** é implicada pelas outras duas.
///
/// ⚠️ Este gate nasceu de uma mutação SOBREVIVENTE: tirar o id do array
/// `PAINTER_WETPAINT_FIELDS` deixa a row pintada, hit-registrada e clicável — o
/// gate de "vivo sob o mouse" fica **VERDE** — e o `ValueChanged` morre no
/// painel sem virar `SetValue`. *Pintado, vivo e mudo* é um estado que só este
/// gate distingue.
///
/// ⚠️ **E o gate que parecia cobri-lo tem ORÁCULO AUTO-REFERENTE:** o
/// `the_knob_rows_are_offered_only_while_armed` ITERA
/// `PAINTER_WETPAINT_FIELDS`, então tirar um id do array **encolhe a lista que
/// o gate percorre** e ele segue verde afirmando menos. Um gate que enumera a
/// própria coisa sob teste não pode falhar por ela sumir — por isso este aqui
/// nomeia o id por LITERAL.
#[test]
fn the_flow_grid_edit_forwards_through_the_panel() {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::panel::EventOutcome;
    use ph2d_editor_core::tool::PanelEvent;
    let mut wet = PainterTool::default();
    wet.set_wetpaint_armed(true);
    set_current_brush(Some(wet.brush_settings()));
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    let _ = host.paint::<PainterLayersPanel>(&mut st, viewport());
    let id = core_ids::PAINTER_WETPAINT_FLOW;
    assert_eq!(
        host.apply_panel_event::<PainterLayersPanel>(&mut st, WidgetEvent::ValueChanged(id)),
        EventOutcome::Consumed,
        "a edicao do Flow Grid morreu no painel — o braco ValueChanged nao conhece o id"
    );
    let actions = host.drained_actions();
    assert!(
        actions.iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(i, _)) if *i == id
        )),
        "a edicao do Flow Grid nunca virou SetValue — pintada, viva e MUDA. drained = {actions:?}"
    );
}
