//! Seam dos **dois knobs do LÁPIS** — vivos sob o MOUSE, e o valor chega ao barramento.
//!
//! O irmão `seam_pencil.rs` prova o CHIP de modo; este prova a SEÇÃO que o modo abre. As duas
//! metades de cada slider são independentes: sair do `populate` mata a 1ª asserção (o ponteiro não
//! vira `ValueChanged`, porque a checagem de focabilidade mora no store), sair do `event` mata a 2ª
//! (o valor não chega ao bus e o slider anda sem fazer nada).
//!
//! ⚠️ **E há uma 3ª condição que só esta seção tem: ela SÓ EXISTE no modo Pencil.** Um seam que
//! pintasse o painel no modo default não acharia retângulo nenhum e passaria em silêncio se as
//! asserções fossem `if let Some(..)`; por isso o `painted_rect` aqui é `expect`, e a fixture
//! começa por escolher o modo.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
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

/// Publica o modo que a shell publicaria — sem isto a seção não é pintada e não há knob a clicar.
fn publish_mode(mode: ph2d_tool_vector::DrawMode) {
    ph2d_panel_vector::state::set_current_vector_style(Some(
        ph2d_tool_vector::VectorStyleSnapshot {
            mode,
            ..ph2d_tool_vector::VectorStyleSnapshot::default()
        },
    ));
}

/// Arrasta o botão de um slider e devolve os eventos que o gesto produziu.
fn drag_slider(
    host: &mut MockPanelHost,
    panel_state: &mut VectorPanelState,
    id: ph2d_a11y::NodeId,
) -> Vec<WidgetEvent> {
    let r = host
        .painted_rect::<VectorPanel>(panel_state, VIEWPORT, id)
        .unwrap_or_else(|| {
            panic!("o slider {id:?} nao foi PINTADO com area clicavel na secao Pencil")
        });
    let cy = r.y + r.h * 0.5;
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
    evs
}

/// **Os dois knobs são alcançáveis por um ponteiro e o valor chega ao bus.**
#[test]
fn the_pencil_knobs_are_reachable_by_a_pointer_and_reach_the_bus() {
    publish_mode(ph2d_tool_vector::DrawMode::Pencil);
    for id in [ids::VECTOR_PENCIL_FIDELITY, ids::VECTOR_PENCIL_STABILIZER] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let evs = drag_slider(&mut host, &mut panel_state, id);
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::ValueChanged(c) if *c == id)),
            "arrastar o slider {id:?} nao produziu ValueChanged — ele esta' desenhado e nao existe \
             para o dispatcher (falta o `register` no populate)"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, _)) if c == id
            )),
            "o valor do slider {id:?} nao chegou ao bus — o knob anda sob o mouse e nao faz nada \
             (falta o braco no event.rs)"
        );
    }
}

/// **A seção só existe no modo Pencil** — noutro modo os dois knobs não são nem pintados.
///
/// A metade da AUSÊNCIA vale tanto quanto a da presença: dois sliders que descrevem como a mão é
/// capturada, oferecidos num modo em que não há mão a capturar, são dois controles mortos.
#[test]
fn the_pencil_section_is_absent_outside_pencil_mode() {
    publish_mode(ph2d_tool_vector::DrawMode::Select);

    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for id in [ids::VECTOR_PENCIL_FIDELITY, ids::VECTOR_PENCIL_STABILIZER] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "o knob {id:?} foi pintado no modo Select — a secao descreve a captura da MAO livre, e \
             fora do lapis nao ha mao a capturar"
        );
    }
}
