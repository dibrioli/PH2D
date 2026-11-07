//! **A mão do artista num tilemap**: destino do traço, modo, carimbo em curso, e o funil do traço.

use ph2d_pixel_core::{Forma, FrameId, LayerId, Op, Sprite, Stroke, TileRef, Tileset};

use crate::desenho::{self, Modo};
use crate::mapa::{Mapa, carimba};

/// Para onde vai o traço numa camada de tilemap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alvo {
    #[default]
    Pixels,
    Tiles,
    /// O traço pinta a área do terreno escolhido.
    Terreno,
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
    /// O terreno de [`Alvo::Terreno`], pela ordem do tileset.
    pub terreno: u32,
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
            terreno: 0,
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
            Alvo::Tiles | Alvo::Terreno => Alvo::Pixels,
        };
    }
}

/// A mão agora: o que um arrasto escreve, ou a chave da recusa. `apaga`: borracha ou botão direito.
pub fn marca_de(m: &Mao, set: &Tileset, apaga: bool) -> Result<Marca, &'static str> {
    if m.alvo != Alvo::Terreno {
        let t = if apaga { TileRef::EMPTY } else { m.tile_ref() };
        return Ok(Marca::Tile(t));
    }
    if !set.forma.tem_terreno() {
        return Err("pixel.tiles.refuse.terrain_shape");
    }
    if m.terreno as usize >= set.terrains.len() {
        return Err("pixel.tiles.no_terrain");
    }
    Ok(Marca::Terreno((!apaga).then_some(m.terreno as usize)))
}

/// O conta-gotas: em terreno, fica com o terreno da célula; senão, com o tile dela.
pub fn pega(m: &mut Mao, set: &Tileset, t: TileRef) {
    match terreno_do_tile(set, t.id()) {
        Some(q) if m.alvo == Alvo::Terreno => m.terreno = q,
        _ => m.pega(t),
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

/// O terreno a que um tile pertence, pela ordem do tileset.
#[must_use]
pub fn terreno_do_tile(set: &Tileset, id: u32) -> Option<u32> {
    set.terrains
        .iter()
        .position(|t| id >= t.first && id < t.first + ph2d_pixel_core::TERRAIN_TILES)
        .map(|i| i as u32)
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

/// O que o arrasto escreve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marca {
    /// Este tile; o vazio apaga.
    Tile(TileRef),
    /// A área entra neste terreno; `None` apaga.
    Terreno(Option<usize>),
}

/// Um arrasto de Draw Tiles: as células andadas; a família reescreve um passo do desfazer com elas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carimbo {
    pub layer: LayerId,
    pub frame: FrameId,
    pub marca: Marca,
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
        let m = Mapa::de(s, self.layer, self.frame, set);
        let alvo = m.celula(at.0, at.1);
        if self.celulas.last() == Some(&alvo) {
            return None;
        }
        let mut out = self.celulas.clone();
        let Some(&(c0, r0)) = self.celulas.last() else {
            out.push(alvo);
            return Some(out);
        };
        if m.forma() == Forma::Square {
            let n = (alvo.0 - c0).abs().max((alvo.1 - r0).abs()).max(1);
            out.extend((1..=n).map(|i| (c0 + (alvo.0 - c0) * i / n, r0 + (alvo.1 - r0) * i / n)));
            return Some(out);
        }
        // Em iso e hex anda-se em píxeis: índices lineares saltariam vizinhas.
        let (x0, y0) = m.origem(c0, r0);
        let (x0, y0) = (
            x0 + i32::from(set.tile_w) / 2,
            y0 + i32::from(set.tile_h) / 2,
        );
        let (dx, dy) = (at.0 - x0, at.1 - y0);
        let passo = i32::from(set.tile_w.min(set.tile_h)).max(2) / 2;
        let n = (dx.abs().max(dy.abs()) / passo).max(1);
        for i in 1..=n {
            let c = m.celula(x0 + dx * i / n, y0 + dy * i / n);
            if out.last() != Some(&c) {
                out.push(c);
            }
        }
        Some(out)
    }

    /// O op com estas células.
    #[must_use]
    pub fn op(&self, s: &Sprite, celulas: &[(i32, i32)]) -> Option<Op> {
        match self.marca {
            Marca::Tile(t) => carimba(s, self.layer, self.frame, celulas, t),
            Marca::Terreno(q) => crate::terreno::pinta(s, self.layer, self.frame, celulas, q),
        }
    }
}
