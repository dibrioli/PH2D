//! **Tocar a animação**: a máquina de passos medida no Aseprite Trial (P4 `oracle_trial_frames.json`;
//! P5 `oracle_trial_play_p5.json`, `docs/PixelArt/ferramentas/oraculo_timeline.md` «Ronda 3»). O
//! relógio é do host: [`Play::advance`] recebe os ms.
//!
//! **Tags aninhadas** (*Play Subtags & Repetitions*, ligado por omissão): ao começar, a pilha é a
//! cadeia das tags que contêm o quadro (de fora para dentro; entre duas que se sobrepõem, a que
//! começa primeiro), todas a meio da 1.ª passada; a mais interior é a **raiz**. Ao andar, entra-se
//! numa tag FILHA da de cima (contida nela) pelo início da 1.ª passada dela (Forward/Ping-pong desde
//! `from`, Reverse/Ping-pong Reverse desde `to`). Uma passada acaba na ponta; no Ping-pong a
//! seguinte vai no outro sentido sem repetir a ponta, nos outros recomeça do início. `repeat` N =
//! N passadas; `repeat` 0 é «para sempre» só na raiz — noutra tag conta como uma. Acabada, a tag
//! sai e a de baixo continua depois dela; a linha do tempo inteira volta ao 0 depois do último.
//!
//! Sem subtags só a raiz toca, em ciclo, sem contar passadas (fora de tags: a linha inteira).
//! *Play All* ignora as tags. *Play Once* toca UMA passada da raiz desde o início dela, sem as
//! tags de dentro (a raiz é a tag mais interior se ela repete para sempre; senão, a linha inteira,
//! em frente; o Ping-pong vai e volta) e pára no quadro de partida.

use crate::sprite::{Direction, Sprite, Tag};

/// As opções do tocar (Aseprite: *Play Once*, *Play All Frames*, *Play Subtags & Repetitions*).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayOptions {
    pub once: bool,
    pub all: bool,
    pub subtags: bool,
}

impl Default for PlayOptions {
    fn default() -> Self {
        Self {
            once: false,
            all: false,
            subtags: true,
        }
    }
}

/// A velocidade do tocar (*Playback Speed*): a duração de cada quadro divide-se por ela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaySpeed {
    Quarter,
    Half,
    #[default]
    One,
    OneAndHalf,
    Two,
    Three,
}

impl PlaySpeed {
    pub const ALL: [PlaySpeed; 6] = [
        PlaySpeed::Quarter,
        PlaySpeed::Half,
        PlaySpeed::One,
        PlaySpeed::OneAndHalf,
        PlaySpeed::Two,
        PlaySpeed::Three,
    ];

    /// A velocidade em quartos (`0.25×` = 1): o relógio anda `ms × quartos`, o quadro dura
    /// `ms × 4` — conta inteira, exacta.
    #[must_use]
    pub const fn quarters(self) -> u32 {
        match self {
            PlaySpeed::Quarter => 1,
            PlaySpeed::Half => 2,
            PlaySpeed::One => 4,
            PlaySpeed::OneAndHalf => 6,
            PlaySpeed::Two => 8,
            PlaySpeed::Three => 12,
        }
    }
}

/// Uma tag na pilha do tocar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Nivel {
    tag: usize,
    forward: bool,
    passes: u32,
}

/// O estado do tocar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Play {
    /// A posição do frame mostrado.
    pub index: usize,
    /// O relógio já passado neste frame, em quartos de ms.
    acc: u64,
    /// Ainda a tocar (com `once`, pára no quadro de partida).
    pub playing: bool,
    /// O quadro onde o tocar começou (*Rewind on Stop* e o fim do *Play Once* voltam a ele).
    pub start: usize,
    pub speed: PlaySpeed,
    pilha: Vec<Nivel>,
    raiz: Option<usize>,
    /// *Play Once*: a passada `(de, até, para a frente, falta a volta do Ping-pong)`.
    recta: Option<(usize, usize, bool, bool)>,
    opt: PlayOptions,
}

