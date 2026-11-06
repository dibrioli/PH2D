//! **O gabarito de 47** (`docs/Pixel Art/02_spec_engine_e_porte.md` §15): o artista pinta a área e cada
//! célula recebe o tile da forma da sua vizinhança. O Godot corrido confirma a regra.

use std::collections::BTreeSet;

use ph2d_pixel_core::{
    FrameId, LayerId, Op, Sprite, TERRAIN_TILES, Terrain, TileRef, TilesSet, Tileset, TilesetId,
};

use crate::mapa::{Mapa, op_de_cels};

pub const N: u8 = 1;
pub const NE: u8 = 2;
pub const E: u8 = 4;
pub const SE: u8 = 8;
pub const S: u8 = 16;
pub const SW: u8 = 32;
pub const W: u8 = 64;
pub const NW: u8 = 128;

/// Os vizinhos, na ordem dos bits.
pub const VIZINHOS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// A regra única: um canto só conta com as duas arestas dele presentes.
#[must_use]
pub const fn reduz(m: u8) -> u8 {
    let mut r = m & (N | E | S | W);
    if m & N != 0 && m & E != 0 && m & NE != 0 {
        r |= NE;
    }
    if m & E != 0 && m & S != 0 && m & SE != 0 {
        r |= SE;
    }
    if m & S != 0 && m & W != 0 && m & SW != 0 {
        r |= SW;
    }
    if m & W != 0 && m & N != 0 && m & NW != 0 {
        r |= NW;
    }
    r
}

/// As 47 formas, por ordem crescente de máscara: a ordem das casas do gabarito.
#[must_use]
pub fn formas() -> Vec<u8> {
    (0..=u8::MAX)
        .map(reduz)
        .collect::<BTreeSet<u8>>()
        .into_iter()
        .collect()
}

/// A casa do gabarito (`0..47`) de uma vizinhança qualquer.
#[must_use]
pub fn casa(mascara: u8) -> u32 {
    let r = reduz(mascara);
    let mut i = 0;
    for m in 0..r {
        if reduz(m) == m {
            i += 1;
        }
    }
    i
}

/// O terreno a que um tile pertence.
fn terreno_de(set: &Tileset, t: TileRef) -> Option<&Terrain> {
    set.terrains
        .iter()
        .find(|k| t.id() >= k.first && t.id() < k.first + TERRAIN_TILES)
}

/// Dá à célula o tile da sua vizinhança dentro do terreno dela; quem não é de terreno não muda.
fn resolve(m: &mut Mapa, set: &Tileset, c: i32, r: i32) {
    let Some(k) = terreno_de(set, m.get(c, r)) else {
        return;
    };
    let first = k.first;
    let mut mascara = 0_u8;
    for (b, (dx, dy)) in VIZINHOS.iter().enumerate() {
        let viz = terreno_de(set, m.get(c + dx, r + dy));
        if viz.is_some_and(|v| v.first == first) {
            mascara |= 1 << b;
        }
    }
    m.set(
        c,
        r,
        TileRef::new(first + casa(mascara), false, false, false),
    );
}

/// **Pintar área**: as células entram no terreno `qual` (`None` = apagar), e elas e as vizinhas
/// recebem o tile certo. Um passo de desfazer.
#[must_use]
pub fn pinta(
    s: &Sprite,
    layer: LayerId,
    frame: FrameId,
    celulas: &[(i32, i32)],
    qual: Option<usize>,
) -> Option<Op> {
    let set = s.tileset(s.layer(layer)?.tileset()?)?;
    let marca = match qual {
        Some(i) => TileRef::new(set.terrains.get(i)?.first, false, false, false),
        None => TileRef::EMPTY,
    };
    let mut m = Mapa::de(s, layer, frame, set);
    let mut toca: BTreeSet<(i32, i32)> = BTreeSet::new();
    for &(c, r) in celulas {
        m.set(c, r, marca);
        toca.insert((c, r));
        toca.extend(VIZINHOS.iter().map(|(dx, dy)| (c + dx, r + dy)));
    }
    for (c, r) in toca {
        resolve(&mut m, set, c, r);
    }
    let key = s.owner((layer, frame));
    op_de_cels(m.mudanca(s, key).into_iter().collect())
}

/// Um terreno novo: 47 tiles vazios no fim do tileset. Um passo de desfazer.
#[must_use]
pub fn novo_terreno(s: &Sprite, set: TilesetId, nome: &str) -> Option<Op> {
    let mut t = s.tileset(set)?.clone();
    let first = t.tiles.len() as u32;
    let vazio = t.tiles[0].clone();
    t.tiles.resize((first + TERRAIN_TILES) as usize, vazio);
    t.terrains.push(Terrain {
        name: nome.to_owned(),
        first,
    });
    Some(Op::Tiles(Box::new(TilesSet {
        sets: vec![(set, Some(t))],
        ..TilesSet::default()
    })))
}
