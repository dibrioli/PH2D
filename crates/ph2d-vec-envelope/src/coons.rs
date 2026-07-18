//! O gesto **4 curvas de lado** (ADR-0129 §4, Fatia D): a gaiola cujos LADOS DOBRAM, como um mapa
//! [`Warp`]. É o *Mesh Warp* do Affinity / o *Warp* do Photoshop.
//!
//! # O patch de Coons, e por que o termo bilinear existe
//!
//! Dadas as 4 curvas de bordo, a superfície é a soma das duas **réguas** (interpolação linear entre
//! os bordos opostos, uma por eixo) **menos** o patch bilinear dos 4 cantos:
//!
//! ```text
//! S(u,v) = (1−v)·B(u) + v·T(u) + (1−u)·L(v) + u·R(v)
//!        − [(1−u)(1−v)·BL + u(1−v)·BR + u·v·TR + (1−u)v·TL]
//! ```
//!
//! O termo negativo não é correção estética: cada régua já entrega os cantos por conta própria, então
//! somá-las os conta **duas vezes**. Subtrair o bilinear cancela a duplicata *exatamente*, e é isso
//! que faz `S(u,0) = B(u)` ao bit — a garantia de que **o bordo desenhado é o bordo do mapa**. Sem
//! ela a alça não pousaria onde o artista a soltou.
//!
//! # Por que ele NÃO substitui o [`QuadWarp`](crate::QuadWarp)
//!
//! Com os 4 lados retos, este patch é **bilinear** — e bilinear **não é** projetivo: sob bilinear uma
//! reta interior vira parábola, sob homografia toda reta continua reta. Os dois concordam nos 4
//! cantos e divergem no miolo, então **são dois gestos**, exatamente como a tabela do ADR-0129 §4 os
//! lista (e como Photoshop separa *Distort* de *Warp*). Em **repouso** — gaiola no retângulo-fonte,
//! lados retos — os dois são a **identidade**, então a escolha do gesto só é visível depois de
//! deformar. Há um par de gates sobre exatamente isso (concordam em repouso · divergem fora dele).
//!
//! # A reta entra na forma CANÔNICA (⅓, ⅔)
//!
//! Um lado reto é uma cúbica degenerada, e a degenerada `(P0,P0,P3,P3)` **não** é afim em `t` — a
//! `ph2d-vec-blend` já pagou esse preço (as intermediárias ondulavam). Aqui a consequência seria
//! pior: o repouso deixaria de ser a identidade *exata*. [`rest_edges`] emite (⅓, ⅔), que é afim, e
//! é a **porta única** de "os lados são retos".

use crate::Warp;

/// Piso de degenerescência — o mesmo do [`crate::QuadWarp`], e com a mesma ressalva de escala
/// absoluta (a gaiola opera em coordenadas de mundo do documento).
const EPS: f64 = 1e-12;

/// Resolução do amostrador de dobra ([`CoonsWarp::folds`]) por eixo: `FOLD_GRID+1` amostras.
///
/// ⚠️ **É uma AMOSTRAGEM, e a diferença para o guard do Quad importa.** A convexidade dos cantos tem
/// um *teorema* atrás (uma homografia de retângulo para quad estritamente convexo não põe o horizonte
/// dentro — ADR-0129 §5); para um patch de Coons não existe critério fechado equivalente, então a
/// pergunta *"este mapa dobra?"* é respondida por grade. Uma dobra menor que a célula escapa. 16 dá
/// 289 avaliações de jacobiana por frame de arrasto (custo desprezível) e uma célula de ~6% do lado —
/// bem abaixo de qualquer dobra que a mão produza numa alça.
const FOLD_GRID: usize = 16;

/// Uma cúbica de bordo, em pontos de controle — orientada no sentido do eixo que ela parametriza.
#[derive(Clone, Copy, Debug)]
struct Side([[f64; 2]; 4]);

impl Side {
    /// `t ∈ [0,1]` pelo esquema de Bernstein (de Casteljau expandido, sem alocação).
    fn eval(&self, t: f64) -> [f64; 2] {
        let s = 1.0 - t;
        let (a, b, c, d) = (s * s * s, 3.0 * s * s * t, 3.0 * s * t * t, t * t * t);
        let p = &self.0;
        [
            a * p[0][0] + b * p[1][0] + c * p[2][0] + d * p[3][0],
            a * p[0][1] + b * p[1][1] + c * p[2][1] + d * p[3][1],
        ]
    }

