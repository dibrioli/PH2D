//! Seam da caixa **Stroke** (plano 34) — *esta forma tem traço?*
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — a lacuna que já deixou 36 células da matriz de física e dez chips do Painter
//! *pintados, hit-registrados e mortos sob o ponteiro*.
//!
//! ⚠️ **Nasceu do report do Enio de 2026-08-27** (*"o contorno funciona com as shapes que eu
//! desejo, mas não funcionam com os teus desenhos"*): uma forma que chegou ao documento sem traço
//! não tinha por onde ganhar um, e a secção *Stroke* oferecia controlos que não a alcançavam.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
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

fn rect(id: ph2d_a11y::NodeId) -> Option<Rect> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
}

/// **A caixa está viva e o Click chega ao bus** — nos DOIS estados.
///
/// ⚠️ Os dois valores são exercitados de propósito: o `paint_checkbox` ramifica na marcação, e um
/// gate que só clicasse o marcado ficaria verde sobre um desmarcado que não pinta o retângulo (e
/// portanto não regista o hit).
#[test]
fn the_stroke_checkbox_is_reachable_and_reaches_the_bus_in_both_states() {
    for tem in [true, false] {
        state::set_stroke_present(Some(tem));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::VECTOR_STROKE_PRESENT)
            .unwrap_or_else(|| panic!("a caixa ({tem}) nao foi PINTADA com area clicavel"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ids::VECTOR_STROKE_PRESENT)),
            "o ponteiro sobre a caixa ({tem}) nao virou Click - ela esta' desenhada e nao existe \
             para o dispatcher (falta o `register` no populate)"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ids::VECTOR_STROKE_PRESENT
            )),
            "o Click da caixa ({tem}) nao chegou ao bus - ela acende sob o rato e nao faz nada \
             (falta a linha na allowlist do event_clicks)"
        );
    }
    state::set_stroke_present(None);
}

/// **Sem resposta publicada a linha não existe** — a metade da AUSÊNCIA.
///
/// ⚠️ Sem ela o gate acima ficaria verde sobre uma caixa que aparece em TODA selecção, incluindo a
/// múltipla — onde ela descreveria um objecto que não está lá e o clique não teria sujeito.
#[test]
fn the_row_is_not_painted_without_an_answer_to_show() {
    state::set_stroke_present(None);
    assert!(
        rect(ids::VECTOR_STROKE_PRESENT).is_none(),
        "a caixa foi pintada sem resposta publicada"
    );
    state::set_stroke_present(Some(false));
    assert!(
        rect(ids::VECTOR_STROKE_PRESENT).is_some(),
        "e com resposta ela existe"
    );
    state::set_stroke_present(None);
}

/// ⛔⛔ **AS ROWS DO TRAÇO FICAM VISÍVEIS MESMO SEM TRAÇO — e isso é DELIBERADO.**
///
/// É o **oposto** da lei que o Enio pediu para a secção *Pattern* (*"os parâmetros que um modo não
/// usa não devem aparecer"*), e a diferença é real: a secção *Stroke* é a ficha da **FERRAMENTA**
/// espelhada na selecção, então cada row aqui autora **o traço da próxima forma que se desenhar**.
/// Escondê-las tiraria ao artista o único sítio onde se afina o traço de desenho.
///
/// ⚠️ Este gate existe para que a próxima janela **não** "corrija" isto por simetria com o Pattern.
/// O que a caixa cura não é *"o controlo está morto"* — é *"o controlo não dizia que não alcança
/// ESTA forma"*.
#[test]
fn the_stroke_rows_stay_visible_without_a_stroke() {
    state::set_stroke_present(Some(false));
    for (id, what) in [
        (ids::VECTOR_WIDTH, "a largura"),
        (ids::VECTOR_STROKE_SWATCH, "a cor"),
        (ids::VECTOR_CAP_BUTT, "a ponta"),
        (ids::VECTOR_JOIN_MITER, "a junta"),
    ] {
        assert!(
            rect(id).is_some(),
            "{what} sumiu numa forma sem traco - ela autora o traco da PROXIMA forma, e escondê-la \
             tira o unico sitio onde o desenho se afina"
        );
    }
    state::set_stroke_present(None);
}
