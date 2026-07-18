//! **O X da timeline FECHA a timeline** (Enio, 2026-07-16).
//!
//! O botão era pintado e o handler existia; o que faltava era alguém provar que os dois
//! se encontram. Este gate pinta o painel de verdade, acha o retângulo que a pintura
//! registrou, põe o ponteiro nele pelo `dispatch_pointer` REAL e exige que o painel
//! fique invisível — [[feedback_widget_is_done_when_a_test_clicks_it]] e o irmão dele,
//! [[feedback_painted_is_not_populated_paint_gate]].

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);

#[test]
fn the_close_button_is_painted_registered_and_actually_closes_the_panel() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    set_current_timeline(Some(TimelineViewSnapshot::default()));
    let regs = host.paint::<TimelinePanel>(&mut state, VIEWPORT);

    assert!(
        host.panel_visible(TimelinePanel::ID),
        "o painel começa aberto — senão o gate abaixo é vacuously true"
    );

    let r = regs
        .iter()
        .find(|(w, _)| *w == ids::TIMELINE_CLOSE)
        .map(|(_, r)| *r)
        .expect("o X foi pintado mas nunca registrado: ele clica no nada");
    assert!(r.w > 0.0 && r.h > 0.0, "o X não tem área pra clicar: {r:?}");

    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let evs = host.click_at(cx, cy);
    assert!(
        evs.contains(&WidgetEvent::Click(ids::TIMELINE_CLOSE)),
        "o ponteiro caiu em {:?}, não no X — got {evs:?}",
        host.hit_at(cx, cy)
    );
    for ev in evs {
        host.apply_panel_event::<TimelinePanel>(&mut state, ev);
    }

    assert!(
        !host.panel_visible(TimelinePanel::ID),
        "clicar no X tem de FECHAR a timeline"
    );
}
