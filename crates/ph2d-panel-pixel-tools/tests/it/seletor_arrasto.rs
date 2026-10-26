//! **Arrastar no seletor** (smoke do dono 25/10: «os sliders da roda não arrastam, apenas clicam»):
//! premir na barra/quadro e mover SEM largar já muda a cor de frente — pelo despacho real do rato.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelRequest, PixelSelector, PixelTool};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 4000.0,
};

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
    for modo in [
        PixelSelector::TintShadeTone,
        PixelSelector::RgbWheel,
        PixelSelector::NormalMap,
    ] {
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
            let r = rects
                .iter()
                .find(|(n, _)| n == area)
                .map(|(_, r)| *r)
                .expect("pintado");
            let (y, x0, x1) = (r.y + r.h * 0.5, r.x + r.w * 0.3, r.x + r.w * 0.8);
            let _ = host.dispatch_pointer_event(ev(PointerKind::Down, x0, y, 1));
            let _ = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
            let antes = fgs(&host.drained_actions());
            let _ = host.dispatch_pointer_event(ev(PointerKind::Move, x1, y, 2));
            let _ = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
            let depois = fgs(&host.drained_actions());
            assert!(
                !depois.is_empty(),
                "{modo:?} parte {k}: mover premido não mudou a cor"
            );
            assert_ne!(
                antes.last(),
                depois.last(),
                "{modo:?} parte {k}: a cor não seguiu o ponteiro"
            );
        }
    }
}

/// A matiz (graus) de uma cor, `None` para cinza.
fn matiz(c: [u8; 4]) -> Option<f64> {
    let [r, g, b] = [c[0], c[1], c[2]].map(f64::from);
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    if mx - mn < 1.0 {
        return None;
    }
    let h = if mx == r {
        (g - b) / (mx - mn)
    } else if mx == g {
        2.0 + (b - r) / (mx - mn)
    } else {
        4.0 + (r - g) / (mx - mn)
    };
    Some((h * 60.0).rem_euclid(360.0))
}

#[test]
fn dragging_the_brightness_bar_keeps_the_wheel_hue_with_a_lagging_snapshot() {
    // smoke do dono 25/10: «arrastar o slider que escurece deixa o seletor da roda vibrando»
    for modo in [
        PixelSelector::RgbWheel,
        PixelSelector::RybWheel,
        PixelSelector::NormalMap,
    ] {
        let mut host = MockPanelHost::with_panel::<PixelToolsPanel>();
        let mut st = PixelToolsPanelState::default();
        let mut info = PixelPanelInfo {
            editing: true,
            tool: PixelTool::Pencil,
            fg: [200, 60, 140, 255],
            bg: [0, 0, 0, 255],
            palette: vec![[1, 2, 3, 255]; 8],
            ..Default::default()
        };
        info.options.selector = modo;
        set_current_pixel(info.clone());
        let rects = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
        let r = rects
            .iter()
            .find(|(n, _)| *n == ids::SELECTOR_AREAS[1].0)
            .map(|(_, r)| *r)
            .expect("barra");
        let y = r.y + r.h * 0.5;
        let _ = host.dispatch_pointer_event(ev(PointerKind::Down, r.x + r.w * 0.95, y, 1));
        let _ = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
        let mut atrasada = info.fg;
        let mut mandadas = fgs(&host.drained_actions());
        // o quadro seguinte ao Down já traz a cor que o Down mandou (o atraso é de UM quadro)
        if let Some(&c) = mandadas.last() {
            atrasada = c;
        }
        let mut brilhos = vec![st.selector_params().c];
        let p0 = st.selector_params();
        for i in 0..30u32 {
            let x = r.x + r.w * (0.95 - 0.03 * i as f32);
            let _ = host.dispatch_pointer_event(ev(PointerKind::Move, x, y, 2 + u128::from(i)));
            // o instantâneo chega UM quadro atrasado: o painel vê a cor de antes
            info.fg = atrasada;
            set_current_pixel(info.clone());
            let _ = host.paint::<PixelToolsPanel>(&mut st, VIEWPORT);
            let novas = fgs(&host.drained_actions());
            if let Some(&c) = novas.last() {
                atrasada = c;
            }
            mandadas.extend(novas);
            let p = st.selector_params();
            brilhos.push(p.c);
            // o anel da roda não se mexe quando só o brilho muda
            assert!(
                (p.a - p0.a).abs() < 1e-9 && (p.b - p0.b).abs() < 1e-9,
                "{modo:?}: o anel da roda saltou ({:.4},{:.4}) → ({:.4},{:.4})",
                p0.a,
                p0.b,
                p.a,
                p.b
            );
        }
        // e o anel da barra só anda para onde o ponteiro anda (para a esquerda): nunca volta atrás
        assert!(
            brilhos.windows(2).all(|w| w[1] <= w[0] + 1e-9),
            "{modo:?}: o anel da barra vibrou {brilhos:?}"
        );
        let hs: Vec<f64> = mandadas
            .iter()
            .filter(|c| c[0].max(c[1]).max(c[2]) > 12)
            .filter_map(|&c| matiz(c))
            .collect();
        assert!(hs.len() > 10, "{modo:?}: a barra mandou cores");
        let (mn, mx) = hs
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &h| (a.min(h), b.max(h)));
        assert!(mx - mn < 6.0, "{modo:?}: a matiz tremeu {mn:.1}..{mx:.1}");
        // e o raio do anel (a saturação) também não pode saltar
        let ss: Vec<f64> = mandadas
            .iter()
            .filter(|c| c[0].max(c[1]).max(c[2]) > 12)
            .map(|c| {
                let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
                f64::from(mx - mn) / f64::from(mx)
            })
            .collect();
        let (smn, smx) = ss
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &s| (a.min(s), b.max(s)));
        assert!(
            smx - smn < 0.08,
            "{modo:?}: a saturação tremeu {smn:.3}..{smx:.3}"
        );
    }
}
