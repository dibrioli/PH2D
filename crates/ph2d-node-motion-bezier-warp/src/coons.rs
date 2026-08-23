//! A geometria do `motion.bezier_warp`: as quatro curvas de Bézier da fronteira e o
//! **patch de Coons** que as interpola.
//!
//! ## O que um patch de Coons é, e por que ele é o mapa certo aqui
//!
//! Dadas as quatro curvas da fronteira de um quadrilátero curvo — `bottom(u)`,
//! `top(u)`, `left(v)`, `right(v)` —, o patch de Coons **bilinearmente misturado** é
//!
//! ```text
//! S(u,v) = (1−v)·bottom(u) + v·top(u)          ← a régua em v
//!        + (1−u)·left(v)   + u·right(v)        ← a régua em u
//!        − [ (1−u)(1−v)·P00 + u(1−v)·P10 + (1−u)v·P01 + uv·P11 ]   ← o bilinear a MAIS
//! ```
//!
//! As duas primeiras linhas interpolam cada par de bordas opostas; somadas, elas
//! contam a superfície bilinear dos quatro cantos **duas** vezes, e a terceira linha
//! subtrai-a. É a construção de Coons (1967), e ela tem a propriedade que interessa:
//! **a fronteira do patch é EXACTAMENTE as quatro curvas dadas** — nada de aproximar
//! a borda que o artista desenhou.
//!
//! ## ⚠️ Ele NÃO reduz à homografia do `motion.four_point_warp`, e é por isso que
//! ## este nó existe
//!
//! Com as tangentes nos terços a Bézier degenera na RECTA, então a fronteira vira um
//! quadrilátero de lados rectos — e aí o Coons é o mapa **BILINEAR** daquele quad, que
//! **não** é a homografia projectiva (Heckbert) do nó irmão. Os dois concordam nos
//! quatro CANTOS e divergem no interior: o bilinear arqueia as rectas interiores, o
//! projectivo mantém-nas rectas (é a propriedade que define uma projectividade). A
//! célula P1 da folha 04 mediu isto antes de decidir, e é a razão de o *Bezier Warp*
//! do AE não ser um param do Corner Pin.
//!
//! ⭐ **E com TUDO no neutro os dois são a identidade**, porque a fronteira passa a ser
//! a do próprio quadrado unitário e o Coons de um bilinear é o bilinear — que ali é
//! `S(u,v) = (u,v)`. É isso que faz o nó recém-largado não mover um pixel.
//!
//! Sem transcendentais (HR-5): só somas e produtos.

/// Um ponto de controle da fronteira, em coordenadas do **quadrado unitário**.
pub type P2 = [f32; 2];

/// Os **doze** pontos de controle da fronteira, na ordem canónica.
///
/// Quatro cantos e, entre cada par vizinho, as duas tangentes da cúbica daquele lado.
/// A ordem é a do relógio começando no canto superior-esquerdo — a mesma do
/// `motion.four_point_warp`, para os dois nós se lerem igual num painel.
#[derive(Clone, Copy, Debug)]
pub struct Boundary {
    /// Os cantos: TL, TR, BR, BL.
    pub corner: [P2; 4],
    /// As tangentes, duas por lado, na ordem TOP, RIGHT, BOTTOM, LEFT — e dentro de
    /// cada lado, na direcção em que aquele lado é percorrido.
    pub tangent: [[P2; 2]; 4],
}

/// Índices dos lados em [`Boundary::tangent`].
pub const TOP: usize = 0;
pub const RIGHT: usize = 1;
pub const BOTTOM: usize = 2;
pub const LEFT: usize = 3;

/// Índices dos cantos em [`Boundary::corner`].
pub const TL: usize = 0;
pub const TR: usize = 1;
pub const BR: usize = 2;
pub const BL: usize = 3;

