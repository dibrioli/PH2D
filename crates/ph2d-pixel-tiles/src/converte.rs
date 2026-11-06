//! **Layer ▸ Convert To**: imagem ⇄ tilemap, um passo de desfazer, o composto igual.

use std::collections::BTreeMap;

use crate::desenho::no_terreno;
use ph2d_pixel_core::tiles::expand;
use ph2d_pixel_core::{
    Cel, CelKey, LayerId, LayerKind, LayerSet, Op, Rect, Sprite, TileGrid, TileRef, TilesSet,
    Tileset, TilesetId,
};

/// As imagens já vistas e a referência que as dá.
struct Dedupe {
    vistos: BTreeMap<Vec<u8>, TileRef>,
    tiles: Vec<Vec<u8>>,
    flips: bool,
    tw: u32,
    th: u32,
    bpp: usize,
}

impl Dedupe {
    /// `px` visto pelos olhos de `t`: a imagem que `t` desenha quando o tile é `px`.
    fn virado(&self, px: &[u8], t: TileRef) -> Vec<u8> {
        let mut out = vec![0; px.len()];
        for sy in 0..self.th {
            for sx in 0..self.tw {
                let (dx, dy) = t.place(sx, sy, self.tw, self.th);
                let s = (sy * self.tw + sx) as usize * self.bpp;
                let d = (dy * self.tw + dx) as usize * self.bpp;
                out[d..d + self.bpp].copy_from_slice(&px[s..s + self.bpp]);
            }
        }
        out
    }

    /// A referência para `px`, fazendo nascer o tile se for preciso.
    fn refere(&mut self, px: Vec<u8>) -> TileRef {
        if let Some(t) = self.vistos.get(&px) {
            return *t;
        }
        let id = self.tiles.len() as u32;
        self.conhece(id, &px);
        self.tiles.push(px);
        TileRef::new(id, false, false, false)
    }

    /// O tile `id` passa a responder por `px`; quem já respondia fica.
    fn conhece(&mut self, id: u32, px: &[u8]) {
        if self.flips {
            for b in 0_u8..8 {
                let v = TileRef::new(id, b & 1 != 0, b & 2 != 0, b & 4 != 0);
                let img = self.virado(px, v);
                self.vistos.entry(img).or_insert(v);
            }
        } else {
            let t = TileRef::new(id, false, false, false);
            self.vistos.entry(px.to_vec()).or_insert(t);
        }
    }
}

/// O mapa de `cel` sobre a grelha de origem `(0, 0)`, com a imagem dele partida em tiles.
fn parte(s: &Sprite, key: CelKey, bbox: Rect, d: &mut Dedupe) -> TileGrid {
    let (tw, th) = (d.tw as i32, d.th as i32);
    let c0 = bbox.x.div_euclid(tw);
    let r0 = bbox.y.div_euclid(th);
    let c1 = (bbox.x + bbox.w as i32 - 1).div_euclid(tw);
    let r1 = (bbox.y + bbox.h as i32 - 1).div_euclid(th);
    let mut tiles = Vec::new();
    for r in r0..=r1 {
        for c in c0..=c1 {
            let caixa = Rect::new(c * tw, r * th, d.tw, d.th);
            tiles.push(d.refere(s.read_rect(key, caixa)));
        }
    }
    TileGrid {
        x: c0 * tw,
        y: r0 * th,
        w: (c1 - c0 + 1) as u32,
        h: (r1 - r0 + 1) as u32,
        tiles,
    }
}

/// **Imagem → tilemap**: os cels partem-se nos tiles de `tile_w × tile_h`, deduplicados por cópia
/// exata; com `permitir_flips` e tile quadrado, também por X/Y/D. Os ligados continuam ligados.
/// `None`: grupo, referência, Background, ou já é tilemap.
pub fn para_tilemap(
    s: &mut Sprite,
    layer: LayerId,
    tile_w: u16,
    tile_h: u16,
    permitir_flips: bool,
) -> Option<(Op, TilesetId)> {
    if !pode_converter(s, layer) || tile_w == 0 || tile_h == 0 {
        return None;
    }
    let id = TilesetId(s.reserve_id());
    let bpp = s.mode().bpp();
    let vazio = s.transparent_value().to_le_bytes()[..bpp]
        .repeat(usize::from(tile_w) * usize::from(tile_h));
    let set = Tileset::new(id, "Tileset", tile_w, tile_h, vazio);
    Some((corta(s, layer, set, permitir_flips)?, id))
}

