//! Seam dos **COMPONENTES** (plano UI/UX W5) — os quatro verbos estão vivos sob o MOUSE, chegam
//! ao barramento, e cada um aparece **só onde faz sentido**.
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
use ph2d_panel_vector::state::{ComponentState, VectorPanelState};
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

fn clear() {
    state::set_component_state(None);
}

/// Uma forma comum (só o *Create* faz sentido).
fn plain() -> ComponentState {
    ComponentState::default()
}

/// Um mestre (só o *Place*).
fn main_shape() -> ComponentState {
    ComponentState {
        is_main: true,
        ..ComponentState::default()
    }
}

/// Uma instância COM overrides (Detach + Reset).
fn instance_with_overrides() -> ComponentState {
    ComponentState {
        is_instance: true,
        has_overrides: true,
        ..ComponentState::default()
    }
}

fn rect_under(st: ComponentState, id: ph2d_a11y::NodeId) -> Option<Rect> {
    state::set_component_state(Some(st));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
fn click_reaches_bus(st: ComponentState, id: ph2d_a11y::NodeId, what: &str) {
    state::set_component_state(Some(st));
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

/// **Os QUATRO verbos estão vivos e chegam ao bus**, cada um no estado em que é oferecido.
#[test]
fn all_four_component_verbs_are_reachable_and_reach_the_bus() {
    clear();
    click_reaches_bus(plain(), ids::VECTOR_COMPONENT_CREATE, "Create Component");
    click_reaches_bus(main_shape(), ids::VECTOR_COMPONENT_PLACE, "Place Instance");
    click_reaches_bus(
        instance_with_overrides(),
        ids::VECTOR_COMPONENT_DETACH,
        "Detach Instance",
    );
    click_reaches_bus(
        instance_with_overrides(),
        ids::VECTOR_COMPONENT_RESET,
        "Reset Overrides",
    );
    clear();
}

/// **Cada verbo aparece SÓ onde faz sentido** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela, o gate acima ficaria verde sobre uma seção que pinta os quatro botões sempre, três
/// deles inertes — que é o botão-morto que este repo persegue e que ensina o artista a duvidar
/// dos outros.
#[test]
fn each_verb_appears_only_where_it_makes_sense() {
    clear();
    // Uma forma comum: Create sim, os outros três não.
    assert!(rect_under(plain(), ids::VECTOR_COMPONENT_CREATE).is_some());
    for id in [
        ids::VECTOR_COMPONENT_PLACE,
        ids::VECTOR_COMPONENT_DETACH,
        ids::VECTOR_COMPONENT_RESET,
    ] {
        assert!(
            rect_under(plain(), id).is_none(),
            "um verbo de instância foi oferecido sobre uma forma comum"
        );
    }
    // Um mestre: Place sim, Create não (ele já é um).
    assert!(rect_under(main_shape(), ids::VECTOR_COMPONENT_PLACE).is_some());
    assert!(rect_under(main_shape(), ids::VECTOR_COMPONENT_CREATE).is_none());
    // Uma instância LIMPA: Detach sim, Reset não — um reset que não reseta nada é um clique que
    // não faz nada, e o artista não tem como o saber antes de o dar.
    let clean = ComponentState {
        is_instance: true,
        ..ComponentState::default()
    };
    assert!(rect_under(clean, ids::VECTOR_COMPONENT_DETACH).is_some());
    assert!(rect_under(clean, ids::VECTOR_COMPONENT_RESET).is_none());
    clear();
}

/// **Sem estado publicado a seção não existe.**
#[test]
fn the_section_is_not_painted_without_a_selection() {
    clear();
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for id in [
        ids::VECTOR_SECTION_COMPONENT,
        ids::VECTOR_COMPONENT_CREATE,
        ids::VECTOR_COMPONENT_PLACE,
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "a secao de componente foi pintada sem selecao"
        );
    }
}
