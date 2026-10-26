//! **Arrastar no seletor** (smoke do dono 25/10: «os sliders da roda não arrastam, apenas clicam»):
//! premir na barra/quadro e mover SEM largar já muda a cor de frente — pelo despacho real do rato.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelRequest, PixelSelector, PixelTool};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect { x: 0.0, y: 0.0, w: 320.0, h: 4000.0 };

fn ev(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
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

fn fgs(a: &[EditorAction]) -> Vec<[u8; 4]> {
    a.iter()
        .filter_map(|e| match e {
            EditorAction::Pixel(PixelRequest::Fg(c)) => Some(*c),
            _ => None,
        })
        .collect()
}

#[test]
fn holding_and_moving_on_every_selector_part_follows_the_pointer_before_the_release() {
    for modo in [PixelSelector::TintShadeTone, PixelSelector::RgbWheel, PixelSelector::NormalMap] {
        for (k, (area, _)) in ids::SELECTOR_AREAS.iter().enumerate() {
            let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
            let mut st = PixelToolsPanelState::default();
            let mut info = PixelPanelInfo {
                editing: true,
                tool: PixelTool::Pencil,
                fg: [200, 30, 40, 255],
                bg: [0, 0, 0, 255],
                palette: vec![[1, 2, 3, 255]; 8],
                ..Default::default()
            };
            info.options.selector = modo;
            set_current_pixel(info);
            let rects = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
            let r = rects.iter().find(|(n, _)| n == area).map(|(_, r)| *r).expect("pintado");
            let (y, x0, x1) = (r.y + r.h * 0.5, r.x + r.w * 0.3, r.x + r.w * 0.8);
            let _ = host.dispatch_pointer_event(ev(PointerKind::Down, x0, y, 1));
            let _ = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
            let antes = fgs(&host.drained_actions());
            let _ = host.dispatch_pointer_event(ev(PointerKind::Move, x1, y, 2));
            let _ = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
            let depois = fgs(&host.drained_actions());
            assert!(!depois.is_empty(), "{modo:?} parte {k}: mover premido não mudou a cor");
            assert_ne!(antes.last(), depois.last(), "{modo:?} parte {k}: a cor não seguiu o ponteiro");
        }
    }
}
