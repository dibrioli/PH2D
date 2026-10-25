//! Kernels inteiros (`docs/Pixel Art/02_spec_engine_e_porte.md` §8). Não existe meio pixel.

/// As células do segmento `(x0,y0)→(x1,y1)`, inclusivo, em `cb(x, y)`.
///
/// **Canonicalizado**: os extremos ordenam-se (menor x; empate → menor y) antes de rasterizar, logo
/// A→B e B→A dão o MESMO conjunto de células (o Bresenham cru desempata pela direção). A ordem do
/// `cb` segue a direção canónica, não a de quem chama.
pub fn line_cells(x0: i32, y0: i32, x1: i32, y1: i32, mut cb: impl FnMut(i32, i32)) {
    let (mut ax, mut ay, mut bx, mut by) = (x0, y0, x1, y1);
    if bx < ax || (bx == ax && by < ay) {
        std::mem::swap(&mut ax, &mut bx);
        std::mem::swap(&mut ay, &mut by);
    }
    let dx = i64::from(bx - ax).abs();
    let dy = -i64::from(by - ay).abs();
    let sx = if ax < bx { 1 } else { -1 };
    let sy = if ay < by { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (ax, ay);
    loop {
        cb(x, y);
        if x == bx && y == by {
            return;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
