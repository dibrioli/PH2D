//! **O pivô e as formas de colisão POR CÉLULA** (P14e D1, o padrão da Unity: cada célula tem o
//! próprio pivô e a própria forma). O valor da folha (`Sprite.anchor`, `ExtraSheet.anchor`) é o
//! padrão; a célula que tem o seu manda. Gravado.

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Entity, World};
use serde::{Deserialize, Serialize};

use crate::{FrameRef, SimComponent, SpriteGrid};

/// Uma forma de colisão (uma slice do Aseprite) célula a célula.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FormaPorCelula {
    pub nome: String,
    /// `[x, y, w, h]` em píxeis da célula (Y para baixo); `None` = escondida nessa célula.
    pub rects_px: Vec<Option<[f32; 4]>>,
}

/// As células de UMA folha.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CelulasDaFolha {
    /// Tamanho de uma célula, em píxeis da imagem.
    pub celula_px: [u32; 2],
    /// O pivô de cada célula em píxeis da célula (Y para baixo); `None`/fora = o da folha.
    pub pivos_px: Vec<Option<[f32; 2]>>,
    pub formas: Vec<FormaPorCelula>,
}

impl CelulasDaFolha {
    /// O `anchor` (metros locais, Y para cima, do pivô ao centro do quad) da célula `cell`, para
    /// um quad de `size` metros; `None` = a célula não tem pivô próprio.
    #[must_use]
    pub fn anchor(&self, cell: u32, size: [f32; 2]) -> Option<[f32; 2]> {
        let [px, py] = (*self.pivos_px.get(cell as usize)?)?;
        let [w, h] = self.celula_px;
        if w == 0 || h == 0 {
            return None;
        }
        Some([
            size[0] * (0.5 - px / w as f32),
            size[1] * (py / h as f32 - 0.5),
        ])
    }

    /// O pivô da célula em píxeis; sem pivô próprio, o centro.
    #[must_use]
    pub fn pivo_px(&self, cell: u32) -> (f64, f64) {
        self.pivos_px.get(cell as usize).copied().flatten().map_or(
            (
                f64::from(self.celula_px[0]) / 2.0,
                f64::from(self.celula_px[1]) / 2.0,
            ),
            |[x, y]| (f64::from(x), f64::from(y)),
        )
    }
}

/// As células por folha (`folhas[k]` = a folha `k`; `0` = a própria imagem).
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteCells {
    pub folhas: Vec<CelulasDaFolha>,
}

impl SimComponent for SpriteCells {}

impl SpriteCells {
    #[must_use]
    pub fn folha(&self, k: u8) -> Option<&CelulasDaFolha> {
        self.folhas.get(usize::from(k))
    }
}

/// O anchor da célula na tela do objecto `e` (folha `k`, célula), se ela tem pivô próprio.
#[must_use]
pub fn anchor_da_celula(
    world: &World,
    e: Entity,
    k: u8,
    cell: u32,
    size: [f32; 2],
) -> Option<[f32; 2]> {
    world.get::<SpriteCells>(e)?.folha(k)?.anchor(cell, size)
}

/// A célula da folha `0` que o objecto mostra (`None` = vazio ou outra folha).
#[must_use]
pub fn celula_mostrada(world: &World, e: Entity) -> Option<u32> {
    match world.get::<SpriteGrid>(e)?.shown() {
        FrameRef::Cell { sheet: 0, cell } => Some(cell),
        _ => None,
    }
}
