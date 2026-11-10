//! O desenho de uma célula para a prévia Align ([`super::align`]): a caixa e as arestas da silhueta.

/// O desenho de uma célula (os píxeis com alfa), em frações dela a partir do canto de cima: a caixa `[x0, y0, x1,
/// y1]` e as arestas da silhueta `[xa, ya, xb, yb]` (as bordas entre um píxel com alfa e um sem), fundidas em corridas.
#[derive(Debug, PartialEq)]
pub struct Silhueta {
    pub caixa: [f32; 4],
    pub arestas: Vec<[f32; 4]>,
}

impl Silhueta {
    /// `None` = a célula não tem desenho.
    #[must_use]
    pub fn of(px: &[u8], w: usize, h: usize) -> Option<Self> {
        let on = |x: isize, y: isize| {
            x >= 0
                && y >= 0
                && (x as usize) < w
                && (y as usize) < h
                && px[(y as usize * w + x as usize) * 4 + 3] > 0
        };
        let mut caixa: Option<[usize; 4]> = None;
        for y in 0..h {
            for x in (0..w).filter(|x| on(*x as isize, y as isize)) {
                let o = caixa.get_or_insert([x, y, x, y]);
                *o = [o[0].min(x), o[1].min(y), o[2].max(x), o[3].max(y)];
            }
        }
        let [a, t, r, d] = caixa?;
        let (fw, fh) = (w.max(1) as f32, h.max(1) as f32);
        let mut arestas = Vec::new();
        // As bordas horizontais (por cima de cada linha `y`, entre `y - 1` e `y`) e as verticais, em corridas.
        for y in 0..=h as isize {
            let mut run: Option<usize> = None;
            for x in 0..=w {
                let borda = x < w && on(x as isize, y) != on(x as isize, y - 1);
                match (borda, run) {
                    (true, None) => run = Some(x),
                    (false, Some(x0)) => {
                        arestas.push([x0 as f32 / fw, y as f32 / fh, x as f32 / fw, y as f32 / fh]);
                        run = None;
                    }
                    _ => {}
                }
            }
        }
        for x in 0..=w as isize {
            let mut run: Option<usize> = None;
            for y in 0..=h {
                let borda = y < h && on(x, y as isize) != on(x - 1, y as isize);
                match (borda, run) {
                    (true, None) => run = Some(y),
                    (false, Some(y0)) => {
                        arestas.push([x as f32 / fw, y0 as f32 / fh, x as f32 / fw, y as f32 / fh]);
                        run = None;
                    }
                    _ => {}
                }
            }
        }
        Some(Self {
            caixa: [
                a as f32 / fw,
                t as f32 / fh,
                (r + 1) as f32 / fw,
                (d + 1) as f32 / fh,
            ],
            arestas,
        })
    }
}
