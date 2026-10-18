use super::*;
use ProtocolEvent as P;

const T: ToolKey = 7;

fn feed(s: &mut Sessao, evs: &[ProtocolEvent]) -> Vec<PenEvent> {
    let mut out = Vec::new();
    for e in evs {
        s.feed(T, *e, &mut out);
    }
    out
}

fn entra(s: &mut Sessao, kind: ToolKind) {
    let out = feed(
        s,
        &[P::ProximityIn {
            on_our_surface: true,
            kind,
        }],
    );
    assert!(out.is_empty(), "nada sai antes do frame");
}

fn amostras(out: &[PenEvent]) -> Vec<PenSample> {
    out.iter()
        .filter_map(|e| match e {
            PenEvent::Sample(s) => Some(*s),
            _ => None,
        })
        .collect()
}

fn uma(out: &[PenEvent]) -> PenSample {
    assert_eq!(out.len(), 1, "um frame, uma saída: {out:?}");
    amostras(out)[0]
}

#[test]
fn proximity_in_and_out_hover_then_leave_at_their_frames() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    let out = feed(
        &mut s,
        &[
            P::Motion { x: 10.5, y: 20.25 },
            P::Distance(32768),
            P::Frame { time_ms: 100 },
        ],
    );
    let a = uma(&out);
    assert_eq!(a.phase, PenPhase::Hover);
    assert_eq!(a.pos, [10.5, 20.25]);
    assert_eq!(a.time_ms, 100);
    assert_eq!(a.pressure, 0.0);
    assert!((a.distance - 0.5).abs() < 1e-4);
    let out = feed(&mut s, &[P::ProximityOut]);
    assert!(out.is_empty(), "a saída espera o frame");
    let out = feed(&mut s, &[P::Frame { time_ms: 140 }]);
    assert_eq!(
        out,
        [PenEvent::Leave {
            tool: ToolKind::Pen,
            time_ms: 140
        }]
    );
    let out = feed(
        &mut s,
        &[P::Motion { x: 1.0, y: 1.0 }, P::Frame { time_ms: 150 }],
    );
    assert!(
        out.is_empty(),
        "fora do alcance nada sai até ao próximo proximity_in"
    );
}

#[test]
fn a_proximity_in_without_motion_emits_no_sample() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    assert!(feed(&mut s, &[P::Frame { time_ms: 1 }]).is_empty());
}

#[test]
fn tip_down_move_up_with_pressure_persisting_and_reset_on_up() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    feed(
        &mut s,
        &[P::Motion { x: 1.0, y: 1.0 }, P::Frame { time_ms: 1 }],
    );
    let d = uma(&feed(
        &mut s,
        &[
            P::Motion { x: 2.0, y: 1.0 },
            P::Pressure(30000),
            P::Down,
            P::Frame { time_ms: 2 },
        ],
    ));
    assert_eq!(d.phase, PenPhase::Down);
    assert_eq!(d.pressure, 30000.0 / 65535.0);
    // O protocolo só manda o que muda: a pressão do frame anterior continua.
    let m = uma(&feed(
        &mut s,
        &[P::Motion { x: 3.0, y: 1.0 }, P::Frame { time_ms: 3 }],
    ));
    assert_eq!(m.phase, PenPhase::Move);
    assert_eq!(m.pressure, d.pressure);
    let u = uma(&feed(&mut s, &[P::Up, P::Frame { time_ms: 4 }]));
    assert_eq!(u.phase, PenPhase::Up);
    assert_eq!(u.pressure, 0.0, "o Up zera a pressão");
    assert_eq!(u.pos, [3.0, 1.0]);
    let h = uma(&feed(
        &mut s,
        &[P::Motion { x: 4.0, y: 1.0 }, P::Frame { time_ms: 5 }],
    ));
    assert_eq!(h.phase, PenPhase::Hover);
    assert_eq!(h.pressure, 0.0, "e continua zero depois");
}

#[test]
fn pressure_extremes_are_exact() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    let a = uma(&feed(
        &mut s,
        &[P::Pressure(65535), P::Down, P::Frame { time_ms: 1 }],
    ));
    assert_eq!(a.pressure, 1.0);
    let b = uma(&feed(&mut s, &[P::Pressure(0), P::Frame { time_ms: 2 }]));
    assert_eq!(b.pressure, 0.0);
    let c = uma(&feed(
        &mut s,
        &[P::Pressure(70000), P::Frame { time_ms: 3 }],
    ));
    assert_eq!(c.pressure, 1.0, "acima do máximo do protocolo satura");
}

