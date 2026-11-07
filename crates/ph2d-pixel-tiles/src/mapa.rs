//! **O mapa de um cel como grelha do mundo**: lê-se e escreve-se por `(coluna, linha)` sem pensar
//! em caixa; ao gravar, encolhe às células usadas, como o cel de imagem.

use ph2d_pixel_core::{
    Cel, CelKey, Forma, FrameId, LayerId, Op, Sprite, TileGrid, TileRef, TilesSet, Tileset,
};

/// A grelha em edição. A célula `(c, r)` começa no píxel `fase + forma.origem(c, r)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapa {
    forma: Forma,
    tw: i32,
    th: i32,
    fase_x: i32,
    fase_y: i32,
    col0: i32,
    row0: i32,
    w: i32,
    h: i32,
    cells: Vec<TileRef>,
}

impl Mapa {
    /// O mapa do cel: o dono da imagem, se for ligado.
    #[must_use]
    pub fn de(s: &Sprite, layer: LayerId, frame: FrameId, set: &Tileset) -> Self {
        let (tw, th) = (i32::from(set.tile_w), i32::from(set.tile_h));
        let forma = set.forma;
        let dono = s.owner((layer, frame));
        // Um quadro ainda sem mapa segue a grelha da camada: o canvas pode tê-la deslocado.
        let fase = s
            .entries()
            .filter(|(k, _)| k.0 == layer)
            .find_map(|(_, c)| c.tiles.as_deref())
            .map_or((0, 0), |g| g.ancora(forma, tw, th).1);
        match s.entry(dono).and_then(|c| c.tiles.as_deref()) {
            Some(g) => {
                let ((col0, row0), (fase_x, fase_y)) = g.ancora(forma, tw, th);
                Self {
                    forma,
                    tw,
                    th,
                    fase_x,
                    fase_y,
                    col0,
                    row0,
                    w: g.w as i32,
                    h: g.h as i32,
                    cells: g.tiles.clone(),
                }
            }
            None => Self {
                forma,
                tw,
                th,
                fase_x: fase.0,
                fase_y: fase.1,
                col0: 0,
                row0: 0,
                w: 0,
                h: 0,
                cells: Vec::new(),
            },
        }
    }

    /// A célula sob o píxel `(x, y)` do sprite.
    #[must_use]
    pub const fn celula(&self, x: i32, y: i32) -> (i32, i32) {
        self.forma
            .celula(x - self.fase_x, y - self.fase_y, self.tw, self.th)
    }

    /// O píxel do sprite onde a caixa da célula `(c, r)` começa.
    #[must_use]
    pub const fn origem(&self, c: i32, r: i32) -> (i32, i32) {
        let (x, y) = self.forma.origem(c, r, self.tw, self.th);
        (self.fase_x + x, self.fase_y + y)
    }

    /// A forma da grelha.
    #[must_use]
    pub const fn forma(&self) -> Forma {
        self.forma
    }

    /// A caixa das células ocupadas: `(c0, r0, c1, r1)`, com o fim de fora.
    #[must_use]
    pub const fn caixa(&self) -> (i32, i32, i32, i32) {
        (self.col0, self.row0, self.col0 + self.w, self.row0 + self.h)
    }

    fn grade_de(&self, col: i32, row: i32, w: u32, h: u32, tiles: Vec<TileRef>) -> TileGrid {
        TileGrid::com_ancora(
            self.forma,
            (self.tw, self.th),
            (col, row),
            (self.fase_x, self.fase_y),
            (w, h),
            tiles,
        )
    }

