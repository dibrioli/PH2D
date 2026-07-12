//! **Pontas de traço** (arrowheads / markers) — módulo irmão de [`crate::shapes`].
//!
//! ## Por que isto NÃO é uma forma do catálogo
//!
//! Já existe uma família de setas em BLOCO (`arrows.rs`): silhuetas preenchíveis, com corpo
//! e cabeça, que se desenham arrastando uma caixa. São ótimas para *apontar para algo num
//! cartaz*, e péssimas para o que se quer aqui.
//!
//! Uma ponta de traço é outra coisa: é uma **propriedade do stroke**. Ela nasce na ponta de
//! um caminho — qualquer caminho aberto: uma linha, um arco, uma espiral, uma curva
//! desenhada à mão, e (o que vem a seguir) um **conector**. Ela herda a cor e a largura do
//! traço, gira sozinha com a tangente da curva, e não tem caixa própria. Modelá-la como
//! forma obrigaria o usuário a posicioná-la e girá-la à mão a cada vez que a linha mudasse
//! — que é exatamente o trabalho que um editor existe para não fazer.
//!
//! É também a razão de ela vir ANTES dos conectores: um conector sem ponta é uma linha.
//!
//! ## O modelo (o do SVG, que é o que todo mundo usa)
//!
//! - A geometria do marcador é autorada numa caixa unitária e **escalada pela largura do
//!   traço** (`markerUnits = "strokeWidth"`, o default do SVG). Engrossar a linha engrossa a
//!   ponta na proporção — senão uma linha grossa terminaria num alfinete.
//! - Ele é **orientado pela tangente** da curva no extremo (`orient = "auto"`).
//! - A linha é **encurtada** pelo recuo do marcador ([`Marker::inset`]), senão o traço
//!   apareceria por dentro de uma ponta vazada (a `Open` e a `Bar` não têm o que esconder,
//!   mas a `Triangle` e o `Diamond` têm). É a mesma razão pela qual um conector tem de parar
//!   na borda da caixa, não no centro dela — e é o mesmo mecanismo.

use crate::{VecPath, VecVertex};

/// A ponta de um traço. **Append-only**: o discriminante é gravado no documento.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Default, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(u8)]
pub enum Marker {
    /// Sem ponta (o default — uma linha é uma linha).
    #[default]
    None = 0,
    /// Triângulo cheio: a seta clássica de fluxograma.
    Triangle = 1,
    /// Ponta ABERTA (duas riscas em "V"): a seta de diagrama de classes / UML.
    Open = 2,
    /// Losango cheio (agregação, em UML).
    Diamond = 3,
    /// Losango VAZADO (composição, em UML) — o mesmo contorno, sem preenchimento.
    DiamondOpen = 4,
    /// Círculo cheio (o "ponto" de uma âncora, ou o bullet de um diagrama ER).
    Circle = 5,
    /// Círculo VAZADO.
    CircleOpen = 6,
    /// Barra perpendicular (o "1" de uma cardinalidade; ou um fim-de-linha).
    Bar = 7,
}

/// Todos os marcadores, na ordem do seletor.
pub const ALL_MARKERS: &[Marker] = &[
    Marker::None,
    Marker::Triangle,
    Marker::Open,
    Marker::Diamond,
    Marker::DiamondOpen,
    Marker::Circle,
    Marker::CircleOpen,
    Marker::Bar,
];

/// Meia-largura padrão de uma ponta, em múltiplos da largura do traço.
const HALF_W: f64 = 2.0;
/// Comprimento padrão de uma ponta, em múltiplos da largura do traço.
const LEN: f64 = 4.0;
/// Raio de uma ponta redonda.
const R: f64 = 2.0;

impl Marker {
    /// O discriminante gravado no documento.
    #[must_use]
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// O marcador de um discriminante gravado. `None` (o `Option`) = veio de uma versão
    /// futura — o chamador trata como "sem ponta" em vez de entrar em pânico.
    #[must_use]
    pub fn from_u8(v: u8) -> Option<Self> {
        ALL_MARKERS.iter().copied().find(|m| m.as_u8() == v)
    }

