//! **O popup Dynamics** do Pixel Art (o do Aseprite 1.3.18, foto `docs/PixelArt/aseprite/55_dinamica.png`)
//! no vocabulário do chrome: inteiros, para o painel e o `Eq` das opções; a família traduz para
//! `ph2d_pixel_core::dynamics`.

/// O sensor de uma linha da grelha Size/Angle/Gradient × Pressure/Velocity (uma coluna por linha).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelSensor {
    #[default]
    Off,
    Pressure,
    Velocity,
}

/// Uma linha: o sensor e o `min` (o `max` do Size e do Angle é o da barra do pincel, como no Trial).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PixelChannel {
    pub sensor: PixelSensor,
    pub min: i32,
}

/// As duas pegas do *Sensor Threshold* de um sensor, em milésimos da barra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelThreshold {
    pub lo: u16,
    pub hi: u16,
}

/// O fim da barra das pegas, em milésimos.
pub const PIXEL_THRESHOLD_FULL: u16 = 1000;

impl Default for PixelThreshold {
    /// Perto das pontas, como no Trial (lidas 0,10 e 0,89).
    fn default() -> Self {
        Self { lo: 100, hi: 890 }
    }
}

/// A matriz do Gradient (o dropdown dele, legenda «BG > FG»).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelGradientDither {
    #[default]
    None,
    Bayer2,
    Bayer4,
    Bayer8,
}

impl PixelGradientDither {
    pub const ALL: [Self; 4] = [Self::None, Self::Bayer2, Self::Bayer4, Self::Bayer8];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::None => "panel.pixel.dynamics.dither_none",
            Self::Bayer2 => "panel.pixel.dynamics.dither_bayer2",
            Self::Bayer4 => "panel.pixel.dynamics.dither_bayer4",
            Self::Bayer8 => "panel.pixel.dynamics.dither_bayer8",
        }
    }
}

/// P13: a métrica do *Color Mode ▸ Indexed* (Exact RGB = a de sempre, byte a byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelColorMetric {
    #[default]
    ExactRgb,
    Perceptual,
    KeepRamps,
}

impl PixelColorMetric {
    pub const ALL: [Self; 3] = [Self::ExactRgb, Self::Perceptual, Self::KeepRamps];

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::ExactRgb => "pixel.convert.metric.rgb",
            Self::Perceptual => "pixel.convert.metric.perceptual",
            Self::KeepRamps => "pixel.convert.metric.keep_ramps",
        }
    }
}

/// O popup inteiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelDynamics {
    /// A corda, em unidades do slider (`0` = desligada; ligar põe [`PIXEL_STABILIZER_ON`]).
    pub stabilizer: u32,
    pub size: PixelChannel,
    pub angle: PixelChannel,
    pub gradient: PixelChannel,
    pub dither: PixelGradientDither,
    pub pressure: PixelThreshold,
    pub velocity: PixelThreshold,
}

/// O valor do Stabilizer ao ligar a caixa (Trial).
pub const PIXEL_STABILIZER_ON: u32 = 16;

impl Default for PixelDynamics {
    fn default() -> Self {
        Self {
            stabilizer: 0,
            size: PixelChannel {
                sensor: PixelSensor::Off,
                min: 1,
            },
            angle: PixelChannel::default(),
            gradient: PixelChannel::default(),
            dither: PixelGradientDither::None,
            pressure: PixelThreshold::default(),
            velocity: PixelThreshold::default(),
        }
    }
}

impl PixelDynamics {
    /// Algum sensor ligado ou a corda: o traço precisa da pressão, do relógio e do ecrã.
    #[must_use]
    pub fn active(&self) -> bool {
        self.stabilizer > 0
            || [self.size, self.angle, self.gradient]
                .iter()
                .any(|c| c.sensor != PixelSensor::Off)
    }
}