    /// O tile em `(c, r)`; fora da caixa é o vazio.
    #[must_use]
    pub fn get(&self, c: i32, r: i32) -> TileRef {
        let (x, y) = (c - self.col0, r - self.row0);
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return TileRef::EMPTY;
        }
        self.cells[(y * self.w + x) as usize]
    }

    /// Escreve em `(c, r)`, alargando a caixa se for preciso. Vazio fora da caixa não alarga.
    pub fn set(&mut self, c: i32, r: i32, t: TileRef) {
        if t == TileRef::EMPTY && self.get(c, r) == TileRef::EMPTY {
            return;
        }
        self.cobre(c, r);
        let i = (r - self.row0) * self.w + (c - self.col0);
        self.cells[i as usize] = t;
    }

    fn cobre(&mut self, c: i32, r: i32) {
        if self.w == 0 {
            (self.col0, self.row0, self.w, self.h) = (c, r, 1, 1);
            self.cells = vec![TileRef::EMPTY];
            return;
        }
        let (x0, y0) = (self.col0.min(c), self.row0.min(r));
        let x1 = (self.col0 + self.w).max(c + 1);
        let y1 = (self.row0 + self.h).max(r + 1);
        if (x0, y0, x1 - x0, y1 - y0) == (self.col0, self.row0, self.w, self.h) {
            return;
        }
        let (nw, nh) = (x1 - x0, y1 - y0);
        let mut novo = vec![TileRef::EMPTY; (nw * nh) as usize];
        for y in 0..self.h {
            for x in 0..self.w {
                let o = (y + self.row0 - y0) * nw + (x + self.col0 - x0);
                novo[o as usize] = self.cells[(y * self.w + x) as usize];
            }
        }
        (self.col0, self.row0, self.w, self.h, self.cells) = (x0, y0, nw, nh, novo);
    }

    /// A grelha encolhida às células usadas; `None` se não sobrar nenhuma.
    #[must_use]
    pub fn grade(&self) -> Option<TileGrid> {
        let usada = |x: i32, y: i32| self.cells[(y * self.w + x) as usize] != TileRef::EMPTY;
        let xs: Vec<i32> = (0..self.w)
            .filter(|&x| (0..self.h).any(|y| usada(x, y)))
            .collect();
        let ys: Vec<i32> = (0..self.h)
            .filter(|&y| (0..self.w).any(|x| usada(x, y)))
            .collect();
        let (x0, x1) = (*xs.first()?, *xs.last()?);
        let (y0, y1) = (*ys.first()?, *ys.last()?);
        let mut tiles = Vec::with_capacity(((x1 - x0 + 1) * (y1 - y0 + 1)) as usize);
        for y in y0..=y1 {
            for x in x0..=x1 {
                tiles.push(self.cells[(y * self.w + x) as usize]);
            }
        }
        Some(self.grade_de(
            self.col0 + x0,
            self.row0 + y0,
            (x1 - x0 + 1) as u32,
            (y1 - y0 + 1) as u32,
            tiles,
        ))
    }

    /// O que grava este mapa no cel `key`, sem mudar a opacidade nem o `z` dele. `None`: nada mudou.
    #[must_use]
    pub fn mudanca(&self, s: &Sprite, key: CelKey) -> Option<(CelKey, Option<Cel>)> {
        let antes = s.entry(key);
        let grade = self.grade();
        if antes.and_then(|c| c.tiles.as_deref()) == grade.as_ref() {
            return None;
        }
        let mantem = |c: Cel| Cel {
            opacity: antes.map_or(255, |o| o.opacity),
            z: antes.map_or(0, |o| o.z),
            ..c
        };
        let ligados = s
            .entries()
            .any(|(k, c)| k.0 == key.0 && c.link == Some(key.1));
        match grade {
            Some(g) => Some((key, Some(mantem(Cel::tilemap(g))))),
            // Quem tem ligados não sai: fica com uma célula vazia, e eles continuam a apontar.
            None if ligados => {
                let g = self.grade_de(self.col0, self.row0, 1, 1, vec![TileRef::EMPTY]);
                Some((key, Some(mantem(Cel::tilemap(g)))))
            }
            None => antes.map(|_| (key, None)),
        }
    }
}

/// O op que grava os cels de `mudancas`.
#[must_use]
pub fn op_de_cels(mudancas: Vec<(CelKey, Option<Cel>)>) -> Option<Op> {
    (!mudancas.is_empty()).then(|| {
        Op::Tiles(Box::new(TilesSet {
            cels: mudancas,
            ..TilesSet::default()
        }))
    })
}

/// **Draw Tiles**: `tile` nas células dadas. O tile vazio apaga.
#[must_use]
pub fn carimba(
    s: &Sprite,
    layer: LayerId,
    frame: FrameId,
    celulas: &[(i32, i32)],
    tile: TileRef,
) -> Option<Op> {
    let set = s.tileset(s.layer(layer)?.tileset()?)?;
    if tile.id() as usize >= set.tiles.len() {
        return None;
    }
    let mut m = Mapa::de(s, layer, frame, set);
    for &(c, r) in celulas {
        m.set(c, r, tile);
    }
    let key = s.owner((layer, frame));
    op_de_cels(m.mudanca(s, key).into_iter().collect())
}
