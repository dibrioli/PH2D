//! Seam da **MOLA** (W7m) — *ela troca as linhas, não as soma*.
//!
//! ⚠️ O gate central é de **PRESENÇA E AUSÊNCIA**: sem a metade da ausência, um painel que
//! pintasse as quatro linhas ao mesmo tempo passaria — e o artista teria dois modelos para manter
//! de acordo, com a cena a obedecer a um deles sem dizer qual.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{UiStatesState, VectorPanelState};
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

/// A seção UI States com um hospedeiro autorado — e a mola ligada ou não.
fn arm(spring: Option<(f32, f32)>) {
    ph2d_panel_vector::state::set_ui_states_state(Some(UiStatesState {
        recorded: [true, true, false, false],
        role_labels: [
            "Default".into(),
            "Hover".into(),
            "Pressed".into(),
            "Disabled".into(),
        ],
        live: None,
        duration_s: 0.15,
        easing: ph2d_anim::Easing::new(ph2d_anim::EasingFamily::Cubic, ph2d_anim::EasingMode::Out),
        spring,
        preview: Some(false),
        move_all: Some(true),
        // A tabela sinal -> papel: vazia por omissao nesta fixture.
        bindings: Vec::new(),
    }));
}

/// ⭐ **PRESENÇA e AUSÊNCIA: uma família de linhas de cada vez.**
#[test]
fn the_spring_rows_replace_the_duration_and_curve_rows() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;

    arm(None);
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_DURATION)
            .is_some(),
        "sem mola, a DURACAO tem de estar pintada"
    );
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_STIFFNESS)
            .is_none(),
        "sem mola, a RIGIDEZ nao pode ser pintada — seria um controle que nao faz nada"
    );

    arm(Some((12.0, 1.0)));
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_STIFFNESS)
            .is_some(),
        "com mola, a RIGIDEZ tem de estar pintada"
    );
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_DAMPING)
            .is_some(),
        "com mola, o AMORTECIMENTO tem de estar pintado"
    );
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_DURATION)
            .is_none(),
        "com mola, a DURACAO nao pode sobrar — uma mola nao TEM duracao, e deixa-la ali seria \
         um numero que a cena ignora"
    );
    // ⚠️ O controle: o checkbox continua lá nos dois modos, senão não haveria como voltar.
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_SPRING)
            .is_some(),
        "o checkbox tem de sobreviver ao proprio modo — sem ele a mola seria um caminho sem volta"
    );
    arm(None);
}

/// **O checkbox chega ao bus por um par Down+Up REAL.**
///
/// ⚠️ `WidgetEvent::Click` sintético **pula a checagem de focabilidade do store**, então uma caixa
/// tirada do `populate` continuaria a "passar": pintada, com área de hit, e morta sob o mouse.
#[test]
fn the_spring_checkbox_reaches_the_bus() {
    arm(None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_STATE_SPRING)
        .expect("o checkbox da mola e' pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_STATE_SPRING)),
        "o ponteiro sobre o checkbox nao virou Click"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_STATE_SPRING
        )),
        "o Click do checkbox nao chegou ao bus — a mola seria inalcancavel"
    );
}

/// **Os dois sliders chegam ao bus como `SetValue`.**
#[test]
fn both_spring_knobs_reach_the_bus() {
    for id in [ids::VECTOR_STATE_STIFFNESS, ids::VECTOR_STATE_DAMPING] {
        arm(Some((12.0, 1.0)));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.set_slider_value(id, 0.75);
        host.apply_panel_event::<VectorPanel>(&mut st, WidgetEvent::ValueChanged(id));
        assert!(
            host.drained_actions().into_iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, _)) if c == id)
            ),
            "o knob {id:?} nao chegou ao bus — ele arrastaria e nao mudaria nada"
        );
    }
    arm(None);
}
