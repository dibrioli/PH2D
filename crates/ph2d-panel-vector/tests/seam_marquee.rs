//! Seam da **FORMA do marquee** (`Box | Lasso`) — os dois chips estão vivos sob o MOUSE, e só no
//! modo em que a região existe.
//!
//! O gesto é REAL (Down+Up sobre o retângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: o sintético prova a allowlist e o switch da tool mas **pula a checagem de
//! focabilidade no store** — a lacuna que deixou as 36 células da matriz de física e os dez chips
//! de ferramenta do Painter *pintados, hit-registrados e mortos sob o ponteiro*.
//!
//! ⚠️ **A metade da AUSÊNCIA é a que carrega o gate.** Um par de chips pintado em todo modo é um
//! controle que não faz nada em treze deles: só no Node existe um arrasto no vazio para lhes
//! obedecer. E a *presença* sozinha fica verde num painel que os pinte sempre.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{VectorPanelState, set_current_vector_style};
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::{DrawMode, MarqueeShape};
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

fn arm(mode: DrawMode, marquee: MarqueeShape) {
    set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        marquee,
        ..VectorStyleSnapshot::default()
    }));
}

/// **Os dois chips são alcançáveis por um ponteiro e o clique chega ao bus.**
///
/// Duas metades independentes: sair do `populate_modes` mata a 1ª asserção (o ponteiro não vira
/// Click), sair do `event_clicks` mata a 2ª (o Click morre no painel, e o artista clica "Lasso" e
/// nada acontece — com o log a dizer `[hero] unhandled event`).
#[test]
fn both_marquee_chips_are_reachable_by_a_pointer_and_reach_the_bus() {
    for id in [ids::VECTOR_MARQUEE_BOX, ids::VECTOR_MARQUEE_LASSO] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        arm(DrawMode::Node, MarqueeShape::Box);
        let r = host
            .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
            .expect("o chip de marquee nao foi PINTADO com area clicavel no modo Node");
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre {id:?} nao virou Click — esta' desenhado e nao existe para o \
             dispatcher (falta o `register` no populate_modes)"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
            )),
            "o Click de {id:?} nao chegou ao bus — o chip acende sob o mouse e nao faz nada \
             (falta a linha na allowlist do event_clicks)"
        );
    }
}

/// **Fora do modo Node os chips NÃO existem** — a metade da ausência.
///
/// A região só nasce ao arrastar do vazio no Node; num modo em que ela não existe, um par que diz
/// que forma ela tem é um controle mudo. E o gate a percorre em vários modos, porque um `!= Node`
/// escrito ao contrário passa em qualquer um deles isolado.
#[test]
fn the_marquee_chips_do_not_exist_outside_the_node_mode() {
    for mode in [
        DrawMode::Select,
        DrawMode::Pen,
        DrawMode::Pencil,
        DrawMode::Shape,
        DrawMode::Cut,
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        arm(mode, MarqueeShape::Lasso);
        for id in [ids::VECTOR_MARQUEE_BOX, ids::VECTOR_MARQUEE_LASSO] {
            assert!(
                host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                    .is_none(),
                "{id:?} foi pintado no modo {mode:?}, onde nao ha' regiao nenhuma para ele \
                 qualificar"
            );
        }
    }
}

/// **O chip aceso é o do valor publicado** — o painel não guarda uma segunda cópia da escolha.
///
/// ⚠️ Ele lê o retângulo dos DOIS e exige que existam nos dois estados: um painel que só pintasse
/// o chip activo passaria numa asserção de presença ingénua e deixaria o artista sem como voltar.
#[test]
fn both_chips_exist_whichever_one_is_live() {
    for m in [MarqueeShape::Box, MarqueeShape::Lasso] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;
        arm(DrawMode::Node, m);
        for id in [ids::VECTOR_MARQUEE_BOX, ids::VECTOR_MARQUEE_LASSO] {
            assert!(
                host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
                    .is_some(),
                "{id:?} sumiu com o chip em {m:?} — quem escolheu o laco ficaria sem como voltar \
                 ao retangulo"
            );
        }
    }
}
