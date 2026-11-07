//! **O cartão Tileset**: acrescentar um tile vazio e apagar um tile, com os mapas a seguir.

use ph2d_pixel_core::{
    Cel, Forma, Op, Sprite, TERRAIN_TILES, TileGrid, TileRef, TilesSet, TilesetId,
};

/// Um tile vazio no fim do conjunto. Um passo de desfazer.
#[must_use]
pub fn novo_tile(s: &Sprite, set: TilesetId) -> Option<Op> {
    let n = s.tileset(set)?.tiles.len() as u32;
    Some(Op::Tiles(Box::new(TilesSet {
        counts: vec![(set, n + 1)],
        ..TilesSet::default()
    })))
}

/// O mapa sem o tile `id`: as células dele esvaziam e os tiles acima descem um número.
fn sem_o_tile(g: &TileGrid, id: u32) -> TileGrid {
    let desce = |t: TileRef| {
        let i = t.id();
        if i == id {
            TileRef::EMPTY
        } else if i > id {
            TileRef::new(i - 1, t.flip_x(), t.flip_y(), t.flip_d())
        } else {
            t
        }
    };
    TileGrid {
        tiles: g.tiles.iter().copied().map(desce).collect(),
        ..g.clone()
    }
}

/// Apaga o tile `id`. `None`: o tile vazio, um tile de gabarito, ou que não existe. Um passo de desfazer.
#[must_use]
pub fn apaga_tile(s: &Sprite, set: TilesetId, id: u32) -> Option<Op> {
    let velho = s.tileset(set)?;
    let de_gabarito = velho
        .terrains
        .iter()
        .any(|t| id >= t.first && id < t.first + TERRAIN_TILES);
    if id == 0 || id as usize >= velho.tiles.len() || de_gabarito {
        return None;
    }
    let mut novo = velho.clone();
    novo.tiles.remove(id as usize);
    for t in novo.terrains.iter_mut().filter(|t| t.first > id) {
        t.first -= 1;
    }
    let cels = s
        .entries()
        .filter(|(k, c)| c.link.is_none() && s.layer(k.0).and_then(|l| l.tileset()) == Some(set))
        .filter_map(|(k, c)| {
            let g = c.tiles.as_deref()?;
            let grade = Box::new(sem_o_tile(g, id));
            Some((
                *k,
                Some(Cel {
                    tiles: Some(grade),
                    ..c.clone()
                }),
            ))
        })
        .collect();
    Some(Op::Tiles(Box::new(TilesSet {
        sets: vec![(set, Some(novo))],
        cels,
        ..TilesSet::default()
    })))
}

/// Apaga o terreno `qual` e os 47 tiles dele: as células esvaziam, os tiles acima descem. Um passo.
#[must_use]
pub fn apaga_terreno(s: &Sprite, set: TilesetId, qual: usize) -> Option<Op> {
    let velho = s.tileset(set)?;
    let first = velho.terrains.get(qual)?.first;
    let fim = first + TERRAIN_TILES;
    let mut novo = velho.clone();
    novo.terrains.remove(qual);
    novo.tiles.drain(first as usize..fim as usize);
    for t in novo.terrains.iter_mut().filter(|t| t.first >= fim) {
        t.first -= TERRAIN_TILES;
    }
    let desce = |t: TileRef| {
        let i = t.id();
        if i >= fim {
            TileRef::new(i - TERRAIN_TILES, t.flip_x(), t.flip_y(), t.flip_d())
        } else if i >= first {
            TileRef::EMPTY
        } else {
            t
        }
    };
    let cels = cels_com(s, set, |g| TileGrid {
        tiles: g.tiles.iter().copied().map(desce).collect(),
        ..g.clone()
    });
    Some(Op::Tiles(Box::new(TilesSet {
        sets: vec![(set, Some(novo))],
        cels,
        ..TilesSet::default()
    })))
}

/// Os cels donos das camadas deste tileset, com o mapa refeito.
fn cels_com(
    s: &Sprite,
    set: TilesetId,
    f: impl Fn(&TileGrid) -> TileGrid,
) -> Vec<(ph2d_pixel_core::CelKey, Option<Cel>)> {
    s.entries()
        .filter(|(k, c)| c.link.is_none() && s.layer(k.0).and_then(|l| l.tileset()) == Some(set))
        .filter_map(|(k, c)| {
            let tiles = Some(Box::new(f(c.tiles.as_deref()?)));
            Some((*k, Some(Cel { tiles, ..c.clone() })))
        })
        .collect()
}

/// **Muda a forma da grelha**: as células ficam com o mesmo índice e os tiles com a mesma arte. Em hex
/// não há gabarito, logo um tileset com terrenos recusa. Um passo de desfazer.
#[must_use]
pub fn muda_forma(s: &Sprite, set: TilesetId, forma: Forma) -> Option<Op> {
    let velho = s.tileset(set)?;
    if velho.forma == forma || (!forma.tem_terreno() && !velho.terrains.is_empty()) {
        return None;
    }
    let lados = (i32::from(velho.tile_w), i32::from(velho.tile_h));
    let de = velho.forma;
    let cels = cels_com(s, set, |g| {
        let (indice, fase) = g.ancora(de, lados.0, lados.1);
        TileGrid::com_ancora(forma, lados, indice, fase, (g.w, g.h), g.tiles.clone())
    });
    let mut novo = velho.clone();
    novo.forma = forma;
    Some(Op::Tiles(Box::new(TilesSet {
        sets: vec![(set, Some(novo))],
        cels,
        ..TilesSet::default()
    })))
}
