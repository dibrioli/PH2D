//! O desenho das cenas de smoke: na 21, três tiles feitos à mão e um mapa com um flip; na 22, ilhas
//! sobre a água, com as 47 formas do terreno desenhadas por regra. Em ops de um passo.

use ph2d_pixel_core::{FrameId, LayerId, Op, Patch, Rect, Sprite, TileRef, TilesSet, TilesetId};

use crate::mapa::carimba;
use crate::terreno::{self, E, N, NE, NW, S, SE, SW, W};

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

/// O mapa real da aceitação e da cena 22: `#` é terreno. Mais de 200 células, buracos e corredores de
/// uma célula (`docs/PixelArt/P10_plano.md` §7).
pub const MAPA_REAL: [&str; 16] = [
    "........................",
    "..#########....#######..",
    "..#########....#######..",
    "..###...###....##...##..",
    "..###...###....##...##..",
    "..#########....#######..",
    "..#########.#..#######..",
    ".....#......#......#....",
    ".....#......#......#....",
    "..####################..",
    "..############.########.",
    "..############.########.",
    "..###..#######.##...###.",
    "..###..#######.##...###.",
    "..############.########.",
    "........................",
];

/// As células de terreno de [`MAPA_REAL`].
#[must_use]
pub fn celulas_reais() -> Vec<(i32, i32)> {
    celulas_de(&MAPA_REAL)
}

fn celulas_de(mapa: &[&str]) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for (r, linha) in mapa.iter().enumerate() {
        for (c, b) in linha.bytes().enumerate() {
            if b == b'#' {
                out.push((c as i32, r as i32));
            }
        }
    }
    out
}

/// O mapa da cena 22: ilhas com um lago, corredores de uma célula e água livre para pintar.
pub const ILHAS: [&str; 10] = [
    "................",
    "..####....###...",
    ".######..#####..",
    ".##..#######.##.",
    ".##..######..##.",
    ".#######...####.",
    "..###.#.....##..",
    "...#..#.........",
    "..###.##..###...",
    "................",
];

/// As células de terreno de [`ILHAS`].
#[must_use]
pub fn celulas_ilhas() -> Vec<(i32, i32)> {
    celulas_de(&ILHAS)
}

/// O raio dos cantos de fora da ilha, em píxeis.
const RAIO: i32 = 6;

/// A distância do píxel à água, na forma `m`: 0 é o píxel que toca a água. Cantos de fora redondos,
/// cantos de dentro em arco, e o mesmo número dos dois lados da emenda entre tiles.
fn ate_a_agua(m: u8, x: i32, y: i32) -> i32 {
    let ultimo = i32::from(LADO) - 1;
    let lados = [(N, y), (E, ultimo - x), (S, ultimo - y), (W, x)];
    let tem = |b: u8| m & b != 0;
    let mut d = i32::MAX;
    for (i, &(a, ea)) in lados.iter().enumerate() {
        if !tem(a) {
            d = d.min(ea);
        }
        let (b, eb) = lados[(i + 1) % 4];
        let diagonal = [NE, SE, SW, NW][i];
        if !tem(a) && !tem(b) && ea < RAIO && eb < RAIO {
            let (u, v) = (RAIO - ea, RAIO - eb);
            d = d.min(RAIO - (u * u + v * v).isqrt());
        } else if tem(a) && tem(b) && !tem(diagonal) {
            d = d.min((ea * ea + eb * eb).isqrt());
        }
    }
    d
}

/// A cor (índice da DB32) do píxel `(x, y)` da forma `m`; `None` deixa ver a água. Faixas da margem:
/// espuma, areia, relva escura; dentro, relva com textura e flores.
fn cor_da_ilha(m: u8, x: i32, y: i32) -> Option<usize> {
    let grao = (x * 7 + y * 13 + x * y * 3) % 29;
    match ate_a_agua(m, x, y) {
        ..=0 => None,
        1 => Some(20),
        2 | 3 => Some(if grao % 5 == 0 { 6 } else { 7 }),
        4 => Some(12),
        _ if (x, y) == (4, 10) || (x, y) == (11, 5) => Some(8),
        _ if grao == 3 || grao == 17 => Some(9),
        _ if grao == 11 || grao == 23 => Some(11),
        _ => Some(10),
    }
}

/// **O terreno desenhado da cena 22**: as 47 formas, em ilha de relva com praia. Um passo de desfazer.
#[must_use]
pub fn terreno_ilhas(s: &Sprite, set: TilesetId, nome: &str) -> Option<Op> {
    let Op::Tiles(mut op) = terreno::novo_terreno(s, set, nome)? else {
        return None;
    };
    let bpp = s.mode().bpp();
    let t = op.sets.first_mut()?.1.as_mut()?;
    let first = t.terrains.last()?.first as usize;
    let lado = i32::from(LADO);
    for (i, m) in terreno::formas().into_iter().enumerate() {
        for y in 0..lado {
            for x in 0..lado {
                if let Some(c) = cor_da_ilha(m, x, y) {
                    let cor = s.value_of_rgba(*s.palette().get(c)?).to_le_bytes();
                    let p = (y * lado + x) as usize * bpp;
                    t.tiles[first + i][p..p + bpp].copy_from_slice(&cor[..bpp]);
                }
            }
        }
    }
    Some(Op::Tiles(op))
}

/// **A água** sob o mapa da cena 22, na camada `layer`: é o que aparece onde não há terreno.
#[must_use]
pub fn agua(s: &Sprite, layer: LayerId, frame: FrameId) -> Option<Op> {
    let (w, h) = (s.width(), s.height());
    let rect = Rect::new(0, 0, w, h);
    let bpp = s.mode().bpp();
    let fundo = s.value_of_rgba(*s.palette().get(16)?).to_le_bytes();
    let onda = s.value_of_rgba(*s.palette().get(18)?).to_le_bytes();
    let mut pixels = Vec::with_capacity((w * h) as usize * bpp);
    for y in 0..h {
        for x in 0..w {
            let crista = y % 6 == 2 && (x + y * 3) % 16 < 4;
            pixels.extend_from_slice(&(if crista { onda } else { fundo })[..bpp]);
        }
    }
    Some(Op::Patch(Patch {
        layer,
        frame,
        bbox: Some(rect),
        rect,
        pixels,
    }))
}
