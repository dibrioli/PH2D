use super::*;
use crate::sessao::{PenSample, ProtocolEvent as P, Sessao};

const ESCALA: f64 = 1.5;
const AGORA: u128 = 1_000_000_000_000;

/// Os eventos do protocolo pela `Sessao` e daí pelo conversor, num lote só.
fn corre(c: &mut Conversor, s: &mut Sessao, evs: &[P], agora: u128) -> Vec<Saida> {
    let mut pen = Vec::new();
    for e in evs {
        s.feed(1, *e, &mut pen);
    }
    let mut out = Vec::new();
    c.converte(&pen, ESCALA, agora, &mut out);
    out
}

fn ponteiros(out: &[Saida]) -> Vec<PointerEvent> {
    out.iter()
        .filter_map(|s| match s {
            Saida::Ponteiro(e) => Some(*e),
            Saida::Saiu => None,
        })
        .collect()
}

fn resumo(out: &[Saida]) -> Vec<(PointerKind, PointerButton, f32)> {
    ponteiros(out)
        .iter()
        .map(|e| (e.kind, e.button, e.pressure))
        .collect()
}

fn entra(kind: ToolKind) -> P {
    P::ProximityIn {
        on_our_surface: true,
        kind,
    }
}

fn pressao(v: f32) -> P {
    P::Pressure((v * 65535.0).round() as u32)
}

fn amostra(phase: PenPhase, pressure: f32, time_ms: u32) -> PenEvent {
    PenEvent::Sample(PenSample {
        pos: [10.0, 20.0],
        pressure,
        tilt_deg: [0.0, 0.0],
        distance: 0.0,
        tool: ToolKind::Pen,
        phase,
        time_ms,
    })
}

#[test]
fn the_mouse_is_pressure_one_and_source_mouse() {
    let e = do_rato(3.0, 4.0, PointerKind::Down, PointerButton::Secondary, 9);
    assert_eq!((e.pressure, e.source), (1.0, PointerSource::Mouse));
    assert_eq!(
        (e.x, e.y, e.kind, e.button, e.timestamp_ns),
        (3.0, 4.0, PointerKind::Down, PointerButton::Secondary, 9)
    );
    assert_eq!(Tinta::de(&e), Tinta::RATO);
    assert_eq!(
        (Tinta::RATO.tilt(), Tinta::RATO.borracha()),
        ([0.0, 0.0], false)
    );
}

#[test]
fn hover_is_a_move_with_zero_pressure_in_physical_pixels() {
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    let out = corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Pen),
            P::Motion { x: 100.0, y: 40.0 },
            P::Distance(30000),
            P::Frame { time_ms: 5 },
        ],
        AGORA,
    );
    let e = ponteiros(&out);
    assert_eq!(e.len(), 1, "{out:?}");
    assert_eq!(
        (e[0].kind, e[0].button, e[0].pressure),
        (PointerKind::Move, PointerButton::Primary, 0.0)
    );
    assert_eq!((e[0].x, e[0].y), (150.0, 60.0), "lógico × 1,5");
    assert_eq!(e[0].source, PointerSource::Pen(PenInfo::default()));
}

/// A ponta é o clique primário, com a pressão da amostra; o arrasto leva a pressão de cada amostra.
#[test]
fn the_tip_is_a_primary_click_and_moves_carry_the_pressure() {
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Pen),
            P::Motion { x: 10.0, y: 10.0 },
            P::Frame { time_ms: 1 },
        ],
        AGORA,
    );
    let out = corre(
        &mut c,
        &mut s,
        &[P::Down, pressao(0.25), P::Frame { time_ms: 2 }],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [(PointerKind::Down, PointerButton::Primary, 16384.0 / 65535.0)]
    );
    let out = corre(
        &mut c,
        &mut s,
        &[
            P::Motion { x: 12.0, y: 10.0 },
            pressao(0.75),
            P::Frame { time_ms: 7 },
        ],
        AGORA,
    );
    let e = ponteiros(&out);
    assert_eq!(
        (e[0].kind, e[0].button),
        (PointerKind::Move, PointerButton::Primary)
    );
    assert!((e[0].pressure - 0.75).abs() < 1e-4, "{e:?}");
    let out = corre(&mut c, &mut s, &[P::Up, P::Frame { time_ms: 9 }], AGORA);
    assert_eq!(
        resumo(&out),
        [(PointerKind::Up, PointerButton::Primary, 0.0)]
    );
}

/// Os extremos do eixo chegam inteiros: `0` e `65535` são `0.0` e `1.0`.
#[test]
fn pressure_extremes_reach_zero_and_one() {
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Pen),
            P::Motion { x: 1.0, y: 1.0 },
            P::Frame { time_ms: 1 },
        ],
        AGORA,
    );
    let out = corre(
        &mut c,
        &mut s,
        &[P::Down, P::Pressure(0), P::Frame { time_ms: 2 }],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [(PointerKind::Down, PointerButton::Primary, 0.0)]
    );
    let out = corre(
        &mut c,
        &mut s,
        &[P::Pressure(65535), P::Frame { time_ms: 3 }],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [(PointerKind::Move, PointerButton::Primary, 1.0)]
    );
}

