//! O ORÁCULO: 180 s do dono a riscar com a Intuos5 (`sudo libinput debug-events --device
//! /dev/input/event14`, 17/10, `docs/Painter/ferramentas/caneta/README.md`), lido tal como foi
//! gravado (`.zst`).
//!
//! O KWin passa o libinput 1:1 a frames do `zwp_tablet_tool_v2`; cada linha vira os eventos do
//! protocolo que implica + um `frame`:
//! - proximidade-in → `proximity_in` + `motion` + todos os eixos; proximidade-out → `proximity_out`;
//! - eixo/ponta → `motion` + só os eixos que MUDARAM (o protocolo só manda mudanças: a sessão tem de
//!   os segurar) + `down`/`up`;
//! - botão → `button`.
//!
//! Valores como o KWin os manda: pressão e distância `(v × 65535) as u32`, posição e inclinação em
//! `wl_fixed` (1/256). O libinput imprime `pressure` só quando a `distance` é `0` (senão a pressão é
//! `0`), com `*` no eixo que mudou. A posição em mm vira px × 10 (a escala não importa à lei).

use std::path::Path;

use ph2d_host::{PointerButton, PointerKind};
use ph2d_pen::ponteiro::{Conversor, Saida};
use ph2d_pen::{PenButton, PenEvent, PenPhase, ProtocolEvent as P, Sessao, ToolKind};

const GRAVACAO: &str = "../../docs/Painter/ferramentas/caneta/medicoes_2026-10-17/libinput_debug_events_event14.txt.zst";

const PASSO: f32 = 1.0 / 65535.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tipo {
    ProxIn,
    ProxOut,
    Axis,
    TipDown,
    TipUp,
    Button { code: u32, pressed: bool },
}

#[derive(Debug, Clone, Copy)]
struct Linha {
    tipo: Tipo,
    time_ms: u32,
    pos: [f64; 2],
    tilt: [f64; 2],
    pressure: f32,
    distance: f32,
    kind: ToolKind,
}

fn fixo(v: f64) -> f64 {
    (v * 256.0).round() / 256.0
}

fn bruto(v: f32) -> u32 {
    (v * 65535.0) as u32
}

/// `"72.28*/49.17*"` ou `"57.58 /98.42 "` → os dois números (o `*` só diz que mudou).
fn par(s: &str) -> [f64; 2] {
    let s: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '*')
        .collect();
    let (a, b) = s.split_once('/').unwrap_or_else(|| panic!("par: {s:?}"));
    [a.parse().unwrap(), b.parse().unwrap()]
}

fn valor_depois(linha: &str, chave: &str) -> Option<f32> {
    let i = linha.find(chave)? + chave.len();
    let tok = linha[i..].split_whitespace().next()?;
    Some(tok.trim_end_matches('*').parse().unwrap())
}

fn le(linha: &str) -> Option<Linha> {
    let mut toks = linha.split_whitespace();
    let _dev = toks.next()?;
    let ev = toks.next()?;
    if !ev.starts_with("TABLET_TOOL_") {
        return None;
    }
    let t_tok = toks.find(|t| t.starts_with('+') && t.ends_with('s'))?;
    let time_ms = (t_tok[1..t_tok.len() - 1].parse::<f64>().unwrap() * 1000.0).round() as u32;
    let resto = &linha[linha.find(t_tok).unwrap() + t_tok.len()..];
    if ev == "TABLET_TOOL_BUTTON" {
        let code = resto.split_whitespace().next().unwrap()[1..]
            .parse()
            .unwrap();
        return Some(Linha {
            tipo: Tipo::Button {
                code,
                pressed: resto.contains(" pressed"),
            },
            time_ms,
            pos: [0.0; 2],
            tilt: [0.0; 2],
            pressure: 0.0,
            distance: 0.0,
            kind: ToolKind::Pen,
        });
    }
    let ti = resto.find("tilt:").unwrap();
    let fim_tilt = resto
        .find("distance:")
        .or_else(|| resto.find("pressure:"))
        .unwrap();
    let pos = par(&resto[..ti]);
    let tilt = par(&resto[ti + 5..fim_tilt]);
    let tipo = match ev {
        "TABLET_TOOL_PROXIMITY" if resto.contains("proximity-in") => Tipo::ProxIn,
        "TABLET_TOOL_PROXIMITY" => Tipo::ProxOut,
        "TABLET_TOOL_AXIS" => Tipo::Axis,
        "TABLET_TOOL_TIP" if resto.trim_end().ends_with("down") => Tipo::TipDown,
        "TABLET_TOOL_TIP" => Tipo::TipUp,
        outro => panic!("evento desconhecido {outro}"),
    };
    Some(Linha {
        tipo,
        time_ms,
        pos: [fixo(pos[0] * 10.0), fixo(pos[1] * 10.0)],
        tilt: [fixo(tilt[0]), fixo(tilt[1])],
        pressure: valor_depois(resto, "pressure:").unwrap_or(0.0),
        distance: valor_depois(resto, "distance:").unwrap_or(0.0),
        kind: if resto.contains("eraser") {
            ToolKind::Eraser
        } else {
            ToolKind::Pen
        },
    })
}

