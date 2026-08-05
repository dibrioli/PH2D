//! Seam dos **ESTADOS de UI** (plano UI/UX W7) — os três verbos estão vivos sob o MOUSE, chegam
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
use ph2d_panel_vector::state::{UiStatesState, VectorPanelState};
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
    state::set_ui_states_state(None);
}

/// Uma seleção com os papéis de `recorded` gravados.
fn with(recorded: [bool; 4]) -> UiStatesState {
    UiStatesState {
        recorded,
        role_labels: [
            "Default".into(),
            "Hover".into(),
            "Pressed".into(),
            "Disabled".into(),
        ],
        live: None,
        duration_s: 0.15,
    }
}

fn rect_under(st: UiStatesState, id: ph2d_a11y::NodeId) -> Option<Rect> {
    state::set_ui_states_state(Some(st));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

/// Clica de verdade no widget `id` e exige que o Click chegue ao barramento.
fn click_reaches_bus(st: UiStatesState, id: ph2d_a11y::NodeId, what: &str) {
    state::set_ui_states_state(Some(st));
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

/// **O Rec de TODO papel está vivo e chega ao bus, com a seleção em branco.**
///
/// ⚠️ A face VAZIA é a que importa: se o Rec só existisse onde já há pose, a feature seria
/// alcançável apenas onde ela já foi usada — ou seja, em lugar nenhum.
#[test]
fn every_role_can_be_recorded_from_an_empty_selection() {
    clear();
    for i in 0..ids::MAX_STATE_ROLES {
        click_reaches_bus(
            with([false; 4]),
            ids::vector_state_record_id(i),
            &format!("Rec do papel {i}"),
        );
    }
    clear();
}

/// **Show e Clear existem, e chegam ao bus, no papel que TEM pose.**
#[test]
fn show_and_clear_are_reachable_on_a_recorded_role() {
    clear();
    let mut rec = [false; 4];
    rec[1] = true;
    click_reaches_bus(with(rec), ids::vector_state_apply_id(1), "Show do Hover");
    click_reaches_bus(with(rec), ids::vector_state_clear_id(1), "Clear do Hover");
    clear();
}

/// **Show e Clear NÃO existem num papel vazio** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela o gate acima ficaria verde sobre uma seção que pinta os três botões sempre, dois
/// deles inertes — o botão-morto que este repo persegue e que ensina o artista a duvidar dos
/// outros.
#[test]
fn an_empty_role_offers_only_the_recorder() {
    clear();
    assert!(
        rect_under(with([false; 4]), ids::vector_state_record_id(0)).is_some(),
        "o Rec sumiu do papel vazio — nao ha' como comecar"
    );
    assert!(
        rect_under(with([false; 4]), ids::vector_state_apply_id(0)).is_none(),
        "o Show foi pintado num papel SEM pose: um clique que nao pode fazer nada"
    );
    assert!(
        rect_under(with([false; 4]), ids::vector_state_clear_id(0)).is_none(),
        "o Clear foi pintado num papel SEM pose: um clique que nao pode fazer nada"
    );
    clear();
}

/// **Sem seleção única, a seção inteira não é pintada.**
#[test]
fn no_host_no_section() {
    clear();
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for i in 0..ids::MAX_STATE_ROLES {
        assert!(
            host.painted_rect::<VectorPanel>(
                &mut panel_state,
                VIEWPORT,
                ids::vector_state_record_id(i)
            )
            .is_none(),
            "a secao de estados foi pintada sem hospedeiro"
        );
    }
}

/// **A DURAÇÃO está viva e o valor chega ao bus.**
///
/// ⚠️ O slider é arrastado de verdade: um `ValueChanged` sintético provaria o `apply_event` e
/// pularia o registro no store, que é onde um slider pintado nasce imóvel.
#[test]
fn the_duration_slider_is_alive_and_reaches_the_bus() {
    clear();
    state::set_ui_states_state(Some(with([true, false, false, false])));
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_STATE_DURATION)
        .expect("o slider de duracao nao foi PINTADO");
    let cy = r.y + r.h * 0.5;
    // ⚠️ O Down cai num ponto e o Move noutro: um Move para o MESMO x não move o valor, e o gate
    // ficaria verde-sobre-nada por não haver `ValueChanged` para ninguém encaminhar.
    host.dispatch_pointer_event(pointer(PointerKind::Down, r.x + r.w * 0.25, cy, SEC));
    let mut evs = host.dispatch_pointer_event(pointer(
        PointerKind::Move,
        r.x + r.w * 0.75,
        cy,
        SEC + SEC / 100,
    ));
    evs.extend(host.dispatch_pointer_event(pointer(
        PointerKind::Up,
        r.x + r.w * 0.75,
        cy,
        SEC + SEC / 50,
    )));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(c) if *c == ids::VECTOR_STATE_DURATION)),
        "arrastar a duracao nao produziu ValueChanged — ela esta' desenhada e nao existe para o \
         dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, _))
                if c == ids::VECTOR_STATE_DURATION
        )),
        "arrastar a duracao nao chegou ao bus — o numero autorado nunca alcanca o documento"
    );
    clear();
}
