//! Behavioral SEAM test for the Vector Style panel ↔ tool (blindagem Fase 1.2).
//!
//! Unit tests in `ph2d-tool-vector` exercise `handle_panel_event` directly, and
//! `populate.rs` asserts widgets are registered — but NEITHER proves the wire
//! between them is intact. A forgotten `event.rs` arm or a wrong projection
//! leaves the control painted, draggable and SILENTLY DEAD while every unit test
//! + the `*_contract_surface` gates stay green.
//!
//! These tests run the full path the desktop shell runs, headless:
//!   populate → set widget value → apply_event → bus → handle_panel_event
//!   → assert the tool's Style actually changed.
//!
//! (The Stroke / Fill colour swatches go through the OKLCH picker read-back in
//! the shell's `vector_bridge`, not through `apply_event`, so they are covered
//! by the tool's `set_stroke_rgba` / `set_fill_rgba` unit tests + the bridge —
//! not this panel seam.)

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::{PanelEvent, Tool}; // brings `handle_panel_event` into scope
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::params::slider_to_px;
use ph2d_tool_vector::{DrawMode, VectorTool};
use ph2d_ui_testkit::MockPanelHost;

/// Forward every drained `ToolPanelEvent` into the tool (what the shell does
/// each frame). Returns whether at least one was forwarded.
fn drain_into_tool(host: &mut MockPanelHost, tool: &mut VectorTool) -> bool {
    let mut forwarded = false;
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
            forwarded = true;
        }
    }
    forwarded
}