    /// A derivada `dP/dt` — fechada (a hodógrafa quadrática), nunca por diferença finita: a espinha
    /// exige que a jacobiana seja a derivada REAL do `map`, senão o fit não converge
    /// ([`Warp::jacobian`]).
    fn deriv(&self, t: f64) -> [f64; 2] {
        let s = 1.0 - t;
        let (a, b, c) = (3.0 * s * s, 6.0 * s * t, 3.0 * t * t);
        let p = &self.0;
        [
            a * (p[1][0] - p[0][0]) + b * (p[2][0] - p[1][0]) + c * (p[3][0] - p[2][0]),
            a * (p[1][1] - p[0][1]) + b * (p[2][1] - p[1][1]) + c * (p[3][1] - p[2][1]),
        ]
    }
}

/// Os 2 pontos de controle interiores de cada lado, na ordem dos cantos: `edges[i]` vai de
/// `corners[i]` a `corners[(i+1) % 4]`.
pub type CageEdges = [[[f64; 2]; 2]; 4];

/// Os lados **RETOS** de uma gaiola, na forma canônica (⅓, ⅔) do segmento.
///
/// **Porta única de "os lados são retos".** Três chamadores: o nascimento do envelope, a troca para o
/// gesto Perspective, e o arrasto de canto NO Perspective (que tem de manter o invariante — em
/// Perspective os lados *são* retos, então o que está guardado não pode dizer outra coisa). Duas
/// cópias divergiriam e o repouso deixaria de ser a identidade num dos caminhos.
#[must_use]
pub fn rest_edges(corners: &[[f64; 2]; 4]) -> CageEdges {
    std::array::from_fn(|i| {
        let a = corners[i];
        let b = corners[(i + 1) % 4];
        let d = [b[0] - a[0], b[1] - a[1]];
        [
            [a[0] + d[0] / 3.0, a[1] + d[1] / 3.0],
            [a[0] + d[0] * 2.0 / 3.0, a[1] + d[1] * 2.0 / 3.0],
        ]
    })
}

/// O gesto **Mesh** como [`Warp`]: normaliza o ponto do bbox-fonte para o quadrado unitário e aplica
/// o patch de Coons das 4 curvas de bordo.
///
/// **Ordem dos cantos:** `[BL, BR, TR, TL]`, a mesma do [`QuadWarp`](crate::QuadWarp). Os lados
/// `edges[2]` (TR→TL) e `edges[3]` (TL→BL) são guardados no sentido do polígono e **invertidos** aqui
/// para parametrizarem `u` e `v` crescentes — a inversão é exata (trocar a ordem dos 4 controles).
#[derive(Clone, Copy, Debug)]
pub struct CoonsWarp {
    origin: [f64; 2],
    size: [f64; 2],
    corners: [[f64; 2]; 4],
    /// `[bottom(u), right(v), top(u), left(v)]`, já orientados no sentido crescente do seu eixo.
    sides: [Side; 4],
}

impl CoonsWarp {
    /// `None` se o bbox-fonte for degenerado (largura ou altura ~0). ⚠️ Ao contrário do
    /// [`QuadWarp::new`](crate::QuadWarp::new), **não há gaiola-destino degenerada a recusar aqui**:
    /// o patch de Coons é definido para qualquer bordo. O que pode dar errado é ele **dobrar**, e
    /// isso é pergunta separada ([`CoonsWarp::folds`]) porque a resposta é amostrada, não fechada.
    #[must_use]
    pub fn new(
        origin: [f64; 2],
        size: [f64; 2],
        corners: [[f64; 2]; 4],
        edges: &CageEdges,
    ) -> Option<Self> {
        if size[0].abs() < EPS || size[1].abs() < EPS {
            return None;
        }
        let [bl, br, tr, tl] = corners;
        Some(Self {
            origin,
            size,
            corners,
            sides: [
                Side([bl, edges[0][0], edges[0][1], br]),
                Side([br, edges[1][0], edges[1][1], tr]),
                // TR→TL guardado, TL→TR consumido: os 4 controles ao contrário.
                Side([tl, edges[2][1], edges[2][0], tr]),
                // TL→BL guardado, BL→TL consumido.
                Side([bl, edges[3][1], edges[3][0], tl]),
            ],
        })
    }

    /// O patch em coordenadas do quadrado unitário.
    fn patch(&self, u: f64, v: f64) -> [f64; 2] {
        let (b, r, t, l) = (
            self.sides[0].eval(u),
            self.sides[1].eval(v),
            self.sides[2].eval(u),
            self.sides[3].eval(v),
        );
        std::array::from_fn(|k| {
            let ruled_v = (1.0 - v) * b[k] + v * t[k];
            let ruled_u = (1.0 - u) * l[k] + u * r[k];
            ruled_v + ruled_u - self.bilinear(u, v, k)
        })
    }

