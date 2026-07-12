#![forbid(unsafe_code)]
//! **O roteador de conectores** — a linha que gruda em duas formas e as segue.
//!
//! É o que separa um editor de diagrama de um editor vetorial. A crate é
//! deliberadamente **pura**: entra uma descrição de rota (dois pontos, duas direções de
//! saída, uma lista de caixas), sai uma polilinha. Sem ECS, sem kurbo, sem documento — e
//! por isso testável sem montar um mundo.
//!
//! # A decisão de arquitetura: UM roteador, desde o primeiro dia
//!
//! O caminho óbvio seria a tabela de casos do mxGraph (o `OrthConnector`): uma matriz 4×4
//! bit-packed que enumera os formatos de rota (o "Z", o "L", o "U", o "S") em função de
//! como as duas caixas estão dispostas. Ela foi **rejeitada**, por dois achados da pesquisa:
//!
//! 1. **Ela não desvia de obstáculo nenhum.** Verificado na fonte (Apache-2.0, lida, não
//!    copiada). O draw.io teve de vendorizar a *libavoid* (LGPL) compilada em WASM para ter
//!    desvio de verdade. Ou seja: a tabela é um caminho de código que **morre** no dia em
//!    que o desvio chegar — e reescrevê-la seria construir dívida com data marcada.
//! 2. **O A\* sobre o grafo de visibilidade ortogonal** (Wybrow/Marriott/Stuckey, GD 2009)
//!    resolve os dois problemas de uma vez. Com o conjunto de obstáculos = *só as duas
//!    caixas terminais*, o grafo tem ≤36 nós e a busca custa **microssegundos** — e o Z, o
//!    L, o U e o S **caem do custo**, não de uma tabela escrita à mão. Quando o desvio de
//!    obstáculo chegar, ele é literalmente *acrescentar as outras formas ao slice*
//!    [`RouteInput::obstacles`]. O roteador não muda.
//!
//! Um roteador, um conjunto de gates, zero código morto.
//!
//! # O peso de dobra é ADIMENSIONAL — e isso não é um detalhe
//!
//! Uma rota boa não é a mais curta: é a que tem **menos dobras**. O custo, portanto, é
//! `comprimento + W · dobras`. A armadilha está no `W`.
//!
//! A libavoid usa `segmentPenalty = 50`, em pixels. Copiar esse número para cá seria fatal:
//! no mundo do PH2D uma forma tem ~2 unidades de largura, então uma penalidade de 50 tornaria
//! **toda** rota uma reta — a dobra custaria vinte formas de comprimento. O peso tem de ser
//! **relativo ao tamanho da própria rota**:
//!
//! ```text
//! W = BEND_K · manhattan(s0, s1)     com BEND_K = 0,3
//! ```
//!
//! ou seja: *uma dobra a menos vale até 30% de comprimento a mais*. Imune à escala do mundo.
//! E como `manhattan(s0, s1)` é **constante durante a busca**, `W` é constante — o que
//! mantém a heurística `manhattan + min_dobras · W` **admissível** (Wybrow §4), e portanto o
//! A\* ótimo.
//!
//! # O detalhe que separa "correto" de "bonito"
//!
//! Entre duas caixas lado a lado, **toda** linha vertical dentro do vão dá uma rota com o
//! mesmo comprimento e o mesmo número de dobras. Elas são *todas ótimas*. Um A\* puro escolhe
//! uma qualquer — e ela costuma grudar na borda de uma das caixas, o que fica **feio, e
//! correto**. É preciso um **desempate por centralidade** para o "Z" dobrar no meio do vão,
//! que é onde o olho o espera.
//!
//! Nenhum paper menciona isso. O mxGraph o esconde dentro da tabela (o `CENTER_MASK`). Aqui
//! ele é explícito, e tem gate próprio.

use serde::{Deserialize, Serialize};

mod grid;
mod route;
pub use route::route;

mod loops;

#[cfg(test)]
#[path = "route_tests.rs"]
mod tests;

/// Uma caixa alinhada aos eixos, em MUNDO. É como o roteador vê uma forma.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: [f64; 2],
    pub max: [f64; 2],
}

impl Aabb {
    #[must_use]
    pub fn new(min: [f64; 2], max: [f64; 2]) -> Self {
        Self {
            min: [min[0].min(max[0]), min[1].min(max[1])],
            max: [min[0].max(max[0]), min[1].max(max[1])],
        }
    }