impl Boundary {
    /// **A fronteira NEUTRA: o quadrado unitário com as tangentes nos terços.**
    ///
    /// ⚠️ Os terços não são decoração — é a posição em que uma cúbica de Bézier
    /// **degenera exactamente na recta** entre os extremos (`B(t) = (1−t)P₀ + tP₃`
    /// quando `P₁ = P₀ + (P₃−P₀)/3` e `P₂ = P₀ + 2(P₃−P₀)/3`, por identidade
    /// polinomial e não por aproximação). É isto que faz o offset zero ser a
    /// identidade **ao bit**, e não *"quase"*.
    pub fn unit() -> Self {
        let (tl, tr, br, bl) = ([0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]);
        Self {
            corner: [tl, tr, br, bl],
            tangent: [
                thirds(tl, tr), // TOP: TL → TR
                thirds(tr, br), // RIGHT: TR → BR
                thirds(br, bl), // BOTTOM: BR → BL
                thirds(bl, tl), // LEFT: BL → TL
            ],
        }
    }
}

/// As duas tangentes que põem a cúbica exactamente sobre o segmento `a → b`.
fn thirds(a: P2, b: P2) -> [P2; 2] {
    let d = [(b[0] - a[0]) / 3.0, (b[1] - a[1]) / 3.0];
    [[a[0] + d[0], a[1] + d[1]], [b[0] - d[0], b[1] - d[1]]]
}

/// Uma cúbica de Bézier em `t ∈ [0,1]`, na forma de Bernstein.
///
/// ⚠️ Escrita com os quatro pesos explícitos e não por de Casteljau: os dois dão o
/// mesmo número, e esta forma é a que o WGSL porta linha a linha (uma `fma` por
/// termo, sem laço), que é o que mantém a paridade barata de ler.
pub fn bezier(p0: P2, p1: P2, p2: P2, p3: P2, t: f32) -> P2 {
    let s = 1.0 - t;
    let (w0, w1) = (s * s * s, 3.0 * s * s * t);
    let (w2, w3) = (3.0 * s * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// **O patch de Coons** em `(u, v) ∈ [0,1]²`. `v = 0` é a borda de BAIXO.
///
/// ⚠️ As quatro curvas são avaliadas na direcção em que a [`Boundary`] as declara e
/// depois lidas no sentido do patch: o lado `BOTTOM` corre `BR → BL`, então
/// `bottom(u)` é ele em `1 − u`. Errar um destes sentidos dá um patch que ainda
/// interpola os quatro cantos e cruza-se no meio — o modo de falha que parece
/// *"o warp está a torcer"* e não *"um índice está trocado"*.
pub fn coons(b: &Boundary, u: f32, v: f32) -> P2 {
    // As quatro bordas, já orientadas no sentido do patch.
    let top = bezier(
        b.corner[TL],
        b.tangent[TOP][0],
        b.tangent[TOP][1],
        b.corner[TR],
        u,
    );
    let bottom = bezier(
        b.corner[BL],
        b.tangent[BOTTOM][1],
        b.tangent[BOTTOM][0],
        b.corner[BR],
        u,
    );
    let left = bezier(
        b.corner[BL],
        b.tangent[LEFT][0],
        b.tangent[LEFT][1],
        b.corner[TL],
        v,
    );
    let right = bezier(
        b.corner[TR],
        b.tangent[RIGHT][0],
        b.tangent[RIGHT][1],
        b.corner[BR],
        1.0 - v,
    );
    let mut out = [0.0f32; 2];
    for k in 0..2 {
        let ruled_v = (1.0 - v) * bottom[k] + v * top[k];
        let ruled_u = (1.0 - u) * left[k] + u * right[k];
        let bilinear = (1.0 - u) * (1.0 - v) * b.corner[BL][k]
            + u * (1.0 - v) * b.corner[BR][k]
            + (1.0 - u) * v * b.corner[TL][k]
            + u * v * b.corner[TR][k];
        out[k] = ruled_v + ruled_u - bilinear;
    }
    out
}

#[cfg(test)]
#[path = "coons_tests.rs"]
mod tests;