fn first_forward(d: Direction) -> bool {
    matches!(d, Direction::Forward | Direction::PingPong)
}

fn contem(t: &Tag, i: usize) -> bool {
    t.from as usize <= i && i <= t.to as usize
}

/// A tag `a` está DENTRO de `b` (b é a de fora): contida, e com o mesmo trecho a primeira da
/// lista é a de fora.
fn dentro(tags: &[Tag], a: usize, b: usize) -> bool {
    let (x, y) = (&tags[a], &tags[b]);
    a != b && y.from <= x.from && x.to <= y.to && ((x.from, x.to) != (y.from, y.to) || b < a)
}

/// A tag que governa a posição `i` (a raiz se o tocar começasse ali): a mais interior da cadeia.
#[must_use]
pub fn tag_at(tags: &[Tag], i: usize) -> Option<usize> {
    cadeia(tags, i).last().copied()
}

/// A cadeia de tags que contêm `i`, de fora para dentro.
fn cadeia(tags: &[Tag], i: usize) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    while let Some(t) = filha(tags, &out, |t| contem(t, i)) {
        out.push(t);
    }
    out
}

/// A filha da última de `pilha` (vazia = a linha inteira) que passa `ok`: das que não estão dentro
/// de outra candidata, a que começa primeiro.
fn filha(tags: &[Tag], pilha: &[usize], ok: impl Fn(&Tag) -> bool) -> Option<usize> {
    let cand: Vec<usize> = (0..tags.len())
        .filter(|&t| !pilha.contains(&t) && ok(&tags[t]))
        .filter(|&t| pilha.last().is_none_or(|&m| dentro(tags, t, m)))
        .collect();
    cand.iter()
        .copied()
        .filter(|&t| !cand.iter().any(|&o| dentro(tags, t, o)))
        .min_by_key(|&t| (tags[t].from, t))
}

/// A duração com o piso de 1 ms (um frame de 0 conta como 1, a lei do MVP), em quartos de ms.
fn dur(s: &Sprite, i: usize) -> u64 {
    u64::from(s.frames().get(i).map_or(1, |f| f.duration_ms.max(1))) * 4
}

impl Play {
    /// Começa no frame da posição `index`.
    #[must_use]
    pub fn start(s: &Sprite, index: usize, opt: PlayOptions) -> Self {
        let n = s.frames().len();
        let index = index.min(n.saturating_sub(1));
        let mut p = Self {
            index,
            acc: 0,
            playing: n > 0,
            start: index,
            speed: PlaySpeed::One,
            pilha: Vec::new(),
            raiz: None,
            recta: None,
            opt,
        };
        let tags = s.tags();
        let cadeia = if opt.all {
            Vec::new()
        } else {
            cadeia(tags, index)
        };
        if opt.once {
            let raiz = cadeia.last().map(|&t| &tags[t]).filter(|t| t.repeat == 0);
            let (a, b, f, ping) = raiz.map_or((0, n.saturating_sub(1), true, false), |t| {
                let ping = matches!(
                    t.direction,
                    Direction::PingPong | Direction::PingPongReverse
                );
                (
                    t.from as usize,
                    t.to as usize,
                    first_forward(t.direction),
                    ping,
                )
            });
            p.index = if f { a } else { b };
            p.recta = Some((a, b, f, ping));
            return p;
        }
        p.raiz = cadeia.last().copied();
        let ate = if opt.subtags {
            cadeia.len()
        } else {
            cadeia.len().min(1)
        };
        let escolha = if opt.subtags {
            &cadeia[..]
        } else {
            &cadeia[cadeia.len() - ate..]
        };
        p.pilha = escolha
            .iter()
            .map(|&t| Nivel {
                tag: t,
                forward: first_forward(tags[t].direction),
                passes: 0,
            })
            .collect();
        p
    }