    /// A caixa dilatada por `m` em todas as direções — a folga entre o obstáculo e a linha.
    #[must_use]
    pub fn inflate(self, m: f64) -> Self {
        Self {
            min: [self.min[0] - m, self.min[1] - m],
            max: [self.max[0] + m, self.max[1] + m],
        }
    }

    /// `true` se `p` está **estritamente** dentro (a borda não conta — é onde as rotas
    /// tangenciam de propósito).
    #[must_use]
    pub fn contains(self, p: [f64; 2]) -> bool {
        p[0] > self.min[0] + EPS
            && p[0] < self.max[0] - EPS
            && p[1] > self.min[1] + EPS
            && p[1] < self.max[1] - EPS
    }

    #[must_use]
    pub fn center(self) -> [f64; 2] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
        ]
    }
}

/// Uma direção cardeal — a saída de uma ponta, e o rumo de um passo do A\*.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dir {
    East,
    North,
    West,
    South,
}

impl Dir {
    /// O vetor unitário (mundo Y-para-CIMA: `North` é `+y`).
    #[must_use]
    pub fn vec(self) -> [f64; 2] {
        match self {
            Dir::East => [1.0, 0.0],
            Dir::North => [0.0, 1.0],
            Dir::West => [-1.0, 0.0],
            Dir::South => [0.0, -1.0],
        }
    }

    #[must_use]
    pub fn opposite(self) -> Self {
        match self {
            Dir::East => Dir::West,
            Dir::North => Dir::South,
            Dir::West => Dir::East,
            Dir::South => Dir::North,
        }
    }

    /// Todas, em ordem determinística (o desempate do A\* depende disso).
    pub const ALL: [Dir; 4] = [Dir::East, Dir::North, Dir::West, Dir::South];
}

/// Como a rota é desenhada.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum RouteKind {
    /// Reta, de borda a borda.
    Straight = 0,
    /// **Ortogonal** (o cotovelo do fluxograma) — só segmentos horizontais e verticais.
    #[default]
    Orthogonal = 1,
}

/// Uma ponta da rota: onde ela começa/termina e **para que lado sai**.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EndSpec {
    /// O ponto na BORDA da forma (já calculado pelo chamador — ver `boundary_hit`).
    pub at: [f64; 2],
    /// A direção de saída, apontando para **fora** da forma.
    pub dir: Dir,
}

/// Tudo o que o roteador precisa saber.
#[derive(Copy, Clone, Debug)]
pub struct RouteInput<'a> {
    pub start: EndSpec,
    pub end: EndSpec,
    pub kind: RouteKind,
    /// O **jetty**: o quanto a linha avança em linha reta antes de poder dobrar. É o que faz
    /// um conector "sair para cima" antes de virar, em vez de dobrar colado na caixa.
    pub jetty: f64,
    /// **Os obstáculos.** Hoje = as duas caixas terminais. Amanhã = + as outras formas do
    /// diagrama. **O roteador não muda** — só este slice cresce. Era essa a aposta.
    pub obstacles: &'a [Aabb],
    /// Deslocamento perpendicular, para dois conectores no mesmo par de formas não se
    /// sobreporem. `0` = o único.
    pub spread: f64,
    /// Fonte e destino são a MESMA forma? Aí não há rota a buscar: é um laço, e ele é
    /// construído, não roteado.
    pub self_loop: Option<Aabb>,
}

/// Tolerância geométrica. Uma rota é feita de números redondos; o épsilon existe só para
/// "estritamente dentro" não pegar a própria borda que a rota tangencia.
pub(crate) const EPS: f64 = 1e-9;

/// **O peso de uma dobra**, como fração do tamanho da rota. `0,3` = uma dobra a menos vale
/// até 30% de comprimento a mais. Adimensional de propósito (ver o doc do módulo).
pub(crate) const BEND_K: f64 = 0.3;

/// A folga entre um obstáculo e a linha, em múltiplos do jetty. O draw.io usa 16px de buffer
/// com 10px de jetty (1,6×); o yFiles, 10 com stub de 5–10 (1–2×). Convergem.
pub(crate) const MARGIN_K: f64 = 1.5;