/// Um toque rápido (desce e sobe no MESMO `frame`) é um clique inteiro, pela ordem, e o `Move` até o
/// sítio novo vem antes dele.
#[test]
fn a_tap_inside_one_frame_is_move_down_up_in_order() {
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Pen),
            P::Motion { x: 1.0, y: 1.0 },
            P::Frame { time_ms: 1 },
        ],
        AGORA,
    );
    let out = corre(
        &mut c,
        &mut s,
        &[
            P::Motion { x: 4.0, y: 2.0 },
            P::Down,
            pressao(0.5),
            P::Up,
            P::Frame { time_ms: 2 },
        ],
        AGORA,
    );
    let e = ponteiros(&out);
    let k: Vec<_> = e.iter().map(|e| (e.kind, e.button)).collect();
    assert_eq!(
        k,
        [
            (PointerKind::Move, PointerButton::Primary),
            (PointerKind::Down, PointerButton::Primary),
            (PointerKind::Up, PointerButton::Primary)
        ]
    );
    assert!(e.iter().all(|e| (e.x, e.y) == (6.0, 3.0)), "{e:?}");
    assert_eq!(e[0].pressure, 0.0, "o Move antes do toque é no ar");
}

/// Botão de baixo = meio, de cima = secundário, no ar sem pressão; o `Move` leva o premido; outro
/// código não existe para a UI.
#[test]
fn barrel_buttons_are_middle_and_secondary_and_others_vanish() {
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Pen),
            P::Motion { x: 1.0, y: 1.0 },
            P::Frame { time_ms: 1 },
        ],
        AGORA,
    );
    let out = corre(
        &mut c,
        &mut s,
        &[
            P::Button {
                code: 0x14b,
                pressed: true,
            },
            P::Frame { time_ms: 2 },
        ],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [(PointerKind::Down, PointerButton::Middle, 0.0)]
    );
    let out = corre(
        &mut c,
        &mut s,
        &[P::Motion { x: 2.0, y: 1.0 }, P::Frame { time_ms: 3 }],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [(PointerKind::Move, PointerButton::Middle, 0.0)]
    );
    let out = corre(
        &mut c,
        &mut s,
        &[
            P::Button {
                code: 0x14b,
                pressed: false,
            },
            P::Frame { time_ms: 4 },
        ],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [(PointerKind::Up, PointerButton::Middle, 0.0)]
    );
    let out = corre(
        &mut c,
        &mut s,
        &[
            P::Button {
                code: 0x14c,
                pressed: true,
            },
            P::Button {
                code: 0x14c,
                pressed: false,
            },
            P::Frame { time_ms: 5 },
        ],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [
            (PointerKind::Down, PointerButton::Secondary, 0.0),
            (PointerKind::Up, PointerButton::Secondary, 0.0)
        ]
    );
    let out = corre(
        &mut c,
        &mut s,
        &[
            P::Button {
                code: 0x110,
                pressed: true,
            },
            P::Frame { time_ms: 6 },
        ],
        AGORA,
    );
    assert!(out.is_empty(), "{out:?}");
}

/// Com a ponta na mesa, um botão leva a pressão do contacto.
#[test]
fn a_barrel_button_during_contact_carries_the_contact_pressure() {
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Pen),
            P::Motion { x: 1.0, y: 1.0 },
            P::Frame { time_ms: 1 },
        ],
        AGORA,
    );
    corre(
        &mut c,
        &mut s,
        &[P::Down, P::Pressure(32768), P::Frame { time_ms: 2 }],
        AGORA,
    );
    let out = corre(
        &mut c,
        &mut s,
        &[
            P::Button {
                code: 0x14c,
                pressed: true,
            },
            P::Frame { time_ms: 3 },
        ],
        AGORA,
    );
    assert_eq!(
        resumo(&out),
        [(
            PointerKind::Down,
            PointerButton::Secondary,
            32768.0 / 65535.0
        )]
    );
}

