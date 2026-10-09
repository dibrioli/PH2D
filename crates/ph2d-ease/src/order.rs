//! **A POSIÇÃO DE CADA LINHA NA ONDA, `0..1`** — por elemento, sem olhar para o vizinho (é o que a
//! deixa no dispositivo).
//!
//! `Index` · `From Center` · `Random` são as do ciclo 2 (doc 105), ao bit. O ciclo 17 (doc 126
//! §3.2) apendou:
//! - `Grid` — a linha `i` é a célula `(i mod cols, i div cols)`; a onda parte de uma célula
//!   (primeira · centro da grelha · última · uma dada) e a distância é euclidiana, ou só em `x`,
//!   ou só em `y`. Normaliza pela MAIOR distância às células ocupadas, que está num vértice do
//!   invólucro delas — fechada, sem redução. Lei conferida contra o `stagger({grid, from, axis})`
//!   do anime.js (MIT), corrido sobre as nossas grelhas (`tests/fixtures/animejs.txt`).
//! - `From Point` — a distância da POSIÇÃO da linha a um ponto do mundo, sobre o `reach` (onde a
//!   onda chega a `1`). É o «By X / By Y» que o ciclo 2 recusou por pedir um POSTO, sem o posto:
//!   a pergunta deixa de ser «que lugar ocupo» e passa a ser «a que distância estou».

/// A escada do param `order` — append-only, é o wire format.
pub const ORDER_INDEX: i32 = 0;
pub const ORDER_FROM_CENTER: i32 = 1;
pub const ORDER_RANDOM: i32 = 2;
pub const ORDER_GRID: i32 = 3;
pub const ORDER_FROM_POINT: i32 = 4;
/// A última ordem da escada.
pub const ORDER_LAST: i32 = ORDER_FROM_POINT;

/// De onde parte a onda na grelha.
pub const FROM_FIRST: i32 = 0;
pub const FROM_CENTER: i32 = 1;
pub const FROM_LAST: i32 = 2;
pub const FROM_CELL: i32 = 3;

/// Que distância conta.
pub const AXIS_BOTH: i32 = 0;
pub const AXIS_X: i32 = 1;
pub const AXIS_Y: i32 = 2;

/// Os números que a ordem lê (os params do nó, já arredondados onde escolhem um ramo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Order {
    pub kind: i32,
    pub seed: u32,
    /// Colunas da grelha (mínimo `1`).
    pub cols: u32,
    pub from: i32,
    /// A célula de partida com [`FROM_CELL`] (presa à última linha).
    pub cell: u32,
    pub axis: i32,
    pub origin: [f32; 2],
    /// Onde a onda do `From Point` chega a `1`; `≤ 0` ⇒ todos em `0`.
    pub reach: f32,
}

impl Order {
    /// A ordem de sempre: `Index`.
    pub const INDEX: Order = Order {
        kind: ORDER_INDEX,
        seed: 0,
        cols: 10,
        from: FROM_CENTER,
        cell: 0,
        axis: AXIS_BOTH,
        origin: [0.0, 0.0],
        reach: 5.0,
    };
}

/// A distância que conta para o `axis`.
#[must_use]
pub fn axis_dist(dx: f32, dy: f32, axis: i32) -> f32 {
    match axis {
        AXIS_X => dx.abs(),
        AXIS_Y => dy.abs(),
        _ => (dx * dx + dy * dy).sqrt(),
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "uma célula de grelha cabe num f32"
)]
fn grid_cell(k: u32, cols: u32) -> [f32; 2] {
    [(k % cols) as f32, (k / cols) as f32]
}

/// A posição `0..1` da célula `i` numa grelha de `n` ocupadas.
#[expect(
    clippy::cast_precision_loss,
    reason = "uma célula de grelha cabe num f32"
)]
fn grid_raw(o: &Order, i: u32, n: u32) -> f32 {
    let cols = o.cols.max(1);
    let rows = n.div_ceil(cols);
    let origem = match o.from {
        FROM_CENTER => [(cols - 1) as f32 * 0.5, (rows - 1) as f32 * 0.5],
        FROM_LAST => grid_cell(n - 1, cols),
        FROM_CELL => grid_cell(o.cell.min(n - 1), cols),
        _ => [0.0, 0.0],
    };
    let dist = |c: [f32; 2]| axis_dist(c[0] - origem[0], c[1] - origem[1], o.axis);
    // Os vértices do invólucro das células ocupadas: `rows − 1` linhas cheias e a última até
    // `last_col`. A maior distância (euclidiana ou num eixo) está num deles.
    let last_row = (rows - 1) as f32;
    let last_col = ((n - 1) % cols) as f32;
    let largura = if rows > 1 { cols } else { n };
    let mut dmax = dist([0.0, 0.0])
        .max(dist([(largura - 1) as f32, 0.0]))
        .max(dist([0.0, last_row]))
        .max(dist([last_col, last_row]));
    if rows > 1 {
        dmax = dmax.max(dist([(cols - 1) as f32, last_row - 1.0]));
    }
    if dmax > 0.0 {
        dist(grid_cell(i, cols)) / dmax
    } else {
        0.0
    }
}

/// ⭐ **A posição de `i` na onda, `0..1`** — a porta única, lida pela CPU; o [`crate::wgsl_lib!`]
/// faz a mesma conta. `p` é a posição da linha (só o `From Point` a lê).
#[must_use]
pub fn raw_at(o: &Order, i: u32, n: u32, p: [f32; 2]) -> f32 {
    if o.kind == ORDER_FROM_POINT {
        let d = axis_dist(p[0] - o.origin[0], p[1] - o.origin[1], o.axis);
        return if o.reach > 0.0 {
            (d / o.reach).min(1.0)
        } else {
            0.0
        };
    }
    if n <= 1 {
        return 0.0;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "uma contagem de linhas cabe num f32"
    )]
    let last = (n - 1) as f32;
    match o.kind {
        ORDER_FROM_CENTER => {
            #[expect(clippy::cast_precision_loss, reason = "um índice cabe num f32")]
            let d = (2.0 * i as f32 - last).abs();
            d / last
        }
        ORDER_RANDOM => hash01(i, o.seed),
        ORDER_GRID => grid_raw(o, i, n),
        #[expect(clippy::cast_precision_loss, reason = "um índice cabe num f32")]
        _ => i as f32 / last,
    }
}

/// Um `0..1` determinista a partir de `(i, seed)` — o finalizador do splitmix32, em `u32` com
/// `wrapping` para o WGSL fazer a MESMA aritmética.
#[must_use]
pub fn hash01(i: u32, seed: u32) -> f32 {
    let mut x = i
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add(seed.wrapping_mul(0x85EB_CA6B));
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    #[expect(
        clippy::cast_precision_loss,
        reason = "a divisão exacta que o WGSL escreve igual"
    )]
    let r = x as f32 / 4_294_967_296.0;
    r
}
