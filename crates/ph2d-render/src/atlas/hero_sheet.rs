//! ⭐ **O boneco de fábrica** — a folha do *Character* e do *Animated Sprite* do menu Add: o boneco verde de
//! capacete do Kenney Pixel Platformer em vetor plano (`docs/GameEngineFuncional/06_arte_vetorial_plano.md`),
//! 4 linhas (parado · correr · saltar · cair; agachado e a andar agachado nas células livres) × 8 células,
//! cada desenho na pose e no tempo da referência do Blender (`docs/GameEngineFuncional/ferramentas/boneco_poses.json`). O SVG é [`super::fabrica::HERO_ART`];
//! o boneco olha para a DIREITA (`facing = +1`).

use ph2d_color::SrgbRgba;

/// Chave reservada da folha — abaixo do disco, no topo do espaço de chaves.
pub const HERO_SHEET_KEY: u32 = u32::MAX - 2;
/// Lado de uma célula, em texels.
pub const HERO_SHEET_CELL_PX: u32 = 192;
/// Células por linha (a animação mais longa, correr).
pub const HERO_SHEET_COLS: u32 = 8;
/// Linhas.
pub const HERO_SHEET_ROWS: u32 = 4;

const _: () =
    assert!(HERO_SHEET_KEY > super::FIRST_IMPORT_KEY && HERO_SHEET_KEY < super::DISC_TILE_KEY);

/// Do centro da célula à sola do boneco, em fracção do lado da célula: o desenho põe a sola a 0,05 do fundo
/// (`sola_margem` da referência). Quem põe o boneco no mundo faz o personagem pairar a esta altura.
pub const HERO_SHEET_SOLE_BELOW_CENTRE: f32 = 0.45;

/// Uma animação da folha: a tag, as células (inclusivas), os ms de cada célula e se toca UMA vez (e fica no
/// último desenho) em vez de repetir.
#[derive(Clone, Copy, Debug)]
pub struct HeroTag {
    pub name: &'static str,
    pub from: u32,
    pub to: u32,
    pub per_frame_ms: &'static [u32],
    pub once: bool,
}

/// As animações — os tempos são os da referência do Blender (30 q/s). As 4 primeiras começam cada uma na sua
/// linha; `crouch` e `crouch_walk` (plano 10) ocupam as células que `jump` e `fall` deixam livres, para a grelha
/// gravada nos ficheiros (8 × 4) não mudar.
pub const HERO_SHEET_TAGS: [HeroTag; 6] = [
    HeroTag {
        name: "idle",
        from: 0,
        to: 5,
        per_frame_ms: &[167; 6],
        once: false,
    },
    HeroTag {
        name: "run",
        from: 8,
        to: 15,
        per_frame_ms: &[67; 8],
        once: false,
    },
    HeroTag {
        name: "jump",
        from: 16,
        to: 19,
        per_frame_ms: &[100, 133, 167, 133],
        once: true,
    },
    HeroTag {
        name: "fall",
        from: 24,
        to: 27,
        per_frame_ms: &[100; 4],
        once: false,
    },
    HeroTag {
        name: "crouch",
        from: 20,
        to: 23,
        per_frame_ms: &[200; 4],
        once: false,
    },
    HeroTag {
        name: "crouch_walk",
        from: 28,
        to: 31,
        per_frame_ms: &[133; 4],
        once: false,
    },
];

/// ⭐ Os texels da folha inteira (`COLS × CELL` por `ROWS × CELL`), sRGB em alfa reto, linha a linha.
#[must_use]
pub fn hero_sheet_pixels() -> Vec<SrgbRgba> {
    super::fabrica::factory_pixels(&super::fabrica::HERO_ART)
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| SrgbRgba::new(p[0], p[1], p[2], p[3]))
        .collect()
}

#[cfg(test)]
#[path = "hero_sheet_tests.rs"]
mod tests;
