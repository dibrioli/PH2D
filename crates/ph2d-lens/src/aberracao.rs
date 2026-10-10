//! **A ABERRAÇÃO CROMÁTICA** — cada canal lido numa posição própria, bilinear, borda replicada.
//!
//! Conferida AO BIT contra o ImageMagick (`-distort SRT` por canal, erro `8e-6` = o arredondamento
//! de 16 bits, doc 129 §5.1): radial = o canal ESCALADO a partir do centro, `pos = c + (p-c)/s`;
//! linear = o canal DESLOCADO, `pos = p - d`. O verde fica; o vermelho e o azul vão em sentidos
//! opostos.

/// Os params do artista.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aberracao {
    /// Radial: a escala do vermelho é `1 + amount` e a do azul `1 - amount`. Linear: o
    /// deslocamento é `amount × meia-diagonal`. `0` desliga.
    pub amount: f32,
    /// `false` = radial (a lente real) · `true` = linear (o deslocamento do vídeo analógico).
    pub linear: bool,
    /// A direcção do modo linear, em VOLTAS.
    pub angle: f32,
}

impl Aberracao {
    #[must_use]
    pub fn neutra(&self) -> bool {
        self.amount == 0.0
    }
}

/// As posições (R, G, B) onde o pixel de centro `p` lê a fonte.
#[must_use]
pub fn posicoes(p: [f32; 2], centro: [f32; 2], meia_diag: f32, a: &Aberracao) -> [[f32; 2]; 3] {
    if a.linear {
        let ang = a.angle * std::f32::consts::TAU;
        let d = [
            a.amount * meia_diag * ang.cos(),
            a.amount * meia_diag * ang.sin(),
        ];
        return [[p[0] - d[0], p[1] - d[1]], p, [p[0] + d[0], p[1] + d[1]]];
    }
    let em = |s: f32| {
        [
            centro[0] + (p[0] - centro[0]) / s,
            centro[1] + (p[1] - centro[1]) / s,
        ]
    };
    [em(1.0 + a.amount), p, em(1.0 - a.amount)]
}

/// O deslocamento linear `d` (em px) que estes params dão — a porta do gate contra o oráculo, que
/// fala em pixels.
#[must_use]
pub fn de_deslocamento(d: [f32; 2], meia_diag: f32) -> Aberracao {
    Aberracao {
        amount: d[0].hypot(d[1]) / meia_diag,
        linear: true,
        angle: d[1].atan2(d[0]) / std::f32::consts::TAU,
    }
}

/// Bilinear sobre os centros dos pixels, com a borda REPLICADA (fora do quadro lê-se a aresta: um
/// canal encolhido não traz preto para dentro da imagem).
#[must_use]
pub fn bilinear(canal: impl Fn(i32, i32) -> f32, w: i32, h: i32, pos: [f32; 2]) -> f32 {
    let x = pos[0] - 0.5;
    let y = pos[1] - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let at = |i: i32, j: i32| canal(i.clamp(0, w - 1), j.clamp(0, h - 1));
    let (i, j) = (x0 as i32, y0 as i32);
    let topo = at(i, j) + (at(i + 1, j) - at(i, j)) * fx;
    let base = at(i, j + 1) + (at(i + 1, j + 1) - at(i, j + 1)) * fx;
    topo + (base - topo) * fy
}
