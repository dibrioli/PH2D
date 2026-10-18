//! A caneta vista pela app, sem compositor: os eventos do protocolo (`zwp_tablet_tool_v2`)
//! acumulam por ferramenta e saem no `frame`.
//!
//! A lei (ADR-0189):
//! - UMA [`PenSample`] por `frame` que mexeu num eixo (posição, pressão, distância, inclinação) ou
//!   na ponta; nunca se juntam `frame`s; dentro de um `frame` o último valor ganha;
//! - a fase é `Down`/`Up` no `frame` da ponta, `Move` com a ponta na mesa, `Hover` no ar;
//! - os botões saem no `frame` deles, pela ordem de chegada, ANTES da amostra;
//! - `ProximityOut` → [`PenEvent::Leave`] no `frame` dele (depois da amostra);
//! - uma ferramenta que entrou noutra superfície cala-se até entrar na nossa;
//! - os eixos persistem entre `frame`s (o protocolo só manda o que muda); `Up` zera a pressão.

use std::collections::BTreeMap;
use std::fmt;

/// O que a ponta é. A borracha (a outra ponta da caneta) apaga; o resto desenha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToolKind {
    #[default]
    Pen,
    Eraser,
}

impl ToolKind {
    /// O `type` do `zwp_tablet_tool_v2` (os `BTN_TOOL_*` do kernel): só `eraser` (`0x141`) apaga;
    /// pen, brush, pencil, airbrush, finger, mouse e lens desenham.
    pub fn from_protocol_type(code: u32) -> Self {
        if code == 0x141 {
            Self::Eraser
        } else {
            Self::Pen
        }
    }
}

/// Os botões do corpo da caneta (os códigos de botão do evdev, que o `zwp_tablet_tool_v2` repassa).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenButton {
    /// `BTN_STYLUS` (`0x14b`) — o de baixo, mais perto da ponta.
    Lower,
    /// `BTN_STYLUS2` (`0x14c`) — o de cima.
    Upper,
    Other(u32),
}