/// O que o compositor manda por linha, só com os eixos que mudaram desde o último `frame`.
#[derive(Default)]
struct Compositor {
    enviado: Option<(u32, u32, [f64; 2])>,
}

impl Compositor {
    fn eventos(&mut self, l: &Linha) -> Vec<P> {
        let mut v = Vec::new();
        match l.tipo {
            Tipo::ProxOut => {
                v.push(P::ProximityOut);
                self.enviado = None;
            }
            Tipo::Button { code, pressed } => v.push(P::Button { code, pressed }),
            Tipo::ProxIn | Tipo::Axis | Tipo::TipDown | Tipo::TipUp => {
                if l.tipo == Tipo::ProxIn {
                    v.push(P::ProximityIn {
                        on_our_surface: true,
                        kind: l.kind,
                    });
                    self.enviado = None;
                }
                v.push(P::Motion {
                    x: l.pos[0],
                    y: l.pos[1],
                });
                let (p, d, t) = (bruto(l.pressure), bruto(l.distance), l.tilt);
                let antes = self.enviado;
                if antes.map(|a| a.0) != Some(p) {
                    v.push(P::Pressure(p));
                }
                if antes.map(|a| a.1) != Some(d) {
                    v.push(P::Distance(d));
                }
                if antes.map(|a| a.2) != Some(t) {
                    v.push(P::Tilt { x: t[0], y: t[1] });
                }
                self.enviado = Some((p, d, t));
                match l.tipo {
                    Tipo::TipDown => v.push(P::Down),
                    Tipo::TipUp => v.push(P::Up),
                    _ => {}
                }
            }
        }
        v.push(P::Frame { time_ms: l.time_ms });
        v
    }
}

#[derive(Debug, Default, PartialEq)]
struct Contas {
    axis: usize,
    tip_down: usize,
    tip_up: usize,
    button: usize,
    prox_in: usize,
    prox_out: usize,
    samples: usize,
    downs: usize,
    ups: usize,
    buttons: usize,
    leaves: usize,
}

