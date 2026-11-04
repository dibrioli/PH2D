//! A costura da P7b no painel PIXEL, pelo rato de verdade: o pincel de padrão (alinhamento,
//! virar/girar), os pincéis guardados (recuperar, cadeado, apagar, Save Brush Here), o New Brush
//! do menu Edit e a Dynamics (sensores, Stabilizer, Min, a matriz e as barras de duas pegas).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{
    PIXEL_STABILIZER_ON, PixelBrushAction, PixelBrushShape, PixelBrushSlotInfo, PixelBrushSlotKind,
    PixelBrushThumb, PixelEditAction, PixelGradientDither, PixelOptions, PixelPanelInfo,
    PixelPatternAlign, PixelPatternTurn, PixelRequest, PixelSensor, PixelTool,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tools::{PixelToolsPanel, PixelToolsPanelState, ids, set_current_pixel};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 6000.0,
};

const R: [u8; 4] = [255, 0, 0, 255];

fn info() -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        tool: PixelTool::Pencil,
        palette: vec![R],
        size: (64, 32),
        ..Default::default()
    }
}

fn com_padrao() -> PixelPanelInfo {
    let thumb = PixelBrushThumb {
        w: 2,
        h: 1,
        cells: vec![Some(R), None],
    };
    PixelPanelInfo {
        pattern: Some((2, 1)),
        brushes: vec![
            PixelBrushSlotInfo {
                locked: false,
                kind: PixelBrushSlotKind::Pattern(thumb),
            },
            PixelBrushSlotInfo {
                locked: true,
                kind: PixelBrushSlotKind::Shape {
                    shape: PixelBrushShape::Square,
                    size: 5,
                },
            },
        ],
        ..info()
    }
}

struct Palco {
    host: MockPanelHost,
    st: PixelToolsPanelState,
    rects: Vec<(ph2d_a11y::NodeId, Rect)>,
}

impl Palco {
    fn novo(i: PixelPanelInfo) -> Self {
        let mut p = Self {
            host: crate::host_de_antes(),
            st: PixelToolsPanelState::default(),
            rects: Vec::new(),
        };
        set_current_pixel(i);
        p.pinta();
        p
    }

    fn pinta(&mut self) {
        self.rects = self.host.paint::<PixelToolsPanel>(&mut self.st, VIEWPORT);
    }

    fn rect(&self, id: ph2d_a11y::NodeId) -> Option<Rect> {
        self.rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
    }

    fn carrega(&mut self, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        let ev = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) | WidgetEvent::Toggled(c) if *c == id)),
            "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
        );
        for e in ev {
            let _ = self
                .host
                .apply_panel_event::<PixelToolsPanel>(&mut self.st, e);
        }
        self.host.drained_actions()
    }
}

fn so(a: &[EditorAction], r: PixelRequest) {
    assert_eq!(a, [EditorAction::Pixel(r)]);
}

fn opcoes(a: &[EditorAction]) -> PixelOptions {
    match a {
        [EditorAction::Pixel(PixelRequest::Options(o))] => **o,
        _ => panic!("esperava UMAS opções, veio {a:?}"),
    }
}

#[test]
fn the_pattern_brush_aligns_turns_and_a_shape_drops_it() {
    let mut p = Palco::novo(com_padrao());
    for (a, id, _) in ids::PATTERN_ALIGNS {
        let o = opcoes(&p.carrega(id));
        assert_eq!(o.pattern_align, a, "{a:?}");
    }
    for (t, id) in ids::TURNS {
        so(
            &p.carrega(id),
            PixelRequest::Brush(PixelBrushAction::Turn(t)),
        );
    }
    let (s, id) = ids::SHAPES[1];
    so(
        &p.carrega(id),
        PixelRequest::Brush(PixelBrushAction::Shape(s)),
    );
    assert!(p.rect(ids::SIZE).is_none(), "o padrão não tem Size");
    // Sem padrão: nem alinhamento nem virar/girar, e o Size volta.
    let p = Palco::novo(info());
    for id in ids::PATTERN_ALIGNS.map(|(_, id, _)| id) {
        assert!(p.rect(id).is_none());
    }
    for id in ids::TURNS.map(|(_, id)| id) {
        assert!(p.rect(id).is_none());
    }
    assert!(p.rect(ids::SIZE).is_some());
    assert_eq!(PixelPatternAlign::default(), PixelPatternAlign::Source);
    assert_eq!(PixelPatternTurn::ALL.len(), ids::TURNS.len());
}

#[test]
fn each_saved_brush_recalls_locks_and_deletes_and_save_brush_here_saves() {
    let mut p = Palco::novo(com_padrao());
    for n in 0..2 {
        let b = |a| PixelRequest::Brush(a);
        so(
            &p.carrega(ids::brush_slot_id(n)),
            b(PixelBrushAction::Recall(n)),
        );
        so(
            &p.carrega(ids::brush_lock_id(n)),
            b(PixelBrushAction::ToggleLock(n)),
        );
        so(
            &p.carrega(ids::brush_delete_id(n)),
            b(PixelBrushAction::Delete(n)),
        );
    }
    assert!(p.rect(ids::brush_slot_id(2)).is_none());
    so(
        &p.carrega(ids::SAVE_HERE),
        PixelRequest::Brush(PixelBrushAction::SaveHere),
    );
    let dica = |p: &Palco, n| {
        p.host
            .store()
            .tooltip_for(ids::brush_lock_id(n))
            .map(str::to_owned)
    };
    assert_ne!(dica(&p, 0), dica(&p, 1), "o cadeado diz se está trancado");
}