    /// Rótulo para a UI (inglês).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Marker::None => "None",
            Marker::Triangle => "Arrow",
            Marker::Open => "Open",
            Marker::Diamond => "Diamond",
            Marker::DiamondOpen => "Diamond (hollow)",
            Marker::Circle => "Circle",
            Marker::CircleOpen => "Circle (hollow)",
            Marker::Bar => "Bar",
        }
    }

    /// A ponta é PREENCHIDA? As vazadas (`Open`, `DiamondOpen`, `CircleOpen`) são desenhadas
    /// com o mesmo traço da linha — é o que as faz parecerem "da mesma caneta".
    #[must_use]
    pub fn is_filled(self) -> bool {
        matches!(self, Marker::Triangle | Marker::Diamond | Marker::Circle)
    }

    /// **Quanto a linha tem de RECUAR**, em múltiplos da largura do traço, para não aparecer
    /// por dentro da ponta.
    ///
    /// Uma ponta VAZADA precisa do recuo INTEIRO (o traço cruzaria o vão e se veria por
    /// dentro dela); uma ponta CHEIA esconde a linha e pode recuar um pouco menos, mas ainda
    /// recua — senão a junção do traço com a base do triângulo forma uma bolha nas larguras
    /// grandes. A `Open` e a `Bar` não fecham região nenhuma: recuo zero.
    #[must_use]
    pub fn inset(self) -> f64 {
        match self {
            Marker::None | Marker::Open | Marker::Bar => 0.0,
            Marker::Triangle => LEN,
            Marker::Diamond | Marker::DiamondOpen => 2.0 * LEN,
            Marker::Circle | Marker::CircleOpen => 2.0 * R,
        }
    }

    /// A geometria da ponta, em MUNDO: na ponta `tip`, apontando na direção `dir` (unitária,
    /// a tangente da curva no extremo, apontando **para fora**), com o traço de largura `w`.
    ///
    /// A caixa é escalada por `w` (o `markerUnits = "strokeWidth"` do SVG): a ponta engrossa
    /// junto com a linha. Contorno FECHADO nas cheias e nas vazadas de região (elas são
    /// preenchidas ou traçadas conforme [`Marker::is_filled`]); ABERTO na `Open` e na `Bar`,
    /// que são riscos.
    #[must_use]
    pub fn build(self, tip: [f64; 2], dir: [f64; 2], w: f64) -> Option<VecPath> {
        let w = if w > 1e-9 { w } else { return None };
        let (dx, dy) = (dir[0], dir[1]);
        let len = dx.hypot(dy);
        if len < 1e-12 {
            return None; // sem tangente não há para onde apontar
        }
        let (ux, uy) = (dx / len, dy / len); // ao longo da linha, para FORA
        let (nx, ny) = (-uy, ux); // a normal
        // `(a, b)` = `a` unidades para trás da ponta + `b` unidades para o lado, tudo em
        // múltiplos da largura do traço. Um sistema de coordenadas local, para as tabelas
        // abaixo se lerem como um desenho.
        let p = |a: f64, b: f64| -> [f64; 2] {
            [
                tip[0] - ux * a * w + nx * b * w,
                tip[1] - uy * a * w + ny * b * w,
            ]
        };

        let path = match self {
            Marker::None => return None,
            Marker::Triangle => closed_poly(&[p(0.0, 0.0), p(LEN, HALF_W), p(LEN, -HALF_W)]),
            // Duas riscas em "V": um contorno ABERTO que passa pela ponta.
            Marker::Open => open_poly(&[p(LEN, HALF_W), p(0.0, 0.0), p(LEN, -HALF_W)]),
            Marker::Diamond | Marker::DiamondOpen => closed_poly(&[
                p(0.0, 0.0),
                p(LEN, HALF_W),
                p(2.0 * LEN, 0.0),
                p(LEN, -HALF_W),
            ]),
            Marker::Circle | Marker::CircleOpen => circle(p(R, 0.0), R * w),
            // Perpendicular à linha, centrada na ponta.
            Marker::Bar => open_poly(&[p(0.0, HALF_W), p(0.0, -HALF_W)]),
        };
        Some(path)
    }
}

fn closed_poly(pts: &[[f64; 2]]) -> VecPath {
    VecPath {
        verts: pts.iter().copied().map(VecVertex::corner).collect(),
        closed: true,
        ..VecPath::default()
    }
}

fn open_poly(pts: &[[f64; 2]]) -> VecPath {
    VecPath {
        verts: pts.iter().copied().map(VecVertex::corner).collect(),
        closed: false,
        ..VecPath::default()
    }
}