    /// Anda `dt_ms` no relógio; devolve se o frame mudou. A fronteira é do frame seguinte.
    pub fn advance(&mut self, s: &Sprite, dt_ms: u32) -> bool {
        let before = self.index;
        self.acc = self
            .acc
            .saturating_add(u64::from(dt_ms) * u64::from(self.speed.quarters()));
        while self.playing && self.acc >= dur(s, self.index) {
            self.acc -= dur(s, self.index);
            self.step(s);
        }
        self.index != before
    }

    /// O frame seguinte.
    pub fn step(&mut self, s: &Sprite) {
        let n = s.frames().len();
        if n == 0 || !self.playing {
            return;
        }
        if let Some((a, b, f, ping)) = self.recta {
            let fim = if f { b } else { a };
            if self.index != fim {
                self.index = if f { self.index + 1 } else { self.index - 1 };
            } else if ping && a < b {
                self.recta = Some((a, b, !f, false));
                self.index = if f { self.index - 1 } else { self.index + 1 };
            } else {
                self.index = self.start;
                self.playing = false;
            }
            return;
        }
        let tags = s.tags();
        loop {
            let Some(&nivel) = self.pilha.last() else {
                self.vai(
                    s,
                    if self.index + 1 >= n {
                        0
                    } else {
                        self.index + 1
                    },
                );
                return;
            };
            let t = &tags[nivel.tag];
            let (from, to) = (t.from as usize, t.to as usize);
            let fim = if nivel.forward { to } else { from };
            if self.index != fim {
                let i = if nivel.forward {
                    self.index + 1
                } else {
                    self.index - 1
                };
                self.vai(s, i);
                return;
            }
            let raiz = self.raiz == Some(nivel.tag);
            let top = self.pilha.len() - 1;
            self.pilha[top].passes += 1;
            let passes = self.pilha[top].passes;
            let reps = match (t.repeat, raiz || !self.opt.subtags) {
                (_, true) if !self.opt.subtags => 0,
                (0, true) => 0,
                (0, false) => 1,
                (r, _) => u32::from(r),
            };
            if reps == 0 || passes < reps {
                if matches!(
                    t.direction,
                    Direction::PingPong | Direction::PingPongReverse
                ) {
                    let f = !nivel.forward;
                    self.pilha[top].forward = f;
                    let i = if f {
                        (self.index + 1).min(to)
                    } else {
                        self.index.saturating_sub(1).max(from)
                    };
                    self.vai(s, i);
                } else {
                    let i = if nivel.forward { from } else { to };
                    self.vai(s, i);
                }
                return;
            }
            self.pilha.pop();
            if raiz {
                self.raiz = None;
            }
            let (lo, hi, f) = match self.pilha.last() {
                Some(m) => (
                    tags[m.tag].from as usize,
                    tags[m.tag].to as usize,
                    m.forward,
                ),
                None => (0, n - 1, true),
            };
            if f && to < hi {
                self.vai(s, to + 1);
                return;
            }
            if !f && from > lo {
                self.vai(s, from - 1);
                return;
            }
            // A tag acabou na ponta da de baixo: é o fim da passada dela também.
            self.index = if f { hi } else { lo };
            if self.pilha.is_empty() {
                self.vai(s, 0);
                return;
            }
        }
    }

    /// Vai para a posição `i`, entrando nas tags filhas que a contêm (com subtags).
    fn vai(&mut self, s: &Sprite, i: usize) {
        self.index = i;
        if self.opt.all || !self.opt.subtags {
            return;
        }
        let tags = s.tags();
        loop {
            let em: Vec<usize> = self.pilha.iter().map(|n| n.tag).collect();
            let Some(t) = filha(tags, &em, |t| contem(t, self.index)) else {
                return;
            };
            let forward = first_forward(tags[t].direction);
            self.pilha.push(Nivel {
                tag: t,
                forward,
                passes: 0,
            });
            self.index = if forward {
                tags[t].from as usize
            } else {
                tags[t].to as usize
            };
        }
    }
}