#[test]
fn new_brush_is_in_the_edit_menu() {
    let mut p = Palco::novo(info());
    let id = ids::EDIT_ACTIONS
        .iter()
        .find(|(_, r, _)| *r == PixelRequest::Edit(PixelEditAction::NewBrush))
        .map(|(id, ..)| *id)
        .expect("New Brush no menu Edit");
    so(
        &p.carrega(id),
        PixelRequest::Edit(PixelEditAction::NewBrush),
    );
}

#[test]
fn dynamics_opens_and_each_control_writes_its_option() {
    let mut p = Palco::novo(info());
    assert!(p.rect(ids::DYN_STABILIZER).is_none(), "nasce fechada");
    assert!(p.carrega(ids::DYNAMICS).is_empty());
    p.pinta();
    assert!(p.st.dynamics_open());
    let o = opcoes(&p.carrega(ids::DYN_STABILIZER));
    assert_eq!(o.dynamics.stabilizer, PIXEL_STABILIZER_ON);
    for (row, s, id) in ids::sensor_ids() {
        let o = opcoes(&p.carrega(id));
        let c = match row {
            ids::DynRow::Size => o.dynamics.size,
            ids::DynRow::Angle => o.dynamics.angle,
            ids::DynRow::Gradient => o.dynamics.gradient,
        };
        assert_eq!(c.sensor, s, "{row:?}");
    }
    // Com sensores ligados aparecem o Min, a matriz e as barras.
    let mut i = info();
    let d = &mut i.options.dynamics;
    d.size.sensor = PixelSensor::Velocity;
    d.angle.sensor = PixelSensor::Pressure;
    d.gradient.sensor = PixelSensor::Velocity;
    d.stabilizer = 4;
    set_current_pixel(i);
    p.pinta();
    for (m, id) in ids::DITHERS {
        assert_eq!(opcoes(&p.carrega(id)).dynamics.dither, m);
    }
    assert_eq!(PixelGradientDither::ALL.len(), ids::DITHERS.len());
    let casos = [
        (ids::DYN_MIN_SIZE, 3.0),
        (ids::DYN_MIN_ANGLE, -45.0),
        (ids::DYN_STABILIZER_LEN, 32.0),
    ];
    for (id, v) in casos {
        assert!(p.rect(id).is_some(), "{id:?} pintado");
        let ev = p.host.type_into_number(id, &v.to_string());
        for e in ev {
            let _ = p.host.apply_panel_event::<PixelToolsPanel>(&mut p.st, e);
        }
        let o = opcoes(&p.host.drained_actions());
        let d = o.dynamics;
        let lido = [
            d.size.min,
            d.angle.min,
            i32::try_from(d.stabilizer).unwrap_or(0),
        ];
        assert!(lido.contains(&(v as i32)), "{id:?} → {lido:?}");
    }
    for (_, id, _) in ids::THRESHOLDS {
        assert!(p.rect(id).is_some(), "as duas barras com os dois sensores");
    }
    // Fechar esconde-a.
    assert!(p.carrega(ids::DYNAMICS).is_empty());
    p.pinta();
    assert!(p.rect(ids::DYN_STABILIZER).is_none());
}

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

/// Premir perto da pega de baixo e arrastar move SÓ ela, presa à de cima; a outra barra não mexe.
#[test]
fn dragging_a_threshold_handle_moves_only_that_handle_and_never_past_the_other() {
    let mut i = info();
    i.options.dynamics.size.sensor = PixelSensor::Velocity;
    let mut p = Palco::novo(i);
    p.carrega(ids::DYNAMICS);
    p.pinta();
    let (_, pressao, _) = ids::THRESHOLDS[0];
    let (_, vel, _) = ids::THRESHOLDS[1];
    assert!(p.rect(pressao).is_none(), "só a barra do sensor em uso");
    let r = p.rect(vel).expect("a barra da velocidade");
    let x = |f: f32| r.x + r.w * f;
    let y = r.y + r.h * 0.5;
    let _ = p
        .host
        .dispatch_pointer_event(ev(PointerKind::Down, x(0.12), y, 1));
    p.pinta();
    let _ = p
        .host
        .dispatch_pointer_event(ev(PointerKind::Move, x(0.5), y, 2));
    p.pinta();
    let a = p.host.drained_actions();
    let o = opcoes(&a[a.len() - 1..]);
    let t = o.dynamics.velocity;
    assert!((480..=520).contains(&t.lo), "a de baixo seguiu: {t:?}");
    assert_eq!(t.hi, 890, "a de cima ficou");
    assert_eq!(o.dynamics.pressure, info().options.dynamics.pressure);
    // Para lá da de cima: encosta nela.
    let _ = p
        .host
        .dispatch_pointer_event(ev(PointerKind::Move, x(0.99), y, 3));
    p.pinta();
    let a = p.host.drained_actions();
    let t = opcoes(&a[a.len() - 1..]).dynamics.velocity;
    assert_eq!((t.lo, t.hi), (890, 890), "nunca se cruzam");
    let _ = p
        .host
        .dispatch_pointer_event(ev(PointerKind::Up, x(0.99), y, 4));
    p.pinta();
    assert!(p.host.drained_actions().iter().all(|a| !matches!(
        a,
        EditorAction::Pixel(PixelRequest::Options(o)) if o.dynamics.velocity.lo != 890
    )));
}
