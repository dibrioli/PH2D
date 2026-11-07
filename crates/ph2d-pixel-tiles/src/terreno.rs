//! **O gabarito de 47** (`docs/Pixel Art/02_spec_engine_e_porte.md` §15): o artista pinta a área e cada
//! célula recebe o tile da forma da sua vizinhança. O Godot corrido confirma a regra.

use std::collections::BTreeSet;

use ph2d_pixel_core::canvas::{Borders, canvas_size};
use ph2d_pixel_core::tiles::tilemap_layer_using;
use ph2d_pixel_core::{
    Cel, Forma, FrameId, LayerId, Op, Sprite, TERRAIN_TILES, Terrain, TileGrid, TileRef, TilesSet,
    Tileset, TilesetId,
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
    if !set.forma.tem_terreno() {
        return None;
    }
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
    if !t.forma.tem_terreno() {
        return None;
    }
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

/// Em que terço de `n / d` cai: `0`, a borda de baixo; `1`, o meio; `2`, a de cima. `None`: fora.
const fn zona(n: i64, d: i64) -> Option<u8> {
    if n < 0 || n >= d {
        None
    } else if 4 * n < d {
        Some(0)
    } else if 4 * n < 3 * d {
        Some(1)
    } else {
        Some(2)
    }
}

/// O píxel `(x, y)` do tile da forma `mascara` está cheio? O meio sempre; cada braço e cada canto só
/// com o vizinho dele. Em iso, as direções são as do índice, medidas no Godot (M2iso).
#[must_use]
pub const fn guia_cheio(forma: Forma, mascara: u8, x: u32, y: u32, w: u32, h: u32) -> bool {
    let (x, y, w, h) = (x as i64, y as i64, w as i64, h as i64);
    let (a, b) = (2 * x + 1, 2 * y + 1);
    let (u, v, d) = match forma {
        Forma::Iso => (a * h + b * w - w * h, b * w - a * h + w * h, 2 * w * h),
        _ => (a * h, b * w, 2 * w * h),
    };
    let (Some(zu), Some(zv)) = (zona(u, d), zona(v, d)) else {
        return false;
    };
    let bit = match (zu, zv) {
        (1, 1) => return true,
        (1, 0) => N,
        (2, 0) => NE,
        (2, 1) => E,
        (2, 2) => SE,
        (1, 2) => S,
        (0, 2) => SW,
        (0, 1) => W,
        _ => NW,
    };
    mascara & bit != 0
}

/// **Terreno novo já com o guia**: as 47 formas pintadas com `cor`, para o terreno funcionar logo e o
/// artista pintar por cima. Um passo de desfazer.
#[must_use]
pub fn novo_terreno_com_guia(s: &Sprite, set: TilesetId, nome: &str, cor: u32) -> Option<Op> {
    let Op::Tiles(mut op) = novo_terreno(s, set, nome)? else {
        return None;
    };
    let t = op.sets.first_mut()?.1.as_mut()?;
    let first = t.terrains.last()?.first as usize;
    let (w, h) = (u32::from(t.tile_w), u32::from(t.tile_h));
    let forma = t.forma;
    let bpp = s.mode().bpp();
    let valor = cor.to_le_bytes();
    for (i, m) in formas().into_iter().enumerate() {
        let tile = &mut t.tiles[first + i];
        for y in 0..h {
            for x in 0..w {
                if guia_cheio(forma, m, x, y, w, h) {
                    let p = (y * w + x) as usize * bpp;
                    tile[p..p + bpp].copy_from_slice(&valor[..bpp]);
                }
            }
        }
    }
    Some(Op::Tiles(op))
}

/// As colunas do gabarito na tela: 8 × 6, como na especificação.
pub const COLUNAS_DO_GABARITO: u32 = 8;

/// **O gabarito em camada**: uma camada nova com as 47 formas de `qual` à vista, para pintar por cima.
/// Cresce o canvas se não couberem (nunca encolhe). Um passo de desfazer.
#[must_use]
pub fn gabarito(
    s: &mut Sprite,
    acima: LayerId,
    frame: FrameId,
    set: TilesetId,
    qual: usize,
    nome: &str,
) -> Option<(Op, LayerId)> {
    let conjunto = s.tileset(set)?.clone();
    let first = conjunto.terrains.get(qual)?.first;
    let linhas = TERRAIN_TILES.div_ceil(COLUNAS_DO_GABARITO);
    let tiles = (0..COLUNAS_DO_GABARITO * linhas)
        .map(|i| {
            if i < TERRAIN_TILES {
                TileRef::new(first + i, false, false, false)
            } else {
                TileRef::EMPTY
            }
        })
        .collect();
    let lados = (i32::from(conjunto.tile_w), i32::from(conjunto.tile_h));
    // Em iso, a coluna de partida deixa a primeira linha toda em x não negativo.
    let col = if conjunto.forma == Forma::Iso {
        linhas as i32 - 1
    } else {
        0
    };
    let g = TileGrid::com_ancora(
        conjunto.forma,
        lados,
        (col, 0),
        (0, 0),
        (COLUNAS_DO_GABARITO, linhas),
        tiles,
    );
    let caixa = g.caixa(&conjunto);
    let (fim_x, fim_y) = (caixa.x + caixa.w as i32, caixa.y + caixa.h as i32);
    let bordas = Borders {
        left: 0,
        top: 0,
        right: (fim_x - s.width() as i32).max(0),
        bottom: (fim_y - s.height() as i32).max(0),
    };
    let (camada, nova) = tilemap_layer_using(s, acima, set)?;
    let camada = match camada {
        Op::Layers(mut l) => {
            l.layers.iter_mut().find(|x| x.id == nova)?.name = nome.to_owned();
            Op::Layers(l)
        }
        o => o,
    };
    let mut lista = Vec::new();
    if bordas.right > 0 || bordas.bottom > 0 {
        lista.push(canvas_size(s, bordas, s.transparent_value())?);
    }
    lista.push(camada);
    lista.push(Op::Tiles(Box::new(TilesSet {
        cels: vec![((nova, frame), Some(Cel::tilemap(g)))],
        ..TilesSet::default()
    })));
    Some((Op::Batch(lista), nova))
}
