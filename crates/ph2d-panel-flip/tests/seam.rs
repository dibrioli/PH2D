//! Behavioral SEAM test for the Flip panel ↔ tool (blindagem Fase 1.2).
//!
//! Unit tests in `ph2d-tool-flip` exercise `handle_panel_event` directly, and
//! `populate.rs` registers widgets — but NEITHER proves the wire between them is
//! intact. A forgotten `event.rs` arm or a wrong id would leave a slider painted,
//! draggable and SILENTLY DEAD while every unit test + contract gate stays green.
//!
//! These run the full path the desktop shell runs, headless:
//!   populate → set value / click → apply_event → bus → handle_panel_event
//!   → assert the tool's state actually changed.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_flip::state::FlipPanelState;
use ph2d_panel_flip::{FlipPanel, ids};
use ph2d_tool_flip::{FlipMode, FlipTool, WIDTH_MAX_PX};
use ph2d_ui_testkit::MockPanelHost;

/// Drag the Size slider to its full end and prove the width reaches the tool —
/// exercising every site from `populate` to `width_px()`.
#[test]
fn size_slider_drag_reaches_tool() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut panel_state = FlipPanelState;
    let mut tool = FlipTool::default();

    host.set_slider_value(ids::FLIP_SIZE, 1.0);
    let outcome = host.apply_panel_event::<FlipPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::FLIP_SIZE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` arm for FLIP_SIZE is missing"
    );

    let mut forwarded = false;
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
            forwarded = true;
        }
    }
    assert!(
        forwarded,
        "slider edit never reached the bus as a ToolPanelEvent — the seam is dead"
    );
    assert_eq!(
        tool.width_px(),
        WIDTH_MAX_PX,
        "slider→tool seam delivered the wrong px for Size"
    );
}

/// Clicking the Draw mode button must switch the tool's canvas mode through the
/// seam (Select → Draw).
#[test]
fn draw_mode_button_switches_the_tool_mode() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut panel_state = FlipPanelState;
    let mut tool = FlipTool::default();
    assert_eq!(tool.mode(), FlipMode::Select, "fresh tool starts in Select");

    let outcome = host
        .apply_panel_event::<FlipPanel>(&mut panel_state, WidgetEvent::Click(ids::FLIP_MODE_DRAW));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Draw button click ignored — `event.rs` arm for FLIP_MODE_DRAW is missing"
    );

    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(
        tool.mode(),
        FlipMode::Draw,
        "mode button never switched the tool mode through the seam"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PAINTED ≠ POPULATED (a auditoria do "não existe botão fill")
//
// Tudo acima prova que o clique CHEGA na tool. Nada acima prova que o botão está
// NA TELA. O gate `architecture_panel_wiring_parity` lê o TEXTO-FONTE; os seams
// leem o barramento. Nenhum dos dois roda `paint`. Então um widget pode estar
// registrado, wirado, unit-testado e contract-limpo enquanto a chamada de pintura
// dele mora atrás de um `return` — e o relato do usuário é "o botão não existe",
// com todos os gates verdes.
//
// O que o usuário pode clicar é o que a PINTURA registrou. É isso que se lê aqui.
// ─────────────────────────────────────────────────────────────────────────────

/// Um viewport de desktop plausível — o painel ancora no dock direito do layout.
fn viewport() -> ph2d_editor_core::zones::Rect {
    ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0)
}

/// **Todo botão de MODO é pintado e clicável** — inclusive o Fill (W4).
///
/// Mutação que sangra: apague a entrada do Fill do `mode_row` e este teste fica
/// vermelho, enquanto TODOS os outros gates do projeto seguem verdes.
#[test]
fn every_mode_button_is_painted_and_clickable() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState;
    let painted = host.paint::<FlipPanel>(&mut st, viewport());

    for (id, name) in [
        (ids::FLIP_MODE_SELECT, "Select"),
        (ids::FLIP_MODE_DRAW, "Draw"),
        (ids::FLIP_MODE_ERASE, "Erase"),
        (ids::FLIP_MODE_FILL, "Fill"),
    ] {
        let hit = painted.iter().find(|(w, _)| *w == id);
        let Some((_, r)) = hit else {
            panic!("o botao de modo {name} NAO e pintado: nao existe na tela");
        };
        assert!(
            r.w > 0.0 && r.h > 0.0,
            "o botao de modo {name} foi pintado com area ZERO: invisivel e inclicavel ({r:?})"
        );
    }
}

/// A seção do balde é **modal**: só aparece no modo Fill. Fora dele, os widgets do
/// balde não podem estar clicáveis (um widget clicável e invisível é uma armadilha).
///
/// O snapshot vive num global (o shell publica; o painel lê), então o teste o
/// escreve como o `flip_bridge` escreve.
#[test]
fn the_bucket_widgets_appear_only_in_fill_mode() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut st = FlipPanelState;
    let bucket = [
        ids::FLIP_FILL_SWATCH,
        ids::FLIP_FILL_PAINT,
        ids::FLIP_FILL_BEHIND,
        ids::FLIP_FILL_UNPAINT,
        ids::FLIP_GAP,
        ids::FLIP_GROW,
        ids::FLIP_PRECISION,
    ];

    // Modo Draw: o balde não existe na tela.
    let mut snap = ph2d_tool_flip::FlipStyleSnapshot {
        mode: FlipMode::Draw,
        ..Default::default()
    };
    ph2d_panel_flip::set_current_flip_style(Some(snap));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    for id in bucket {
        assert!(
            !painted.iter().any(|(w, _)| *w == id),
            "widget do balde {id:?} pintado FORA do modo Fill"
        );
    }

    // Modo Fill: todos existem, com área.
    snap.mode = FlipMode::Fill;
    ph2d_panel_flip::set_current_flip_style(Some(snap));
    let painted = host.paint::<FlipPanel>(&mut st, viewport());
    for id in bucket {
        let hit = painted
            .iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0);
        assert!(
            hit.is_some(),
            "widget do balde {id:?} NAO e pintado no modo Fill: a secao esta morta"
        );
    }
}

/// **Toda caixa numérica registra o seu RANGE.**
///
/// Sem `set_number_range`, a caixa continua pintando, aceitando digitação e passando no
/// gate de wiring — mas o ARRASTO deriva o passo do texto do buffer e anda ~50 unidades
/// por pixel: um pixel de gesto vai do mínimo ao máximo. O widget parece vivo e é
/// inutilizável. Nenhum teste via isso, porque todos digitavam o valor.
///
/// Mutação que sangra: tire o `set_number_range` do `slider_chip` e este teste cai.
#[test]
fn every_number_box_has_a_registered_range() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::panel::PanelHostInternal; // traz o `store()`
    let host = MockPanelHost::with_panel::<FlipPanel>();
    let store = host.store();

    let boxes = [
        ("Size", ids::FLIP_SIZE_NUM),
        ("Hardness", ids::FLIP_HARDNESS_NUM),
        ("Opacity", ids::FLIP_OPACITY_NUM),
        ("Smoothing", ids::FLIP_SMOOTHING_NUM),
        ("Gap", ids::FLIP_GAP_NUM),
        ("Grow", ids::FLIP_GROW_NUM),
        ("Precision", ids::FLIP_PRECISION_NUM),
    ];
    for (name, id) in boxes {
        assert!(
            matches!(store.get(id), Some(InteractiveState::NumberInput { .. })),
            "a caixa {name} nem esta registrada"
        );
        let range = store.number_range(id);
        let Some((min, max, step)) = range else {
            panic!(
                "a caixa {name} nao registrou range: o arrasto vai andar ~50 unidades por pixel"
            );
        };
        assert!(max > min, "range invertido em {name}: [{min}, {max}]");
        assert!(step > 0.0, "step nao-positivo em {name}: {step}");
    }
}

/// **O painel de cada ferramenta não exibe os atributos das outras** (Enio 2026-07-12).
///
/// Um controle que não faz nada é pior que a ausência dele: o usuário mexe, nada muda, e
/// conclui que o app está quebrado. O Hardness do pincel no modo balde era exatamente isso.
///
/// A tabela é a especificação viva. Mutação que sangra: pinte o Brush no modo Fill e este
/// teste cai.
#[test]
fn each_mode_shows_only_its_own_attributes() {
    let stroke_only = [
        ("Hardness", ids::FLIP_HARDNESS),
        ("Smoothing", ids::FLIP_SMOOTHING),
        ("Stroke color", ids::FLIP_STROKE_SWATCH),
    ];
    let eraser_only = [
        ("Erase soft", ids::FLIP_ERASE_SOFT),
        ("Erase hard", ids::FLIP_ERASE_HARD),
        ("Erase stroke", ids::FLIP_ERASE_STROKE),
    ];
    let bucket_only = [
        ("Fill color", ids::FLIP_FILL_SWATCH),
        ("Gap", ids::FLIP_GAP),
        ("Grow", ids::FLIP_GROW),
        ("Precision", ids::FLIP_PRECISION),
    ];

    // (modo, o que TEM de aparecer, o que NÃO pode aparecer)
    #[allow(clippy::type_complexity)] // (modo, o que aparece, o que NAO pode aparecer)
    let cases: [(
        FlipMode,
        &[(&str, ph2d_a11y::NodeId)],
        &[&[(&str, ph2d_a11y::NodeId)]],
    ); 4] = [
        (FlipMode::Draw, &stroke_only, &[&eraser_only, &bucket_only]),
        (FlipMode::Erase, &eraser_only, &[&stroke_only, &bucket_only]),
        (FlipMode::Fill, &bucket_only, &[&stroke_only, &eraser_only]),
        // Select move/gira o objeto: não tem atributo de pintura nenhum.
        (
            FlipMode::Select,
            &[],
            &[&stroke_only, &eraser_only, &bucket_only],
        ),
    ];

    for (mode, expected, forbidden) in cases {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState;
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        let on_screen = |id: ph2d_a11y::NodeId| painted.iter().any(|(w, r)| *w == id && r.w > 0.0);

        for (name, id) in expected {
            assert!(
                on_screen(*id),
                "modo {mode:?}: o proprio controle '{name}' NAO aparece"
            );
        }
        for group in forbidden {
            for (name, id) in *group {
                assert!(
                    !on_screen(*id),
                    "modo {mode:?}: aparece '{name}', que e atributo de OUTRA ferramenta"
                );
            }
        }
    }
}

/// O **Size** é o único atributo compartilhado: é a espessura do pincel E o raio da
/// borracha. Ele aparece nos dois — e some no balde e no Select.
#[test]
fn size_is_shared_by_brush_and_eraser_and_absent_elsewhere() {
    for (mode, want) in [
        (FlipMode::Draw, true),
        (FlipMode::Erase, true),
        (FlipMode::Fill, false),
        (FlipMode::Select, false),
    ] {
        let mut host = MockPanelHost::with_panel::<FlipPanel>();
        let mut st = FlipPanelState;
        ph2d_panel_flip::set_current_flip_style(Some(ph2d_tool_flip::FlipStyleSnapshot {
            mode,
            ..Default::default()
        }));
        let painted = host.paint::<FlipPanel>(&mut st, viewport());
        let shown = painted
            .iter()
            .any(|(w, r)| *w == ids::FLIP_SIZE && r.w > 0.0);
        assert_eq!(shown, want, "modo {mode:?}: Size deveria aparecer? {want}");
    }
}
