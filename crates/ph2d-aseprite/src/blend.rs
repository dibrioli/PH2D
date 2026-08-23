//! **Os 19 modos de mistura do Aseprite**, em RGBA8 de alfa reto.
//!
//! ⚠️ **As fórmulas são as da compositing spec do W3C** (que é o que o Aseprite implementa, e que
//! o Photoshop documenta) — escritas a partir dela, não traduzidas de fonte GPLv2. As separáveis
//! agem canal a canal; as quatro não-separáveis (`Hue`/`Saturation`/`Color`/`Luminosity`)
//! trabalham sobre a cor inteira, e é por isso que vivem numa função à parte.
//!
//! # ⛔ Não há oráculo, e é por isso que os gates são IDENTIDADES
//!
//! Comparar com um render do Aseprite exigiria o Aseprite. O que se pode afirmar sem ele é
//! **álgebra**: multiplicar por branco não muda nada, `screen` com preto não muda nada, `darken`
//! com branco não muda nada, `difference` consigo mesmo é preto. Cada modo tem pelo menos uma
//! dessas, e um erro de canal ou de sinal parte-as. *Uma identidade verificável vale mais que uma
//! comparação com um número que ninguém sabe de onde veio.*

/// O modo de mistura de uma camada. Os discriminantes **são** os do ficheiro (spec §Layer Chunk).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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
    /// O número que o ficheiro traz. Desconhecido ⇒ `None`, e quem chama transforma isso numa nota
    /// para o artista em vez de escolher um modo por ele.
    #[must_use]
    pub fn from_file(v: u16) -> Option<Self> {
        Some(match v {
            0 => Self::Normal,
            1 => Self::Multiply,
            2 => Self::Screen,
            3 => Self::Overlay,
            4 => Self::Darken,
            5 => Self::Lighten,
            6 => Self::ColorDodge,
            7 => Self::ColorBurn,
            8 => Self::HardLight,
            9 => Self::SoftLight,
            10 => Self::Difference,
            11 => Self::Exclusion,
            12 => Self::Hue,
            13 => Self::Saturation,
            14 => Self::Color,
            15 => Self::Luminosity,
            16 => Self::Addition,
            17 => Self::Subtract,
            18 => Self::Divide,
            _ => return None,
        })
    }

    /// O nome que aparece na UI do Aseprite — é por ele que o artista reconhece a camada numa nota.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Multiply => "Multiply",
            Self::Screen => "Screen",
            Self::Overlay => "Overlay",
            Self::Darken => "Darken",
            Self::Lighten => "Lighten",
            Self::ColorDodge => "Color Dodge",
            Self::ColorBurn => "Color Burn",
            Self::HardLight => "Hard Light",
            Self::SoftLight => "Soft Light",
            Self::Difference => "Difference",
            Self::Exclusion => "Exclusion",
            Self::Hue => "Hue",
            Self::Saturation => "Saturation",
            Self::Color => "Color",
            Self::Luminosity => "Luminosity",
            Self::Addition => "Addition",
            Self::Subtract => "Subtract",
            Self::Divide => "Divide",
        }
    }

    fn is_separable(self) -> bool {
        !matches!(self, Self::Hue | Self::Saturation | Self::Color | Self::Luminosity)
    }
}

/// `a·b/255` com arredondamento — a multiplicação de 8 bits do formato.
fn mul(a: u8, b: u8) -> u8 {
    let t = u32::from(a) * u32::from(b) + 0x80;
    (((t >> 8) + t) >> 8) as u8
}

fn div255(v: u32) -> u8 {
    let t = v + 0x80;
    (((t >> 8) + t) >> 8) as u8
}

fn separable(mode: BlendMode, b: u8, s: u8) -> u8 {
    let (bi, si) = (u32::from(b), u32::from(s));
    match mode {
        BlendMode::Normal => s,
        BlendMode::Multiply => mul(b, s),
        BlendMode::Screen => (bi + si - u32::from(mul(b, s))) as u8,
        // ⚠️ `overlay(b, s) = hard_light(s, b)` — os argumentos TROCAM, e é a única diferença
        // entre os dois. Escrevê-los como duas fórmulas independentes é como uma delas fica
        // errada sem que ninguém repare.
        BlendMode::Overlay => separable(BlendMode::HardLight, s, b),
        BlendMode::Darken => b.min(s),
        BlendMode::Lighten => b.max(s),
        BlendMode::ColorDodge => {
            if b == 0 {
                0
            } else if s == 255 {
                255
            } else {
                (bi * 255 / (255 - si)).min(255) as u8
            }
        }
        BlendMode::ColorBurn => {
            if b == 255 {
                255
            } else if s == 0 {
                0
            } else {
                255 - ((255 - bi) * 255 / si).min(255) as u8
            }
        }
        BlendMode::HardLight => {
            if s < 128 {
                mul(b, (si * 2) as u8)
            } else {
                let d = (si * 2 - 255) as u8;
                (u32::from(b) + u32::from(d) - u32::from(mul(b, d))) as u8
            }
        }
        BlendMode::SoftLight => soft_light(bi, si),
        BlendMode::Difference => bi.abs_diff(si) as u8,
        BlendMode::Exclusion => (bi + si - 2 * u32::from(mul(b, s))) as u8,
        BlendMode::Addition => (bi + si).min(255) as u8,
        BlendMode::Subtract => bi.saturating_sub(si) as u8,
        BlendMode::Divide => {
            if b == 0 {
                0
            } else if bi >= si {
                255
            } else {
                (bi * 255 / si).min(255) as u8
            }
        }
        // As não-separáveis nunca chegam aqui — a `blend` desvia-as antes.
        BlendMode::Hue | BlendMode::Saturation | BlendMode::Color | BlendMode::Luminosity => s,
    }
}