#[test]
fn the_owners_recording_replays_step_by_step() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(GRAVACAO);
    let zst = std::fs::File::open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let texto = String::from_utf8(zstd::stream::decode_all(zst).unwrap()).unwrap();

    let mut sessao = Sessao::default();
    let mut conv = Conversor::default();
    let mut cliques = 0;
    let mut comp = Compositor::default();
    let mut c = Contas::default();
    let mut na_mesa = false;
    let mut ultima_pos = [0.0; 2];
    let mut pressao_max = 0.0f32;
    for (n, texto_linha) in texto.lines().enumerate() {
        let Some(l) = le(texto_linha) else {
            continue;
        };
        let mut out = Vec::new();
        for e in comp.eventos(&l) {
            sessao.feed(1, e, &mut out);
        }
        let ctx = || format!("linha {}: {texto_linha}\n→ {out:?}", n + 1);
        assert_eq!(out.len(), 1, "uma linha do libinput, um evento: {}", ctx());
        match (l.tipo, out[0]) {
            (
                Tipo::Button { code, pressed },
                PenEvent::Button {
                    button,
                    pressed: p,
                    pos,
                    tool,
                    time_ms,
                },
            ) => {
                c.button += 1;
                c.buttons += 1;
                assert_eq!(button, PenButton::from_code(code), "{}", ctx());
                assert_ne!(
                    button,
                    PenButton::Other(code),
                    "a Intuos5 só tem os dois: {}",
                    ctx()
                );
                assert_eq!(
                    (p, pos, tool, time_ms),
                    (pressed, ultima_pos, l.kind, l.time_ms),
                    "{}",
                    ctx()
                );
            }
            (Tipo::ProxOut, PenEvent::Leave { tool, time_ms }) => {
                c.prox_out += 1;
                c.leaves += 1;
                assert_eq!((tool, time_ms), (l.kind, l.time_ms), "{}", ctx());
                assert!(!na_mesa, "{}", ctx());
            }
            (tipo, PenEvent::Sample(s)) => {
                c.samples += 1;
                let fase = match tipo {
                    Tipo::ProxIn => {
                        c.prox_in += 1;
                        PenPhase::Hover
                    }
                    Tipo::Axis => {
                        c.axis += 1;
                        if na_mesa {
                            PenPhase::Move
                        } else {
                            PenPhase::Hover
                        }
                    }
                    Tipo::TipDown => {
                        c.tip_down += 1;
                        c.downs += 1;
                        na_mesa = true;
                        PenPhase::Down
                    }
                    Tipo::TipUp => {
                        c.tip_up += 1;
                        c.ups += 1;
                        na_mesa = false;
                        PenPhase::Up
                    }
                    _ => panic!("tipo sem amostra: {}", ctx()),
                };
                assert_eq!(s.phase, fase, "{}", ctx());
                assert_eq!(
                    (s.pos, s.time_ms, s.tool),
                    (l.pos, l.time_ms, l.kind),
                    "{}",
                    ctx()
                );
                assert_eq!(
                    [f64::from(s.tilt_deg[0]), f64::from(s.tilt_deg[1])],
                    l.tilt,
                    "{}",
                    ctx()
                );
                let esperada = if fase == PenPhase::Up {
                    0.0
                } else {
                    l.pressure
                };
                assert!(
                    (s.pressure - esperada).abs() <= PASSO,
                    "pressão {}: {}",
                    s.pressure,
                    ctx()
                );
                assert!(
                    (s.distance - l.distance).abs() <= PASSO,
                    "distância: {}",
                    ctx()
                );
                pressao_max = pressao_max.max(s.pressure);
                ultima_pos = s.pos;
            }
            _ => panic!("evento trocado: {}", ctx()),
        }
        // …e pelo conversor (ADR-0189 §3): a MESMA pressão sai no `PointerEvent`, evento a evento.
        let mut ponteiro = Vec::new();
        conv.converte(&out, 1.0, u128::from(l.time_ms) * 1_000_000, &mut ponteiro);
        if let (PenEvent::Sample(s), Some(Saida::Ponteiro(e))) = (out[0], ponteiro.last()) {
            let esperada = if s.phase == PenPhase::Hover {
                0.0
            } else {
                s.pressure
            };
            assert_eq!(e.pressure, esperada, "{}", ctx());
            assert_eq!(
                e.kind == PointerKind::Down,
                s.phase == PenPhase::Down,
                "{}",
                ctx()
            );
            if e.kind == PointerKind::Down {
                assert_eq!(e.button, PointerButton::Primary, "{}", ctx());
                cliques += 1;
            }
        }
    }
    println!("{c:#?}");
    // A gravação é fixa: estas contas são as do README da pasta.
    assert_eq!(
        (
            c.axis,
            c.tip_down + c.tip_up,
            c.button,
            c.prox_in + c.prox_out
        ),
        (9890, 210, 16, 14)
    );
    assert_eq!(c.samples, c.axis + c.tip_down + c.tip_up + c.prox_in);
    assert_eq!((c.downs, c.ups), (c.tip_down, c.tip_up));
    assert_eq!(
        cliques, c.tip_down,
        "cada toque da ponta é um clique primário"
    );
    assert_eq!((c.buttons, c.leaves), (c.button, c.prox_out));
    // O máximo gravado é `0.97` (15 linhas); o `65535 → 1.0` exacto prova-o `sessao_tests`.
    assert!(
        (pressao_max - 0.97).abs() <= PASSO,
        "pressão máxima {pressao_max}"
    );
}