/// A borracha e a inclinação viajam na fonte, em centésimos de grau; a `Tinta` normaliza o tilt.
#[test]
fn eraser_and_tilt_travel_in_the_source() {
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    let out = corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Eraser),
            P::Motion { x: 1.0, y: 1.0 },
            P::Tilt { x: 45.5, y: -30.25 },
            P::Down,
            pressao(0.5),
            P::Frame { time_ms: 2 },
        ],
        AGORA,
    );
    let e = ponteiros(&out);
    let info = PenInfo {
        tilt_cdeg: [4550, -3025],
        eraser: true,
    };
    assert!(
        e.iter().all(|e| e.source == PointerSource::Pen(info)),
        "{e:?}"
    );
    let t = Tinta::de(&e[1]);
    assert!(t.borracha());
    assert_eq!(t.tilt(), [4550.0 / 9000.0, -3025.0 / 9000.0]);
    // E a ponta de desenhar não apaga; o tilt satura no alcance do protocolo.
    let mut c = Conversor::default();
    let mut out = Vec::new();
    let mut a = amostra(PenPhase::Hover, 0.0, 1);
    if let PenEvent::Sample(s) = &mut a {
        s.tilt_deg = [120.0, -91.0];
    }
    c.converte(&[a], 1.0, AGORA, &mut out);
    let t = Tinta::de(&ponteiros(&out)[0]);
    assert!(!t.borracha());
    assert_eq!(t.tilt(), [1.0, -1.0]);
}

/// `Leave` solta o que ficou premido e diz que saiu; nada mais.
#[test]
fn leave_releases_what_is_held_and_says_so() {
    let mut c = Conversor::default();
    let mut out = Vec::new();
    let lote = [
        amostra(PenPhase::Hover, 0.0, 1),
        PenEvent::Button {
            button: PenButton::Lower,
            pressed: true,
            pos: [10.0, 20.0],
            tool: ToolKind::Pen,
            time_ms: 2,
        },
        PenEvent::Leave {
            tool: ToolKind::Pen,
            time_ms: 3,
        },
    ];
    c.converte(&lote, 1.0, AGORA, &mut out);
    assert_eq!(
        resumo(&out),
        [
            (PointerKind::Move, PointerButton::Primary, 0.0),
            (PointerKind::Down, PointerButton::Middle, 0.0),
            (PointerKind::Up, PointerButton::Middle, 0.0)
        ]
    );
    assert_eq!(out.last(), Some(&Saida::Saiu));
    // Uma saída limpa (a `Sessao` já mandou o `Up`) é só o `Saiu`.
    let (mut c, mut s) = (Conversor::default(), Sessao::default());
    corre(
        &mut c,
        &mut s,
        &[
            entra(ToolKind::Pen),
            P::Motion { x: 1.0, y: 1.0 },
            P::Frame { time_ms: 1 },
        ],
        AGORA,
    );
    let out = corre(
        &mut c,
        &mut s,
        &[P::ProximityOut, P::Frame { time_ms: 2 }],
        AGORA,
    );
    assert_eq!(out, [Saida::Saiu]);
}

/// O relógio: a amostra mais nova do lote é o `agora`; as outras guardam o espaçamento do compositor;
/// o lote seguinte conta da âncora; nunca recua nem passa o `agora`; a volta dos 32 bits não parte nada.
#[test]
fn timestamps_keep_the_compositor_spacing_anchored_to_the_host_clock() {
    let ms = 1_000_000;
    let ts = |out: &[Saida]| {
        ponteiros(out)
            .iter()
            .map(|e| e.timestamp_ns)
            .collect::<Vec<_>>()
    };
    let m = |d: u32| (u32::MAX - 6).wrapping_add(d);
    let mut c = Conversor::default();
    let mut out = Vec::new();
    let lote: Vec<_> = (0..3)
        .map(|k| amostra(PenPhase::Hover, 0.0, m(5 * k)))
        .collect();
    c.converte(&lote, 1.0, AGORA, &mut out);
    assert_eq!(
        ts(&out),
        [AGORA - 10 * ms, AGORA - 5 * ms, AGORA],
        "u32::MAX dá a volta no meio"
    );
    // Lote seguinte, drenado 20 ms depois, com amostras 5 ms à frente: contam da âncora.
    let mut out = Vec::new();
    c.converte(
        &[amostra(PenPhase::Hover, 0.0, m(15))],
        1.0,
        AGORA + 20 * ms,
        &mut out,
    );
    assert_eq!(ts(&out), [AGORA + 5 * ms]);
    // Um lote que cairia no futuro re-ancora no `agora` (o atraso diminuiu).
    let mut out = Vec::new();
    c.converte(
        &[amostra(PenPhase::Hover, 0.0, m(100))],
        1.0,
        AGORA + 30 * ms,
        &mut out,
    );
    assert_eq!(ts(&out), [AGORA + 30 * ms]);
    // Uma amostra atrasada de outra ferramenta não faz o tempo recuar.
    let mut out = Vec::new();
    c.converte(
        &[
            amostra(PenPhase::Hover, 0.0, m(50)),
            amostra(PenPhase::Hover, 0.0, m(101)),
        ],
        1.0,
        AGORA + 31 * ms,
        &mut out,
    );
    let t = ts(&out);
    assert_eq!(t, [AGORA + 30 * ms, AGORA + 31 * ms]);
}
