//! Seam da seção **VERTEX** — os três chips de tipo continuam VIVOS numa seleção MISTA.
//!
//! O que este arquivo prova é o que nenhum teste de unidade alcança: que os chips estão registados,
//! que o ponteiro sobre eles vira `Click`, e que esse `Click` atravessa o painel até o barramento —
//! **inclusive quando a seleção mistura tipos**, que é o estado em que nenhum deles acende.
//!
//! ⚠️ **É a metade que a cura podia ter custado.** Ao parar de acender um chip sobre seleção mista
//! (auditoria do plano 25, item 5), a saída errada seria desligá-los: retipar é justamente o gesto de
//! *tornar* a seleção uniforme, e sem ele o artista fica preso no estado que o painel não descreve —
//! um controle que só existe quando já não é preciso.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::{VertexSel, VertexType};
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

/// **Os três chips + o Delete Node chegam ao bus, com a seleção UNIFORME e com a MISTA.**
///
/// A varredura completa (id → populate → paint → hit → dispatch → event → bus) rodada nas DUAS
/// caras: se a mista desligasse os controles, o gesto que sai do estado misto deixaria de existir.
#[test]
fn every_vertex_chip_reaches_the_bus_in_both_faces() {
    for (sel, face) in [
        (VertexSel::Uniform(VertexType::Smooth), "uniforme"),
        (VertexSel::Mixed, "MISTA"),
    ] {
        for (id, name) in [
            (ids::VECTOR_VERT_CORNER, "Corner"),
            (ids::VECTOR_VERT_SMOOTH, "Smooth"),
            (ids::VECTOR_VERT_SYMMETRIC, "Symm"),
            (ids::VECTOR_VERT_DELETE, "Delete Node"),
        ] {
            ph2d_panel_vector::set_selected_vertex_type(Some(sel));
            let mut host = MockPanelHost::with_panel::<VectorPanel>();
            let mut panel_state = VectorPanelState;
            let r = host
                .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .unwrap_or_else(|| {
                    panic!("{name} nao foi PINTADO com area clicavel na secao Vertex ({face})")
                });
            let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
            host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
            let evs =
                host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
            assert!(
                evs.iter()
                    .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
                "o ponteiro sobre {name} nao virou Click ({face}) — o chip esta' desenhado e nao \
                 existe para o dispatcher"
            );
            for ev in evs {
                host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
            }
            assert!(
                host.drained_actions().into_iter().any(
                    |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id)
                ),
                "o Click de {name} nao chegou ao bus ({face}) — o chip acende sob o mouse e nao \
                 faz nada"
            );
        }
    }
    ph2d_panel_vector::set_selected_vertex_type(None);
}

/// **Sem vértice selecionado a seção INTEIRA some** — a metade da AUSÊNCIA.
///
/// É o estado de um caminho inteiro selecionado (o resultado de uma booleana): sem vértice, um chip
/// de tipo não tem alvo, e um `Delete Node` sem nó é o knob morto na forma mais cara.
#[test]
fn no_selected_vertex_hides_the_whole_section() {
    ph2d_panel_vector::set_selected_vertex_type(None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for (id, name) in [
        (ids::VECTOR_VERT_CORNER, "Corner"),
        (ids::VECTOR_VERT_DELETE, "Delete Node"),
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "{name} foi pintado sem vertice selecionado — nao ha alvo para ele escrever"
        );
    }
}