#[test]
fn buttons_lower_upper_other_come_in_order_before_the_sample() {
    assert_eq!(PenButton::from_code(0x14b), PenButton::Lower);
    assert_eq!(PenButton::from_code(0x14c), PenButton::Upper);
    assert_eq!(PenButton::from_code(0x110), PenButton::Other(0x110));
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    let out = feed(
        &mut s,
        &[
            P::Motion { x: 5.0, y: 6.0 },
            P::Button {
                code: 0x14c,
                pressed: true,
            },
            P::Button {
                code: 0x14b,
                pressed: true,
            },
            P::Button {
                code: 0x115,
                pressed: false,
            },
            P::Frame { time_ms: 9 },
        ],
    );
    let botoes: Vec<(PenButton, bool, [f64; 2], u32)> = out
        .iter()
        .filter_map(|e| match *e {
            PenEvent::Button {
                button,
                pressed,
                pos,
                time_ms,
                ..
            } => Some((button, pressed, pos, time_ms)),
            _ => None,
        })
        .collect();
    assert_eq!(
        botoes,
        [
            (PenButton::Upper, true, [5.0, 6.0], 9),
            (PenButton::Lower, true, [5.0, 6.0], 9),
            (PenButton::Other(0x115), false, [5.0, 6.0], 9),
        ]
    );
    assert!(
        matches!(out.last(), Some(PenEvent::Sample(_))),
        "a amostra vem depois: {out:?}"
    );
    let so_botao = feed(
        &mut s,
        &[
            P::Button {
                code: 0x14b,
                pressed: false,
            },
            P::Frame { time_ms: 10 },
        ],
    );
    assert_eq!(
        so_botao.len(),
        1,
        "um frame só de botão não inventa amostra: {so_botao:?}"
    );
}

#[test]
fn the_eraser_end_is_an_eraser_and_everything_else_draws() {
    assert_eq!(ToolKind::from_protocol_type(0x141), ToolKind::Eraser);
    for code in [0x140, 0x142, 0x143, 0x144, 0x145, 0x146, 0x147] {
        assert_eq!(
            ToolKind::from_protocol_type(code),
            ToolKind::Pen,
            "{code:#x}"
        );
    }
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Eraser);
    let a = uma(&feed(
        &mut s,
        &[
            P::Motion { x: 1.0, y: 2.0 },
            P::Down,
            P::Frame { time_ms: 1 },
        ],
    ));
    assert_eq!(a.tool, ToolKind::Eraser);
    let out = feed(&mut s, &[P::Up, P::ProximityOut, P::Frame { time_ms: 2 }]);
    assert_eq!(
        out.last(),
        Some(&PenEvent::Leave {
            tool: ToolKind::Eraser,
            time_ms: 2
        })
    );
}

#[test]
fn tilt_and_distance_persist_until_changed() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    let a = uma(&feed(
        &mut s,
        &[
            P::Motion { x: 0.0, y: 0.0 },
            P::Tilt { x: 22.5, y: -4.0 },
            P::Frame { time_ms: 1 },
        ],
    ));
    assert_eq!(a.tilt_deg, [22.5, -4.0]);
    let b = uma(&feed(
        &mut s,
        &[P::Motion { x: 1.0, y: 0.0 }, P::Frame { time_ms: 2 }],
    ));
    assert_eq!(b.tilt_deg, [22.5, -4.0]);
    let c = uma(&feed(
        &mut s,
        &[P::Tilt { x: -60.0, y: 60.0 }, P::Frame { time_ms: 3 }],
    ));
    assert_eq!(
        c.tilt_deg,
        [-60.0, 60.0],
        "só a inclinação mudou: ainda é uma amostra"
    );
    assert_eq!(c.pos, [1.0, 0.0]);
}

#[test]
fn two_motions_in_one_frame_are_one_sample_with_the_last_position() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    let a = uma(&feed(
        &mut s,
        &[
            P::Motion { x: 1.0, y: 1.0 },
            P::Pressure(100),
            P::Motion { x: 2.0, y: 3.0 },
            P::Pressure(200),
            P::Frame { time_ms: 7 },
        ],
    ));
    assert_eq!(a.pos, [2.0, 3.0]);
    assert_eq!(a.pressure, 200.0 / 65535.0);
}

#[test]
fn n_frames_are_n_samples_never_coalesced() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    let mut evs = Vec::new();
    for i in 0..50u32 {
        evs.push(P::Motion {
            x: f64::from(i),
            y: 0.0,
        });
        evs.push(P::Frame { time_ms: i });
    }
    let out = feed(&mut s, &evs);
    let a = amostras(&out);
    assert_eq!(a.len(), 50);
    assert!(
        a.iter()
            .enumerate()
            .all(|(i, s)| s.time_ms == i as u32 && s.pos[0] == i as f64)
    );
}