/// **Imagem → tilemap sobre um tileset que já existe**: um pedaço igual a um tile dele usa esse tile,
/// e o que for novo acrescenta-se ao fim. O gabarito fica de fora, como no traço.
pub fn para_tilemap_em(
    s: &Sprite,
    layer: LayerId,
    set: TilesetId,
    permitir_flips: bool,
) -> Option<Op> {
    if !pode_converter(s, layer) {
        return None;
    }
    corta(s, layer, s.tileset(set)?.clone(), permitir_flips)
}

fn pode_converter(s: &Sprite, layer: LayerId) -> bool {
    s.layer(layer)
        .is_some_and(|l| matches!(l.kind, LayerKind::Image) && !l.reference && !l.background)
}

fn corta(s: &Sprite, layer: LayerId, mut set: Tileset, permitir_flips: bool) -> Option<Op> {
    let (id, tile_w, tile_h) = (set.id, set.tile_w, set.tile_h);
    let bpp = s.mode().bpp();
    let mut d = Dedupe {
        vistos: BTreeMap::new(),
        tiles: Vec::new(),
        flips: permitir_flips && tile_w == tile_h,
        tw: u32::from(tile_w),
        th: u32::from(tile_h),
        bpp,
    };
    // O tile 0 é o vazio, sem flips: nunca ganha uma imagem virada, que seria a mesma.
    d.vistos.insert(set.tiles[0].clone(), TileRef::EMPTY);
    for (i, px) in set.tiles.iter().enumerate().skip(1) {
        if !no_terreno(&set, i as u32) {
            d.conhece(i as u32, px);
        }
    }
    d.tiles = std::mem::take(&mut set.tiles);
    let mut grelhas = Vec::new();
    for (&key, cel) in s
        .entries()
        .filter(|(k, c)| k.0 == layer && c.link.is_none())
    {
        let g = if cel.bbox.is_empty() {
            // O dono vazio de ligados: uma célula vazia, e eles continuam a apontar.
            TileGrid {
                w: 1,
                h: 1,
                tiles: vec![TileRef::EMPTY],
                ..TileGrid::default()
            }
        } else {
            parte(s, key, cel.bbox, &mut d)
        };
        grelhas.push((key, cel.clone(), g));
    }
    set.tiles = d.tiles;
    let cels = grelhas
        .into_iter()
        .map(|(k, c, g)| {
            let data = expand(&g, &set, bpp);
            let bbox = g.rect(tile_w, tile_h);
            let tiles = Some(Box::new(g));
            (
                k,
                Some(Cel {
                    bbox,
                    data,
                    tiles,
                    ..c
                }),
            )
        })
        .collect();
    let mut layers = s.layers().to_vec();
    layers.iter_mut().find(|x| x.id == layer)?.kind = LayerKind::Tilemap { tileset: id };
    let novo = TilesSet {
        sets: vec![(id, Some(set))],
        ..TilesSet::default()
    };
    Some(Op::Batch(vec![
        Op::Tiles(Box::new(novo)),
        Op::Layers(Box::new(LayerSet { layers, cels })),
    ]))
}

/// **Tilemap → imagem**: os mesmos píxeis, numa camada de imagem. O tileset fica no sprite.
#[must_use]
pub fn para_imagem(s: &Sprite, layer: LayerId) -> Option<Op> {
    s.layer(layer)?.tileset()?;
    let cels = s
        .entries()
        .filter(|(k, c)| k.0 == layer && c.link.is_none())
        .map(|(k, c)| {
            (
                *k,
                Some(Cel {
                    tiles: None,
                    ..c.clone()
                }),
            )
        })
        .collect();
    let mut layers = s.layers().to_vec();
    layers.iter_mut().find(|x| x.id == layer)?.kind = LayerKind::Image;
    Some(Op::Layers(Box::new(LayerSet { layers, cels })))
}
