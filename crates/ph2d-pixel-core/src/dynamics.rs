//! **A dinâmica do pincel** (o popup *Dynamics* do Aseprite): Size, Angle e Gradient seguem a
//! Pressure ou a Velocity por UMA rampa entre as duas pegas do *Sensor Threshold*. Lei MEDIDA no
//! Trial 1.3.18.6 (`tests/fixtures/oracle_trial_dynamics_velocity.json`, `round2_*`) e, para a
//! pressão, no Pixelorama 1.2.3 (MIT, rampa linear entre limiares). Puro: o tempo entra como dado.

use crate::dither::Dither;

/// O que move um canal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sensor {
    #[default]
    Off,
    Pressure,
    Velocity,
}

/// A escala da barra do limiar, em px de ECRÃ por evento: a pega na fracção `f` vale
/// `VELOCITY_FULL_PX · f − VELOCITY_ZERO_PX`. Ajuste dos 80 arrastos do Trial com pegas 0,10/0,89,
/// 0,49/0,89 e 0,085/0,29 (`dynamics_oracle`): erro médio 0,15 de tamanho; `66 · f` sem o −1 dá 0,31.
pub const VELOCITY_FULL_PX: f64 = 66.0;
pub const VELOCITY_ZERO_PX: f64 = 1.0;

/// O τ do filtro da velocidade, em ns (grelha 30..70 ms no mesmo ajuste; 45 ms dá 0,16).
pub const VELOCITY_TAU_NS: f64 = 50e6;

/// Um canal: o sensor, as pegas `lo`/`hi` (fracções da barra) e o `min`/`max` do que move.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Channel {
    pub sensor: Sensor,
    pub lo: f64,
    pub hi: f64,
    pub min: f64,
    pub max: f64,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            sensor: Sensor::Off,
            lo: 0.1,
            hi: 0.89,
            min: 1.0,
            max: 1.0,
        }
    }
}

/// O que a mão dá num ponto: a pressão `0..=1` e a velocidade filtrada na escala da barra.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sense {
    pub pressure: f64,
    pub velocity: f64,
}

impl Channel {
    /// A fracção `0..=1` andada entre as pegas; `None` = o canal está desligado.
    #[must_use]
    pub fn ramp(&self, s: Sense) -> Option<f64> {
        let x = match self.sensor {
            Sensor::Off => return None,
            Sensor::Pressure => s.pressure,
            Sensor::Velocity => s.velocity,
        };
        let span = self.hi - self.lo;
        Some(if span <= 0.0 {
            f64::from(u8::from(x >= self.hi))
        } else {
            ((x - self.lo) / span).clamp(0.0, 1.0)
        })
    }

    /// `min + (max − min) · rampa`; `None` = desligado (fica o valor da barra).
    #[must_use]
    pub fn map(&self, s: Sense) -> Option<f64> {
        self.ramp(s).map(|t| self.min + (self.max - self.min) * t)
    }
}

/// Os três canais e a matriz do Gradient.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dynamics {
    pub size: Channel,
    pub angle: Channel,
    pub gradient: Channel,
    pub dither: Dither,
}

/// O resto de vírgula flutuante que o `floor` do tamanho perdoa (`1 + 8 · 0,5` dá 4,999…).
const FLOOR_SLACK: f64 = 1e-9;

/// O carimbo de um ponto do traço.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Dab {
    pub size: Option<u32>,
    pub angle: Option<i32>,
    /// A fracção do caminho fundo → frente (`None` = a cor do traço).
    pub gradient: Option<f64>,
}

impl Dynamics {
    /// O carimbo para o que a mão dá: o tamanho por `floor` (Trial), o ângulo arredondado.
    #[must_use]
    pub fn dab(&self, s: Sense) -> Dab {
        Dab {
            size: self
                .size
                .map(s)
                .map(|v| ((v + FLOOR_SLACK).floor() as u32).max(1)),
            angle: self.angle.map(s).map(|v| v.round() as i32),
            gradient: self.gradient.ramp(s),
        }
    }

    /// Algum canal liga a velocidade (só então o traço a filtra).
    #[must_use]
    pub fn uses(&self, sensor: Sensor) -> bool {
        [self.size, self.angle, self.gradient]
            .iter()
            .any(|c| c.sensor == sensor)
    }
}

/// **O filtro da velocidade**: o passo é o deslocamento em px de ecrã desde o evento anterior (não
/// se divide por `dt`), filtrado no VECTOR por `v += (1 − e^(−dt/τ)) · (passo − v)` a cada evento.
/// Parado não corre (sem evento não há passo); inverter o sentido recomeça do zero. O filtro nasce
/// a zero a cada premir e o 1.º evento só marca a partida (o Trial não conta o passo desde o premir:
/// com passos de 64 px o traço começa em 1).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity {
    v: [f64; 2],
    last: Option<([f64; 2], u64)>,
}

impl Velocity {
    /// Um evento em `pos` (px de ecrã) no instante `t_ns`; devolve a velocidade na escala da barra.
    pub fn sample(&mut self, pos: [f64; 2], t_ns: u64) -> f64 {
        let Some((p, t)) = self.last else {
            self.last = Some((pos, t_ns));
            return self.value();
        };
        let dt = t_ns.saturating_sub(t) as f64;
        let a = 1.0 - (-dt / VELOCITY_TAU_NS).exp();
        for i in 0..2 {
            self.v[i] += a * ((pos[i] - p[i]) - self.v[i]);
        }
        self.last = Some((pos, t_ns));
        self.value()
    }

    /// A velocidade filtrada agora, na escala da barra.
    #[must_use]
    pub fn value(&self) -> f64 {
        (self.v[0].hypot(self.v[1]) + VELOCITY_ZERO_PX) / VELOCITY_FULL_PX
    }
}
