//! O relógio do Play quando há som: o playhead do mixer, nunca o relógio do host.

use ph2d_audio::StreamPlayhead;

/// O instante do Play em ms = onde começou + o que o mixer já tocou da faixa-mestra.
pub struct Relogio {
    t0_ms: u64,
    playhead: StreamPlayhead,
    /// Os ms que a faixa-mestra tem para tocar a partir de `t0`.
    resta_ms: i64,
    /// Os ms inteiros já entregues ao Play.
    entregue: u64,
    ultimo_quadro: u64,
}

impl Relogio {
    pub(crate) fn novo(t0_ms: u64, playhead: StreamPlayhead, resta_ms: i64) -> Self {
        Self {
            t0_ms,
            playhead,
            resta_ms,
            entregue: t0_ms,
            ultimo_quadro: 0,
        }
    }

    pub(crate) fn resta_ms(&self) -> i64 {
        self.resta_ms
    }

    /// O instante do Play em ms, segundo o mixer.
    #[must_use]
    pub fn ms(&self) -> f64 {
        self.t0_ms as f64 + self.playhead.ms()
    }

    /// O som já acabou (a fonte esgotou e o playhead parou): o Play volta ao relógio do host.
    fn acabou(&self, quadro: u64) -> bool {
        self.playhead.source_finished() && quadro == self.ultimo_quadro && quadro > 0
    }

    /// O `dt` do Play: os ms que o mixer andou desde a última leitura (o resto fica para a próxima,
    /// sem deriva). Acabado o som, `host_ms` — o Play continua como sem som.
    pub fn dt(&mut self, host_ms: u32) -> u32 {
        let q = self.playhead.frame();
        if self.acabou(q) {
            self.entregue = self.entregue.saturating_add(u64::from(host_ms));
            return host_ms;
        }
        self.ultimo_quadro = q;
        let agora = self.ms().floor() as u64;
        let dt = agora.saturating_sub(self.entregue);
        self.entregue = self.entregue.max(agora);
        u32::try_from(dt).unwrap_or(u32::MAX)
    }
}

/// O `dt` que o Play anda: o do som quando há [`Relogio`], senão o do host **tal como hoje**.
pub fn dt_do_play(relogio: Option<&mut Relogio>, host_ms: u32) -> u32 {
    relogio.map_or(host_ms, |r| r.dt(host_ms))
}
