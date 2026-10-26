//! **Tocar a animação**: a máquina de passos medida no Aseprite Trial (`tg_*.ase`, vídeo a 50 q/s,
//! fixtura `oracle_trial_frames.json`). O relógio é do host: [`Play::advance`] recebe os ms.
//!
//! Fora de uma tag anda para a frente e volta ao 0 no fim. Ao entrar numa tag começa a 1.ª passada
//! (Forward/Ping-pong para a frente desde `from`; Reverse/Ping-pong Reverse para trás desde `to`).
//! Uma passada acaba na ponta dela; no Ping-pong a seguinte vai no outro sentido sem repetir a
//! ponta, nos outros recomeça do início. Com `repeat` N, ao fim de N passadas segue para `to + 1`
//! (o 0 depois do último); `repeat` 0 fica na tag para sempre. Começar dentro de uma tag conta
//! como estar na 1.ª passada nesse frame (no fim dela, a passada já acabou).

use crate::sprite::{Direction, Sprite, Tag};

/// As opções do tocar (Aseprite: *Play Once*, *Play All*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlayOptions {
    /// Pára no fim em vez de voltar ao início (e uma tag sem repetição conta como uma passada).
    pub once: bool,
    /// Ignora as tags: a linha do tempo inteira, para a frente.
    pub all: bool,
}

/// O estado do tocar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Play {
    /// A posição do frame mostrado.
    pub index: usize,
    /// Os ms já passados neste frame.
    pub acc_ms: u32,
    /// Ainda a tocar (com `once`, pára no fim).
    pub playing: bool,
    tag: Option<usize>,
    forward: bool,
    passes: u32,
    opt: PlayOptions,
}

fn first_forward(d: Direction) -> bool {
    matches!(d, Direction::Forward | Direction::PingPong)
}

/// A tag que governa a posição `i`: a primeira da lista que a contém.
#[must_use]
pub fn tag_at(tags: &[Tag], i: usize) -> Option<usize> {
    tags.iter()
        .position(|t| t.from as usize <= i && i <= t.to as usize)
}

/// A duração com o piso de 1 ms (um frame de 0 conta como 1, a lei do MVP).
fn dur(s: &Sprite, i: usize) -> u32 {
    s.frames().get(i).map_or(1, |f| f.duration_ms.max(1))
}

impl Play {
    /// Começa no frame da posição `index`.
    #[must_use]
    pub fn start(s: &Sprite, index: usize, opt: PlayOptions) -> Self {
        let mut p = Self {
            index: index.min(s.frames().len().saturating_sub(1)),
            acc_ms: 0,
            playing: true,
            tag: None,
            forward: true,
            passes: 0,
            opt,
        };
        if !opt.all
            && let Some(t) = tag_at(s.tags(), p.index)
        {
            p.tag = Some(t);
            p.forward = first_forward(s.tags()[t].direction);
        }
        p
    }

    /// Anda `dt_ms` no relógio; devolve se o frame mudou. A fronteira é do frame seguinte.
    pub fn advance(&mut self, s: &Sprite, dt_ms: u32) -> bool {
        let before = self.index;
        self.acc_ms = self.acc_ms.saturating_add(dt_ms);
        while self.playing && self.acc_ms >= dur(s, self.index) {
            self.acc_ms -= dur(s, self.index);
            self.step(s);
        }
        self.index != before
    }

    /// O frame seguinte.
    pub fn step(&mut self, s: &Sprite) {
        let n = s.frames().len();
        if n == 0 {
            return;
        }
        let tags = s.tags();
        if let Some(ti) = self.tag {
            let t = &tags[ti];
            let (from, to) = (t.from as usize, t.to as usize);
            let end = if self.forward { to } else { from };
            if self.index != end {
                self.index = if self.forward { self.index + 1 } else { self.index - 1 };
                return;
            }
            self.passes += 1;
            let repeat = if t.repeat == 0 && self.opt.once { 1 } else { u32::from(t.repeat) };
            if repeat != 0 && self.passes >= repeat {
                self.tag = None;
                if to + 1 >= n && self.opt.once {
                    self.playing = false;
                    return;
                }
                self.enter(s, if to + 1 >= n { 0 } else { to + 1 });
                return;
            }
            let ping = matches!(t.direction, Direction::PingPong | Direction::PingPongReverse);
            if ping {
                self.forward = !self.forward;
                self.index = if self.forward {
                    (self.index + 1).min(to)
                } else {
                    self.index.saturating_sub(1).max(from)
                };
            } else {
                self.index = if self.forward { from } else { to };
            }
            return;
        }
        if self.index + 1 >= n {
            if self.opt.once {
                self.playing = false;
                return;
            }
            self.enter(s, 0);
        } else {
            self.enter(s, self.index + 1);
        }
    }

    /// Vai para a posição `i`, entrando na tag dela pelo início da 1.ª passada.
    fn enter(&mut self, s: &Sprite, i: usize) {
        self.index = i;
        if self.opt.all {
            return;
        }
        if let Some(ti) = tag_at(s.tags(), i) {
            let t = &s.tags()[ti];
            self.tag = Some(ti);
            self.passes = 0;
            self.forward = first_forward(t.direction);
            self.index = if self.forward { t.from as usize } else { t.to as usize };
        }
    }
}