/// A fórmula do W3C, em ponto flutuante porque ela tem uma raiz quadrada.
fn soft_light(b: u32, s: u32) -> u8 {
    let (b, s) = (b as f64 / 255.0, s as f64 / 255.0);
    let d = if b <= 0.25 {
        ((16.0 * b - 12.0) * b + 4.0) * b
    } else {
        b.sqrt()
    };
    let r = if s <= 0.5 {
        b - (1.0 - 2.0 * s) * b * (1.0 - b)
    } else {
        b + (2.0 * s - 1.0) * (d - b)
    };
    (r.clamp(0.0, 1.0) * 255.0).round() as u8
}

// ─── As quatro não-separáveis (W3C §blending, "non-separable blend modes") ───

fn lum(c: [f64; 3]) -> f64 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

fn clip_color(mut c: [f64; 3]) -> [f64; 3] {
    let l = lum(c);
    let n = c[0].min(c[1]).min(c[2]);
    let x = c[0].max(c[1]).max(c[2]);
    if n < 0.0 {
        for v in &mut c {
            *v = l + (*v - l) * l / (l - n).max(f64::EPSILON);
        }
    }
    if x > 1.0 {
        for v in &mut c {
            *v = l + (*v - l) * (1.0 - l) / (x - l).max(f64::EPSILON);
        }
    }
    c
}

fn set_lum(c: [f64; 3], l: f64) -> [f64; 3] {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

fn sat(c: [f64; 3]) -> f64 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

/// Põe a saturação `s` mantendo a ORDEM dos canais — o mínimo vai a 0, o máximo a `s`, e o do meio
/// fica onde estava proporcionalmente.
fn set_sat(c: [f64; 3], s: f64) -> [f64; 3] {
    let mut idx = [0usize, 1, 2];
    idx.sort_by(|&a, &b| c[a].partial_cmp(&c[b]).unwrap_or(std::cmp::Ordering::Equal));
    let (lo, mid, hi) = (idx[0], idx[1], idx[2]);
    let mut out = [0.0; 3];
    if c[hi] > c[lo] {
        out[mid] = (c[mid] - c[lo]) * s / (c[hi] - c[lo]);
        out[hi] = s;
    }
    out[lo] = 0.0;
    out
}

fn non_separable(mode: BlendMode, b: [u8; 3], s: [u8; 3]) -> [u8; 3] {
    let f = |v: [u8; 3]| [f64::from(v[0]) / 255.0, f64::from(v[1]) / 255.0, f64::from(v[2]) / 255.0];
    let (bf, sf) = (f(b), f(s));
    let r = match mode {
        BlendMode::Hue => set_lum(set_sat(sf, sat(bf)), lum(bf)),
        BlendMode::Saturation => set_lum(set_sat(bf, sat(sf)), lum(bf)),
        BlendMode::Color => set_lum(sf, lum(bf)),
        BlendMode::Luminosity => set_lum(bf, lum(sf)),
        _ => sf,
    };
    [
        (r[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (r[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (r[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

/// **Mistura um pixel sobre outro.** `back` é o que já está lá, `src` o que a camada traz,
/// `opacity` é o produto da opacidade da camada com a do cel.
///
/// ⚠️ **A mistura escolhe a COR; a composição é sempre `source-over`.** É por isso que os 19 modos
/// cabem numa função só: eles trocam a fórmula da cor e nada mais. Foi assim que o formato foi
/// desenhado, e escrever cada modo com a sua própria composição seria dezanove sítios onde o alfa
/// pode estar errado.
#[must_use]
pub fn blend(mode: BlendMode, back: [u8; 4], src: [u8; 4], opacity: u8) -> [u8; 4] {
    let sa = mul(src[3], opacity);
    if sa == 0 {
        return back;
    }
    if back[3] == 0 {
        return [src[0], src[1], src[2], sa];
    }
    // A cor misturada, ainda sem alfa: ela só tem significado onde o fundo é opaco, e a
    // interpolação abaixo é o que a atenua onde ele não é (a fórmula do W3C, `Cs = (1-ab)·Cs +
    // ab·B(Cb, Cs)`).
    let bl = if mode.is_separable() {
        [
            separable(mode, back[0], src[0]),
            separable(mode, back[1], src[1]),
            separable(mode, back[2], src[2]),
        ]
    } else {
        non_separable(mode, [back[0], back[1], back[2]], [src[0], src[1], src[2]])
    };
    let ba = u32::from(back[3]);
    let mixed = [
        div255(u32::from(src[0]) * (255 - ba) + u32::from(bl[0]) * ba),
        div255(u32::from(src[1]) * (255 - ba) + u32::from(bl[1]) * ba),
        div255(u32::from(src[2]) * (255 - ba) + u32::from(bl[2]) * ba),
    ];
    // `source-over` em alfa reto.
    let ra = u32::from(sa) + ba - u32::from(mul(back[3], sa));
    if ra == 0 {
        return [0, 0, 0, 0];
    }
    let ch = |b: u8, s: u8| -> u8 {
        let bb = u32::from(b) * ba;
        let ss = u32::from(s) * u32::from(sa);
        // `Cr = (Cs·as + Cb·ab·(1-as)) / ar`
        let num = ss + bb * (255 - u32::from(sa)) / 255;
        (num / ra).min(255) as u8
    };
    [
        ch(back[0], mixed[0]),
        ch(back[1], mixed[1]),
        ch(back[2], mixed[2]),
        ra.min(255) as u8,
    ]
}

#[cfg(test)]
#[path = "blend_tests.rs"]
mod tests;
