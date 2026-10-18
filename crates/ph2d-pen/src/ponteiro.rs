//! A caneta como o RATO a veria: cada [`PenEvent`] vira os `PointerEvent`s que a porta do rato da
//! shell recebe (ADR-0189 §3), logo a caneta conduz a UI inteira.
//!
//! A lei:
//! - a ponta é o botão PRIMÁRIO; o botão de baixo (`BTN_STYLUS`) o do MEIO; o de cima
//!   (`BTN_STYLUS2`) o SECUNDÁRIO (a convenção do GNOME/Krita); outro botão não existe para a UI;
//! - posição: pixels LÓGICOS da superfície × o factor de escala = os FÍSICOS do `PointerEvent`;
//! - pressão: a da amostra; no ar (`Hover`) `0.0`; o `Up` traz a dele (`0.0`, a ponta largou);
//! - um clique num sítio NOVO vem depois de um `Move` até lá — o rato manda sempre o `CursorMoved`
//!   antes do clique, e a shell resolve o clique no `last_pointer`;
//! - o `button` de um `Move` é o último premido, `Primary` sem nenhum (o `held_button` da shell);
//! - `Leave` não é ponteiro: solta o que ficou premido (compositor que não o soltou) e devolve
//!   [`Saida::Saiu`];
//! - o relógio: ver [`Relogio`].

use ph2d_host::{PenInfo, PointerButton, PointerEvent, PointerKind, PointerSource};

use crate::sessao::{PenButton, PenEvent, PenPhase, ToolKind};

/// O evento do RATO: pressão `1.0`, fonte `Mouse` — a caneta não muda um byte do rato.
#[must_use]
pub fn do_rato(
    x: f32,
    y: f32,
    kind: PointerKind,
    button: PointerButton,
    timestamp_ns: u128,
) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button,
        timestamp_ns,
    }
}

/// O que um evento de ponteiro carrega além da posição — o que os meios de desenho leem.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tinta {
    pub pressure: f32,
    pub source: PointerSource,
}

impl Tinta {
    pub const RATO: Self = Self {
        pressure: 1.0,
        source: PointerSource::Mouse,
    };

    #[must_use]
    pub fn de(evt: &PointerEvent) -> Self {
        Self {
            pressure: evt.pressure,
            source: evt.source,
        }
    }

    /// A inclinação no idioma do `CanvasPointer::tilt`: `[-1, 1]` por eixo = `±90°` (o alcance do
    /// protocolo); `[0, 0]` sem caneta.
    #[must_use]
    pub fn tilt(self) -> [f32; 2] {
        match self.source {
            PointerSource::Pen(i) => i.tilt_cdeg.map(|c| f32::from(c) / 9000.0),
            _ => [0.0, 0.0],
        }
    }

    /// A ponta de borracha está na mesa.
    #[must_use]
    pub fn borracha(self) -> bool {
        matches!(
            self.source,
            PointerSource::Pen(PenInfo { eraser: true, .. })
        )
    }
}

/// O que sai de um [`PenEvent`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Saida {
    Ponteiro(PointerEvent),
    /// A caneta saiu do alcance: a shell volta a ler o rato.
    Saiu,
}

/// O `time_ms` do compositor (base indefinida) no relógio do `PointerEvent::timestamp_ns` da shell.
///
/// Cada lote drenado ancora-se pela amostra MAIS NOVA = o `agora` do dreno, se ela cairia no futuro
/// (ou se não há âncora); as outras contam o intervalo do compositor a partir dela — o espaçamento
/// verdadeiro (~5 ms a 200 Hz) sobrevive a um quadro que drena várias. A âncora converge para o
/// atraso mínimo; o tempo nunca recua nem passa o `agora`.
#[derive(Debug, Default)]
struct Relogio {
    ancora: Option<(u32, u128)>,
    ultimo: u128,
}

impl Relogio {
    fn em((m0, n0): (u32, u128), ms: u32) -> i128 {
        // `i32`: o relógio de 32 bits dá a volta a cada ~49 dias; a diferença é com sinal.
        n0 as i128 + i128::from(ms.wrapping_sub(m0) as i32) * 1_000_000
    }

    fn ancora_lote(&mut self, ms_novo: u32, agora: u128) {
        if self
            .ancora
            .is_none_or(|a| Self::em(a, ms_novo) > agora as i128)
        {
            self.ancora = Some((ms_novo, agora));
        }
    }

    fn ns(&mut self, ms: u32, agora: u128) -> u128 {
        let a = *self.ancora.get_or_insert((ms, agora));
        let t = Self::em(a, ms).clamp(0, agora as i128) as u128;
        self.ultimo = self.ultimo.max(t);
        self.ultimo
    }
}

fn tempo_de(ev: &PenEvent) -> u32 {
    match *ev {
        PenEvent::Sample(s) => s.time_ms,
        PenEvent::Button { time_ms, .. } | PenEvent::Leave { time_ms, .. } => time_ms,
    }
}