/// Drag the Width slider to its full end and prove the width lands in the tool
/// — exercising every site from `populate` to `stroke_width_px()`.
#[test]
fn width_slider_drag_reaches_tool_style() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    // A drag writes the slider's stored value, then the dispatch emits
    // ValueChanged. Simulate both.
    host.set_slider_value(ids::VECTOR_WIDTH, 1.0);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_WIDTH),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` arm for VECTOR_WIDTH is missing"
    );

    assert!(
        drain_into_tool(&mut host, &mut tool),
        "width edit never reached the bus as a ToolPanelEvent — the panel→shell seam is dead"
    );

    // End-to-end proof: the tool's width changed to the slider's px.
    assert_eq!(
        tool.stroke_width_px(),
        slider_to_px(1.0),
        "slider→tool seam delivered the wrong px for Width"
    );
}

/// The Fill Opacity slider owns the fill alpha (replaces the old "None" button):
/// dragging it to 0 makes the fill invisible, through the seam.
#[test]
fn fill_opacity_slider_sets_alpha_through_seam() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_ne!(tool.fill_rgba()[3], 0, "precondition: default fill opaque");

    host.set_slider_value(ids::VECTOR_FILL_OPACITY, 0.0);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_FILL_OPACITY),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Fill Opacity edit ignored — `event.rs` arm for VECTOR_FILL_OPACITY is missing"
    );

    drain_into_tool(&mut host, &mut tool);
    assert_eq!(
        tool.fill_rgba()[3],
        0,
        "Fill Opacity → 0 never cleared the fill alpha through the seam"
    );
    assert!(
        tool.take_apply_to_selected(),
        "Opacity change must flag the selected path for recolour"
    );
}

/// A draw-mode button (Rectangle) must switch the tool's mode through the seam
/// — exercising the mode arm in `event.rs` + `handle_panel_event`.
#[test]
fn mode_button_click_switches_tool_mode_through_seam() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    // ADR-0112: a ferramenta abre na SELEÇÃO, como qualquer editor vetorial.
    assert_eq!(
        tool.mode(),
        DrawMode::Select,
        "precondition: default is Select"
    );

    // Cada botão de modo tem de chegar ao tool pelo seam. Line/Arc foram o smoke
    // que falhou (Enio 2026-07-09): pintados + registrados, mas ausentes da
    // allowlist de `event.rs` → o clique nunca virava `ToolPanelEvent`.
    for (id, want) in [
        (ids::VECTOR_MODE_PEN, DrawMode::Pen),
        (ids::VECTOR_MODE_TEXT, DrawMode::Text),
        (ids::VECTOR_MODE_NODE, DrawMode::Node),
        // O 5º pill (reforma da UI): `DrawMode::Shape` não tinha botão nenhum, então a
        // fileira de modos ficava TODA apagada justamente enquanto se desenhava uma
        // forma. O botão novo é inútil se não chegar ao tool — este arm é o gate.
        (ids::VECTOR_MODE_SHAPE, DrawMode::Shape),
        // O 6º pill: o CONECTOR.
        (ids::VECTOR_MODE_CONNECT, DrawMode::Connect),
        // **Pick Shapes** (Blend) — coleta formas na ordem de clique. O botão dele mora na seção
        // BLEND (não nesta fileira), mas o id/seam é o mesmo.
        (ids::VECTOR_MODE_PICKBLEND, DrawMode::PickBlend),
        // O 9º e 10º pills: **Fillet / Chamfer** (arredondar / chanfrar quina). Pintados +
        // registrados, mas mortos se ausentes da allowlist de `event.rs` ou do switch da tool.
        (ids::VECTOR_MODE_FILLET, DrawMode::Fillet),
        (ids::VECTOR_MODE_CHAMFER, DrawMode::Chamfer),
    ] {
        let outcome =
            host.apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "mode button ignored — `event.rs` allowlist for VECTOR_MODE_* is missing this id"
        );
        drain_into_tool(&mut host, &mut tool);
        assert_eq!(
            tool.mode(),
            want,
            "mode click never reached the tool through the seam"
        );
    }
}

/// **O pill do Conector chega à tool.** Gate próprio (e não só mais uma linha na tabela
/// acima) porque o modo `Connect` é a porta de entrada de uma feature inteira: um pill que
/// PINTA mas não despacha deixa o conector inalcançável, e todo unit test da crate de rota
/// continua verde — foi exatamente assim que Line/Arc morreram (Enio 2026-07-09).
///
/// Percorre o caminho que a shell percorre: populate → clique → `apply_event` → bus →
/// `handle_panel_event` → o modo da tool.
#[test]
fn clicking_connect_pill_reaches_the_tool() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_ne!(
        tool.mode(),
        DrawMode::Connect,
        "precondition: nao e Connect"
    );

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_MODE_CONNECT),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o pill Connect nao foi consumido — falta o id na allowlist de `event.rs`"
    );
    assert!(
        drain_into_tool(&mut host, &mut tool),
        "o clique nunca virou ToolPanelEvent — o seam painel->shell esta morto"
    );
    assert_eq!(
        tool.mode(),
        DrawMode::Connect,
        "o clique chegou ao bus mas nao virou modo — falta o arm em `handle_panel_event`"
    );
}

/// A Cap button + the Dash and Gap sliders reach the tool through the seam —
/// the stroke-detail controls.
#[test]
fn stroke_cap_dash_and_gap_reach_the_tool() {
    use ph2d_tool_vector::StrokeCap;
    use ph2d_tool_vector::params::{DASH_MAX, GAP_MAX};
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    let c = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_CAP_ROUND),
    );
    assert_eq!(c, EventOutcome::Consumed, "Cap button not wired");
    drain_into_tool(&mut host, &mut tool);
    assert_eq!(tool.cap(), StrokeCap::Round);

    host.set_slider_value(ids::VECTOR_DASH, 1.0);
    let d = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_DASH),
    );
    assert_eq!(d, EventOutcome::Consumed, "Dash slider not wired");
    drain_into_tool(&mut host, &mut tool);
    assert!((tool.dash() - DASH_MAX).abs() < 1e-6);

    host.set_slider_value(ids::VECTOR_GAP, 1.0);
    let g = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_GAP),
    );
    assert_eq!(g, EventOutcome::Consumed, "Gap slider not wired");
    drain_into_tool(&mut host, &mut tool);
    assert!((tool.gap() - GAP_MAX).abs() < 1e-6);
}

/// The Star mode button switches the mode, and the Star "Points" slider reaches
/// **Gate do seam do CATÁLOGO** — o que substitui um teste por forma.
///
/// Para TODA forma do catálogo: o botão dela chega ao tool pelo seam (escolhe a forma e
/// arma o gesto), e CADA campo que ela declara chega ao tool como valor. Uma forma nova
/// entra na tabela e este teste já a cobre — nenhum botão e nenhum campo pode nascer
/// pintado-e-morto, que é exatamente o bug que o smoke do Line/Arc pegou (Enio
/// 2026-07-09): registrado, desenhado, e ausente da allowlist do `event.rs`.
#[test]
fn every_shape_and_every_field_in_the_catalog_reaches_the_tool() {
    use ph2d_tool_vector::shapes;
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    for (i, d) in shapes::SHAPES.iter().enumerate() {
        let outcome = host.apply_panel_event::<VectorPanel>(
            &mut panel_state,
            WidgetEvent::Click(ids::vector_shape_id(i)),
        );
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "{:?}: botao do catalogo ignorado pelo painel",
            d.kind
        );
        assert!(
            drain_into_tool(&mut host, &mut tool),
            "{:?}: o clique nunca virou ToolPanelEvent",
            d.kind
        );
        assert_eq!(
            tool.shape(),
            d.kind,
            "{:?}: a tool nao trocou de forma",
            d.kind
        );
        assert_eq!(
            tool.mode(),
            DrawMode::Shape,
            "{:?}: escolher a forma tem de armar o gesto",
            d.kind
        );

        // E cada campo declarado chega ao tool com o VALOR (nao um track 0..1).
        for (fi, f) in d.fields.iter().enumerate() {
            let id = ids::vector_shape_field_id(fi);
            host.set_number_value(id, f.max);
            let outcome = host
                .apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::ValueChanged(id));
            assert_eq!(
                outcome,
                EventOutcome::Consumed,
                "{:?}.{}: campo ignorado pelo painel",
                d.kind,
                f.label
            );
            assert!(
                drain_into_tool(&mut host, &mut tool),
                "{:?}.{}: a edicao nunca virou ToolPanelEvent",
                d.kind,
                f.label
            );
            assert!(
                (tool.draw_config().values[fi] - f.max).abs() < 1e-9,
                "{:?}.{}: o valor nao chegou na tool",
                d.kind,
                f.label
            );
        }
    }
}

/// **Gate do seam das PONTAS de traço** (markers / arrowheads).
///
/// Os dois seletores (Start / End) são chips de dropdown: o clique numa linha do popover
/// tem de (a) ser consumido pelo painel, (b) FECHAR o chip — o light-dismiss genérico não
/// dispara, porque o clique é DENTRO do popover — e (c) chegar à tool como a ponta
/// escolhida. Um controle que pinta e não despacha é exatamente o bug que este arquivo
/// existe para pegar.
///
/// Cobre TODA ponta de `ALL_MARKERS` nos DOIS slots: uma ponta nova entra na tabela e já
/// nasce coberta, e um id de opção trocado entre começo e fim sai vermelho.
#[test]
fn every_marker_option_reaches_the_tool_and_closes_its_chip() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_vec_scene::{ALL_MARKERS, Marker};

    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_eq!(
        tool.marker_start(),
        Marker::None,
        "precondition: uma linha nasce sem ponta"
    );

    for (slot, dd) in [
        (0_usize, ids::VECTOR_MARKER_START_DD),
        (1_usize, ids::VECTOR_MARKER_END_DD),
    ] {
        for (i, &want) in ALL_MARKERS.iter().enumerate() {
            // Abre o chip (o que o dispatch genérico faz num clique nele).
            match host.store_mut().get_mut(dd) {
                Some(InteractiveState::Dropdown { open, .. }) => *open = true,
                _ => panic!("o chip de ponta {slot} nao esta registrado como Dropdown no populate"),
            }

            let outcome = host.apply_panel_event::<VectorPanel>(
                &mut panel_state,
                WidgetEvent::Click(ids::vector_marker_option_id(slot, i)),
            );
            assert_eq!(
                outcome,
                EventOutcome::Consumed,
                "slot {slot} / {want:?}: opcao de ponta ignorada pelo painel"
            );
            assert!(
                drain_into_tool(&mut host, &mut tool),
                "slot {slot} / {want:?}: o clique nunca virou ToolPanelEvent — o seletor esta MORTO"
            );

            let got = if slot == 0 {
                tool.marker_start()
            } else {
                tool.marker_end()
            };
            assert_eq!(
                got, want,
                "slot {slot}: a ponta escolhida nao chegou na tool"
            );

            match host.store().get(dd) {
                Some(InteractiveState::Dropdown {
                    open,
                    selected_index,
                    ..
                }) => {
                    assert!(
                        !open,
                        "slot {slot} / {want:?}: o chip ficou ABERTO apos a escolha"
                    );
                    assert_eq!(
                        *selected_index,
                        Some(i),
                        "slot {slot} / {want:?}: o chip nao registrou a ponta escolhida"
                    );
                }
                _ => panic!("o chip de ponta {slot} deixou de ser um Dropdown"),
            }
        }
    }

    // Os dois seletores são INDEPENDENTES: o último loop deixou End na última ponta, e o
    // Start na mesma — mas escolher no Start não pode mexer no End.
    let end_before = tool.marker_end();
    match host.store_mut().get_mut(ids::VECTOR_MARKER_START_DD) {
        Some(InteractiveState::Dropdown { open, .. }) => *open = true,
        _ => unreachable!(),
    }
    host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::vector_marker_option_id(0, 1)), // Triangle
    );
    drain_into_tool(&mut host, &mut tool);
    assert_eq!(tool.marker_start(), Marker::Triangle);
    assert_eq!(
        tool.marker_end(),
        end_before,
        "escolher a ponta do COMECO mexeu na do FIM — os ids de opcao colidiram"
    );
    assert!(
        tool.take_apply_to_selected(),
        "a ponta e Style: a escolha tem de reestilizar o caminho selecionado"
    );
}

#[test]
fn boolean_button_click_forwards_to_the_bus_for_the_shell() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_BOOL_UNION),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Boolean button ignored — `event.rs` arm for VECTOR_BOOL_* is missing"
    );

    let forwarded = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == ids::VECTOR_BOOL_UNION
        )
    });
    assert!(
        forwarded,
        "Boolean click never reached the bus as a ToolPanelEvent — the shell can't apply the op"
    );
}

/// The Arrange buttons (Duplicate + z-order) are DOCUMENT commands acting on the
/// selected path — the tool ignores them, so the seam proof is that each `Click`
/// reaches the bus as a `ToolPanelEvent` for the shell drain to apply.
#[test]
fn arrange_buttons_forward_to_the_bus_for_the_shell() {
    for id in [
        ids::VECTOR_ARRANGE_DUPLICATE,
        ids::VECTOR_ARRANGE_TO_BACK,
        ids::VECTOR_ARRANGE_BACKWARD,
        ids::VECTOR_ARRANGE_FORWARD,
        ids::VECTOR_ARRANGE_TO_FRONT,
        ids::VECTOR_ARRANGE_FLIP_H,
        ids::VECTOR_ARRANGE_FLIP_V,
        ids::VECTOR_ARRANGE_ROTATE_CW,
        ids::VECTOR_ARRANGE_ROTATE_CCW,
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;

        let outcome =
            host.apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "Arrange button ignored — `event.rs` arm for VECTOR_ARRANGE_* is missing"
        );

        let forwarded = host.drained_actions().iter().any(|a| {
            matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(fid)) if *fid == id
            )
        });
        assert!(
            forwarded,
            "Arrange click never reached the bus as a ToolPanelEvent — the shell can't apply it"
        );
    }
}

/// A Vertex-type button (Smooth) is a DOCUMENT command (retypes the selected
/// vertex via the shell-side Pen), so — like the Boolean buttons — the seam
/// proof is that the panel forwards the `Click` onto the bus for the shell drain.
#[test]
fn vertex_type_button_click_forwards_to_the_bus_for_the_shell() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_VERT_SMOOTH),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Vertex-type button ignored — `event.rs` arm for VECTOR_VERT_* is missing"
    );

    let forwarded = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == ids::VECTOR_VERT_SMOOTH
        )
    });
    assert!(
        forwarded,
        "Vertex-type click never reached the bus as a ToolPanelEvent — the shell can't retype it"
    );
}

/// The "Delete Node" button is a DOCUMENT command (removes the selected vertex
/// via the shell Pen), so — like Boolean/Vertex-type — the seam proof is that the
/// panel forwards the `Click` onto the bus for the shell drain.
#[test]
fn delete_node_button_click_forwards_to_the_bus_for_the_shell() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_VERT_DELETE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Delete-Node click ignored — `event.rs` arm for VECTOR_VERT_DELETE is missing"
    );

    let forwarded = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == ids::VECTOR_VERT_DELETE
        )
    });
    assert!(
        forwarded,
        "Delete-Node click never reached the bus as a ToolPanelEvent — the shell can't delete it"
    );
}

/// **O toggle Chamfer** (ADR-0121) só é pintado quando a seleção tem uma quina com raio
/// (`current_corner_chamfer()` é `Some`), e o Click chega ao bus pelo caminho INTEIRO (paint →
/// hit-rect → ponteiro → dispatch → `event.rs` → bus). O gate cobre a AUSÊNCIA (sem quina, não
/// pinta) e a PRESENÇA (com quina, pinta e despacha) — sem a 1ª metade, "não vaza" ficaria
/// verde com a seção apagada. [[feedback_absence_gate_needs_a_presence_sibling]]
#[test]
fn the_chamfer_toggle_paints_and_clicks_only_with_a_corner_selected() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    const SEC: u128 = 1_000_000_000;

    // A seção Vertex precisa estar visível para o toggle ser sequer alcançável.
    ph2d_panel_vector::set_selected_vertex_type(Some(ph2d_tool_vector::VertexType::Corner));

    // SEM quina com raio → o toggle não é pintado.
    ph2d_panel_vector::set_current_corner_chamfer(None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_VERT_CHAMFER)
            .is_none(),
        "o toggle Chamfer foi pintado sem uma quina com raio na seleção"
    );

    // COM quina → pintado, clicável, e o Click chega ao bus (a shell alterna o estilo).
    ph2d_panel_vector::set_current_corner_chamfer(Some(false));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_VERT_CHAMFER)
        .expect("o toggle Chamfer não foi pintado com uma quina selecionada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_VERT_CHAMFER)),
        "o ponteiro sobre o Chamfer não virou Click — falta o registro em `populate`"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_VERT_CHAMFER
        )),
        "o Click do Chamfer não chegou ao bus — falta o id em `forwards_plain_click`"
    );

    // Não vazar o estado publicado (thread-local) para os outros testes.
    ph2d_panel_vector::set_current_corner_chamfer(None);
    ph2d_panel_vector::set_selected_vertex_type(None);
}

/// **Gate anti-cabeçalho-morto.** O collapse de uma seção é dispatch GENÉRICO e exige
/// DOIS sites: a marca no `populate` (`mark_collapsible_section`) e o hit-rect do header
/// no paint. Faltando a marca, `apply_click` nunca chama `toggle_collapsed` — o header
/// pinta um chevron, promete dobrar, e não dobra. É o mesmo gênero de bug do botão
/// pintado-e-morto, só que na chrome de seção.
///
/// O painel migrou 14 `section_label` caseiros para `SectionHeader` colapsáveis (canon
/// `docs/UI_Padrao/components/section_header.md`): este teste prova que TODOS os 17
/// nasceram vivos, e uma seção nova sem a marca sai vermelha.
#[test]
fn every_section_header_is_registered_as_collapsible() {
    let host = MockPanelHost::with_panel::<VectorPanel>();
    assert_eq!(
        ids::VECTOR_SECTIONS.len(),
        22, // +1: EFFECTS (ADR-0132, a pilha de Live Path Effects)
        "a lista de secoes mudou — confira que o paint pinta um header para cada uma"
    );
    for &id in ids::VECTOR_SECTIONS {
        assert!(
            host.store().is_collapsible_section(id),
            "{id:?}: header pintado mas NAO marcado colapsavel — o chevron nao dobra \
             (falta `mark_collapsible_section` no populate)"
        );
    }
}

/// **A separação categoria ≠ tipo, pelo seam.** A categoria virou um `Dropdown` (widget
/// visualmente distinto da grade de tipos). Escolher uma família no popover tem de (a)
/// ser consumido pelo painel e (b) FECHAR o chip — o light-dismiss genérico não dispara
/// aqui, porque o clique é DENTRO do popover. Sem o fecho manual o popover ficaria
/// pendurado sobre o painel.
#[test]
fn picking_a_category_closes_the_dropdown_chip() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_tool_vector::shapes;

    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    for (i, _g) in shapes::ALL_GROUPS.iter().enumerate() {
        // Abre o chip (o que o dispatch genérico faz num clique nele).
        if let Some(InteractiveState::Dropdown { open, .. }) =
            host.store_mut().get_mut(ids::VECTOR_SHAPE_GROUP_DD)
        {
            *open = true;
        } else {
            panic!("VECTOR_SHAPE_GROUP_DD nao esta registrado como Dropdown no populate");
        }

        let outcome = host.apply_panel_event::<VectorPanel>(
            &mut panel_state,
            WidgetEvent::Click(ids::vector_shape_group_id(i)),
        );
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "opcao de categoria {i} ignorada pelo painel"
        );

        match host.store().get(ids::VECTOR_SHAPE_GROUP_DD) {
            Some(InteractiveState::Dropdown {
                open,
                selected_index,
                ..
            }) => {
                assert!(!open, "categoria {i}: o chip ficou ABERTO apos a escolha");
                assert_eq!(
                    *selected_index,
                    Some(i),
                    "categoria {i}: o chip nao registrou a familia escolhida"
                );
            }
            _ => panic!("VECTOR_SHAPE_GROUP_DD deixou de ser um Dropdown"),
        }
    }
}

/// **Gate do seam do CONECTOR.** Os três campos (Route / Jetty / Spread) editam a RELAÇÃO,
/// que vive no `VecConnector` de cada conector SELECIONADO — não no Style da tool. Logo a
/// prova do seam é a mesma dos botões de documento: cada edição tem de chegar ao bus como
/// um `ToolPanelEvent::SetValue` **no id do campo**, com o VALOR (a shell é quem sabe quais
/// conectores estão selecionados).
///
/// O Route é o caso que morde: ele é um BOTÃO que cicla, e o valor que ele emite é o
/// PRÓXIMO discriminante — calculado a partir do que está na tela. Um clique que emitisse o
/// valor corrente seria um botão que pinta, despacha e **não muda nada**.
#[test]
fn connector_fields_reach_the_bus_for_the_shell() {
    use ph2d_panel_vector::{ConnectorSnapshot, set_current_connector};

    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    // Um conector selecionado (o que a shell publica a cada frame), em Orthogonal.
    set_current_connector(Some(ConnectorSnapshot {
        route: 1,
        jetty: 0.35,
        spread: 0.0,
        corner: 0.0,
        curve: 1.0 / 3.0,
    }));

    // Jetty + Spread: caixas numéricas — o valor sai como está (não é um track 0..1).
    for (id, v) in [
        (ids::VECTOR_CONNECTOR_JETTY, 0.75),
        (ids::VECTOR_CONNECTOR_SPREAD, -0.5),
        // A quina do PERCURSO: mesmo caminho de volta (SetValue no id do proprio campo).
        (ids::VECTOR_CONNECTOR_CORNER, 0.25),
    ] {
        host.set_number_value(id, v);
        let outcome =
            host.apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::ValueChanged(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "campo do conector ignorado — falta o arm em `paint_connector::apply_event`"
        );
        let forwarded = host.drained_actions().iter().any(|a| {
            matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(fid, fv))
                    if *fid == id && (*fv - v).abs() < 1e-9
            )
        });
        assert!(
            forwarded,
            "a edicao nunca virou ToolPanelEvent — a shell nao pode fixar o valor"
        );
    }

    // Route: o clique CICLA (Orthogonal = 1 → Straight = 0).
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_CONNECTOR_ROUTE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o botao Route foi ignorado pelo painel"
    );
    let cycled = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(fid, fv))
                if *fid == ids::VECTOR_CONNECTOR_ROUTE && (*fv - 2.0).abs() < 1e-9
        )
    });
    assert!(
        cycled,
        "o clique no Route nao emitiu a PROXIMA rota (Orthogonal -> Curved) — \
         um botao que despacha o valor corrente nao muda nada"
    );

    set_current_connector(None);
}

/// Sem conector na seleção a seção nem é pintada — e o botão Route, se um clique fantasma o
/// alcançasse, não pode inventar uma rota para uma seleção que não tem conector nenhum.
#[test]
fn the_route_button_refuses_to_fire_without_a_connector_in_focus() {
    use ph2d_panel_vector::set_current_connector;

    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    set_current_connector(None);

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_CONNECTOR_ROUTE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Ignored,
        "sem conector em foco o Route nao pode ser consumido"
    );
    assert!(
        !host
            .drained_actions()
            .iter()
            .any(|a| matches!(a, EditorAction::ToolPanelEvent(_))),
        "o Route despachou uma rota sem conector nenhum selecionado"
    );
}

/// The Close (X) button must emit `CancelActiveTool` (deactivates the tool),
/// mirror of the Padding panel's Cancel.
#[test]
fn close_button_cancels_active_tool() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host
        .apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::Click(ids::VECTOR_CLOSE));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Close click was ignored — `event.rs` arm for VECTOR_CLOSE is missing"
    );

    let cancelled = host
        .drained_actions()
        .iter()
        .any(|a| matches!(a, EditorAction::CancelActiveTool));
    assert!(
        cancelled,
        "Close click never emitted CancelActiveTool through the seam"
    );
}

/// **Os três controles de PONTA chegam à TOOL** — Head Size, Head Round e Both Ends.
///
/// Gate próprio, e não mais uma linha na tabela de sliders, porque os três atravessam
/// caminhos DIFERENTES: as duas caixas passam pelo `paint_markers::apply_event` (delegado do
/// roteador), e o Both Ends é um `Click` puro que só vive se o id estiver na allowlist
/// `forwards_plain_click`. Tirar qualquer um desses dois sites deixa o controle **pintado,
/// clicável e MORTO** — e todo unit test da tool continua verde. É a classe que matou o
/// Line/Arc (Enio 2026-07-09).
///
/// Percorre o caminho inteiro da shell: populate → evento → `apply_event` → bus →
/// `handle_panel_event` → o Style da tool.
#[test]
fn the_marker_head_controls_reach_the_tool() {
    use ph2d_vec_scene::Marker;

    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    // 1. As duas CAIXAS (Head Size / Head Round): o valor sai como está (não é um track 0..1).
    for (id, v) in [
        (ids::VECTOR_MARKER_SCALE, 2.5),
        (ids::VECTOR_MARKER_ROUND, 0.75),
    ] {
        host.set_number_value(id, v);
        let outcome =
            host.apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::ValueChanged(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "campo da ponta ignorado — falta o arm em `paint_markers::apply_event`"
        );
        assert!(
            drain_into_tool(&mut host, &mut tool),
            "o campo da ponta nunca virou ToolPanelEvent — o seam painel->shell esta morto"
        );
    }
    assert!(
        (tool.marker_scale() - 2.5).abs() < 1e-9,
        "o Head Size chegou ao bus mas nao ao Style — falta o arm em `handle_panel_event`"
    );
    assert!(
        (tool.marker_round() - 0.75).abs() < 1e-9,
        "idem o Head Round"
    );

    // 2. O botão BOTH ENDS: um `Click` puro. Fora da allowlist `forwards_plain_click` ele
    //    pinta, dá hit e NAO faz nada.
    assert!(!tool.both_ends(), "precondicao: a linha nasce de via unica");
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_MARKER_BOTH),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o botao Both Ends nao foi consumido — falta o id na allowlist de `event.rs`"
    );
    assert!(
        drain_into_tool(&mut host, &mut tool),
        "o clique nunca virou ToolPanelEvent — o botao Both Ends esta MORTO"
    );
    assert!(
        tool.both_ends(),
        "o clique chegou ao bus mas nao ligou a dupla via — falta o arm em `handle_panel_event`"
    );
    assert_ne!(tool.marker_start(), Marker::None);
    assert_ne!(tool.marker_end(), Marker::None);

    // 3. E o clique de volta desliga (o estado é derivado das pontas, nos dois sentidos).
    host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_MARKER_BOTH),
    );
    drain_into_tool(&mut host, &mut tool);
    assert!(!tool.both_ends(), "o segundo clique tinha de desligar");
    assert_eq!(
        tool.marker_start(),
        Marker::None,
        "a via unica limpa o COMECO"
    );
}

/// **O pill do Shape Builder chega à tool.** Mesmo gate, mesma razão: um pill que PINTA e
/// não despacha deixa a feature inteira inalcançável, e todo teste do arranjo — que é onde
/// mora a matemática — continua verde. É o modo mais fácil de entregar nada.
#[test]
fn clicking_build_pill_reaches_the_tool() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_ne!(tool.mode(), DrawMode::Build, "precondition: nao e Build");

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_MODE_BUILD),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o pill Build nao foi consumido — falta o id na allowlist de `event.rs`"
    );
    assert!(
        drain_into_tool(&mut host, &mut tool),
        "o clique nunca virou ToolPanelEvent — o seam painel->shell esta morto"
    );
    assert_eq!(
        tool.mode(),
        DrawMode::Build,
        "o clique chegou ao bus mas nao virou modo — falta o arm em `handle_panel_event`"
    );
}

/// **O botão Pick Shapes chega à tool.** Mesmo gate, mesma razão: um botão que PINTA e não
/// despacha deixa a feature (escolher as formas do blend na ordem) inalcançável — e o motor de
/// correspondência, onde mora a matemática, continua verde. (O botão mora na seção BLEND, não na
/// fileira de modos, mas o seam é o mesmo id.)
#[test]
fn clicking_pick_button_reaches_the_tool() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_ne!(
        tool.mode(),
        DrawMode::PickBlend,
        "precondition: nao e PickBlend"
    );

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_MODE_PICKBLEND),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o botão Pick nao foi consumido — falta o id na allowlist de `event.rs`"
    );
    assert!(
        drain_into_tool(&mut host, &mut tool),
        "o clique nunca virou ToolPanelEvent — o seam painel->shell esta morto"
    );
    assert_eq!(
        tool.mode(),
        DrawMode::PickBlend,
        "o clique chegou ao bus mas nao virou modo — falta o arm em `handle_panel_event`"
    );
}

/// Um evento de ponteiro real, para o dispatcher real.
fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t,
    }
}

/// Os botões de COMANDO da seção **Envelope** (ADR-0129) chegam ao bus quando o artista CLICA
/// NELES — os três, pelo caminho inteiro (`paint` → hit-rect → ponteiro real → dispatcher →
/// `event.rs` → bus). Irmão do gate do Blend abaixo, e pela MESMA razão.
///
/// ⚠️ **O `set_current_has_envelope(true)` não é decoração do fixture: é a PREMISSA da seção.**
/// Expand/Release só são PINTADOS quando a seleção é de fato um envelope — sem publicar isso, o
/// `painted_rect` devolve `None` e o gate morre na 1ª asserção. É o mesmo tipo de premissa de ESTADO
/// que o sweep do Painter perdeu (Filter Stroke só existe depois de um traço).
#[test]
fn every_envelope_command_button_reaches_the_bus_when_clicked() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    const SEC: u128 = 1_000_000_000;
    // A seleção É um envelope — sem isto o Expand/Release nem chegam à tela.
    ph2d_panel_vector::set_current_has_envelope(true);
    ph2d_panel_vector::set_current_envelope_presets(&["Arc", "Flag", "Wave"], Some(0), 0.5);
    for (id, name) in [
        (ids::VECTOR_ENVELOPE_RUN, "Envelope"),
        (ids::VECTOR_ENVELOPE_EXPAND, "Expand"),
        (ids::VECTOR_ENVELOPE_RELEASE, "Release"),
        (ids::VECTOR_ENVELOPE_PERSPECTIVE, "Perspective"),
        (ids::VECTOR_ENVELOPE_MESH, "Mesh"),
        (ids::VECTOR_ENVELOPE_PINS, "Pins"),
        // Os presets (ADR-0129 Fatia C) — a lista vem PUBLICADA, então o fixture a publica; sem
        // isso o `paint` não desenha botão nenhum e o gate morre na 1ª asserção, que é a mesma
        // premissa de ESTADO do `set_current_has_envelope`.
        (ids::vector_envelope_preset_id(0), "preset 0"),
        (ids::vector_envelope_preset_id(2), "preset 2"),
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .unwrap_or_else(|| {
                panic!("o botao {name} nao foi PINTADO com area clicavel na secao Envelope")
            });
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {name} nao virou Click — falta `button()` no `populate` \
             (o botao esta desenhado, mas nao existe para o dispatcher)"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        let reached = host
            .drained_actions()
            .into_iter()
            .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id));
        assert!(
            reached,
            "o Click em {name} nao chegou ao bus — falta o id na allowlist do `event.rs` \
             (o botao e clicavel e MORTO)"
        );
    }
}

/// **Sem envelope selecionado, os controles do envelope NÃO existem na tela** — o par de PRESENÇA do gate
/// acima. Um botão que não faz nada é pior que um botão que falta: sobre uma seleção que não é
/// envelope, os dois dissolveriam o nada e o artista aprenderia que "às vezes não funciona"; e os chips de
/// GESTO escolheriam o mapa de uma gaiola que não existe.
/// O **Envelope** (criar) continua lá: ele age sobre a seleção e recusa no shell com mensagem.
#[test]
fn the_envelope_controls_are_not_offered_without_an_envelope() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    ph2d_panel_vector::set_current_has_envelope(false);
    ph2d_panel_vector::set_current_envelope_presets(&["Arc", "Flag", "Wave"], Some(0), 0.5);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for (id, name) in [
        (ids::VECTOR_ENVELOPE_EXPAND, "Expand"),
        (ids::VECTOR_ENVELOPE_RELEASE, "Release"),
        (ids::VECTOR_ENVELOPE_PERSPECTIVE, "Perspective"),
        (ids::VECTOR_ENVELOPE_MESH, "Mesh"),
        (ids::VECTOR_ENVELOPE_PINS, "Pins"),
        (ids::vector_envelope_preset_id(0), "preset 0"),
        (ids::VECTOR_ENVELOPE_BEND, "Bend"),
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "o controle {name} foi pintado SEM envelope na selecao — ele age sobre o nada"
        );
    }
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_ENVELOPE_RUN)
            .is_some(),
        "o botao de CRIAR tem de existir sempre — e' a porta de entrada da feature"
    );
}

/// Os botões de COMANDO da seção Blend chegam ao bus quando o artista CLICA NELES — todos os
/// quatro, pelo caminho inteiro: `paint` → o retângulo que ele registrou → ponteiro real →
/// dispatcher real → `event.rs` → bus.
///
/// Eles não viram estado de tool (a shell os consome como comandos de DOCUMENTO: criar o objeto
/// vivo, resetar o spine, expandir, soltar), então a asserção termina no `ToolPanelEvent` — que é
/// exatamente onde a shell os lê.
///
/// **O clique é achado, não inventado.** Sintetizar `WidgetEvent::Click(id)` provaria só a allowlist
/// do `event.rs`: sem `button()` no `populate` o widget não existe, o `paint` não registra hit-rect
/// nenhum, e um clique de verdade cai no vazio — com o gate sintético verde. Foi um mutante
/// sobrevivente que mostrou isso (2026-07-15): a seção Blend inteira não tinha seam, e apagar o
/// registro deixava o botão pintado, clicável no olho e **morto**. É a classe do Redo órfão da barra
/// (pintado ≠ populado; populado ≠ despachado).
#[test]
fn every_blend_command_button_reaches_the_bus_when_clicked() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    const SEC: u128 = 1_000_000_000;
    for (id, name) in [
        (ids::VECTOR_BLEND_RUN, "Blend"),
        (ids::VECTOR_BLEND_RESET_SPINE, "Reset Spine"),
        (ids::VECTOR_BLEND_EXPAND, "Expand"),
        (ids::VECTOR_BLEND_RELEASE, "Release"),
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .unwrap_or_else(|| {
                panic!("o botao {name} nao foi PINTADO com area clicavel na secao Blend")
            });
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        // O dispatcher produziu o Click? É aqui que o REGISTRO no `populate` se prova: sem widget
        // no store, o ponteiro cai sobre o desenho do botão e não vira evento nenhum.
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {name} nao virou Click — falta `button()` no `populate` \
             (o botao esta desenhado, mas nao existe para o dispatcher)"
        );
        // …e o painel o encaminha ao bus? É aqui que a allowlist do `event.rs` se prova.
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        let reached = host
            .drained_actions()
            .into_iter()
            .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id));
        assert!(
            reached,
            "o Click em {name} nao chegou ao bus — falta o id na allowlist do `event.rs` \
             (o botao e clicavel e MORTO)"
        );
    }
}

/// **O slider Bend chega ao bus com o valor NO DOMÍNIO DO DOCUMENTO** (`-1..1`), não com o track
/// (`0..1`) — a conversão bipolar mora na fronteira do painel, e o shell nunca precisa saber que
/// existe um track.
///
/// É o gate que impede o erro clássico deste seam: o shell recebendo `0.0` e carimbando um preset
/// de força zero (= a gaiola em repouso) quando o artista arrastou o slider até a ponta ESQUERDA.
#[test]
fn the_bend_slider_reaches_the_bus_in_document_units() {
    ph2d_panel_vector::set_current_has_envelope(true);
    ph2d_panel_vector::set_current_envelope_presets(&["Arc"], Some(0), 0.5);
    for (track, want) in [(0.0_f32, -1.0_f64), (0.5, 0.0), (1.0, 1.0)] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        host.set_slider_value(ids::VECTOR_ENVELOPE_BEND, track);
        let outcome = host.apply_panel_event::<VectorPanel>(
            &mut panel_state,
            WidgetEvent::ValueChanged(ids::VECTOR_ENVELOPE_BEND),
        );
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "o Bend foi ignorado — falta o arm de ValueChanged no `event.rs`"
        );
        let got = host.drained_actions().into_iter().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, v))
                if c == ids::VECTOR_ENVELOPE_BEND =>
            {
                Some(v)
            }
            _ => None,
        });
        assert_eq!(
            got,
            Some(want),
            "track {track} devia chegar como {want} no bus, chegou {got:?}"
        );
    }
}

/// **Sem preset ATIVO o Bend não existe** — a gaiola pode estar viva e ser MANUAL (a mão a
/// promoveu), e aí o slider não teria o que re-carimbar. Par de presença do gate acima.
#[test]
fn the_bend_slider_is_not_offered_without_an_active_preset() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    ph2d_panel_vector::set_current_has_envelope(true);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    ph2d_panel_vector::set_current_envelope_presets(&["Arc", "Flag"], None, 0.5);
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_ENVELOPE_BEND)
            .is_none(),
        "o Bend foi pintado numa gaiola MANUAL — ele nao tem o que re-carimbar"
    );
    // E os botões de preset seguem lá: é por eles que a gaiola volta a ser dirigida.
    assert!(
        host.painted_rect::<VectorPanel>(
            &mut panel_state,
            VIEWPORT,
            ids::vector_envelope_preset_id(0)
        )
        .is_some(),
        "os presets sumiram junto — a gaiola manual ficaria sem porta de volta"
    );

    ph2d_panel_vector::set_current_envelope_presets(&["Arc", "Flag"], Some(1), 0.5);
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_ENVELOPE_BEND)
            .is_some(),
        "com preset ativo o Bend tem de existir"
    );
}

/// **O `Clear Pins` só existe no gesto PINOS** — e os presets/Bend só existem FORA dele. Os dois
/// grupos são exclusivos por construção: presets GERAM uma gaiola e o puppet não tem gaiola nenhuma.
///
/// Sem este gate a seção mostraria "Arc" e "Clear Pins" ao mesmo tempo, e cada um agiria sobre um
/// modelo que o outro não usa.
#[test]
fn the_pins_controls_and_the_cage_controls_are_exclusive() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    ph2d_panel_vector::set_current_has_envelope(true);
    ph2d_panel_vector::set_current_envelope_presets(&["Arc", "Flag"], Some(0), 0.5);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    // Gesto de gaiola (Perspective): presets sim, Clear Pins não.
    ph2d_panel_vector::set_current_envelope_mode(0);
    assert!(
        host.painted_rect::<VectorPanel>(
            &mut panel_state,
            VIEWPORT,
            ids::vector_envelope_preset_id(0)
        )
        .is_some(),
        "os presets sumiram no gesto de gaiola"
    );
    assert!(
        host.painted_rect::<VectorPanel>(
            &mut panel_state,
            VIEWPORT,
            ids::VECTOR_ENVELOPE_CLEAR_PINS
        )
        .is_none(),
        "Clear Pins apareceu num gesto que nao tem pinos"
    );

    // Gesto Pinos: o inverso exato.
    ph2d_panel_vector::set_current_envelope_mode(2);
    assert!(
        host.painted_rect::<VectorPanel>(
            &mut panel_state,
            VIEWPORT,
            ids::VECTOR_ENVELOPE_CLEAR_PINS
        )
        .is_some(),
        "Clear Pins nao foi oferecido no gesto Pinos"
    );
    assert!(
        host.painted_rect::<VectorPanel>(
            &mut panel_state,
            VIEWPORT,
            ids::vector_envelope_preset_id(0)
        )
        .is_none(),
        "um preset de GAIOLA foi oferecido no gesto Pinos"
    );
}

/// **Os botões da pilha de efeitos chegam ao bus quando o artista CLICA neles** — pelo
/// caminho inteiro (`paint` → hit-rect → ponteiro real → dispatcher → `event.rs` → bus).
///
/// ⚠️ **`set_current_effects` não é decoração do fixture: é a PREMISSA da seção.** Sem alvo
/// publicado, o `painted_rect` devolve `None` e o gate morre na 1ª asserção.
///
/// O gate varre a tabela PUBLICADA — um tipo de efeito novo é coberto sem que este arquivo
/// seja reescrito, que é a propriedade inteira do refactor dirigido-por-tabela.
#[test]
fn every_effect_stack_button_reaches_the_bus_when_clicked() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    const SEC: u128 = 1_000_000_000;
    const KINDS: &[&str] = &["Trim Path", "Zig Zag"];

    // Uma pilha de DUAS linhas: é o que faz o Up e o Down existirem (nas bordas eles não são
    // oferecidos, de propósito).
    // ⚠️ A linha traz um slider **e** uma CAIXINHA. Sem a caixinha na fixture, a varredura
    // nunca a via — e ela era o unico controlo da secao pintado como BOTAO sem id proprio: o
    // store registava um `Slider` no id partilhado, e *um slider nao emite Click no Up*, entao
    // o ramo que a alternava era codigo morto (Enio, 2026-07-18).
    let row = |label: &'static str| ph2d_panel_vector::FxRowView {
        label,
        enabled: true,
        params: vec![
            ph2d_panel_vector::FxParamView {
                name: "Start",
                min: 0.0,
                max: 1.0,
                toggle: false,
                integer: false,
                value: 0.25,
            },
            ph2d_panel_vector::FxParamView {
                name: "Smooth",
                min: 0.0,
                max: 1.0,
                toggle: true,
                integer: false,
                value: 0.0,
            },
        ],
    };
    let publish = || {
        ph2d_panel_vector::set_current_effects(true, KINDS, vec![row("Trim Path"), row("Zig Zag")]);
    };

    let mut targets: Vec<(ph2d_a11y::NodeId, String)> = Vec::new();
    for (k, name) in KINDS.iter().enumerate() {
        targets.push((ids::vector_fx_add_id(k), format!("Add {name}")));
    }
    targets.push((ids::vector_fx_remove_id(0), "rótulo/remove linha 0".into()));
    targets.push((ids::vector_fx_down_id(0), "Down linha 0".into()));
    targets.push((ids::vector_fx_up_id(1), "Up linha 1".into()));
    targets.push((ids::vector_fx_hide_id(0), "olho linha 0".into()));
    // O botão de SEÇÃO "Apply" — só é pintado com a pilha não-vazia (que a fixture publica).
    targets.push((ids::VECTOR_FX_APPLY, "Apply Effects".into()));
    // A CAIXINHA: e' um botao, e tem de emitir Click como qualquer outro.
    targets.push((
        ids::vector_fx_toggle_id(0, 1),
        "caixinha Smooth linha 0".into(),
    ));

    for (id, name) in targets {
        publish();
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .unwrap_or_else(|| panic!("{name} nao foi PINTADO com area clicavel na secao Effects"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {name} nao virou Click — falta o registro no `populate_effects`"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id)
            ),
            "o Click em {name} nao chegou ao bus — falta o id em `forwards_plain_click` \
             (o botao e clicavel e MORTO)"
        );
    }
    ph2d_panel_vector::set_current_effects(false, &[], Vec::new());
}

/// **Sem alvo, a seção NÃO oferece nada** — nem os botões de Add.
///
/// A seção é por-caminho: com dois selecionados *"a pilha"* não tem referente, e um Add que
/// escrevesse num deles em silêncio seria pior que não existir.
#[test]
fn the_effect_section_offers_nothing_without_a_single_target() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    ph2d_panel_vector::set_current_effects(false, &["Trim Path"], Vec::new());
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::vector_fx_add_id(0))
            .is_none(),
        "o Add foi pintado sem caminho unico selecionado"
    );

    // E COM alvo ele aparece — sem esta metade, "não oferece" ficaria verde com a seção
    // inteira apagada. [[feedback_absence_gate_needs_a_presence_sibling]]
    ph2d_panel_vector::set_current_effects(true, &["Trim Path"], Vec::new());
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::vector_fx_add_id(0))
            .is_some(),
        "com alvo, o Add TEM de ser oferecido"
    );
    ph2d_panel_vector::set_current_effects(false, &[], Vec::new());
}

/// **O chip mostra o número do DOCUMENTO durante o arrasto, não o track `0..1`.**
///
/// Enio, 2026-07-18: *"o número que aparece ao arrastar a caixa numérica é de 0 a 1, sendo que
/// os números reais (quando solta o mouse) são outros"*.
///
/// O slider guarda `0..1` e o chip está ligado a ele. Registados na IDENTIDADE, o chip repetia
/// o track durante todo o gesto e só ganhava o valor certo quando o snapshot voltava. O canal é
/// o `link_slider_number_mapped`, e a faixa de cada parâmetro só se sabe no frame — daí o
/// `seed_effect_ranges`, que corre no `paint`.
///
/// **Este gate afirma a condição, não um proxy:** pergunta ao store a projeção registada e a
/// faixa do chip, e exige que sejam as do EFEITO. Sem o seed, a projeção é `(1, 0)` e a faixa
/// `0..1` — exatamente o que se via na tela.
#[test]
fn the_effect_chip_carries_the_documents_range_not_the_normalised_track() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    // Faixas do produto: uma percentagem que vai a 100 e uma CONTAGEM que vai a 128. Uma faixa
    // `0..1` aqui deixaria o gate verde sobre o bug — a identidade e a verdade coincidiriam.
    let params = vec![
        ph2d_panel_vector::FxParamView {
            name: "Size",
            min: 0.0,
            max: 100.0,
            toggle: false,
            integer: false,
            value: 12.0,
        },
        ph2d_panel_vector::FxParamView {
            name: "Ridges",
            min: 1.0,
            max: 128.0,
            toggle: false,
            integer: true,
            value: 8.0,
        },
    ];
    ph2d_panel_vector::set_current_effects(
        true,
        &["Zig Zag"],
        vec![ph2d_panel_vector::FxRowView {
            label: "Zig Zag",
            enabled: true,
            params,
        }],
    );
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    // Pintar é o que republica — o `populate` corre uma vez e não sabe que efeito caiu na linha.
    let _ = host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::vector_fx_param_num_id(0, 0));

    let store = host.store();
    for (param, name, min, max) in [
        (0usize, "Size", 0.0_f64, 100.0_f64),
        (1, "Ridges", 1.0, 128.0),
    ] {
        let chip = ids::vector_fx_param_num_id(0, param);
        let (scale, offset) = store.linked_slider_mapping(chip);
        assert!(
            (f64::from(scale) - (max - min)).abs() < 1e-3 && (f64::from(offset) - min).abs() < 1e-3,
            "{name}: a projeção registada é ({scale}, {offset}) e devia ser ({}, {min}) — com a \
             identidade o chip repete o track `0..1` durante o gesto inteiro",
            max - min
        );
        let (rmin, rmax, _) = store
            .number_range(chip)
            .unwrap_or_else(|| panic!("{name}: sem `set_number_range` o arrasto escala errado"));
        assert!(
            (rmin - min).abs() < 1e-9 && (rmax - max).abs() < 1e-9,
            "{name}: a faixa do chip é {rmin}..{rmax} e devia ser {min}..{max}"
        );
    }
    ph2d_panel_vector::set_current_effects(false, &[], Vec::new());
}
