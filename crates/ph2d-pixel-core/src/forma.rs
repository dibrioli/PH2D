//! **A forma da grelha de tiles**: onde cai cada célula e que célula tem cada píxel, em inteiros.
//! Lei medida: o Godot corrido e `ph2d-grid` (`docs/PixelArt/P10_oraculos.md`, M3b). A arte do tile
//! continua retangular; só a posição e o clique seguem a forma.

/// A forma do tile. `Iso` é o losango de `ph2d-grid` (`diamond_down` do Godot); `Hex` e `HexFlat`,
/// linhas ou colunas alternadas (`stacked`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Forma {
    #[default]
    Square,
    Iso,
    Hex,
    HexFlat,
}

impl Forma {
    pub const TODAS: [Self; 4] = [Self::Square, Self::Iso, Self::Hex, Self::HexFlat];

    /// O número gravado.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Square => 0,
            Self::Iso => 1,
            Self::Hex => 2,
            Self::HexFlat => 3,
        }
    }

    #[must_use]
    pub const fn from_code(c: u8) -> Option<Self> {
        Some(match c {
            0 => Self::Square,
            1 => Self::Iso,
            2 => Self::Hex,
            3 => Self::HexFlat,
            _ => return None,
        })
    }

    /// O nome no JSON do mapa.
    #[must_use]
    pub const fn nome(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Iso => "iso",
            Self::Hex => "hex",
            Self::HexFlat => "hex_flat",
        }
    }

    /// O gabarito de 47 só existe com 8 vizinhos: em hex há 6.
    #[must_use]
    pub const fn tem_terreno(self) -> bool {
        matches!(self, Self::Square | Self::Iso)
    }

    /// O canto da caixa do tile `(c, r)`, com a célula `(0, 0)` a começar em `(0, 0)`.
    #[must_use]
    pub const fn origem(self, c: i32, r: i32, w: i32, h: i32) -> (i32, i32) {
        match self {
            Self::Square => (c * w, r * h),
            Self::Iso => (((c - r) * w).div_euclid(2), ((c + r) * h).div_euclid(2)),
            Self::Hex => hex_origem(c, r, w, h),
            Self::HexFlat => {
                let (y, x) = hex_origem(r, c, h, w);
                (x, y)
            }
        }
    }

    /// A célula do píxel `(x, y)`, medido a partir do canto da célula `(0, 0)`.
    #[must_use]
    pub const fn celula(self, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
        match self {
            Self::Square => (x.div_euclid(w), y.div_euclid(h)),
            Self::Iso => iso_celula(x as i64, y as i64, w as i64, h as i64),
            Self::Hex => hex_celula(x as i64, y as i64, w as i64, h as i64),
            Self::HexFlat => {
                let (r, c) = hex_celula(y as i64, x as i64, h as i64, w as i64);
                (c, r)
            }
        }
    }

    /// As células `(c, r)` que tocam a caixa `(w, h)` em células, começando em `(c0, r0)`: basta olhar
    /// para as duas primeiras e duas últimas de cada eixo, porque `origem` é monótona por paridade.
    #[must_use]
    pub const fn limites(
        self,
        (c0, r0): (i32, i32),
        (cs, rs): (i32, i32),
        (w, h): (i32, i32),
    ) -> (i32, i32, i32, i32) {
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        let colunas = [c0, c0 + 1, c0 + cs - 2, c0 + cs - 1];
        let linhas = [r0, r0 + 1, r0 + rs - 2, r0 + rs - 1];
        let mut i = 0;
        while i < colunas.len() {
            let mut j = 0;
            while j < linhas.len() {
                let (c, r) = (colunas[i], linhas[j]);
                if c >= c0 && c < c0 + cs && r >= r0 && r < r0 + rs {
                    let (x, y) = self.origem(c, r, w, h);
                    if x < x0 {
                        x0 = x;
                    }
                    if y < y0 {
                        y0 = y;
                    }
                    if x + w > x1 {
                        x1 = x + w;
                    }
                    if y + h > y1 {
                        y1 = y + h;
                    }
                }
                j += 1;
            }
            i += 1;
        }
        (x0, y0, x1, y1)
    }
}

const fn hex_origem(c: i32, r: i32, w: i32, h: i32) -> (i32, i32) {
    (c * w + r.rem_euclid(2) * (w / 2), (r * 3 * h).div_euclid(4))
}

/// Com a aresta exatamente no centro do píxel, ele fica na coluna menor: o desempate medido.
const fn iso_celula(x: i64, y: i64, w: i64, h: i64) -> (i32, i32) {
    let a = (2 * x + 1 - w) * h;
    let b = (2 * y + 1) * w;
    let d = 2 * w * h;
    (
        (a + b - 1).div_euclid(d) as i32,
        (b - a).div_euclid(d) as i32,
    )
}

/// A banda de `3h/4` e, no topo dela, o ziguezague com a linha de cima. Em quartos e meios de píxel,
/// para não perder o exato.
const fn hex_celula(x: i64, y: i64, w: i64, h: i64) -> (i32, i32) {
    let q = 4 * y + 2;
    let banda = q.div_euclid(3 * h);
    let t = q.rem_euclid(3 * h);
    let impar = banda.rem_euclid(2) == 1;
    let meio = 2 * x + 1 - if impar { w } else { 0 };
    let col = meio.div_euclid(2 * w);
    let fx = meio.rem_euclid(2 * w);
    let acima = if fx < w {
        t * w < h * (w - fx)
    } else {
        t * w <= h * (fx - w)
    };
    if t >= h || !acima {
        return (col as i32, banda as i32);
    }
    let vizinha = match (fx < w, impar) {
        (true, true) | (false, false) => col,
        (true, false) => col - 1,
        (false, true) => col + 1,
    };
    (vizinha as i32, (banda - 1) as i32)
}
