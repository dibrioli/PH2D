//! **A mão no traço livre** (P7, o popup Dynamics): a pressão e o relógio de cada evento (a porta
//! da caneta, `ph2d_pen::porta`), a velocidade filtrada e a corda do Stabilizer em px de ECRÃ.
//! As leis vivem em `ph2d_pixel_core::dynamics` e `ph2d_pen_law`; aqui só se liga a mão a elas.

use ph2d_editor_core::pixel_panel::{
    PIXEL_THRESHOLD_FULL, PixelChannel, PixelDynamics, PixelGradientDither, PixelSensor,
    PixelThreshold,
};
use ph2d_pixel_core::dither::Dither;
use ph2d_pixel_core::dynamics::{Channel, Dab, Dynamics, Sense, Sensor, Velocity};

use ph2d_editor_core::pixel_panel::PixelOptions as Options;

/// O comprimento da corda por unidade do slider, em px de ecrã (Trial: 4 → 3, 7 → 6, 32 → 29
/// células de atraso a 400 %; 16 → 128 px).
pub const STRING_PX_PER_UNIT: f32 = 8.0;

/// A mão de um traço: a dinâmica das opções, o filtro e a ponta da corda.
#[derive(Debug, Clone, PartialEq)]
pub struct Mao {
    d: Dynamics,
    vel: Velocity,
    /// A ponta que pinta (px de ecrã) e o comprimento da corda.
    corda: Option<([f32; 2], f32)>,
}

fn sensor(s: PixelSensor) -> Sensor {
    match s {
        PixelSensor::Off => Sensor::Off,
        PixelSensor::Pressure => Sensor::Pressure,
        PixelSensor::Velocity => Sensor::Velocity,
    }
}

fn canal(c: PixelChannel, max: i32, d: &PixelDynamics) -> Channel {
    let t: PixelThreshold = match c.sensor {
        PixelSensor::Velocity => d.velocity,
        _ => d.pressure,
    };
    let f = |v: u16| f64::from(v) / f64::from(PIXEL_THRESHOLD_FULL);
    Channel {
        sensor: sensor(c.sensor),
        lo: f(t.lo),
        hi: f(t.hi),
        min: f64::from(c.min),
        max: f64::from(max),
    }
}

/// A dinâmica do núcleo para as opções: o Max do Size e do Angle é o da barra (Trial).
#[must_use]
pub fn dynamics(o: &Options) -> Dynamics {
    let d = &o.dynamics;
    let size = i32::try_from(o.size).unwrap_or(i32::MAX);
    Dynamics {
        size: canal(d.size, size, d),
        angle: canal(d.angle, o.angle, d),
        gradient: canal(d.gradient, 1, d),
        dither: match d.dither {
            PixelGradientDither::None => Dither::None,
            PixelGradientDither::Bayer2 => Dither::Bayer2,
            PixelGradientDither::Bayer4 => Dither::Bayer4,
            PixelGradientDither::Bayer8 => Dither::Bayer8,
        },
    }
}

impl Mao {
    /// A mão para as opções, premida em `p` (px de ecrã); `None` = sem dinâmica nem corda.
    pub fn of(o: &Options, p: (f32, f32)) -> Option<Self> {
        o.dynamics.active().then(|| Self {
            d: dynamics(o),
            vel: Velocity::default(),
            corda: (o.dynamics.stabilizer > 0).then(|| {
                let len = o.dynamics.stabilizer as f32 * STRING_PX_PER_UNIT;
                ([p.0, p.1], len)
            }),
        })
    }

    /// O Gradient está ligado (o traço precisa do fundo e da matriz).
    pub fn gradient(&self) -> Option<Dither> {
        (self.d.gradient.sensor != Sensor::Off).then_some(self.d.dither)
    }

    /// O carimbo do premir (o filtro ainda a zero).
    pub fn premir(&self, pressure: f32) -> Dab {
        self.d.dab(Sense {
            pressure: f64::from(pressure),
            velocity: self.vel.value(),
        })
    }

    /// Um evento da mão em `p` com a `pressure` e o instante `t_ns`: o ponto que pinta (a ponta da
    /// corda) e o carimbo.
    pub fn ponto(&mut self, p: (f32, f32), pressure: f32, t_ns: u64) -> ((f32, f32), Dab) {
        let v = self.vel.sample([f64::from(p.0), f64::from(p.1)], t_ns);
        let dab = self.d.dab(Sense {
            pressure: f64::from(pressure),
            velocity: v,
        });
        let at = match self.corda.as_mut() {
            Some((ponta, len)) => {
                *ponta = ph2d_pen_law::pulled_string_step(*ponta, [p.0, p.1], *len);
                (ponta[0], ponta[1])
            }
            None => p,
        };
        (at, dab)
    }
}

/// A pressão e o instante do último evento de ponteiro (o rato: pressão 1).
pub fn tinta() -> (f32, u64) {
    let t = ph2d_pen::porta::tinta();
    let ns = u64::try_from(ph2d_pen::porta::evento_ns()).unwrap_or(u64::MAX);
    (t.pressure, ns)
}
