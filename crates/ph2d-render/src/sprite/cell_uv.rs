//! **A UV final de uma sprite** — de que rectângulo do atlas a célula se corta, e onde cai o meio-texel. Desceu da
//! shell (`render_loop::sim_extract_emit`, dois sítios de escolha) para que a lei tenha UM dono e um teste que a
//! chama: trocar `region_uv_exact` por `region_uv` sobrevivia a todos os testes (auditoria GEF 29).

use super::{inset_half_texel, region_subrect, sprite_sheet_subrect};
use crate::{AtlasRegion, TextureAtlas};

/// A textura que a sprite amostra.
#[derive(Clone, Copy, Debug)]
pub enum CellSource {
    /// O atlas partilhado de `size_px`, na região empacotada (`None` = chave desconhecida: UV nula).
    Atlas {
        size_px: u32,
        region: Option<AtlasRegion>,
    },
    /// Uma textura própria, amostrada inteira (`[0,0,1,1]`).
    Own,
}

impl CellSource {
    /// A região `key` do `atlas`.
    pub fn atlas(atlas: &TextureAtlas, key: u32) -> Self {
        Self::Atlas {
            size_px: atlas.size_px,
            region: atlas.region(key),
        }
    }
}

/// O que a sprite amostra e como a corta.
#[derive(Clone, Copy, Debug)]
pub struct CellUvInput {
    pub source: CellSource,
    /// A janela em pixels da fonte e as dimensões da fonte; `None` = a fonte inteira.
    pub region: Option<(ph2d_ecs::SpriteRegion, (u32, u32))>,
    /// A grelha da folha; `None` = a imagem inteira (pré-visualização, Painter, Edit).
    pub grid: Option<ph2d_ecs::SpriteGrid>,
    /// [`super::sprite_wraps`] desta instância.
    pub wraps: bool,
}

/// A UV da grelha inteira (`sheet`, a que a folha aberta percorre) e a UV final da instância (`cell`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellUv {
    pub sheet: [f32; 4],
    pub cell: [f32; 4],
}

/// A célula corta-se do rect EXACTO; o meio-texel (`filter_clip` da região, ou uma sprite do atlas que dá a volta e
/// por isso salta o clamp do shader) move só o rect FINAL, igual para todos os quadros.
pub fn sprite_cell_uv(input: CellUvInput) -> CellUv {
    let (base, atlas_ht) = match input.source {
        CellSource::Atlas { size_px, region } => (
            region.map_or([0.0; 4], |r| r.uv_exact(size_px)),
            Some(0.5 / size_px.max(1) as f32),
        ),
        CellSource::Own => ([0.0, 0.0, 1.0, 1.0], None),
    };
    let (sheet, clip) = match input.region {
        Some((region, (sw, sh))) => {
            // O meio-texel no espaço UV da textura AMOSTRADA: o atlas (1 / size_px) ou a própria (1 / w, 1 / h).
            let clip = region.filter_clip.then(|| match atlas_ht {
                Some(ht) => (ht, ht),
                None => (0.5 / sw.max(1) as f32, 0.5 / sh.max(1) as f32),
            });
            (
                region_subrect(base, region.rect, sw as f32, sh as f32, None),
                clip,
            )
        }
        None => (base, None),
    };
    let cell = input.grid.map_or(sheet, |g| {
        sprite_sheet_subrect(sheet, g.hframes, g.vframes, g.frame)
    });
    let clip = clip.or(atlas_ht.filter(|_| input.wraps).map(|ht| (ht, ht)));
    CellUv {
        sheet,
        cell: clip.map_or(cell, |ht| inset_half_texel(cell, ht)),
    }
}

#[cfg(test)]
#[path = "cell_uv_tests.rs"]
mod tests;
