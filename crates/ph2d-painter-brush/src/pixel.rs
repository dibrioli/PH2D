//! ⭐ **A ponta de PÍXEL** (L18, CAP-16 — `docs/Painter/Pinceis/03_MOTORES.md` §4): o carimbo BINÁRIO alinhado à
//! grelha de píxeis — o lápis do Aseprite / o Pencil do Photoshop. Cada píxel ou é da ponta ou não é (sem AA,
//! sem queda): `hardness 1` não basta, porque o centro do dab cai entre píxeis e a cápsula, a cache bilinear e o
//! AA do filme pintam fracções.
//!
//! A lei, em três partes:
//! 1. **O diâmetro é inteiro** ([`diametro`]) e o centro é uma ÂNCORA da grelha ([`ancora`]): diâmetro ímpar no
//!    centro de um píxel, par no canto — a ponta de 1 px é um píxel, a de 2 px um bloco 2×2, a de 3 px 3×3.
//! 2. **A pegada é a do pincel** (tamanho, achatamento, ângulo, quina — [`crate::FootprintDeform::falloff_t`]) e um
//!    píxel é dela quando o CENTRO dele cai dentro ([`cobre`]). O Shape e o Grain não a recortam (o lápis de píxel
//!    não tem grão; a ponta de imagem binária é a L19 com «Pixel»).
//! 3. **O caminho liga-se** de âncora em âncora pela recta de Bresenham ([`bresenham`]) e, no modo
//!    [`PixelTip::Perfect`] com 1 px, a QUINA de cada L sai ([`e_quina`]: o «pixel-perfect» do Aseprite) — a
//!    linha de 1 px fica sem degraus duplos.

use crate::footprint::FootprintDeform;

/// O modo da ponta de píxel (o campo [`crate::BrushSpec::pixel`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PixelTip {
    /// A ponta de sempre (com AA).
    #[default]
    Off,
    /// Binária, alinhada à grelha, o caminho ligado por Bresenham.
    Aligned,
    /// [`Self::Aligned`] + a quina de cada L tirada na linha de 1 px («pixel-perfect»).
    Perfect,
}

impl PixelTip {
    /// Quantos modos (a faixa da linha do painel `0..COUNT`).
    pub const COUNT: u8 = 3;

    /// O índice estável (preset e painel).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Aligned => 1,
            Self::Perfect => 2,
        }
    }

    /// O inverso de [`Self::to_u8`]; fora da faixa = [`Self::Off`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Aligned,
            2 => Self::Perfect,
            _ => Self::Off,
        }
    }

    /// A ponta de píxel está ligada.
    #[must_use]
    pub fn on(self) -> bool {
        self != Self::Off
    }
}

/// O diâmetro inteiro (≥ 1 px) de um dab de raio `radius`.
#[must_use]
pub fn diametro(radius: f32) -> i32 {
    ((2.0 * radius).round() as i32).max(1)
}

/// A âncora de um centro na grelha do diâmetro `d`: ímpar → o índice do PÍXEL que o contém, par → o CANTO mais
/// perto.
#[must_use]
pub fn ancora(center: [f32; 2], d: i32) -> [i32; 2] {
    if d % 2 == 1 {
        [center[0].floor() as i32, center[1].floor() as i32]
    } else {
        [center[0].round() as i32, center[1].round() as i32]
    }
}

/// O centro, em píxeis da tela, da âncora `a` de diâmetro `d` (o inverso de [`ancora`]).
#[must_use]
pub fn centro(a: [i32; 2], d: i32) -> [f32; 2] {
    if d % 2 == 1 {
        [a[0] as f32 + 0.5, a[1] as f32 + 0.5]
    } else {
        [a[0] as f32, a[1] as f32]
    }
}

/// A recta de Bresenham de `a` (exclusive) até `b` (inclusive), tomando um ponto a cada `passo` (o último sempre).
pub fn bresenham(a: [i32; 2], b: [i32; 2], passo: i32, out: &mut Vec<[i32; 2]>) {
    let (dx, dy) = ((b[0] - a[0]).abs(), -(b[1] - a[1]).abs());
    let (sx, sy) = ((b[0] - a[0]).signum(), (b[1] - a[1]).signum());
    let (mut x, mut y, mut err) = (a[0], a[1], dx + dy);
    let mut k = 0;
    while [x, y] != b {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
        k += 1;
        if k % passo.max(1) == 0 || [x, y] == b {
            out.push([x, y]);
        }
    }
}

/// ⭐ **A quina do L** (o «pixel-perfect» do Aseprite): `p1` toca `p0` e `p2` pelos lados e `p0`, `p2` são
/// vizinhos na DIAGONAL — o degrau duplo que a linha de 1 px não deve ter.
#[must_use]
pub fn e_quina(p0: [i32; 2], p1: [i32; 2], p2: [i32; 2]) -> bool {
    let lado = |a: [i32; 2], b: [i32; 2]| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() == 1;
    lado(p0, p1) && lado(p1, p2) && (p0[0] - p2[0]).abs() == 1 && (p0[1] - p2[1]).abs() == 1
}

