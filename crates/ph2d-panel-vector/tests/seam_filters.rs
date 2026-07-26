//! Behavioral SEAM test for the **Filters** section (FX raster, plano 24) — irmão do `seam.rs`
//! (que bateu o teto de LOC).
//!
//! Prova o que os testes de unidade + o `architecture_panel_wiring_parity` não provam: que o
//! GESTO de armar/desarmar um filtro chega ao bus. Os quatro chips de tipo (None/Blur/Glow/Shadow)
//! têm de virar `Click` no ponteiro real E chegar ao bus como `ToolPanelEvent::Click` — senão o
//! chip pinta, promete armar, e está MORTO (o drain da shell nunca chama `set_filter`).
//!
//! (A cor do Glow/Drop Shadow vai pelo picker OKLCH partilhado — coberta por
//! `register_picker_swatch` + o dispatch genérico, como as swatches de Fill/Contour —, então não
//! passa por este seam. Os sliders viajam como `SetValue`, cobertos pela delegação em `event.rs`.)

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

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

/// Os quatro chips de tipo do **Filters** chegam ao bus quando o artista CLICA neles — pelo
/// caminho inteiro (`paint` → hit-rect → ponteiro real → `event.rs` → bus). É o gesto que arma e
/// desarma o `VecFilter`; sem ele, o produtor GPU existiria e a feature não.
///
/// ⚠️ **A premissa da seção:** ela só pinta com forma selecionada (`can_add`) ou filtro vivo
/// (`present`). O fixture publica AS DUAS — com um Drop Shadow presente todos os controles sobem,
/// e os quatro chips ficam clicáveis.
#[test]
fn every_filter_kind_chip_reaches_the_bus_when_clicked() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    const SEC: u128 = 1_000_000_000;
    for (id, name) in [
        (ids::VECTOR_FILTER_KIND_NONE, "None"),
        (ids::VECTOR_FILTER_KIND_BLUR, "Blur"),
        (ids::VECTOR_FILTER_KIND_GLOW, "Glow"),
        (ids::VECTOR_FILTER_KIND_SHADOW, "Shadow"),
    ] {
        // Drop Shadow presente (kind 2): a seção pinta os quatro chips + todos os parâmetros.
        ph2d_panel_vector::set_current_filter_can_add(true);
        ph2d_panel_vector::set_current_filter(true, 2, 0.1, 0.12, -0.12, [0, 0, 0, 255], 0.6);
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .unwrap_or_else(|| {
                panic!("o chip {name} nao foi PINTADO com area clicavel na secao Filters")
            });
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {name} nao virou Click — falta `button()` no `populate` \
             (o chip esta desenhado, mas nao existe para o dispatcher)"
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
            "o Click em {name} nao chegou ao bus — falta o id no `forwards_plain_click` do \
             `event.rs` (o chip e clicavel e MORTO, e o drain da shell nunca arma o filtro)"
        );
    }
}

/// A seção **NÃO** é oferecida sem seleção nem filtro vivo — o cabeçalho nem sobe. É a metade
/// AUSENTE do seam: um chip que aparece sobre a tela vazia editaria a forma errada.
#[test]
fn the_filters_section_is_not_offered_without_a_shape() {
    const VIEWPORT: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 900.0,
    };
    ph2d_panel_vector::set_current_filter_can_add(false);
    ph2d_panel_vector::set_current_filter(false, 0, 0.0, 0.0, 0.0, [0, 0, 0, 255], 1.0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_FILTER_KIND_BLUR)
            .is_none(),
        "o chip Blur foi pintado sem forma selecionada — a secao Filters vazou"
    );
}