#[test]
fn a_tool_that_entered_another_surface_is_silent_until_it_enters_ours() {
    let mut s = Sessao::default();
    let mut out = Vec::new();
    s.feed(
        T,
        P::ProximityIn {
            on_our_surface: false,
            kind: ToolKind::Pen,
        },
        &mut out,
    );
    for e in [
        P::Motion { x: 1.0, y: 1.0 },
        P::Down,
        P::Button {
            code: 0x14b,
            pressed: true,
        },
        P::Frame { time_ms: 1 },
        P::Up,
        P::ProximityOut,
        P::Frame { time_ms: 2 },
    ] {
        s.feed(T, e, &mut out);
    }
    assert!(out.is_empty(), "outra superfície: {out:?}");
    entra(&mut s, ToolKind::Pen);
    let a = uma(&feed(
        &mut s,
        &[P::Motion { x: 3.0, y: 4.0 }, P::Frame { time_ms: 3 }],
    ));
    assert_eq!((a.phase, a.pos), (PenPhase::Hover, [3.0, 4.0]));
}

#[test]
fn tools_are_independent() {
    let mut s = Sessao::default();
    let mut out = Vec::new();
    for k in [1, 2] {
        s.feed(
            k,
            P::ProximityIn {
                on_our_surface: k == 1,
                kind: ToolKind::Pen,
            },
            &mut out,
        );
        s.feed(
            k,
            P::Motion {
                x: k as f64,
                y: 0.0,
            },
            &mut out,
        );
        s.feed(k, P::Frame { time_ms: 5 }, &mut out);
    }
    assert_eq!(amostras(&out).len(), 1);
    assert_eq!(amostras(&out)[0].pos, [1.0, 0.0]);
}

#[test]
fn a_quick_tap_inside_one_frame_is_not_lost() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    let out = feed(
        &mut s,
        &[
            P::Motion { x: 1.0, y: 1.0 },
            P::Pressure(9000),
            P::Down,
            P::Up,
            P::Frame { time_ms: 3 },
        ],
    );
    let fases: Vec<PenPhase> = amostras(&out).iter().map(|a| a.phase).collect();
    assert_eq!(fases, [PenPhase::Down, PenPhase::Up]);
}

#[test]
fn leaving_with_the_tip_down_lifts_it_first() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    feed(
        &mut s,
        &[
            P::Motion { x: 1.0, y: 1.0 },
            P::Down,
            P::Frame { time_ms: 1 },
        ],
    );
    let out = feed(&mut s, &[P::ProximityOut, P::Frame { time_ms: 2 }]);
    assert!(
        matches!(
            out[0],
            PenEvent::Sample(PenSample {
                phase: PenPhase::Up,
                ..
            })
        ),
        "{out:?}"
    );
    assert!(matches!(out[1], PenEvent::Leave { .. }));
    assert_eq!(out.len(), 2);
}

#[test]
fn removing_a_tool_mid_stroke_lifts_and_leaves() {
    let mut s = Sessao::default();
    entra(&mut s, ToolKind::Pen);
    feed(
        &mut s,
        &[
            P::Motion { x: 1.0, y: 1.0 },
            P::Down,
            P::Frame { time_ms: 8 },
        ],
    );
    let mut out = Vec::new();
    s.remove(T, &mut out);
    assert!(
        matches!(
            out[0],
            PenEvent::Sample(PenSample {
                phase: PenPhase::Up,
                time_ms: 8,
                ..
            })
        ),
        "{out:?}"
    );
    assert_eq!(
        out[1],
        PenEvent::Leave {
            tool: ToolKind::Pen,
            time_ms: 8
        }
    );
    let mut out = Vec::new();
    s.remove(T, &mut out);
    assert!(out.is_empty(), "já esquecida");
}

#[test]
fn the_log_line_names_phase_pressure_and_tilt() {
    let e = PenEvent::Sample(PenSample {
        pos: [76.66, 48.2],
        pressure: 0.46,
        tilt_deg: [22.11, 0.0],
        distance: 0.0,
        tool: ToolKind::Pen,
        phase: PenPhase::Down,
        time_ms: 437,
    });
    assert_eq!(
        e.to_string(),
        "437 pen down 76.66/48.20 pressure: 0.4600 tilt: 22.11/0.00 distance: 0.000"
    );
}
