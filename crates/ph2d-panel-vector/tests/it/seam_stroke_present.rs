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
use ph2d_panel_vector::{VectorPanel, state};

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
    let mut host = crate::host_aberto();
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
        let mut host = crate::host_aberto();
        let mut panel_state = VectorPanelState;
        let r = host
            .painted_rect::<VectorPanel>(
                &mut panel_state,
                VIEWPORT,
                ph2d_panel_vector::ids::VECTOR_STROKE_PRESENT,
            )
            .unwrap_or_else(|| panic!("a caixa ({tem}) nao foi PINTADA com area clicavel"));
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == ph2d_panel_vector::ids::VECTOR_STROKE_PRESENT)),
            "o ponteiro sobre a caixa ({tem}) nao virou Click - ela esta' desenhada e nao existe \
             para o dispatcher (falta o `register` no populate)"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == ph2d_panel_vector::ids::VECTOR_STROKE_PRESENT
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
        rect(ph2d_panel_vector::ids::VECTOR_STROKE_PRESENT).is_none(),
        "a caixa foi pintada sem resposta publicada"
    );
    state::set_stroke_present(Some(false));
    assert!(
        rect(ph2d_panel_vector::ids::VECTOR_STROKE_PRESENT).is_some(),
        "e com resposta ela existe"
    );
    state::set_stroke_present(None);
}

/// ⭐ **F44 (dono 01/11, o Figma): uma forma SEM traço mostra só o «+» do título.**
///
/// ⛔ Revoga a lei de 2026-08 (*«as rows do traço ficam visíveis sem traço, porque autoram o traço da
/// próxima forma»*): no layout do Figma o traço é uma LINHA que existe ou não, e o «+» é quem o dá.
/// Sem forma selecionada (`None`) a secção continua a ser a ficha da FERRAMENTA, com tudo à vista.
#[test]
fn a_shape_without_a_stroke_shows_only_the_plus() {
    let linhas = [
        (ph2d_tool_vector::ids::VECTOR_WIDTH_NUM, "a espessura"),
        (ph2d_tool_vector::ids::VECTOR_STROKE_SWATCH, "a cor"),
        (ph2d_tool_vector::ids::VECTOR_ALIGN_CENTRE, "a posicao"),
        (ph2d_panel_vector::ids::VECTOR_STROKE_SETTINGS, "os ajustes"),
    ];
    state::set_stroke_present(Some(false));
    for (id, what) in linhas {
        assert!(
            rect(id).is_none(),
            "{what} ficou à vista numa forma SEM traço — no Figma só o «+» do título a dá"
        );
    }
    assert!(
        rect(ph2d_panel_vector::ids::VECTOR_STROKE_PRESENT).is_some(),
        "o «+» que dá o traço tem de estar lá"
    );
    state::set_stroke_present(None);
    for (id, what) in linhas {
        assert!(
            rect(id).is_some(),
            "{what} sumiu sem seleção — a secção é a ficha da ferramenta"
        );
    }
}
