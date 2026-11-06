//! O desenho da cena de smoke 21: três tiles feitos à mão e um mapa com um flip, em ops de um passo.

use ph2d_pixel_core::{FrameId, LayerId, Op, Sprite, TileRef, TilesSet, TilesetId};

use crate::mapa::carimba;

/// O lado de um tile da cena.
pub const LADO: u16 = 16;
/// O mapa: 0 é vazio; `-3` é o tile 3 virado na horizontal (a seta para o outro lado).
pub const MAPA: [[i32; 6]; 4] = [
    [0, 0, 3, 0, -3, 0],
    [2, 2, 0, 0, 0, 2],
    [1, 1, 2, 2, 2, 1],
    [1, 1, 1, 1, 1, 1],
];
/// As cores de cada tile na DB32 (fundo, marca): relva, tijolo, seta.
const CORES: [(usize, usize); 3] = [(10, 11), (6, 3), (0, 22)];

/// A marca de cada tile: textura na relva, juntas no tijolo, e uma seta para a direita.
const fn marca(tile: usize, x: u16, y: u16) -> bool {
    match tile {
        0 => (x * 3 + y * 5).is_multiple_of(7),
        1 => y % 8 == 7 || (x + (y / 8) * 8).is_multiple_of(16),
        _ => {
            let h = y.abs_diff(8);
            (h == 0 && x > 2 && x < 13) || (x > 7 && x < 13 && h <= 12 - x)
        }
    }
}

/// Os tiles da cena no tileset `set`.
#[must_use]
pub fn tiles(s: &Sprite, set: TilesetId) -> Option<Op> {
    let bpp = s.mode().bpp();
    let cor = |i: usize| Some(s.value_of_rgba(*s.palette().get(i)?).to_le_bytes());
    let mut tiles = Vec::new();
    for (i, &(a, b)) in CORES.iter().enumerate() {
        let (a, b) = (cor(a)?, cor(b)?);
        let mut px = Vec::new();
        for y in 0..LADO {
            for x in 0..LADO {
                px.extend_from_slice(&(if marca(i, x, y) { b } else { a })[..bpp]);
            }
        }
        tiles.push((set, i as u32 + 1, px));
    }
    Some(Op::Tiles(Box::new(TilesSet {
        counts: vec![(set, CORES.len() as u32 + 1)],
        tiles,
        ..TilesSet::default()
    })))
}

/// A linha `r` do mapa, como um op por tile. Um passo por célula, como à mão.
#[must_use]
pub fn celula(s: &Sprite, l: LayerId, f: FrameId, (c, r): (usize, usize)) -> Option<Op> {
    let t = MAPA[r][c];
    let tile = TileRef::new(t.unsigned_abs(), t < 0, false, false);
    (t != 0).then(|| carimba(s, l, f, &[(c as i32, r as i32)], tile))?
}