/// A máquina da caneta-como-rato (uma por janela).
#[derive(Debug, Default)]
pub struct Conversor {
    relogio: Relogio,
    /// O último botão premido, `None` depois de qualquer `Up` (o `held_button` da shell).
    premido: Option<PointerButton>,
    em_baixo: Vec<PointerButton>,
    /// A última posição FÍSICA emitida.
    pos: Option<[f32; 2]>,
    /// A pressão do contacto em curso.
    pressao: f32,
    info: PenInfo,
}

impl Conversor {
    /// Um lote drenado, por ordem. `escala` = o factor de escala da janela; `agora_ns` = o relógio
    /// da shell no dreno.
    pub fn converte(
        &mut self,
        lote: &[PenEvent],
        escala: f64,
        agora_ns: u128,
        out: &mut Vec<Saida>,
    ) {
        let Some(novo) = lote.last() else {
            return;
        };
        self.relogio.ancora_lote(tempo_de(novo), agora_ns);
        for ev in lote {
            let t = self.relogio.ns(tempo_de(ev), agora_ns);
            self.um(ev, escala, t, out);
        }
    }

    fn um(&mut self, ev: &PenEvent, escala: f64, t: u128, out: &mut Vec<Saida>) {
        match *ev {
            PenEvent::Sample(s) => {
                self.info = PenInfo {
                    tilt_cdeg: s
                        .tilt_deg
                        .map(|g| (g * 100.0).round().clamp(-9000.0, 9000.0) as i16),
                    eraser: s.tool == ToolKind::Eraser,
                };
                let pos = fisico(s.pos, escala);
                match s.phase {
                    PenPhase::Hover => self.mexe(pos, 0.0, t, out),
                    PenPhase::Move => {
                        self.pressao = s.pressure;
                        self.mexe(pos, s.pressure, t, out);
                    }
                    PenPhase::Down => {
                        self.vai(pos, 0.0, t, out);
                        self.pressao = s.pressure;
                        self.clica(
                            PointerKind::Down,
                            PointerButton::Primary,
                            s.pressure,
                            t,
                            out,
                        );
                    }
                    PenPhase::Up => {
                        self.vai(pos, self.pressao, t, out);
                        self.pressao = 0.0;
                        self.clica(PointerKind::Up, PointerButton::Primary, s.pressure, t, out);
                    }
                }
            }
            PenEvent::Button {
                button,
                pressed,
                pos,
                ..
            } => {
                let b = match button {
                    PenButton::Lower => PointerButton::Middle,
                    PenButton::Upper => PointerButton::Secondary,
                    PenButton::Other(_) => return,
                };
                let p = if self.na_mesa() { self.pressao } else { 0.0 };
                self.vai(fisico(pos, escala), p, t, out);
                let kind = if pressed {
                    PointerKind::Down
                } else {
                    PointerKind::Up
                };
                self.clica(kind, b, p, t, out);
            }
            PenEvent::Leave { .. } => {
                while let Some(b) = self.em_baixo.last().copied() {
                    self.clica(PointerKind::Up, b, 0.0, t, out);
                }
                out.push(Saida::Saiu);
                *self = Self {
                    relogio: std::mem::take(&mut self.relogio),
                    ..Self::default()
                };
            }
        }
    }

    fn na_mesa(&self) -> bool {
        self.em_baixo.contains(&PointerButton::Primary)
    }

    fn vai(&mut self, pos: [f32; 2], pressure: f32, t: u128, out: &mut Vec<Saida>) {
        if self.pos != Some(pos) {
            self.mexe(pos, pressure, t, out);
        }
    }

    fn mexe(&mut self, pos: [f32; 2], pressure: f32, t: u128, out: &mut Vec<Saida>) {
        self.pos = Some(pos);
        let b = self.premido.unwrap_or(PointerButton::Primary);
        out.push(self.evento(PointerKind::Move, b, pressure, t));
    }

    fn clica(
        &mut self,
        kind: PointerKind,
        b: PointerButton,
        pressure: f32,
        t: u128,
        out: &mut Vec<Saida>,
    ) {
        if kind == PointerKind::Down {
            self.premido = Some(b);
            self.em_baixo.push(b);
        } else {
            self.premido = None;
            self.em_baixo.retain(|x| *x != b);
        }
        out.push(self.evento(kind, b, pressure, t));
    }

    fn evento(&self, kind: PointerKind, button: PointerButton, pressure: f32, t: u128) -> Saida {
        let [x, y] = self.pos.unwrap_or_default();
        Saida::Ponteiro(PointerEvent {
            x,
            y,
            pressure,
            kind,
            source: PointerSource::Pen(self.info),
            button,
            timestamp_ns: t,
        })
    }
}

fn fisico(pos: [f64; 2], escala: f64) -> [f32; 2] {
    pos.map(|v| (v * escala) as f32)
}

#[cfg(test)]
#[path = "ponteiro_tests.rs"]
mod tests;
