//! Seam da fileira **Width: Auto | Fixed** e do slider de largura (W2a — o refluxo do texto).
//!
//! O que nenhum teste de unidade alcança: que os dois chips estão REGISTADOS, que o ponteiro
//! sobre eles vira `Click`, que esse `Click` atravessa o painel até o barramento — e que o
//! slider **só é pintado no modo Fixed**, que é a lei do knob-morto aplicada a um `Option`.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// A seção Text só é pintada com um texto em foco — a mesma premissa de todos os knobs dela.
fn arm_text() {
    ph2d_panel_vector::set_current_text_visible(true);
    ph2d_panel_vector::set_current_text(Some("hello world".to_owned()));
}

/// **Os dois chips chegam ao bus por um par Down+Up REAL.**
///
/// ⚠️ `WidgetEvent::Click` sintético **pula a checagem de focabilidade do store**, então um chip
/// tirado do `populate` continuaria a "passar": pintado, com área de hit, e morto sob o mouse.
#[test]
fn both_width_chips_reach_the_bus() {
    for (id, name) in [
        (ids::VECTOR_TEXT_WRAP_AUTO, "Width/Auto"),
        (ids::VECTOR_TEXT_WRAP_FIXED, "Width/Fixed"),
    ] {
        arm_text();
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .unwrap_or_else(|| panic!("{name} nao foi PINTADO com area clicavel na secao Text"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {name} nao virou Click - o chip esta' desenhado e nao existe \
             para o dispatcher"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id)
            ),
            "o Click de {name} nao chegou ao bus - o chip acende sob o mouse e nao faz nada"
        );
    }
}

/// ⭐ **PRESENÇA e AUSÊNCIA: o slider de largura só existe no modo Fixed.**
///
/// É a lei do knob-morto sobre um `Option`: em `Auto` não há largura nenhuma a editar, e um
/// slider ali seria um controle que não faz nada. As duas metades num gate só — afirmar apenas
/// a presença deixaria passar um painel que o pinta sempre.
#[test]
fn the_width_slider_lives_only_in_fixed_mode() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    arm_text();
    ph2d_panel_vector::set_current_text_wrap(None); // Auto
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_TEXT_WRAP_W)
            .is_none(),
        "em Auto o slider de largura NAO pode ser pintado - nao ha largura a editar"
    );
    // ⚠️ O controle: a fileira que o comanda continua lá. Sem esta metade o gate ficaria verde
    // sobre uma seção Text que sumiu inteira.
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_TEXT_WRAP_AUTO)
            .is_some(),
        "a fileira Width tem de continuar pintada em Auto"
    );

    arm_text();
    ph2d_panel_vector::set_current_text_wrap(Some(8.0)); // Fixed
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_TEXT_WRAP_W)
            .is_some(),
        "em Fixed o slider de largura TEM de ser pintado - senao a largura e' ineditavel"
    );
    ph2d_panel_vector::set_current_text_wrap(None);
}

/// **O slider de largura chega ao bus como `SetValue`** — o mesmo formato dos irmãos
/// (Size/Weight/Line height/Tracking), que é o que faz a shell drená-lo sem caso especial.
///
/// ⚠️ O gesto é o idioma canónico deste painel (`set_slider_value` + `ValueChanged`), e não um
/// Down/Move/Up: um arrasto REAL sobre o trilho só emite `ValueChanged` se o ponteiro de facto
/// mover o valor, e a primeira versão deste gate media o retângulo do trilho e clicava no meio
/// dele — onde o valor já estava. O que se prova aqui é o BRAÇO do `event.rs`.
#[test]
fn dragging_the_width_slider_reaches_the_bus() {
    arm_text();
    ph2d_panel_vector::set_current_text_wrap(Some(8.0));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.set_slider_value(ids::VECTOR_TEXT_WRAP_W, 0.75);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_TEXT_WRAP_W),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "o painel ignorou um edit real do slider - falta o braco em `event.rs`"
    );
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, _)) if c == ids::VECTOR_TEXT_WRAP_W
        )),
        "arrastar o slider de largura nao chegou ao bus - o refluxo seria ineditavel"
    );
    ph2d_panel_vector::set_current_text_wrap(None);
}
