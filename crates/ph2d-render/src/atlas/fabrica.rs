//! ⭐ **A arte de fábrica do jogo de plataforma** (`docs/GameEngineFuncional/06_arte_vetorial_plano.md`): o boneco
//! verde de capacete e o cenário do Kenney Pixel Platformer (CC0), redesenhados em vetor plano.
//!
//! Os SVG de `fabrica/` são a FONTE — gerados por `docs/GameEngineFuncional/ferramentas/arte_svg.py` sobre as
//! poses que o Blender deu (`boneco_blender.py`). Cada um rasteriza-se UMA vez por processo, ao tamanho da sua
//! entrada, e grava-se pela chave reservada, como o quadrado branco.

use std::sync::OnceLock;

/// Uma imagem de fábrica: a chave reservada, o SVG, os texels da imagem inteira e a grelha (`[colunas, linhas]`).
///
/// ⭐ Um AZULEJO (`tile`) é UM desenho que o SVG dá e a imagem repete em todas as células da grelha; quem o
/// pinta mostra a célula do MEIO ([`Self::middle_frame`]). Assim o vizinho de cada borda no atlas é o próprio
/// azulejo, e o mip (que mistura a borda de uma região com o que está ao lado dela no atlas) não traz um fio
/// transparente a cada repetição.
#[derive(Clone, Copy, Debug)]
pub struct FactoryArt {
    pub key: u32,
    pub svg: &'static [u8],
    pub px: [u32; 2],
    pub grid: [u32; 2],
    pub tile: bool,
}

impl FactoryArt {
    /// O lado de uma célula, em texels.
    #[must_use]
    pub const fn cell_px(&self) -> [u32; 2] {
        [self.px[0] / self.grid[0], self.px[1] / self.grid[1]]
    }

    /// A célula do meio da grelha (a de um azulejo que se pinta).
    #[must_use]
    pub const fn middle_frame(&self) -> u32 {
        self.grid[1] / 2 * self.grid[0] + self.grid[0] / 2
    }
}

/// O azulejo do chão (relva por cima, terra): 1 × 1 m, repete na horizontal.
pub const GROUND_TILE_KEY: u32 = u32::MAX - 3;
/// O azulejo da plataforma: 1 × 0,5 m, repete na horizontal.
pub const PLATFORM_TILE_KEY: u32 = u32::MAX - 4;
/// A terra sem relva (paredes): 1 × 1 m, repete nos dois eixos.
pub const DIRT_TILE_KEY: u32 = u32::MAX - 5;
/// A moeda a girar: 6 × 1 células.
pub const COIN_SHEET_KEY: u32 = u32::MAX - 6;
/// A bandeira a ondular: 4 × 1 células de 1 × 2 m.
pub const FLAG_SHEET_KEY: u32 = u32::MAX - 7;
/// A faixa de nuvens do fundo (plano 11): 8 × 2 m, repete na horizontal.
pub const CLOUDS_TILE_KEY: u32 = u32::MAX - 8;
/// Os montes ao longe do fundo (plano 11): 8 × 6 m, repete na horizontal.
pub const HILLS_TILE_KEY: u32 = u32::MAX - 9;

const _: () = assert!(HILLS_TILE_KEY > super::FIRST_IMPORT_KEY);

