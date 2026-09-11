//! Seam da seção **CONTOUR** (pesquisa `20_*` #9) — as duas caras + o clique que chega ao bus.
//!
//! O que este arquivo prova é o que nenhum teste de unidade alcança: que o controle **pintado**
//! está registado, que o ponteiro sobre ele vira `Click`, e que esse `Click` atravessa o painel
//! até o barramento que a shell drena. Faltar uma ponta é um clique dropado **em silêncio**, não
//! um erro de compilação.
//!
//! E as duas CARAS são metade do gate, com presença **e** ausência: sem contour a seção mostra só
//! *Add Contour*; com contour mostra os controles e a swatch, e o *Add* some. A ausência importa
//! tanto quanto a presença porque a swatch de cor sem alvo é o knob morto na forma mais cara.

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

/// Publica uma seleção COM contour armado (a cara dos controles).
fn armed() {
    ph2d_panel_vector::set_current_contour_can_add(false);
    ph2d_panel_vector::set_current_contour(true, 6.0, 0.1, 1.5, 1, 0, [10, 20, 30, 255]);
}

/// Publica uma seleção SEM contour, mas que permite criar um (a cara do botão).
fn addable() {
    ph2d_panel_vector::set_current_contour_can_add(true);
    ph2d_panel_vector::set_current_contour(false, 4.0, 0.0, 1.0, 1, 0, [255, 255, 255, 255]);
}

/// **Cada botão pintado da seção vira `Click` e chega ao bus.** É a varredura de 8 sítios: id →
/// populate → paint → hit → dispatch → event → bus.
#[test]
fn every_contour_button_reaches_the_bus() {
    for (id, name, arm) in [
        (ids::VECTOR_CONTOUR_ADD, "Add Contour", false),
        (ids::VECTOR_CONTOUR_EXPAND, "Expand Contour", true),
        (ids::VECTOR_CONTOUR_REMOVE, "Remove Contour", true),
        (ids::VECTOR_CONTOUR_JOIN_MITER, "Corner: Miter", true),
        (ids::VECTOR_CONTOUR_JOIN_ROUND, "Corner: Round", true),
        (ids::VECTOR_CONTOUR_JOIN_BEVEL, "Corner: Bevel", true),
        (ids::VECTOR_CONTOUR_SIDE_OUTER, "Side: Outer", true),
        (ids::VECTOR_CONTOUR_SIDE_INNER, "Side: Inner", true),
        (ids::VECTOR_CONTOUR_SIDE_BOTH, "Side: Both", true),
    ] {
        if arm {
            armed();
        } else {
            addable();
        }
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .unwrap_or_else(|| panic!("{name} nao foi PINTADO com area clicavel na secao Contour"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {name} nao virou Click — falta `button()` no `populate_contour` \
             (o botao esta desenhado, mas nao existe para o dispatcher)"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id)
            ),
            "o Click de {name} nao chegou ao bus — falta o braco no `event.rs` (o botao \
             acende sob o mouse e nao faz nada)"
        );
    }
}

/// **As duas caras da seção**, presença E ausência.
///
/// Sem contour a seção é UM botão; com contour são os controles + as duas saídas, e o *Add*
/// **desaparece**. A metade da ausência é a que impede a swatch de cor de existir sem ter para
/// onde escrever — e é a que um gate escrito só com `assert!(pintou)` nunca faria.
#[test]
fn the_section_shows_the_button_or_the_controls_never_both() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    addable();
    let painted = |h: &mut MockPanelHost, s: &mut VectorPanelState, id| {
        h.painted_rect::<VectorPanel>(s, VIEWPORT, id).is_some()
    };
    assert!(
        painted(&mut host, &mut panel_state, ids::VECTOR_CONTOUR_ADD),
        "sem contour a secao tem de oferecer `Add Contour`"
    );
    for (id, name) in [
        (ids::VECTOR_CONTOUR_TO, "a swatch da cor-alvo"),
        (ids::VECTOR_CONTOUR_STEPS, "o slider de Steps"),
        (ids::VECTOR_CONTOUR_EXPAND, "Expand Contour"),
        (ids::VECTOR_CONTOUR_REMOVE, "Remove Contour"),
    ] {
        assert!(
            !painted(&mut host, &mut panel_state, id),
            "{name} foi pintado SEM contour armado — nao ha para onde ele escrever"
        );
    }

    armed();
    assert!(
        !painted(&mut host, &mut panel_state, ids::VECTOR_CONTOUR_ADD),
        "`Add Contour` continuou pintado sobre uma forma que JA tem contour"
    );
    for (id, name) in [
        (ids::VECTOR_CONTOUR_TO, "a swatch da cor-alvo"),
        (ids::VECTOR_CONTOUR_STEPS, "o slider de Steps"),
        (ids::VECTOR_CONTOUR_OFFSET, "o slider de Offset"),
        (ids::VECTOR_CONTOUR_ACCEL, "o slider de Accel"),
        (ids::VECTOR_CONTOUR_EXPAND, "Expand Contour"),
        (ids::VECTOR_CONTOUR_REMOVE, "Remove Contour"),
    ] {
        assert!(
            painted(&mut host, &mut panel_state, id),
            "{name} nao foi pintado com contour armado"
        );
    }
}

/// **A seção some inteira quando nao ha o que dizer** — nem contour, nem selecao que permita
/// criar um. Sem isto ela viraria ruido permanente no painel, que e' a razao de o Pattern on Path
/// ter a mesma regra.
#[test]
fn the_section_is_absent_when_there_is_nothing_to_say() {
    ph2d_panel_vector::set_current_contour_can_add(false);
    ph2d_panel_vector::set_current_contour(false, 4.0, 0.0, 1.0, 1, 0, [255, 255, 255, 255]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    for (id, name) in [
        (ids::VECTOR_SECTION_CONTOUR, "o cabecalho"),
        (ids::VECTOR_CONTOUR_ADD, "`Add Contour`"),
    ] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                .is_none(),
            "{name} foi pintado sem selecao que permita criar um contour"
        );
    }
}