/// Círculo em quatro cúbicas exatas (o `KAPPA`) — não um polígono.
fn circle(c: [f64; 2], r: f64) -> VecPath {
    let k = crate::corners::KAPPA * r;
    let verts = [
        ([c[0] + r, c[1]], [0.0, -k], [0.0, k]),
        ([c[0], c[1] + r], [k, 0.0], [-k, 0.0]),
        ([c[0] - r, c[1]], [0.0, k], [0.0, -k]),
        ([c[0], c[1] - r], [-k, 0.0], [k, 0.0]),
    ]
    .iter()
    .map(|&(a, i, o)| VecVertex::smooth(a, [a[0] + i[0], a[1] + i[1]], [a[0] + o[0], a[1] + o[1]]))
    .collect();
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// **Encurta um caminho aberto** para dar lugar às pontas: recua `start` unidades de mundo
/// no começo e `end` no fim, ao longo da curva.
///
/// Sem isto o traço aparece por dentro de uma ponta vazada, e forma uma bolha na base de uma
/// cheia. É o análogo exato do que um conector faz ao parar na BORDA de uma caixa em vez do
/// centro dela.
///
/// Recua ao longo da **poligonal das âncoras**, não do comprimento de arco exato: o recuo é
/// de poucos múltiplos da largura do traço, e nessa escala a diferença é sub-pixel. Se o
/// caminho for mais curto que os dois recuos somados, devolve `None` — não há linha a
/// desenhar, só as pontas.
#[must_use]
pub fn trim_path(path: &VecPath, start: f64, end: f64) -> Option<VecPath> {
    if path.closed || path.verts.len() < 2 {
        return Some(path.clone()); // um contorno fechado não tem pontas
    }
    if start <= 0.0 && end <= 0.0 {
        return Some(path.clone());
    }
    let mut verts = path.verts.clone();
    if start > 0.0 && !trim_end(&mut verts, start, true) {
        return None;
    }
    if end > 0.0 && !trim_end(&mut verts, end, false) {
        return None;
    }
    Some(VecPath {
        verts,
        ..path.clone()
    })
}

/// Recua `d` a partir de uma das pontas. `false` = o caminho é curto demais.
fn trim_end(verts: &mut Vec<VecVertex>, d: f64, from_start: bool) -> bool {
    let n = verts.len();
    // Caminha do extremo para dentro, consumindo `d`.
    let mut left = d;
    let mut i = 0_usize;
    loop {
        let (a, b) = if from_start {
            (verts[i].anchor, verts[i + 1].anchor)
        } else {
            (verts[n - 1 - i].anchor, verts[n - 2 - i].anchor)
        };
        let seg = (b[0] - a[0]).hypot(b[1] - a[1]);
        if seg >= left {
            // O ponto novo cai DENTRO deste segmento.
            let t = if seg > 1e-12 { left / seg } else { 0.0 };
            let np = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
            let idx = if from_start { i } else { n - 1 - i };
            // A âncora anda; os handles a acompanham (o recuo é curto, então a curva
            // praticamente não muda de forma — e uma quina continua quina).
            let v = &mut verts[idx];
            let (sx, sy) = (np[0] - v.anchor[0], np[1] - v.anchor[1]);
            v.anchor = np;
            v.in_handle = [v.in_handle[0] + sx, v.in_handle[1] + sy];
            v.out_handle = [v.out_handle[0] + sx, v.out_handle[1] + sy];
            // Descarta os vértices que ficaram para fora.
            if from_start {
                verts.drain(0..i);
            } else {
                verts.truncate(n - i);
            }
            return true;
        }
        left -= seg;
        i += 1;
        if i + 1 >= n {
            return false; // consumiu o caminho inteiro
        }
    }
}

/// A tangente de saída num extremo do caminho, apontando **para fora** — a direção em que a
/// ponta olha. `None` se não há caminho (ou se os dois pontos coincidem).
#[must_use]
pub fn end_tangent(path: &VecPath, at_start: bool) -> Option<([f64; 2], [f64; 2])> {
    let n = path.verts.len();
    if n < 2 {
        return None;
    }
    let (v, next) = if at_start {
        (&path.verts[0], &path.verts[1])
    } else {
        (&path.verts[n - 1], &path.verts[n - 2])
    };
    // A tangente da CURVA no extremo é a direção do handle — não a corda até a âncora
    // seguinte. Numa curva bem encurvada as duas divergem visivelmente, e a ponta sairia
    // torta. O handle degenerado (quina) cai de volta na corda.
    let h = if at_start { v.out_handle } else { v.in_handle };
    let (hx, hy) = (h[0] - v.anchor[0], h[1] - v.anchor[1]);
    let (dx, dy) = if hx.hypot(hy) > 1e-9 {
        (-hx, -hy) // para FORA = contra o handle, que aponta para dentro do caminho
    } else {
        (v.anchor[0] - next.anchor[0], v.anchor[1] - next.anchor[1])
    };
    let len = dx.hypot(dy);
    if len < 1e-12 {
        return None;
    }
    Some((v.anchor, [dx / len, dy / len]))
}

#[cfg(test)]
#[path = "marker_tests.rs"]
mod tests;