/// A folha do boneco (8 × 4; ver [`super::hero_sheet`]).
pub const HERO_ART: FactoryArt = FactoryArt {
    key: super::HERO_SHEET_KEY,
    svg: include_bytes!("fabrica/boneco.svg"),
    px: [
        super::HERO_SHEET_CELL_PX * super::HERO_SHEET_COLS,
        super::HERO_SHEET_CELL_PX * super::HERO_SHEET_ROWS,
    ],
    grid: [super::HERO_SHEET_COLS, super::HERO_SHEET_ROWS],
    tile: false,
};
/// O chão e a plataforma repetem só na horizontal: as cópias vão lado a lado (por cima e por baixo a borda é
/// mesmo a do bloco).
pub const GROUND_TILE: FactoryArt = FactoryArt {
    key: GROUND_TILE_KEY,
    svg: include_bytes!("fabrica/chao.svg"),
    px: [768, 256],
    grid: [3, 1],
    tile: true,
};
pub const PLATFORM_TILE: FactoryArt = FactoryArt {
    key: PLATFORM_TILE_KEY,
    svg: include_bytes!("fabrica/plataforma.svg"),
    px: [768, 128],
    grid: [3, 1],
    tile: true,
};
pub const DIRT_TILE: FactoryArt = FactoryArt {
    key: DIRT_TILE_KEY,
    svg: include_bytes!("fabrica/terra.svg"),
    px: [768, 768],
    grid: [3, 3],
    tile: true,
};
pub const COIN_SHEET: FactoryArt = FactoryArt {
    key: COIN_SHEET_KEY,
    svg: include_bytes!("fabrica/moeda.svg"),
    px: [768, 128],
    grid: [6, 1],
    tile: false,
};
pub const FLAG_SHEET: FactoryArt = FactoryArt {
    key: FLAG_SHEET_KEY,
    svg: include_bytes!("fabrica/bandeira.svg"),
    px: [512, 256],
    grid: [4, 1],
    tile: false,
};
/// Texels por metro dos azulejos do fundo (fica atrás e anda devagar: metade do chão chega).
pub const BACKGROUND_TEXELS_PER_M: u32 = 128;
const B: u32 = BACKGROUND_TEXELS_PER_M;
pub const CLOUDS_TILE: FactoryArt = FactoryArt {
    key: CLOUDS_TILE_KEY,
    svg: include_bytes!("fabrica/nuvens.svg"),
    px: [3 * 8 * B, 2 * B],
    grid: [3, 1],
    tile: true,
};
pub const HILLS_TILE: FactoryArt = FactoryArt {
    key: HILLS_TILE_KEY,
    svg: include_bytes!("fabrica/montes.svg"),
    px: [3 * 8 * B, 6 * B],
    grid: [3, 1],
    tile: true,
};

/// Toda a arte de fábrica, na ordem em que entra no atlas. ⚠️ Arte nova entra no FIM (gate
/// `as_artes_antigas_sao_as_de_antes`).
pub const FACTORY_ART: [FactoryArt; 8] = [
    HERO_ART,
    GROUND_TILE,
    PLATFORM_TILE,
    DIRT_TILE,
    COIN_SHEET,
    FLAG_SHEET,
    CLOUDS_TILE,
    HILLS_TILE,
];

/// Milissegundos por célula da moeda e da bandeira (laços uniformes).
pub const COIN_FRAME_MS: u32 = 80;
pub const FLAG_FRAME_MS: u32 = 150;

/// ⭐ Os texels RGBA8 (sRGB, alfa reto) de `art` — rasterizados à 1.ª pergunta e guardados.
///
/// # Panics
/// Se o SVG embebido não abre: é arte do repositório, e o gate `toda_a_arte_rasteriza` prova-a.
#[must_use]
pub fn factory_pixels(art: &FactoryArt) -> &'static [u8] {
    static CACHE: [OnceLock<Vec<u8>>; FACTORY_ART.len()] =
        [const { OnceLock::new() }; FACTORY_ART.len()];
    let i = FACTORY_ART
        .iter()
        .position(|a| a.key == art.key)
        .expect("arte de fábrica registada");
    CACHE[i].get_or_init(|| {
        let tree = ph2d_imageio_svg::raster::parse(art.svg).expect("SVG de fábrica");
        if !art.tile {
            return ph2d_imageio_svg::raster::rasterize(&tree, art.px).expect("tamanho não nulo");
        }
        let [cw, ch] = art.cell_px().map(|v| v as usize);
        let um =
            ph2d_imageio_svg::raster::rasterize(&tree, art.cell_px()).expect("tamanho não nulo");
        let w = art.px[0] as usize;
        let mut px = vec![0; w * art.px[1] as usize * 4];
        for (y, linha) in px.chunks_exact_mut(w * 4).enumerate() {
            let fonte = &um[(y % ch) * cw * 4..(y % ch + 1) * cw * 4];
            for copia in linha.chunks_exact_mut(cw * 4) {
                copia.copy_from_slice(fonte);
            }
        }
        px
    })
}

#[cfg(test)]
#[path = "fabrica_tests.rs"]
mod tests;
