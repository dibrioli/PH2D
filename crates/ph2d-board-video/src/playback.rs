//! W19 — **o estado do tocador**, lido pela API oficial do tocador do YouTube: a página nossa ouve as
//! mensagens do `iframe` (`infoDelivery` ~4 por segundo, `onStateChange` na hora) e passa-as ao PH2D pelo
//! `Runtime.addBinding`. Medido (`youtube_motor/saidas/w19b`): `mute`/`setVolume`/`seekTo` aparecem no estado
//! em 4–10 ms, `pauseVideo` em ~630 ms — por isso a ordem dada MANDA no que se mostra até o tocador a
//! confirmar ([`Playback::want`]), e os nossos controles respondem no quadro seguinte.

use std::time::{Duration, Instant};

use serde_json::Value;

/// Quanto tempo uma ordem de tocar/pausar manda no que se mostra sem o tocador a confirmar.
const WANT_FOR: Duration = Duration::from_millis(1500);

/// A fase do tocador (os códigos da API: -1, 0, 1, 2, 3, 5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Unstarted,
    Ended,
    Playing,
    Paused,
    Buffering,
    Cued,
}

impl Phase {
    fn of(code: i64) -> Option<Self> {
        Some(match code {
            -1 => Self::Unstarted,
            0 => Self::Ended,
            1 => Self::Playing,
            2 => Self::Paused,
            3 => Self::Buffering,
            5 => Self::Cued,
            _ => return None,
        })
    }

    /// A tocar ou a encher (o botão mostra «pausa»).
    #[must_use]
    pub fn running(self) -> bool {
        matches!(self, Self::Playing | Self::Buffering)
    }
}

/// O estado do tocador num instante.
#[derive(Clone, Debug, PartialEq)]
pub struct Playback {
    pub phase: Phase,
    /// O tempo (s) lido em [`Self::read_at`].
    pub time: f64,
    pub duration: f64,
    /// 0..=100.
    pub volume: u8,
    pub muted: bool,
    pub rate: f64,
    pub read_at: Option<Instant>,
    /// A última mensagem que veio do TOCADOR (as ordens escrevem o estado na hora, para o botão; isto não).
    pub heard_at: Option<Instant>,
    /// A fase que a pessoa pediu (e quando), até o tocador a confirmar.
    pub want: Option<(Phase, Instant)>,
    /// W21 — o tocador já chegou a tocar, pausado, no fim ou à espera de play (`Cued`): acabou o ARRANQUE.
    /// Antes disso o vídeo vai tocar sozinho (`autoplay=1`) e um clique na página pausá-lo-ia mal começasse
    /// (medido: `a_click_while_the_player_starts_never_stops_its_autoplay`).
    pub started: bool,
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            phase: Phase::Unstarted,
            time: 0.0,
            duration: 0.0,
            volume: 100,
            muted: false,
            rate: 1.0,
            read_at: None,
            heard_at: None,
            want: None,
            started: false,
        }
    }
}

impl Playback {
    /// Uma mensagem do tocador (o texto JSON que o `postMessage` dele leva).
    pub fn apply(&mut self, msg: &Value, now: Instant) {
        let info = &msg["info"];
        self.heard_at = Some(now);
        match msg["event"].as_str() {
            Some("onStateChange") => {
                if let Some(p) = info.as_i64().and_then(Phase::of) {
                    self.set_phase(p);
                }
            }
            Some("infoDelivery" | "initialDelivery") => {
                if let Some(p) = info["playerState"].as_i64().and_then(Phase::of) {
                    self.set_phase(p);
                }
                if let Some(t) = info["currentTime"].as_f64() {
                    self.time = t;
                    self.read_at = Some(now);
                }
                if let Some(d) = info["duration"].as_f64() {
                    self.duration = d;
                }
                if let Some(v) = info["volume"].as_u64() {
                    self.volume = v.min(100) as u8;
                }
                if let Some(m) = info["muted"].as_bool() {
                    self.muted = m;
                }
                if let Some(r) = info["playbackRate"].as_f64() {
                    self.rate = r;
                }
            }
            _ => {}
        }
    }

    fn set_phase(&mut self, p: Phase) {
        self.phase = p;
        self.started |= matches!(
            p,
            Phase::Playing | Phase::Paused | Phase::Ended | Phase::Cued
        );
        if self.want.is_some_and(|(w, _)| w.running() == p.running()) {
            self.want = None;
        }
    }

    /// A fase que se mostra: a pedida, enquanto o tocador não a confirma (e por pouco tempo).
    #[must_use]
    pub fn shown(&self, now: Instant) -> Phase {
        match self.want {
            Some((w, at)) if now.duration_since(at) < WANT_FOR => w,
            _ => self.phase,
        }
    }

    /// O tempo agora (s): o lido mais o que passou, se está a tocar.
    #[must_use]
    pub fn now(&self, now: Instant) -> f64 {
        let ran = match (self.shown(now), self.read_at) {
            (Phase::Playing, Some(at)) => now.duration_since(at).as_secs_f64() * self.rate,
            _ => 0.0,
        };
        let t = self.time + ran;
        if self.duration > 0.0 {
            t.min(self.duration)
        } else {
            t
        }
    }
}
