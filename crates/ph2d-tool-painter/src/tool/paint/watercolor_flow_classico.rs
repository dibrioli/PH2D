//! **O Classic do Flow com escala e direção** — filho de [`super`] (tecto de LOC): a lei com que o
//! Flow Size e o Flow Angle transformam as coordenadas do ruído do Classic, e a pré-visualização dele na
//! faixa partilhada das texturas (dono, 2026-10-02: «Classic não é uma textura procedural como as
//! outras? então deve ter os ajustes e o preview como as outras»).

use super::super::watercolor_noise::{NoiseTile, warp_offset};
use ph2d_painter_brush::texture::angle_basis;
use ph2d_painter_brush::{TEX_TILE_BASE_PX, TextureKind, TextureSettings};

/// **O Classic com Flow Size e Flow Angle** — o ruído de valor nas coordenadas `R(−θ)·p·Size`, a MESMA
/// lei com que uma textura amostra o canvas (`sample_tiled_rot`: `R(−θ)·(p/256)·Size`), em px em vez de
/// ladrilhos. No neutro (Size `1`, Angle `0`) não existe, e o Classic é o [`warp_offset`] verbatim.
#[derive(Clone, Copy)]
pub(super) struct Classico {
    s: f32,
    rot: [f32; 2],
    tile: NoiseTile,
}

impl Classico {
    pub(super) fn de(f: &TextureSettings, tile: NoiseTile) -> Option<Self> {
        (f.kind == TextureKind::None && (f.size[0] != 1.0 || f.angle_deg != 0)).then(|| Self {
            s: f.size[0],
            rot: angle_basis(f.angle_deg),
            tile: tile.escalado(f.size[0]),
        })
    }

    #[inline]
    pub(super) fn le(&self, x: f32, y: f32) -> (f32, f32) {
        let [c, s] = self.rot;
        warp_offset(
            (x * c + y * s) * self.s,
            (-x * s + y * c) * self.s,
            self.tile,
        )
    }
}

/// **A pré-visualização do Classic** — a faixa partilhada das texturas (`render_texture_preview`: ~3
/// ladrilhos de 256 px na largura, o Angle a rodar em volta do centro, o Size a multiplicar) com o
/// Classic no lugar do padrão: o valor do ruído de duas oitavas na coordenada de textura `q·256` px.
/// Cinza `½ + x/2` (o canal X, o que a borda desloca). O painel passa-a à faixa no lugar do padrão.
pub fn render_classic_flow_preview(size: f32, angle_deg: u16, out: &mut [u8], w: u32, h: u32) {
    if w == 0 || h == 0 || out.len() < (w * h * 4) as usize {
        return;
    }
    let step = 3.0 / w as f32;
    let [ca, sa] = ph2d_painter_brush::texture::rotate_by_degrees(angle_deg);
    let centro = 1.5;
    for py in 0..h {
        let bv0 = (py as f32 + 0.5) * step - centro;
        for px in 0..w {
            let bu0 = (px as f32 + 0.5) * step - centro;
            let bu = bu0 * ca - bv0 * sa + centro;
            let bv = bu0 * sa + bv0 * ca + centro;
            let q = TEX_TILE_BASE_PX * size;
            let x = warp_offset(bu * q, bv * q, NoiseTile::NONE).0;
            let g = ((0.5 + 0.5 * x).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            let i = ((py * w + px) * 4) as usize;
            out[i..i + 4].copy_from_slice(&[g, g, g, 255]);
        }
    }
}
