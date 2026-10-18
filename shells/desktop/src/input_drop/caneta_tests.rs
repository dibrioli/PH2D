//! A porta ÚNICA do ponteiro (ADR-0189), sem janela: a ponta da caneta é o clique primário e a
//! pressão dela chega ao Move do Painter e ao lápis; o rato continua `1.0`/`Mouse`. O conversor
//! inteiro (botões, borracha, tilt, relógio) gateia-se em `ph2d_pen::ponteiro`.

use ph2d_host::{PointerButton, PointerSource};
use ph2d_pen::ponteiro::{Conversor, Tinta};
use ph2d_pen::porta::{evento_ns, tinta};
use ph2d_pen::{PenEvent, PenPhase, PenSample, ToolKind};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, MouseButton};

use crate::input_dispatch::painter_canvas_input::traco_de_teste;

fn caneta(app: &mut crate::App, c: &mut Conversor, phase: PenPhase, pressure: f32, x: f64) {
    let (pos, tilt_deg, distance, tool) = ([x, 40.0], [0.0; 2], 0.0, ToolKind::Pen);
    let s = PenSample {
        pos,
        pressure,
        tilt_deg,
        distance,
        tool,
        phase,
        time_ms: 1,
    };
    let mut out = Vec::new();
    c.converte(
        &[PenEvent::Sample(s)],
        1.0,
        crate::App::timestamp_ns(),
        &mut out,
    );
    app.despacha_caneta(out);
}

#[test]
fn the_pen_tip_is_a_primary_click_and_the_mouse_stays_pressure_one() {
    let (mut app, mut c) = (crate::App::new(), Conversor::default());
    caneta(&mut app, &mut c, PenPhase::Hover, 0.0, 30.0);
    caneta(&mut app, &mut c, PenPhase::Down, 0.4, 31.0);
    let t = tinta();
    assert_eq!(
        (app.held_button, app.last_pointer),
        (Some(PointerButton::Primary), (31.0, 40.0))
    );
    assert!(
        t.pressure == 0.4 && matches!(t.source, PointerSource::Pen(_)),
        "{t:?}"
    );
    caneta(&mut app, &mut c, PenPhase::Up, 0.0, 31.0);
    assert_eq!(app.held_button, None);
    let mut out = Vec::new();
    c.converte(
        &[PenEvent::Leave {
            tool: ToolKind::Pen,
            time_ms: 2,
        }],
        1.0,
        0,
        &mut out,
    );
    app.despacha_caneta(out);
    assert_eq!(tinta(), Tinta::RATO, "a caneta saiu");
    // O rato, ao byte, nas duas portas — mesmo logo depois da caneta.
    caneta(&mut app, &mut c, PenPhase::Hover, 0.0, 9.0);
    app.on_cursor_moved(PhysicalPosition::new(5.0, 6.0));
    assert_eq!((tinta(), app.last_pointer), (Tinta::RATO, (5.0, 6.0)));
    caneta(&mut app, &mut c, PenPhase::Hover, 0.0, 9.0);
    app.on_mouse_input(ElementState::Pressed, MouseButton::Left);
    assert_eq!(
        (tinta(), app.held_button),
        (Tinta::RATO, Some(PointerButton::Primary))
    );
}

#[test]
fn the_pen_pressure_reaches_the_painter_move_and_the_pencil() {
    let (mut app, mut c) = (crate::App::new(), Conversor::default());
    traco_de_teste(true);
    caneta(&mut app, &mut c, PenPhase::Move, 0.73, 12.0);
    assert_eq!(traco_de_teste(false), 0.73, "o Move do Painter");
    let d = app.pointer_dynamics();
    assert_eq!(
        (d.pressure, d.t_ns),
        (0.73, evento_ns()),
        "o lápis: a pressão e o carimbo"
    );
    traco_de_teste(true);
    app.on_cursor_moved(PhysicalPosition::new(14.0, 40.0));
    assert_eq!(
        (traco_de_teste(false), app.pointer_dynamics().pressure),
        (1.0, 1.0),
        "o rato"
    );
}
