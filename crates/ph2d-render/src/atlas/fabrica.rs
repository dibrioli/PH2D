//! ⭐ **A arte de fábrica do jogo de plataforma** (`docs/GameEngineFuncional/06_arte_vetorial_plano.md`): o boneco
//! verde de capacete e o cenário do Kenney Pixel Platformer (CC0), redesenhados em vetor plano.
//!
//! Os SVG de `fabrica/` são a FONTE — gerados por `docs/GameEngineFuncional/ferramentas/arte_svg.py` sobre as
//! poses que o Blender deu (`boneco_blender.py`). Cada um rasteriza-se UMA vez por processo, ao tamanho da sua
//! entrada, e grava-se pela chave reservada, como o quadrado branco.

use std::sync::OnceLock;

/// Uma imagem de fábrica: a chave reservada, o SVG, os texels no atlas e a grelha (`[colunas, linhas]`).
///
/// ⭐ Um AZULEJO (`tile`) é UMA imagem de um desenho ([`Self::image_px`], [`Self::frames`] = 1 × 1); só o ATLAS
/// a guarda envolvida por cópias de si mesma (`grid` = as cópias, `px` = a tira, a imagem é a do meio —
/// [`super::TextureAtlas::insert_wrapped`], plano GEF 13 D1). Assim o vizinho de cada borda no atlas é o próprio
/// azulejo, e o mip não traz um fio transparente a cada repetição — sem o objecto saber das cópias.
#[derive(Clone, Copy, Debug)]
pub struct FactoryArt {
    pub key: u32,
    /// O nome que a biblioteca mostra (chave `ph2d-i18n`).
    pub name: &'static str,
    pub svg: &'static [u8],
    pub px: [u32; 2],
    pub grid: [u32; 2],
    pub tile: bool,
    /// Texels de uma célula por metro do mundo — o tamanho NATIVO da imagem (plano 12, lei 1).
    pub texels_per_m: u32,
}

impl FactoryArt {
    /// O lado de uma célula, em texels.
    #[must_use]
    pub const fn cell_px(&self) -> [u32; 2] {
        [self.px[0] / self.grid[0], self.px[1] / self.grid[1]]
    }

    /// A célula do meio da grelha: num azulejo, a cópia que É a imagem.
    #[must_use]
    pub const fn middle_frame(&self) -> u32 {
        self.grid[1] / 2 * self.grid[0] + self.grid[0] / 2
    }

    /// Os texels da IMAGEM que a biblioteca mostra: um azulejo é uma célula; uma folha, a folha inteira.
    #[must_use]
    pub const fn image_px(&self) -> [u32; 2] {
        if self.tile { self.cell_px() } else { self.px }
    }

    /// O corte da IMAGEM em quadros (`[colunas, linhas]`): um azulejo é um quadro só.
    #[must_use]
    pub const fn frames(&self) -> [u32; 2] {
        if self.tile { [1, 1] } else { self.grid }
    }

