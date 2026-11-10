//! A vista de leitura do cel composto.

/// RGBA de alfa reto, `w * h * 4` bytes, linha a linha.
#[derive(Debug, Clone, Copy)]
pub struct Img<'a> {
    pub w: u32,
    pub h: u32,
    px: &'a [u8],
}

pub(crate) const VIZ4: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
pub(crate) const VIZ8: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

impl<'a> Img<'a> {
    /// # Panics
    /// Se `px` não tiver `w * h * 4` bytes.
    #[must_use]
    pub fn new(w: u32, h: u32, px: &'a [u8]) -> Self {
        assert_eq!(px.len(), w as usize * h as usize * 4, "Img: tamanho errado");
        Self { w, h, px }
    }

    pub(crate) fn idx(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as u32) < self.w && (y as u32) < self.h)
            .then(|| y as usize * self.w as usize + x as usize)
    }

    /// A cor opaca em `(x, y)`; `None` fora do cel ou transparente.
    pub(crate) fn cor(&self, x: i32, y: i32) -> Option<[u8; 4]> {
        let i = self.idx(x, y)? * 4;
        let c = [self.px[i], self.px[i + 1], self.px[i + 2], self.px[i + 3]];
        (c[3] > 0).then_some(c)
    }

    pub(crate) fn opaco(&self, x: i32, y: i32) -> bool {
        self.cor(x, y).is_some()
    }

    /// `(x, y)` é opaco e tem a cor `c`.
    pub(crate) fn e(&self, x: i32, y: i32, c: [u8; 4]) -> bool {
        self.cor(x, y) == Some(c)
    }

    pub(crate) fn len(&self) -> usize {
        self.w as usize * self.h as usize
    }

    pub(crate) fn xy(&self, i: usize) -> (i32, i32) {
        ((i % self.w as usize) as i32, (i / self.w as usize) as i32)
    }
}

/// O brilho aproximado (Rec. 601 inteiro), para ordenar tons sem OKLab.
pub(crate) fn luma(c: [u8; 4]) -> u32 {
    299 * u32::from(c[0]) + 587 * u32::from(c[1]) + 114 * u32::from(c[2])
}

pub(crate) fn cel(x: i32, y: i32) -> [u32; 2] {
    [x as u32, y as u32]
}