/// Chama `cada(px, py, t)` para cada píxel da tela `w × h` cujo CENTRO cai na pegada `fp` do dab de diâmetro `d`
/// ancorado em `centro` (`t` = a distância normalizada da pegada, `< 1`).
pub fn cobre(
    centro: [f32; 2],
    d: i32,
    fp: FootprintDeform,
    w: u32,
    h: u32,
    mut cada: impl FnMut(i64, i64, f32),
) {
    let r = d as f32 * 0.5;
    let inv = 1.0 / r;
    let [ex, ey] = fp.extent();
    let (rx, ry) = (r * ex + 1.0, r * ey + 1.0);
    let x0 = (centro[0] - rx).floor().max(0.0) as i64;
    let y0 = (centro[1] - ry).floor().max(0.0) as i64;
    let x1 = ((centro[0] + rx).ceil() as i64).min(i64::from(w));
    let y1 = ((centro[1] + ry).ceil() as i64).min(i64::from(h));
    for py in y0..y1 {
        let v = (py as f32 + 0.5 - centro[1]) * inv;
        for px in x0..x1 {
            let u = (px as f32 + 0.5 - centro[0]) * inv;
            let t = fp.falloff_t(u, v);
            if t < 1.0 {
                cada(px, py, t);
            }
        }
    }
}

/// ⭐ **Carimba UM dab de píxel** em `buf` (RGBA8 direito, `w × h`): cada píxel da pegada recebe a cor do dab no
/// modo do pincel (e o modo sobre o vazio, L7) com a cobertura TODA do dab (`coverage · flow`) — sem AA, sem
/// queda. Com a rampa de cor da Shape (`ramp`, a «Falloff Color») a COR de cada píxel é a da rampa no perfil da
/// queda dele e o alfa da rampa decide binário (`≥ ½`): o píxel continua inteiro. Com `mask` (o tecto do traço,
/// Accumulate desligado) um píxel já pintado neste traço não volta a receber tinta. Devolve a caixa tocada.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn carimba(
    buf: &mut [u8],
    w: u32,
    h: u32,
    dab: &crate::Dab,
    spec: &crate::BrushSpec,
    preserve_alpha: bool,
    mut mask: Option<&mut [u8]>,
    ramp: Option<&crate::RampPaint<'_>>,
) -> Option<crate::DirtyRect> {
    if buf.len() < (w as usize) * (h as usize) * 4 {
        return None;
    }
    let a = (dab.coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    if a <= 0.0 {
        return None;
    }
    let d = diametro(dab.radius_px);
    let c = centro(ancora(dab.center, d), d);
    let fp = spec.dab_footprint_of(dab);
    let mix = spec.effective_pigment_mix();
    let (mut lo, mut hi) = ([i64::MAX; 2], [i64::MIN; 2]);
    let neutra = crate::texture::GrainLaw::NEUTRA;
    cobre(c, d, fp, w, h, |px, py, t| {
        let cor = match ramp {
            Some(r) => {
                let x = crate::ramp_alpha::ramp_texel(Some(r), spec.falloff_weight(t), None, None, 0.0, 0.0, &neutra);
                if x.g * x.stamp_alpha < 0.5 {
                    return;
                }
                x.color.unwrap_or(dab.color)
            }
            None => dab.color,
        };
        let p = (py as usize) * (w as usize) + px as usize;
        if let Some(m) = mask.as_deref_mut() {
            if m[p] != 0 {
                return;
            }
            m[p] = 255;
        }
        let i = p * 4;
        let prev = [0, 1, 2, 3].map(|k| f32::from(buf[i + k]) / 255.0);
        let a = if preserve_alpha { a * prev[3] } else { a };
        let out =
            crate::blend_vazio::blend_over_pigment_modo(spec.blend, prev, cor, a, mix, spec.blend_sobre_vazio);
        for k in 0..4 {
            buf[i + k] = crate::dab::encode(out[k]);
        }
        lo = [lo[0].min(px), lo[1].min(py)];
        hi = [hi[0].max(px), hi[1].max(py)];
    });
    (lo[0] <= hi[0]).then(|| crate::DirtyRect {
        x: lo[0] as u32,
        y: lo[1] as u32,
        w: (hi[0] - lo[0] + 1) as u32,
        h: (hi[1] - lo[1] + 1) as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O diâmetro 1/2/3/4/5 cobre 1 · 4 · 9 · 12 · 21 píxeis (o disco da grelha), sempre o mesmo onde quer
    /// que o centro caia dentro da célula.
    #[test]
    fn a_ponta_de_pixel_e_o_disco_da_grelha() {
        for (d, n) in [(1, 1), (2, 4), (3, 9), (4, 12), (5, 21)] {
            for c in [[10.0, 10.0], [10.3, 10.7], [10.49, 9.51]] {
                let a = ancora(c, d);
                let mut k = 0;
                cobre(centro(a, d), d, FootprintDeform::identity(), 40, 40, |_, _, _| k += 1);
                assert_eq!(k, n, "d {d} em {c:?}");
            }
        }
    }

    /// Bresenham liga em passos de um píxel (8-conexo) e acaba no destino.
    #[test]
    fn bresenham_e_oito_conexo() {
        let mut v = Vec::new();
        bresenham([0, 0], [7, 3], 1, &mut v);
        assert_eq!(*v.last().unwrap(), [7, 3]);
        let mut p = [0, 0];
        for q in v {
            assert!((q[0] - p[0]).abs() <= 1 && (q[1] - p[1]).abs() <= 1);
            p = q;
        }
    }

    #[test]
    fn a_quina_e_o_l() {
        assert!(e_quina([0, 0], [1, 0], [1, 1]));
        assert!(!e_quina([0, 0], [1, 0], [2, 0]));
        assert!(!e_quina([0, 0], [1, 1], [2, 1]));
    }
}
