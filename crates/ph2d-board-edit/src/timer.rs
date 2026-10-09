//! **O cronómetro do quadro** (W7, `docs/MiroClone/05_facilitacao.md` §2 F1–F6): estado da SESSÃO —
//! nunca do ficheiro nem do desfazer. Puro: quem chama dá o `Instant` de agora (o ecrã redesenha-se a
//! cada quadro e lê-o no desenho; os testes dão um relógio de mentira).

use std::time::{Duration, Instant};

/// F3 — o tecto: 99:59 (FigJam: *«up to 99 minutes and 59 seconds»*).
pub const TIMER_MAX: Duration = Duration::from_secs(99 * 60 + 59);
/// F2 — a duração de nascença, em minutos (o Miro cala; escolha).
pub const TIMER_DEFAULT_MIN: u32 = 5;
/// F2 — até aqui o `±` anda de 1 em 1 minuto; daqui para cima de [`TIMER_COARSE_STEP`] em
/// [`TIMER_COARSE_STEP`] (comunidade do Miro, c16256: *«over 10 minutes … by 5 minutes»*).
pub const TIMER_FINE_UNTIL: u32 = 10;
pub const TIMER_COARSE_STEP: u32 = 5;
/// F4/F6 — o meio-período do piscar (pausado) e do tremor (no zero).
pub const TIMER_BLINK: Duration = Duration::from_millis(500);

/// Onde o relógio está.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerRun {
    /// Armado, parado: mostra a duração armada.
    Set,
    Running { deadline: Instant },
    /// Pausado em `at` com `left` por correr.
    Paused { left: Duration, at: Instant },
    /// Chegou a zero em `at`.
    Done { at: Instant },
}

/// O cronómetro de um quadro.
#[derive(Clone, Debug)]
pub struct Timer {
    minutes: u32,
    run: TimerRun,
    /// F6 — o altifalante do cartão: a campainha soa no zero.
    pub sound: bool,
    /// F7 — minimizado numa pílula com o resto.
    pub minimized: bool,
    /// F5 — o ■ foi carimbado: o cartão pergunta «Stop timer / Cancel».
    pub confirm_stop: bool,
    rang: bool,
}

impl Default for Timer {
    fn default() -> Self {
        Self {
            minutes: TIMER_DEFAULT_MIN,
            run: TimerRun::Set,
            sound: true,
            minimized: false,
            confirm_stop: false,
            rang: false,
        }
    }
}

impl Timer {
    #[must_use]
    pub fn run(&self) -> TimerRun {
        self.run
    }

    /// A duração armada, em minutos.
    #[must_use]
    pub fn minutes(&self) -> u32 {
        self.minutes
    }

    /// F2 — `+` (`up`) ou `−` na duração armada (só parado): de 1 em 1 até 10, de 5 em 5 acima;
    /// entre 1 e 99. `false` = não mexeu.
    pub fn step(&mut self, up: bool) -> bool {
        if self.run != TimerRun::Set {
            return false;
        }
        let m = self.minutes;
        let next = if up {
            if m < TIMER_FINE_UNTIL {
                m + 1
            } else {
                m + TIMER_COARSE_STEP
            }
        } else if m > TIMER_FINE_UNTIL {
            (m - TIMER_COARSE_STEP).max(TIMER_FINE_UNTIL)
        } else {
            m.saturating_sub(1)
        };
        let max = u32::try_from(TIMER_MAX.as_secs() / 60).unwrap_or(u32::MAX);
        let next = next.clamp(1, max);
        let moved = next != m;
        self.minutes = next;
        moved
    }

    /// ▶ — arranca (armado ou no zero) ou retoma (pausado). Esvazia a pergunta do ■.
    pub fn play(&mut self, now: Instant) {
        self.confirm_stop = false;
        self.run = match self.run {
            TimerRun::Paused { left, .. } => TimerRun::Running {
                deadline: now + left,
            },
            TimerRun::Running { .. } => return,
            TimerRun::Set | TimerRun::Done { .. } => {
                self.rang = false;
                TimerRun::Running {
                    deadline: now + Duration::from_secs(u64::from(self.minutes) * 60),
                }
            }
        };
    }

    /// ⏸ — pausa (só a correr).
    pub fn pause(&mut self, now: Instant) {
        if let TimerRun::Running { deadline } = self.run {
            self.run = TimerRun::Paused {
                left: deadline.saturating_duration_since(now),
                at: now,
            };
        }
    }

    /// F4 — `+1m`/`+5m`: soma a correr ou pausado, com o tecto [`TIMER_MAX`]. Nunca tira tempo.
    pub fn add(&mut self, now: Instant, more: Duration) -> bool {
        let cap = |d: Duration| d.min(TIMER_MAX);
        match &mut self.run {
            TimerRun::Running { deadline } => {
                let left = cap(deadline.saturating_duration_since(now) + more);
                *deadline = now + left;
                true
            }
            TimerRun::Paused { left, .. } => {
                *left = cap(*left + more);
                true
            }
            TimerRun::Set | TimerRun::Done { .. } => false,
        }
    }

    /// F5 — o ■ confirmado: volta à duração armada, parado.
    pub fn stop(&mut self) {
        self.run = TimerRun::Set;
        self.confirm_stop = false;
        self.rang = false;
    }

    /// O que falta (armado: a duração armada; no zero: nada).
    #[must_use]
    pub fn left(&self, now: Instant) -> Duration {
        match self.run {
            TimerRun::Set => Duration::from_secs(u64::from(self.minutes) * 60),
            TimerRun::Running { deadline } => deadline.saturating_duration_since(now),
            TimerRun::Paused { left, .. } => left,
            TimerRun::Done { .. } => Duration::ZERO,
        }
    }

    /// `(minutos, segundos)` do mostrador: o resto ARREDONDADO para cima (arranca em `05:00`, chega a
    /// `00:00` só no zero).
    #[must_use]
    pub fn display(&self, now: Instant) -> (u64, u64) {
        let s = self.left(now).as_millis().div_ceil(1000);
        let s = u64::try_from(s).unwrap_or(u64::MAX);
        (s / 60, s % 60)
    }

    /// F6 — anda o relógio: a correr e sem resto passa a `Done`. `true` UMA vez por chegada ao zero, se
    /// o altifalante está ligado — a campainha que quem chama faz soar.
    pub fn tick(&mut self, now: Instant) -> bool {
        if let TimerRun::Running { deadline } = self.run
            && now >= deadline
        {
            self.run = TimerRun::Done { at: deadline };
        }
        if matches!(self.run, TimerRun::Done { .. }) && !self.rang {
            self.rang = true;
            return self.sound;
        }
        false
    }

    /// F4/F6 — a fase «apagada» do piscar (pausado) e do tremor (no zero): `true` meio período sim,
    /// meio não.
    #[must_use]
    pub fn blink_off(&self, now: Instant) -> bool {
        match self.run {
            TimerRun::Paused { at, .. } | TimerRun::Done { at } => {
                phase(now.saturating_duration_since(at))
            }
            _ => false,
        }
    }
}

fn phase(t: Duration) -> bool {
    (t.as_millis() / TIMER_BLINK.as_millis()) % 2 == 1
}