impl PenButton {
    pub fn from_code(code: u32) -> Self {
        match code {
            0x14b => Self::Lower,
            0x14c => Self::Upper,
            c => Self::Other(c),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenPhase {
    /// No ar, ao alcance da mesa.
    Hover,
    /// A ponta tocou NESTE `frame`.
    Down,
    /// A ponta na mesa, a mexer.
    Move,
    /// A ponta saiu da mesa NESTE `frame` (pressão `0`).
    Up,
}

/// Uma amostra da caneta: um `frame` do compositor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PenSample {
    /// Pixels LÓGICOS da superfície (o `wl_fixed` do compositor).
    pub pos: [f64; 2],
    /// `0..=1` = o bruto `0..=65535` do protocolo.
    pub pressure: f32,
    /// Graus, `[x, y]`, positivos para a direita e para o utilizador.
    pub tilt_deg: [f32; 2],
    /// `0..=1` = o bruto `0..=65535` (sem unidade física).
    pub distance: f32,
    pub tool: ToolKind,
    pub phase: PenPhase,
    /// O relógio do `frame` (ms, base indefinida, a do compositor).
    pub time_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PenEvent {
    Sample(PenSample),
    Button {
        button: PenButton,
        pressed: bool,
        pos: [f64; 2],
        tool: ToolKind,
        time_ms: u32,
    },
    /// A ferramenta saiu do alcance (ou da nossa superfície).
    Leave {
        tool: ToolKind,
        time_ms: u32,
    },
}

/// Um evento do `zwp_tablet_tool_v2`, já sem objectos Wayland (a tradução é 1:1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtocolEvent {
    /// `on_our_surface`: o `wl_surface` do `proximity_in` é o da janela.
    ProximityIn {
        on_our_surface: bool,
        kind: ToolKind,
    },
    ProximityOut,
    Down,
    Up,
    Motion {
        x: f64,
        y: f64,
    },
    /// `0..=65535`.
    Pressure(u32),
    /// `0..=65535`.
    Distance(u32),
    /// Graus.
    Tilt {
        x: f64,
        y: f64,
    },
    Button {
        code: u32,
        pressed: bool,
    },
    Frame {
        time_ms: u32,
    },
}

/// Identifica uma ferramenta física (no Wayland, o id do `zwp_tablet_tool_v2`).
pub type ToolKey = u64;

const AXIS_MAX: f32 = 65535.0;

#[derive(Debug, Default)]
struct Ferramenta {
    kind: ToolKind,
    /// Em proximidade sobre a NOSSA superfície: só então se fala.
    nossa: bool,
    pos: [f64; 2],
    pressure: u32,
    distance: u32,
    tilt: [f32; 2],
    na_mesa: bool,
    last_time: u32,
    // o `frame` por fechar
    eixos: bool,
    /// As mudanças da ponta, por ordem (`true` = desceu).
    ponta: Vec<bool>,
    botoes: Vec<(u32, bool)>,
    saiu: bool,
}

impl Ferramenta {
    fn amostra(&self, phase: PenPhase, time_ms: u32) -> PenEvent {
        PenEvent::Sample(PenSample {
            pos: self.pos,
            pressure: self.pressure as f32 / AXIS_MAX,
            tilt_deg: self.tilt,
            distance: self.distance as f32 / AXIS_MAX,
            tool: self.kind,
            phase,
            time_ms,
        })
    }

    fn fecha(&mut self, time_ms: u32, out: &mut Vec<PenEvent>) {
        self.last_time = time_ms;
        for (code, pressed) in self.botoes.drain(..) {
            out.push(PenEvent::Button {
                button: PenButton::from_code(code),
                pressed,
                pos: self.pos,
                tool: self.kind,
                time_ms,
            });
        }
        // Sair com a ponta na mesa sem `up` (o protocolo manda-o antes; um compositor que não o
        // mande deixava o traço preso).
        if self.saiu && self.na_mesa && self.ponta.last() != Some(&false) {
            self.ponta.push(false);
        }
        if self.ponta.is_empty() {
            if self.eixos {
                let phase = if self.na_mesa {
                    PenPhase::Move
                } else {
                    PenPhase::Hover
                };
                out.push(self.amostra(phase, time_ms));
            }
        } else {
            // Um toque rápido (desce e sobe no MESMO `frame`) não se perde: uma amostra por mudança.
            let ponta = std::mem::take(&mut self.ponta);
            for desceu in ponta {
                self.na_mesa = desceu;
                let phase = if desceu {
                    PenPhase::Down
                } else {
                    self.pressure = 0;
                    PenPhase::Up
                };
                out.push(self.amostra(phase, time_ms));
            }
        }
        self.eixos = false;
        if self.saiu {
            out.push(PenEvent::Leave {
                tool: self.kind,
                time_ms,
            });
            self.esquece();
        }
    }

    fn esquece(&mut self) {
        *self = Self {
            kind: self.kind,
            last_time: self.last_time,
            ..Self::default()
        };
    }
}

/// A máquina de todas as ferramentas, por [`ToolKey`].
#[derive(Debug, Default)]
pub struct Sessao {
    tools: BTreeMap<ToolKey, Ferramenta>,
}

impl Sessao {
    /// Um evento do protocolo; o que a app deve ver sai em `out` (só no `Frame`).
    pub fn feed(&mut self, tool: ToolKey, ev: ProtocolEvent, out: &mut Vec<PenEvent>) {
        let t = self.tools.entry(tool).or_default();
        if let ProtocolEvent::ProximityIn {
            on_our_surface,
            kind,
        } = ev
        {
            t.esquece();
            t.kind = kind;
            t.nossa = on_our_surface;
            return;
        }
        if !t.nossa {
            return;
        }
        match ev {
            ProtocolEvent::ProximityIn { .. } => unreachable!("tratado acima"),
            ProtocolEvent::ProximityOut => t.saiu = true,
            ProtocolEvent::Down => t.ponta.push(true),
            ProtocolEvent::Up => t.ponta.push(false),
            ProtocolEvent::Motion { x, y } => {
                t.pos = [x, y];
                t.eixos = true;
            }
            ProtocolEvent::Pressure(p) => {
                t.pressure = p.min(u16::MAX.into());
                t.eixos = true;
            }
            ProtocolEvent::Distance(d) => {
                t.distance = d.min(u16::MAX.into());
                t.eixos = true;
            }
            ProtocolEvent::Tilt { x, y } => {
                t.tilt = [x as f32, y as f32];
                t.eixos = true;
            }
            ProtocolEvent::Button { code, pressed } => t.botoes.push((code, pressed)),
            ProtocolEvent::Frame { time_ms } => t.fecha(time_ms, out),
        }
    }

    /// A ferramenta deixou de existir (`removed`, a mesa desligada): fecha o traço e sai.
    pub fn remove(&mut self, tool: ToolKey, out: &mut Vec<PenEvent>) {
        let Some(mut t) = self.tools.remove(&tool) else {
            return;
        };
        if !t.nossa {
            return;
        }
        t.eixos = false;
        t.botoes.clear();
        t.ponta.clear();
        t.saiu = true;
        let time = t.last_time;
        t.fecha(time, out);
    }
}

impl fmt::Display for PenPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Hover => "hover",
            Self::Down => "down",
            Self::Move => "move",
            Self::Up => "up",
        })
    }
}

impl fmt::Display for ToolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pen => "pen",
            Self::Eraser => "eraser",
        })
    }
}

/// Uma linha por evento — o registo `PH2D_PEN_LOG`, comparável ao `libinput debug-events`.
impl fmt::Display for PenEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sample(s) => write!(
                f,
                "{} {} {} {:.2}/{:.2} pressure: {:.4} tilt: {:.2}/{:.2} distance: {:.3}",
                s.time_ms,
                s.tool,
                s.phase,
                s.pos[0],
                s.pos[1],
                s.pressure,
                s.tilt_deg[0],
                s.tilt_deg[1],
                s.distance
            ),
            Self::Button {
                button,
                pressed,
                pos,
                tool,
                time_ms,
            } => write!(
                f,
                "{time_ms} {tool} button {button:?} {} {:.2}/{:.2}",
                if *pressed { "pressed" } else { "released" },
                pos[0],
                pos[1]
            ),
            Self::Leave { tool, time_ms } => write!(f, "{time_ms} {tool} leave"),
        }
    }
}

#[cfg(test)]
#[path = "sessao_tests.rs"]
mod tests;
