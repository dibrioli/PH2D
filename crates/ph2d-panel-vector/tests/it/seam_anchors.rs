//! Seam das **ÂNCORAS** (plano UI/UX W3) — os oito chips estão vivos sob o MOUSE, chegam ao
//! barramento, e a seção **não é pintada** onde não há regra a autorar.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou as 36 células da matriz de física e os dez chips de ferramenta
//! do Painter *pintados, hit-registrados e mortos sob o ponteiro*.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{AnchorState, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
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

/// O neutro — a publicação que faz a seção existir para um filho sem regra armada.
fn neutral() -> AnchorState {
    AnchorState {
        h: Some(ids::VECTOR_ANCHOR_H_START),
        v: Some(ids::VECTOR_ANCHOR_V_END),
    }
}

fn clear() {
    state::set_anchor_state(None);
}

fn rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click — ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus — ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// **Os OITO chips estão vivos e chegam ao bus.**
///
/// ⚠️ A lista é percorrida inteira de propósito: um chip que fique de fora do `populate` pinta,
/// acende sob o mouse e o Click morre no painel — o artista clicaria "Right" e nada aconteceria.
#[test]
fn all_eight_anchor_chips_are_reachable_and_reach_the_bus() {
    clear();
    state::set_anchor_state(Some(neutral()));
    for (id, what) in [
        (ids::VECTOR_ANCHOR_H_START, "H Left"),
        (ids::VECTOR_ANCHOR_H_CENTER, "H Center"),
        (ids::VECTOR_ANCHOR_H_END, "H Right"),
        (ids::VECTOR_ANCHOR_H_STRETCH, "H Stretch"),
        (ids::VECTOR_ANCHOR_V_START, "V Top"),
        (ids::VECTOR_ANCHOR_V_CENTER, "V Center"),
        (ids::VECTOR_ANCHOR_V_END, "V Bottom"),
        (ids::VECTOR_ANCHOR_V_STRETCH, "V Stretch"),
    ] {
        state::set_anchor_state(Some(neutral()));
        click_reaches_bus(id, what);
    }
    clear();
}

/// **Sem regra publicada a seção não existe** — a metade da AUSÊNCIA.
///
/// Sem ela, o gate acima ficaria verde sobre uma seção que aparece em toda seleção, e o artista
/// veria dois controlos de posição sobre uma forma solta que não tem moldura nenhuma.
#[test]
fn the_section_is_not_painted_without_an_anchorable_child() {
    clear();
    for id in [
        ids::VECTOR_SECTION_ANCHORS,
        ids::VECTOR_ANCHOR_H_START,
        ids::VECTOR_ANCHOR_V_STRETCH,
    ] {
        assert!(
            rect(id).is_none(),
            "a secao de ancoras foi pintada sem filho ancoravel"
        );
    }
}
