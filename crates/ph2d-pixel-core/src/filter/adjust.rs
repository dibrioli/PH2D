//! **Edit ▸ Invert e Edit ▸ Adjustments** — filtros por célula. Trial: `oracle_trial_adjust.json`.

use super::{Filter, Source};

/// **Edit ▸ Invert**: `255 − v` em R, G e B; o alfa fica (Trial: 2 048 células, 0 desvios).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Invert;

impl Filter for Invert {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        let c = src.get(x, y).unwrap_or_default();
        [255 - c[0], 255 - c[1], 255 - c[2], c[3]]
    }
}

/// Uma tabela por canal (R, G e B pela mesma), o alfa fica.
fn lut_at(lut: &[u8; 256], src: &Source, x: i32, y: i32) -> [u8; 4] {
    let c = src.get(x, y).unwrap_or_default();
    [
        lut[usize::from(c[0])],
        lut[usize::from(c[1])],
        lut[usize::from(c[2])],
        c[3],
    ]
}

/// **Brightness/Contrast** (`-100..=100` cada): o contraste em torno de ½ ANTES do brilho
/// multiplicativo, e `⌊y·255,5⌋` — o factor que o Trial obriga (9 LUTs, 0 desvios; `·255` deixa 177,
/// o brilho primeiro deixa 127).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrightnessContrast {
    lut: [u8; 256],
}

impl BrightnessContrast {
    #[must_use]
    pub fn new(brightness: i32, contrast: i32) -> Self {
        let (b, c) = (f64::from(brightness), f64::from(contrast));
        let lut = std::array::from_fn(|v| {
            let y = (v as f64 / 255.0 - 0.5) * (1.0 + c / 100.0) + 0.5;
            ((y * (1.0 + b / 100.0)).clamp(0.0, 1.0) * 255.5).floor() as u8
        });
        Self { lut }
    }
}

impl Filter for BrightnessContrast {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        lut_at(&self.lut, src, x, y)
    }
}

/// **Color Curve**: rectas entre os pontos (com `(0,0)` e `(255,255)` nas pontas) e `⌊y⌋` (Trial:
/// 0 desvios; spline natural, Catmull-Rom e PCHIP não casam).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Curve {
    lut: [u8; 256],
}

impl Curve {
    /// `points` por `x` crescente; um `x` repetido fica com o último.
    #[must_use]
    pub fn new(points: &[(u8, u8)]) -> Self {
        let mut p: Vec<(u8, u8)> = vec![(0, 0)];
        let mut inner: Vec<(u8, u8)> = points.to_vec();
        inner.sort_by_key(|q| q.0);
        for q in inner {
            match p.last_mut() {
                Some(l) if l.0 == q.0 => *l = q,
                _ => p.push(q),
            }
        }
        if p.last().is_some_and(|l| l.0 < 255) {
            p.push((255, 255));
        }
        let lut = std::array::from_fn(|v| {
            let k = p.windows(2).position(|w| v as u8 <= w[1].0).unwrap_or(0);
            let ((x0, y0), (x1, y1)) = (p[k], p[k + 1]);
            let (x0, y0, x1, y1) = (i32::from(x0), i32::from(y0), i32::from(x1), i32::from(y1));
            (y0 + ((y1 - y0) * (v as i32 - x0)).div_euclid(x1 - x0)) as u8
        });
        Self { lut }
    }
}

impl Filter for Curve {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        lut_at(&self.lut, src, x, y)
    }
}

/// Os quatro modelos do diálogo Hue/Saturation: HSL (omissão) e HSV multiplicam a saturação e a
/// luz por `1 + x/100`; os «+» somam `x/100`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HueMode {
    #[default]
    Hsl,
    Hsv,
    HslPlus,
    HsvPlus,
}

/// **Hue/Saturation** (Ctrl+U): matiz `-180..=180` graus, saturação e luz `-100..=100`. A lei do
/// livro (Foley/van Dam) com a luz HSL de `(max+min)/510` e `⌊x·255 + ½⌋` casa o Trial em todo
/// canal que não seja um empate exacto `k + ½`; nos empates o Trial desempata por ruído de vírgula
/// flutuante (±1, não reproduzível por fórmula — `docs/PixelArt/ferramentas/fit_ajustes.py`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HueSaturation {
    pub mode: HueMode,
    pub hue: i32,
    pub saturation: i32,
    pub lightness: i32,
}

impl HueSaturation {
    #[must_use]
    pub fn rgb(&self, c: [u8; 3]) -> [u8; 3] {
        self.scaled(c)
            .map(|v| (v + 0.5).floor().clamp(0.0, 255.0) as u8)
    }

    /// A cor nova em `0..=255` antes de arredondar (os empates `k + ½` vêem-se aqui).
    #[must_use]
    pub fn scaled(&self, c: [u8; 3]) -> [f64; 3] {
        let [r, g, b] = c.map(|v| f64::from(v) / 255.0);
        let (mx8, mn8) = (c.iter().max().copied(), c.iter().min().copied());
        let (mx8, mn8) = (f64::from(mx8.unwrap_or(0)), f64::from(mn8.unwrap_or(0)));
        let (mx, mn) = (mx8 / 255.0, mn8 / 255.0);
        let ch = mx - mn;
        let hsl = matches!(self.mode, HueMode::Hsl | HueMode::HslPlus);
        let mut h = if ch == 0.0 {
            0.0
        } else if mx == r {
            60.0 * ((g - b) / ch).rem_euclid(6.0)
        } else if mx == g {
            60.0 * ((b - r) / ch + 2.0)
        } else {
            60.0 * ((r - g) / ch + 4.0)
        };
        let mut l = if hsl { (mx8 + mn8) / 510.0 } else { mx };
        let mut s = match (ch == 0.0, hsl) {
            (true, _) => 0.0,
            (false, true) => ch / (1.0 - (2.0 * l - 1.0).abs()),
            (false, false) => ch / mx,
        };
        h = (h + f64::from(self.hue)).rem_euclid(360.0);
        let (ds, dl) = (
            f64::from(self.saturation) / 100.0,
            f64::from(self.lightness) / 100.0,
        );
        if matches!(self.mode, HueMode::HslPlus | HueMode::HsvPlus) {
            s += ds;
            l += dl;
        } else {
            s *= 1.0 + ds;
            l *= 1.0 + dl;
        }
        let (s, l) = (s.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
        let (c2, m) = if hsl {
            let c2 = (1.0 - (2.0 * l - 1.0).abs()) * s;
            (c2, l - c2 / 2.0)
        } else {
            (l * s, l - l * s)
        };
        let x = c2 * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let (r1, g1, b1) = match (h / 60.0) as u32 {
            0 => (c2, x, 0.0),
            1 => (x, c2, 0.0),
            2 => (0.0, c2, x),
            3 => (0.0, x, c2),
            4 => (x, 0.0, c2),
            _ => (c2, 0.0, x),
        };
        [r1, g1, b1].map(|v| (v + m) * 255.0)
    }
}

impl Filter for HueSaturation {
    fn at(&self, src: &Source, x: i32, y: i32) -> [u8; 4] {
        let c = src.get(x, y).unwrap_or_default();
        let [r, g, b] = self.rgb([c[0], c[1], c[2]]);
        [r, g, b, c[3]]
    }
}
