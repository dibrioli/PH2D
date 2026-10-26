//! **O pincel**: forma, tamanho e ângulo → os offsets de UM carimbo, relativos à célula âncora.
//! A tabela é AUTORADA (o MVP fixa os redondos 1..10 a olho), não geometria.

use crate::raster::{Cell, line_cells};

/// O maior tamanho do pincel: o do Aseprite 1.3.18 (o slider *Brush Size* vai a 64), que é o
/// produto de referência do dono. O custo não é o tecto: um carimbo de 64 são 4 096 offsets.
pub const MAX_BRUSH: u32 = 64;

/// A forma do pincel (Aseprite: *Brush Type*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrushShape {
    #[default]
    Circle,
    Square,
    /// Um segmento de `size` células no ângulo do pincel.
    Line,
}

impl BrushShape {
    pub const ALL: [Self; 3] = [Self::Circle, Self::Square, Self::Line];
}

/// O pincel: forma, tamanho em células (`1..=MAX_BRUSH`) e ângulo em graus (anti-horário, como no
/// Aseprite; o redondo ignora-o).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Brush {
    pub shape: BrushShape,
    pub size: u32,
    pub angle: i32,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            shape: BrushShape::Circle,
            size: 1,
            angle: 0,
        }
    }
}

/// Folga estética no raio do redondo, em células — ESCOLHIDA A OLHO (MVP): com 0, d=3 sai um
/// quadrado 3×3, d=9 empata e d=10 repete uma linha. Os goldens 1..10 fixam-na.
const CIRCLE_SLACK: f64 = 0.1;

impl Brush {
    /// O tamanho como o carimbo o usa (`1..=MAX_BRUSH`).
    #[must_use]
    pub fn diameter(self) -> u32 {
        self.size.clamp(1, MAX_BRUSH)
    }

    /// **Os offsets do carimbo** relativos à âncora `floor((d-1)/2)`: ímpar centra no cursor, par
    /// cresce para a direita/baixo (a convenção de todo editor).
    #[must_use]
    pub fn offsets(self) -> Vec<Cell> {
        let d = self.diameter() as i32;
        let anchor = (d - 1) >> 1;
        let c = f64::from(d - 1) / 2.0;
        let mut out = Vec::new();
        match self.shape {
            BrushShape::Circle => {
                let r = f64::from(d) / 2.0 - CIRCLE_SLACK;
                for y in 0..d {
                    for x in 0..d {
                        let (ddx, ddy) = (f64::from(x) - c, f64::from(y) - c);
                        if ddx * ddx + ddy * ddy <= r * r {
                            out.push((x - anchor, y - anchor));
                        }
                    }
                }
            }
            BrushShape::Square if self.angle.rem_euclid(90) == 0 => {
                for y in 0..d {
                    for x in 0..d {
                        out.push((x - anchor, y - anchor));
                    }
                }
            }
            BrushShape::Square => {
                // As células cujo CENTRO cai no quadrado de lado d rodado à volta do centro do carimbo.
                let (s, co) = f64::from(self.angle).to_radians().sin_cos();
                let half = f64::from(d) / 2.0 - CIRCLE_SLACK;
                let reach = (half * std::f64::consts::SQRT_2).ceil() as i32 + 1;
                for y in -reach..=reach + 1 {
                    for x in -reach..=reach + 1 {
                        let (px, py) = (f64::from(x + anchor) - c, f64::from(y + anchor) - c);
                        // Ecrã com y para baixo: o anti-horário do artista é −ângulo aqui.
                        let u = px * co - py * s;
                        let v = px * s + py * co;
                        if u.abs() <= half && v.abs() <= half {
                            out.push((x, y));
                        }
                    }
                }
            }
            BrushShape::Line => {
                let (s, co) = f64::from(self.angle).to_radians().sin_cos();
                let h = f64::from(d - 1) / 2.0;
                // Meio → para baixo: num pincel PAR o centro está a meia célula, e a linha a 0°
                // tem de cair na linha da âncora (a 90°, na coluna dela).
                let r = |v: f64| (v - 0.5).ceil() as i32 - anchor;
                let p0 = (r(c - h * co), r(c + h * s));
                let p1 = (r(c + h * co), r(c - h * s));
                line_cells(p0.0, p0.1, p1.0, p1.1, |x, y| out.push((x, y)));
            }
        }
        out
    }
}

/// **splitmix32** (MVP `engine/rng.js`): o sorteio do spray é função pura da semente — o mesmo
/// gesto dá o mesmo spray.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitMix32(u32);

impl SplitMix32 {
    #[must_use]
    pub const fn new(seed: u32) -> Self {
        Self(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(0x9e37_79b9);
        let mut t = self.0 ^ (self.0 >> 15);
        t = t.wrapping_mul(0x85eb_ca6b);
        t ^= t >> 13;
        t = t.wrapping_mul(0xc2b2_ae35);
        t ^ (t >> 16)
    }

    /// Uniforme em `[0, 1)` (2⁻³² é exato em f64).
    pub fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) * 2.328_306_436_538_696_3e-10
    }
}

/// A semente do spray a partir da célula onde começou (o `hash3` do MVP — outra lei que a do
/// `ph2d_motion_kit::hash3`; só o nome coincidia).
#[must_use]
pub fn spray_seed(seed: u32, a: i32, b: i32) -> u32 {
    let mut x = seed ^ (a as u32).wrapping_mul(0x27d4_eb2d) ^ (b as u32).wrapping_mul(0x1656_67b1);
    x ^= x >> 15;
    x = x.wrapping_mul(0x85eb_ca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2_ae35);
    x ^ (x >> 16)
}
