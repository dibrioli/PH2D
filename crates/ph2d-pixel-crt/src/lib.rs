//! `ph2d-pixel-crt` — o filtro CRT do Pixel Art (P13 bloco V, `docs/PixelArt/P13_plano.md` §4/§11/§12).
//!
//! Duas implementações da MESMA lei: [`crt_cpu`] (gabarito determinista e caminho do export) e o
//! passe WGSL [`gpu::CrtGpu`] (a vista: prévia e janela do jogo). O filtro é da VISTA: nunca grava
//! no documento (dono 09/11).
//!
//! # A lei
//!
//! Entrada: `rgba` de `w × h`, ampliada por `escala` inteira ⇒ saída `W × H = w·escala × h·escala`.
//! Para cada pixel de saída `(x, y)`:
//!
//! 1. **Curvatura barril, em ponto fixo Q14** (inteiros: a CPU e a placa dão o MESMO texel).
//!    `cx = 2x+1−W`, `u = cx·2¹⁴ / W` (divisão truncada para zero; `v` igual com `y`, `H`);
//!    `r² = (u²+v²) >> 14`; `f = 2¹⁴ + (k·r² >> 12)` com `k = round(curvatura·2¹²)`;
//!    `du = (u·f) >> 14` (deslocamento aritmético). Fora de `[−2¹⁴, 2¹⁴)` ⇒ **preto opaco**.
//!    Senão `sx = ((du + 2¹⁴)·W) >> 15`, e o texel de origem é `sx / escala` (nearest).
//!    Com `k = 0` o mapa é a identidade para `W, H < 2¹⁴` (o erro de truncatura é `< W/2¹⁵ < ½`).
//! 2. **Cor**, por canal `c ∈ {R,G,B}`, em níveis `0..255` (`s` = texel de origem, `e`/`d` os
//!    vizinhos horizontais na origem, presos à borda):
//!    `g = brilho · (y ímpar ? 1−scanline : 1) · (c == x mod 3 ? 1 : 1−mascara)`,
//!    `out = clamp(floor(s·g + bloom·(e+s+d)/3 + ½), 0, 255)`. Alfa = o da origem.
//!    Scanline = a linha ÍMPAR do ecrã escurece; máscara = aperture grille R,G,B por coluna.
//!
//! Neutro ([`CrtParams::default`]) ⇒ a saída é a ampliação nearest, byte a byte.

pub mod gpu;

/// Bits da fracção do ponto fixo da curvatura. É também o tecto da lei: `W, H < 2^Q` (a
/// truncatura da identidade e o `i32` da placa — ver o doc do módulo).
pub const Q: u32 = 14;
/// Lado máximo da saída: o limite do ponto fixo Q14 (`(du+2¹⁴)·W < 2³¹` e identidade exacta);
/// coincide com o `max_texture_dimension_2d` 16384 das placas de secretária.
pub const LADO_MAXIMO: u32 = 1 << Q;
/// Curvatura máxima: com `k ≤ 1` o produto `u·f` cabe em `i32` (`2¹⁴·3·2¹⁴ < 2³¹`).
pub const CURVATURA_MAXIMA: f32 = 1.0;

/// Parâmetros da CRT — dado da lei (a UI que os edita é a onda seguinte).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CrtParams {
    /// Quanto escurece a linha ímpar do ecrã, `0..=1`.
    pub scanline: f32,
    /// Quanto a máscara de fósforo apaga os dois canais que não são o da coluna, `0..=1`.
    pub mascara: f32,
    /// Curvatura barril, `0..=CURVATURA_MAXIMA`.
    pub curvatura: f32,
    /// Ganho multiplicativo (compensa o que a scanline e a máscara tiram); neutro = 1.
    pub brilho: f32,
    /// Brilho espalhado: a média horizontal de 3 texels somada por cima, `0..`.
    pub bloom: f32,
}

impl Default for CrtParams {
    /// Desligada: a saída é a ampliação nearest.
    fn default() -> Self {
        Self {
            scanline: 0.0,
            mascara: 0.0,
            curvatura: 0.0,
            brilho: 1.0,
            bloom: 0.0,
        }
    }
}

impl CrtParams {
    /// `k` da lei em Q12, já preso a `[0, CURVATURA_MAXIMA]`.
    #[must_use]
    pub fn k_q12(&self) -> i32 {
        #[allow(clippy::cast_possible_truncation)]
        let k = (self.curvatura.clamp(0.0, CURVATURA_MAXIMA) * 4096.0).round() as i32;
        k
    }
}

