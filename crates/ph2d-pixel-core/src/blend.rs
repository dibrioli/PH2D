//! Os 19 modos de mistura de camada, em inteiros, medidos contra o Aseprite Trial 1.3.18.6
//! (fixtura `oracle_trial_blend.json`, método em `docs/PixelArt/ferramentas/trial_camadas.py`).
//!
//! Lei da composição (medida, 6 144 células por modo e opacidade, exacta): o alfa efectivo da
//! fonte é `mul(Sa, opacidade)`; a cor misturada `B(b, s)` entra por source-over sobre o destino.
//! Os ids são os do formato `.ase` (0 Normal … 18 Divide).

/// Modo de mistura (Aseprite: Layer Properties ▸ Mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    Addition,
    Subtract,
    Divide,
}

impl BlendMode {
    /// Todos, na ordem dos ids do `.ase` (a ordem do menu do Aseprite).
    pub const ALL: [BlendMode; 19] = [
        BlendMode::Normal,
        BlendMode::Multiply,
        BlendMode::Screen,
        BlendMode::Overlay,
        BlendMode::Darken,
        BlendMode::Lighten,
        BlendMode::ColorDodge,
        BlendMode::ColorBurn,
        BlendMode::HardLight,
        BlendMode::SoftLight,
        BlendMode::Difference,
        BlendMode::Exclusion,
        BlendMode::Hue,
        BlendMode::Saturation,
        BlendMode::Color,
        BlendMode::Luminosity,
        BlendMode::Addition,
        BlendMode::Subtract,
        BlendMode::Divide,
    ];

    /// O id do `.ase`.
    #[must_use]
    pub fn code(self) -> u8 {
        Self::ALL.iter().position(|&m| m == self).unwrap_or(0) as u8
    }

    #[must_use]
    pub fn from_code(c: u8) -> Option<Self> {
        Self::ALL.get(usize::from(c)).copied()
    }
}

/// `a · b / 255` arredondado (o produto u8 clássico).
#[must_use]
pub const fn mul(a: u32, b: u32) -> u32 {
    let t = a * b + 0x80;
    ((t >> 8) + t) >> 8
}

fn screen(b: u32, s: u32) -> u32 {
    b + s - mul(b, s)
}

fn hard_light(b: u32, s: u32) -> u32 {
    if s < 128 {
        mul(b, s * 2)
    } else {
        screen(b, s * 2 - 255)
    }
}

fn soft_light(b: u32, s: u32) -> u32 {
    let (bf, sf) = (f64::from(b) / 255.0, f64::from(s) / 255.0);
    let d = if bf <= 0.25 {
        ((16.0 * bf - 12.0) * bf + 4.0) * bf
    } else {
        bf.sqrt()
    };
    let r = if sf <= 0.5 {
        bf - (1.0 - 2.0 * sf) * bf * (1.0 - bf)
    } else {
        bf + (2.0 * sf - 1.0) * (d - bf)
    };
    (r * 255.0 + 0.5) as u32
}

fn separable(mode: BlendMode, b: u32, s: u32) -> u32 {
    match mode {
        BlendMode::Multiply => mul(b, s),
        BlendMode::Screen => screen(b, s),
        BlendMode::Overlay => hard_light(s, b),
        BlendMode::Darken => b.min(s),
        BlendMode::Lighten => b.max(s),
        // As divisões ARREDONDAM (Dodge, Burn e Divide; truncar erra 391, 1 405 e 405 de 6 144).
        BlendMode::ColorDodge => match (b, s) {
            (0, _) => 0,
            (_, 255) => 255,
            _ => ((b * 255 + (255 - s) / 2) / (255 - s)).min(255),
        },
        BlendMode::ColorBurn => match (b, s) {
            (255, _) => 255,
            (_, 0) => 0,
            _ => 255 - (((255 - b) * 255 + s / 2) / s).min(255),
        },
        BlendMode::HardLight => hard_light(b, s),
        BlendMode::SoftLight => soft_light(b, s),
        BlendMode::Difference => b.abs_diff(s),
        BlendMode::Exclusion => b + s - 2 * mul(b, s),
        BlendMode::Addition => (b + s).min(255),
        BlendMode::Subtract => b.saturating_sub(s),
        BlendMode::Divide => {
            if b == 0 {
                0
            } else if b >= s {
                255
            } else {
                (b * 255 + s / 2) / s
            }
        }
        _ => s,
    }
}

