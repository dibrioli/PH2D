//! Seam da seção **SNAP** — as seis opções vivas sob o MOUSE, e o clique chegando ao barramento.
//!
//! A W6 acrescentou duas linhas (Path e Cross) à linha que já existia (Shapes). O que este arquivo
//! prova é o que nenhum teste de unidade alcança: que os seis botões estão REGISTADOS, que o
//! ponteiro sobre eles vira `Click`, e que esse `Click` atravessa o painel até o barramento.
//!
//! ⚠️ **A varredura é sobre as três linhas, não só sobre as novas.** Um seam que só olhasse a
//! feature do dia deixaria de ver o dia em que um `populate` reescrito derrubasse a antiga — e a
//! falha seria *o interruptor parou de responder*, que ninguém atribui a uma wave de precisão.

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

/// As seis opções das três linhas chegam ao bus por um par Down+Up REAL.
///
/// ⚠️ `WidgetEvent::Click` sintético **pula a checagem de focabilidade do store**, então um botão
/// tirado do `populate` continuaria a "passar": ele fica pintado, com área de hit, e morto sob o
/// mouse. É por isso que o gesto é dirigido, e não fabricado.
#[test]
fn every_snap_option_reaches_the_bus() {
    for (id, name) in [
        (ids::VECTOR_SNAP_OFF, "Shapes/Off"),
        (ids::VECTOR_SNAP_ON, "Shapes/On"),
        (ids::VECTOR_SNAP_PATH_OFF, "Path/Off"),
        (ids::VECTOR_SNAP_PATH_ON, "Path/On"),
        (ids::VECTOR_SNAP_CROSS_OFF, "Cross/Off"),
        (ids::VECTOR_SNAP_CROSS_ON, "Cross/On"),
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .unwrap_or_else(|| panic!("{name} nao foi PINTADO com area clicavel na secao Snap"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {name} nao virou Click — o botao esta' desenhado e nao existe para \
             o dispatcher"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id)
            ),
            "o Click de {name} nao chegou ao bus — o botao acende sob o mouse e nao faz nada"
        );
    }
}

/// **A seção empilha TRÊS linhas independentes**, cada uma com o seu par Off/On lado a lado.
///
/// ⚠️ **Isto é menos do que eu quis afirmar, e a mutação me corrigiu.** A primeira versão
/// chamava-se *"cada linha mostra o seu próprio estado"* e mutar `current_snap_crossings()` para
/// `current_snap_path()` **não a fazia falhar**: `painted_rect` devolve GEOMETRIA, e as duas
/// opções de uma linha são pintadas nas mesmas posições esteja qual estiver acesa — o realce não
/// é observável por aqui. Quem prova que os dois interruptores não estão cruzados é o arch-gate
/// da shell (`the_snap_toggles_are_not_crossed`), onde a fiação de facto mora.
#[test]
fn the_snap_section_stacks_three_independent_rows() {
    ph2d_panel_vector::set_current_snap(true);
    ph2d_panel_vector::set_current_snap_position(true, false);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let rect = |host: &mut MockPanelHost, st: &mut VectorPanelState, id| {
        host.painted_rect::<VectorPanel>(st, VIEWPORT, id)
            .expect("a linha e' pintada")
    };
    // As três linhas empilham: cada uma num `y` próprio, e nenhuma some.
    let shapes = rect(&mut host, &mut panel_state, ids::VECTOR_SNAP_ON);
    let path = rect(&mut host, &mut panel_state, ids::VECTOR_SNAP_PATH_ON);
    let cross = rect(&mut host, &mut panel_state, ids::VECTOR_SNAP_CROSS_ON);
    assert!(
        shapes.y < path.y && path.y < cross.y,
        "as tres linhas ocupam alturas distintas: {shapes:?} {path:?} {cross:?}"
    );
    // E o par Off/On de uma linha são dois retângulos, não um: o estado é escolhível.
    let path_off = rect(&mut host, &mut panel_state, ids::VECTOR_SNAP_PATH_OFF);
    assert!(
        (path_off.x - path.x).abs() > 1.0 && (path_off.y - path.y).abs() < 1.0,
        "Off e On da linha Path sao lado a lado: {path_off:?} vs {path:?}"
    );
    ph2d_panel_vector::set_current_snap_position(false, false);
}