    /// O patch bilinear dos 4 cantos, na componente `k` — o termo que cancela a contagem dupla.
    fn bilinear(&self, u: f64, v: f64, k: usize) -> f64 {
        let [bl, br, tr, tl] = self.corners;
        (1.0 - u) * (1.0 - v) * bl[k]
            + u * (1.0 - v) * br[k]
            + u * v * tr[k]
            + (1.0 - u) * v * tl[k]
    }

    /// A jacobiana `d(x,y)/d(u,v)` do patch, em ordem de linha — derivando a expressão termo a termo.
    fn patch_jacobian(&self, u: f64, v: f64) -> [[f64; 2]; 2] {
        let (b, r, t, l) = (
            self.sides[0].eval(u),
            self.sides[1].eval(v),
            self.sides[2].eval(u),
            self.sides[3].eval(v),
        );
        let (db, dr, dt, dl) = (
            self.sides[0].deriv(u),
            self.sides[1].deriv(v),
            self.sides[2].deriv(u),
            self.sides[3].deriv(v),
        );
        let [bl, br, tr, tl] = self.corners;
        std::array::from_fn(|k| {
            // ∂/∂u: as réguas em `u` derivam pelas curvas; a régua em `v` derivou para R − L.
            let du = (1.0 - v) * db[k] + v * dt[k] + (r[k] - l[k])
                - ((1.0 - v) * (br[k] - bl[k]) + v * (tr[k] - tl[k]));
            let dv = (t[k] - b[k]) + (1.0 - u) * dl[k] + u * dr[k]
                - ((1.0 - u) * (tl[k] - bl[k]) + u * (tr[k] - br[k]));
            [du, dv]
        })
    }

    /// **O mapa dobra?** — `true` se o determinante da jacobiana muda de sinal (ou zera) em qualquer
    /// amostra da grade.
    ///
    /// É o guard do gesto Mesh, e o irmão de propósito do `QuadWarp::is_convex`: recusar o movimento
    /// que dobraria torna o caso degenerado **inalcançável pela mão**. Dobra em vetor é pior que em
    /// raster — o contorno sai **auto-interseccionado**, que é a saga da lasca da booleana de volta —
    /// e o `break_cusp` do fitter devolve `None` de propósito (ADR-0129, `fit.rs`), então uma dobra
    /// não seria nem sequer bem aproximada.
    ///
    /// Em repouso `det = w·h > 0` em toda a grade, logo o repouso nunca é recusado.
    #[must_use]
    pub fn folds(&self) -> bool {
        for i in 0..=FOLD_GRID {
            for j in 0..=FOLD_GRID {
                let (u, v) = (i as f64 / FOLD_GRID as f64, j as f64 / FOLD_GRID as f64);
                let j2 = self.patch_jacobian(u, v);
                let det = j2[0][0] * j2[1][1] - j2[0][1] * j2[1][0];
                if det <= 0.0 {
                    return true;
                }
            }
        }
        false
    }

    /// Coordenada normalizada `(u,v)` do ponto de mundo no quadrado unitário do bbox-fonte.
    fn uv(&self, p: [f64; 2]) -> (f64, f64) {
        (
            (p[0] - self.origin[0]) / self.size[0],
            (p[1] - self.origin[1]) / self.size[1],
        )
    }
}

impl Warp for CoonsWarp {
    fn map(&self, p: [f64; 2]) -> [f64; 2] {
        let (u, v) = self.uv(p);
        self.patch(u, v)
    }

    fn jacobian(&self, p: [f64; 2]) -> [[f64; 2]; 2] {
        let (u, v) = self.uv(p);
        let j = self.patch_jacobian(u, v);
        // Regra da cadeia com a normalização afim `diag(1/w, 1/h)` — cada coluna divide pela sua
        // dimensão, como no `QuadWarp`.
        [
            [j[0][0] / self.size[0], j[0][1] / self.size[1]],
            [j[1][0] / self.size[0], j[1][1] / self.size[1]],
        ]
    }
}

/// **Esta gaiola dobraria?** — a pergunta do guard do gesto, sem exigir o domínio-fonte.
///
/// A normalização `(u,v)` é um afim de escala **positiva** (`diag(1/w, 1/h)`, com `w,h > 0` porque a
/// bbox-união é não-degenerada), e um afim de escala positiva multiplica o determinante por um número
/// positivo: **não muda o sinal**. Logo dobrar é propriedade só da gaiola, e o guard não precisa
/// saber sobre que arte ela está. É por isso que o gesto pode perguntar isto antes de qualquer
/// consulta à cena.
#[must_use]
pub fn cage_folds(corners: &[[f64; 2]; 4], edges: &CageEdges) -> bool {
    CoonsWarp::new([0.0, 0.0], [1.0, 1.0], *corners, edges).is_some_and(|w| w.folds())
}

#[cfg(test)]
#[path = "coons_tests.rs"]
mod tests;