    /// O lado (m) de uma célula no tamanho nativo.
    #[must_use]
    pub fn native_m(&self) -> [f32; 2] {
        self.cell_px().map(|v| v as f32 / self.texels_per_m as f32)
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
    name: "asset.factory.hero",
    svg: include_bytes!("fabrica/boneco.svg"),
    px: [
        super::HERO_SHEET_CELL_PX * super::HERO_SHEET_COLS,
        super::HERO_SHEET_CELL_PX * super::HERO_SHEET_ROWS,
    ],
    grid: [super::HERO_SHEET_COLS, super::HERO_SHEET_ROWS],
    tile: false,
    texels_per_m: super::HERO_SHEET_CELL_PX,
};
/// O chão e a plataforma repetem só na horizontal: as cópias vão lado a lado (por cima e por baixo a borda é
/// mesmo a do bloco).
pub const GROUND_TILE: FactoryArt = FactoryArt {
    key: GROUND_TILE_KEY,
    name: "asset.factory.ground",
    svg: include_bytes!("fabrica/chao.svg"),
    px: [768, 256],
    grid: [3, 1],
    tile: true,
    texels_per_m: 256,
};
pub const PLATFORM_TILE: FactoryArt = FactoryArt {
    key: PLATFORM_TILE_KEY,
    name: "asset.factory.platform",
    svg: include_bytes!("fabrica/plataforma.svg"),
    px: [768, 128],
    grid: [3, 1],
    tile: true,
    texels_per_m: 256,
};
pub const DIRT_TILE: FactoryArt = FactoryArt {
    key: DIRT_TILE_KEY,
    name: "asset.factory.dirt",
    svg: include_bytes!("fabrica/terra.svg"),
    px: [768, 768],
    grid: [3, 3],
    tile: true,
    texels_per_m: 256,
};
pub const COIN_SHEET: FactoryArt = FactoryArt {
    key: COIN_SHEET_KEY,
    name: "asset.factory.coin",
    svg: include_bytes!("fabrica/moeda.svg"),
    px: [768, 128],
    grid: [6, 1],
    tile: false,
    texels_per_m: 256,
};
pub const FLAG_SHEET: FactoryArt = FactoryArt {
    key: FLAG_SHEET_KEY,
    name: "asset.factory.flag",
    svg: include_bytes!("fabrica/bandeira.svg"),
    px: [512, 256],
    grid: [4, 1],
    tile: false,
    texels_per_m: 128,
};
/// Texels por metro dos azulejos do fundo (fica atrás e anda devagar: metade do chão chega).
pub const BACKGROUND_TEXELS_PER_M: u32 = 128;
const B: u32 = BACKGROUND_TEXELS_PER_M;
pub const CLOUDS_TILE: FactoryArt = FactoryArt {
    key: CLOUDS_TILE_KEY,
    name: "asset.factory.clouds",
    svg: include_bytes!("fabrica/nuvens.svg"),
    px: [3 * 8 * B, 2 * B],
    grid: [3, 1],
    tile: true,
    texels_per_m: B,
};
pub const HILLS_TILE: FactoryArt = FactoryArt {
    key: HILLS_TILE_KEY,
    name: "asset.factory.hills",
    svg: include_bytes!("fabrica/montes.svg"),
    px: [3 * 8 * B, 6 * B],
    grid: [3, 1],
    tile: true,
    texels_per_m: B,
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

fn indice(art: &FactoryArt) -> usize {
    FACTORY_ART
        .iter()
        .position(|a| a.key == art.key)
        .expect("arte de fábrica registada")
}

/// ⭐ Os texels RGBA8 (sRGB, alfa reto) da IMAGEM `art` ([`FactoryArt::image_px`]) — o que a biblioteca guarda e
/// mostra; rasterizados à 1.ª pergunta e guardados.
///
/// # Panics
/// Se o SVG embebido não abre: é arte do repositório, e o gate `toda_a_arte_rasteriza` prova-a.
#[must_use]
pub fn factory_image(art: &FactoryArt) -> &'static [u8] {
    static CACHE: [OnceLock<Vec<u8>>; FACTORY_ART.len()] =
        [const { OnceLock::new() }; FACTORY_ART.len()];
    CACHE[indice(art)].get_or_init(|| {
        let tree = ph2d_imageio_svg::raster::parse(art.svg).expect("SVG de fábrica");
        ph2d_imageio_svg::raster::rasterize(&tree, art.image_px()).expect("tamanho não nulo")
    })
}

/// Os texels de `art` como o ATLAS os guarda (`art.px`): a imagem, e num azulejo a tira das cópias
/// ([`super::wrap_image`]).
#[must_use]
pub fn factory_pixels(art: &FactoryArt) -> &'static [u8] {
    static CACHE: [OnceLock<Vec<u8>>; FACTORY_ART.len()] =
        [const { OnceLock::new() }; FACTORY_ART.len()];
    if !art.tile {
        return factory_image(art);
    }
    CACHE[indice(art)]
        .get_or_init(|| super::wrap_image(factory_image(art), art.cell_px(), art.grid))
}

#[cfg(test)]
#[path = "fabrica_tests.rs"]
mod tests;