// ── não-separáveis (W3C Compositing §9.2), em f64 ─────────────────────────────

type Rgb = [f64; 3];

fn lum(c: Rgb) -> f64 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn clip_color(c: Rgb) -> Rgb {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    let mut c = c;
    if n < 0.0 {
        c = c.map(|v| l + (v - l) * l / (l - n));
    }
    if x > 1.0 {
        c = c.map(|v| l + (v - l) * (1.0 - l) / (x - l));
    }
    c
}

fn set_lum(c: Rgb, l: f64) -> Rgb {
    let d = l - lum(c);
    clip_color(c.map(|v| v + d))
}

fn sat(c: Rgb) -> f64 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

fn set_sat(c: Rgb, s: f64) -> Rgb {
    let mx = c[0].max(c[1]).max(c[2]);
    let mn = c[0].min(c[1]).min(c[2]);
    if mx > mn {
        c.map(|v| (v - mn) * s / (mx - mn))
    } else {
        [0.0; 3]
    }
}

fn non_separable(mode: BlendMode, b: [u8; 3], s: [u8; 3]) -> [u8; 3] {
    let f = |c: [u8; 3]| c.map(|v| f64::from(v) / 255.0);
    let (bf, sf) = (f(b), f(s));
    let r = match mode {
        BlendMode::Hue => set_lum(set_sat(sf, sat(bf)), lum(bf)),
        BlendMode::Saturation => set_lum(set_sat(bf, sat(sf)), lum(bf)),
        BlendMode::Color => set_lum(sf, lum(bf)),
        _ => set_lum(bf, lum(sf)),
    };
    // TRUNCA (medido: arredondar erra 2 119–2 754 de 6 144 por modo).
    r.map(|v| (v * 255.0).clamp(0.0, 255.0) as u8)
}

/// A cor `B(b, s)` do modo, sem alfa.
#[must_use]
pub fn blend_rgb(mode: BlendMode, b: [u8; 3], s: [u8; 3]) -> [u8; 3] {
    match mode {
        BlendMode::Normal => s,
        BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => {
            non_separable(mode, b, s)
        }
        _ => [0, 1, 2].map(|k| separable(mode, u32::from(b[k]), u32::from(s[k])) as u8),
    }
}

/// Mistura a fonte `src` (alfa reto) sobre `dst` com o modo e a opacidade de camada `opacity`.
#[must_use]
pub fn blend(mode: BlendMode, dst: [u8; 4], src: [u8; 4], opacity: u8) -> [u8; 4] {
    let sa = mul(u32::from(src[3]), u32::from(opacity));
    if sa == 0 {
        return dst;
    }
    let da = u32::from(dst[3]);
    if da == 0 {
        return [src[0], src[1], src[2], sa as u8];
    }
    let mixed = blend_rgb(mode, [dst[0], dst[1], dst[2]], [src[0], src[1], src[2]]);
    // Source-over de alfa reto: ra = sa + da − mul(da, sa); c = d + (m − d)·sa/ra (trunca p/ zero).
    let ra = sa + da - mul(da, sa);
    let mut o = [0u8; 4];
    for k in 0..3 {
        let d = i32::from(dst[k]);
        let m = i32::from(mixed[k]);
        o[k] = (d + (m - d) * sa as i32 / ra as i32) as u8;
    }
    o[3] = ra as u8;
    o
}
