//! O que o painel guarda: a janela aberta, as opções dela e as imagens dos tiles.

use std::sync::Arc;

use crate::canal::{Conjunto, Contexto, Tela};

/// O tile de um tilemap novo: o de omissão do Aseprite (`docs/PixelArt/P10_oraculos.md`).
pub(crate) const TILE_INICIAL: u16 = 16;

/// O estado do painel dos tilemaps.
pub struct PixelTilesPanelState {
    pub(crate) tela: Tela,
    pub(crate) tile_w: u16,
    pub(crate) tile_h: u16,
    pub(crate) flips: bool,
    /// A forma de um tileset novo.
    pub(crate) forma: ph2d_pixel_core::Forma,
    /// O tileset escolhido na janela; `None`, um novo.
    pub(crate) conjunto: Option<u32>,
    /// As imagens publicadas e as da placa: só se recria a do tile que mudou.
    pub(crate) fontes: Arc<Vec<Vec<u8>>>,
    pub(crate) imagens: Vec<Option<ph2d_vector::StableImage>>,
}

impl Default for PixelTilesPanelState {
    fn default() -> Self {
        Self {
            tela: Tela::default(),
            tile_w: TILE_INICIAL,
            tile_h: TILE_INICIAL,
            flips: false,
            forma: ph2d_pixel_core::Forma::Square,
            conjunto: None,
            fontes: Arc::default(),
            imagens: Vec::new(),
        }
    }
}

impl PixelTilesPanelState {
    /// A janela aberta.
    #[must_use]
    pub fn tela(&self) -> Tela {
        self.tela
    }

    /// O tileset escolhido, se ainda existir.
    pub(crate) fn escolhido(&self, x: &Contexto) -> Option<Conjunto> {
        let id = self.conjunto?;
        x.conjuntos.iter().copied().find(|c| c.id == id)
    }
}