/// O mapa da curvatura num eixo: `Some(coordenada de saída amostrada)` ou `None` (fora ⇒ preto).
/// Exposto para os gates de propriedade.
#[must_use]
pub fn curva_eixo(x: u32, lado: u32, r2: i32, k: i32) -> Option<u32> {
    let um = 1i32 << Q;
    let u = cx_q(x, lado);
    let f = um + ((k * r2) >> 12);
    let du = (u * f) >> Q;
    if du < -um || du >= um {
        return None;
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    let s = (((du + um) * lado as i32) >> (Q + 1)) as u32;
    Some(s)
}

/// `u` da lei (Q14) para a coluna `x` de um lado `lado`.
#[must_use]
pub fn cx_q(x: u32, lado: u32) -> i32 {
    #[allow(clippy::cast_possible_wrap)]
    let (x, lado) = (x as i32, lado as i32);
    ((2 * x + 1 - lado) << Q) / lado
}

/// `r²` da lei (Q14) no pixel `(x, y)` de uma saída `w × h`.
#[must_use]
pub fn r2_q(x: u32, y: u32, w: u32, h: u32) -> i32 {
    let (u, v) = (cx_q(x, w), cx_q(y, h));
    #[allow(clippy::cast_possible_wrap)]
    let s = ((u * u) as u32 + (v * v) as u32) >> Q;
    #[allow(clippy::cast_possible_wrap)]
    let s = s as i32;
    s
}

/// O gabarito: a CRT sobre `rgba` (`w × h`, RGBA8 directo) ampliada por `escala`.
///
/// # Panics
/// Se `rgba` não tem `w·h·4` bytes, `escala == 0`, ou a saída passa [`LADO_MAXIMO`].
#[must_use]
pub fn crt_cpu(rgba: &[u8], w: u32, h: u32, escala: u32, p: &CrtParams) -> Vec<u8> {
    assert_eq!(rgba.len(), (w * h * 4) as usize, "rgba não tem w·h·4 bytes");
    assert!(escala > 0, "escala 0");
    let (ow, oh) = (w * escala, h * escala);
    assert!(
        ow < LADO_MAXIMO && oh < LADO_MAXIMO,
        "saída passa o tecto Q14"
    );
    let k = p.k_q12();
    let mut out = vec![0u8; (ow * oh * 4) as usize];
    for y in 0..oh {
        for x in 0..ow {
            let o = ((y * ow + x) * 4) as usize;
            let r2 = r2_q(x, y, ow, oh);
            let (Some(sx), Some(sy)) = (curva_eixo(x, ow, r2, k), curva_eixo(y, oh, r2, k)) else {
                out[o..o + 4].copy_from_slice(&[0, 0, 0, 255]);
                continue;
            };
            let (tx, ty) = (sx / escala, sy / escala);
            let texel = |xx: u32, c: usize| f32::from(rgba[((ty * w + xx) * 4) as usize + c]);
            let (e, d) = (tx.saturating_sub(1), (tx + 1).min(w - 1));
            let linha = if y & 1 == 1 { 1.0 - p.scanline } else { 1.0 };
            for c in 0..3 {
                let mascara = if c as u32 == x % 3 {
                    1.0
                } else {
                    1.0 - p.mascara
                };
                let g = p.brilho * linha * mascara;
                let s = texel(tx, c);
                let v = s * g + p.bloom * (texel(e, c) + s + texel(d, c)) / 3.0;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let b = (v + 0.5).floor().clamp(0.0, 255.0) as u8;
                out[o + c] = b;
            }
            out[o + 3] = rgba[((ty * w + tx) * 4) as usize + 3];
        }
    }
    out
}

/// A ampliação nearest pura — a referência do gate da CRT desligada.
#[must_use]
pub fn amplia_nearest(rgba: &[u8], w: u32, h: u32, escala: u32) -> Vec<u8> {
    let ow = w * escala;
    let mut out = Vec::with_capacity(rgba.len() * (escala * escala) as usize);
    for y in 0..h * escala {
        for x in 0..ow {
            let i = (((y / escala) * w + x / escala) * 4) as usize;
            out.extend_from_slice(&rgba[i..i + 4]);
        }
    }
    out
}

#[cfg(test)]
mod testes;
