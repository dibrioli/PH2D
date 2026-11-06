//! **A mão do artista num tilemap**: destino do traço, modo, carimbo em curso, e o funil do traço.

use ph2d_pixel_core::{FrameId, LayerId, Op, Sprite, Stroke, TileRef, Tileset};

use crate::desenho::{self, Modo};
use crate::mapa::{Mapa, carimba};

/// Para onde vai o traço numa camada de tilemap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alvo {
    #[default]
    Pixels,
    Tiles,
}

/// Destino, modo, o tile do carimbo e os flips dele. O modo inicial é o medido no Trial (M5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mao {
    pub alvo: Alvo,
    pub modo: Modo,
    pub tile: u32,
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_d: bool,
}

impl Default for Mao {
    fn default() -> Self {
        Self {
            alvo: Alvo::Pixels,
            modo: Modo::Auto,
            tile: 1,
            flip_x: false,
            flip_y: false,
            flip_d: false,
        }
    }
}

impl Mao {
    /// O que o carimbo põe na célula.
    #[must_use]
    pub const fn tile_ref(&self) -> TileRef {
        TileRef::new(self.tile, self.flip_x, self.flip_y, self.flip_d)
    }

    /// O conta-gotas de tiles: a mão fica com o tile e os flips da célula.
    pub const fn pega(&mut self, t: TileRef) {
        (self.tile, self.flip_x, self.flip_y, self.flip_d) =
            (t.id(), t.flip_x(), t.flip_y(), t.flip_d());
    }

    /// `Space+Tab`.
    pub const fn alterna_alvo(&mut self) {
        self.alvo = match self.alvo {
            Alvo::Pixels => Alvo::Tiles,
            Alvo::Tiles => Alvo::Pixels,
        };
    }
}

/// O tileset da camada `layer`.
#[must_use]
pub fn tileset_de(s: &Sprite, layer: LayerId) -> Option<&Tileset> {
    s.tileset(s.layer(layer)?.tileset()?)
}

/// Um tileset do sprite, como a janela de criar ou converter o oferece.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conjunto {
    pub id: u32,
    pub tile_w: u16,
    pub tile_h: u16,
    /// Os tiles desenhados: o vazio não conta.
    pub tiles: u32,
}

/// Os tilesets do sprite e o que a janela traz escolhido: o da camada activa, ou o último.
#[must_use]
pub fn conjuntos(s: &Sprite, layer: LayerId) -> (Vec<Conjunto>, Option<u32>) {
    let lista: Vec<Conjunto> = s
        .tilesets()
        .iter()
        .map(|t| Conjunto {
            id: t.id.0,
            tile_w: t.tile_w,
            tile_h: t.tile_h,
            tiles: t.tiles.len().saturating_sub(1) as u32,
        })
        .collect();
    let activo = tileset_de(s, layer).map(|t| t.id.0);
    let padrao = activo.or_else(|| lista.last().map(|c| c.id));
    (lista, padrao)
}

/// O traço acabado: em tilemap vai ao funil dos modos; numa imagem, é o op de sempre.
#[must_use]
pub fn op_do_traco(st: &Stroke, s: &Sprite, modo: Modo) -> Option<Op> {
    if s.layer(st.layer)?.tileset().is_none() {
        return st.op(s);
    }
    let escritas: Vec<_> = st.writes(s).iter().map(|&((x, y), v)| (x, y, v)).collect();
    desenho::aplica(s, st.layer, st.frame, &escritas, modo)
}

/// RGBA de cada tile, para o cartão.
#[must_use]
pub fn imagens(s: &Sprite, set: &Tileset) -> Vec<Vec<u8>> {
    let bpp = s.mode().bpp();
    set.tiles
        .iter()
        .map(|t| {
            t.chunks(bpp)
                .flat_map(|p| {
                    let mut v = [0; 4];
                    v[..bpp].copy_from_slice(p);
                    s.rgba_of_value(u32::from_le_bytes(v))
                })
                .collect()
        })
        .collect()
}

/// Um arrasto de Draw Tiles: as células andadas; a família reescreve um passo do desfazer com elas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carimbo {
    pub layer: LayerId,
    pub frame: FrameId,
    pub tile: TileRef,
    pub celulas: Vec<(i32, i32)>,
    /// O gesto já tem um passo no desfazer.
    pub passo: bool,
}

impl Carimbo {
    /// As células depois de o arrasto chegar ao píxel `at`, sem saltar nenhuma. `None`: não saiu da
    /// célula, ou a camada deixou de ser de tilemap.
    #[must_use]
    pub fn ate(&self, s: &Sprite, at: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let set = tileset_de(s, self.layer)?;
        let alvo = Mapa::de(s, self.layer, self.frame, set).celula(at.0, at.1);
        if self.celulas.last() == Some(&alvo) {
            return None;
        }
        let (c0, r0) = self.celulas.last().copied().unwrap_or(alvo);
        let n = (alvo.0 - c0).abs().max((alvo.1 - r0).abs()).max(1);
        let mut out = self.celulas.clone();
        out.extend((1..=n).map(|i| (c0 + (alvo.0 - c0) * i / n, r0 + (alvo.1 - r0) * i / n)));
        Some(out)
    }

    /// O op com estas células.
    #[must_use]
    pub fn op(&self, s: &Sprite, celulas: &[(i32, i32)]) -> Option<Op> {
        carimba(s, self.layer, self.frame, celulas, self.tile)
    }
}
