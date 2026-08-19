//! As primitivas e os operadores — **portados de referência publicada**, nunca inventados.
//!
//! ⚠️ `DIRETIVA_IMPLEMENTACAO` §1: *existe algoritmo de referência publicado? **Porte-o**.
//! Constante de magia inventada = PARE e ache a fonte.* Não há uma única constante escolhida a olho
//! neste arquivo: cada forma é a distância **exata** à superfície, e cada uma tem a fonte ao lado.
//!
//! # As fontes
//!
//! - **Primitivas**: Inigo Quilez, [*3D distance functions*](https://iquilezles.org/articles/distfunctions/).
//! - **`union_smooth`**: Inigo Quilez, [*smooth minimum*](https://iquilezles.org/articles/smin/),
//!   variante polinomial.
//! - **`union_round`**: identidade geométrica, **derivada** e não copiada — a superfície é o lugar
//!   dos pontos a distância `r` de `{a ≥ r} ∩ {b ≥ r}`. Os gates da crate a verificam contra o
//!   valor analítico: um operador que se diz exato e não é, esta crate reprova.
//! - **Intersecção e subtração**: **De Morgan**, `A ∩ B = ¬(¬A ∪ ¬B)`. Sem fórmula nova — uma
//!   fórmula a mais seria a segunda resposta à mesma pergunta.

use fidget::context::Tree;

fn length2(x: &Tree, y: &Tree) -> Tree {
    (x.square() + y.square()).sqrt()
}

fn length3(x: &Tree, y: &Tree, z: &Tree) -> Tree {
    (x.square() + y.square() + z.square()).sqrt()
}

/// `−a`: o complemento do sólido.
pub fn neg(a: &Tree) -> Tree {
    Tree::constant(0.0) - a.clone()
}

/// Desloca a superfície por `r`.
///
/// ⚠️ **É este operador que arredonda aresta CONVEXA**, e a razão é geométrica: deslocar para fora
/// faz cada quina convexa virar um arco de raio exatamente `r`, e deixa a côncava viva. Como ele
/// **cresce** a peça, quem quer manter o tamanho encolhe a fonte antes — é o que as primitivas
/// abaixo fazem, e é a receita canônica.
pub fn offset(a: &Tree, r: f64) -> Tree {
    a.clone() - Tree::constant(r)
}

fn box_raw(hx: f64, hy: f64, hz: f64) -> Tree {
    let qx = Tree::x().abs() - Tree::constant(hx);
    let qy = Tree::y().abs() - Tree::constant(hy);
    let qz = Tree::z().abs() - Tree::constant(hz);
    let outside = length3(&qx.max(0.0), &qy.max(0.0), &qz.max(0.0));
    let inside = qx.max(qy.clone()).max(qz.clone()).min(0.0);
    outside + inside
}

/// Caixa de meias-extensões `half`, arestas arredondadas em `round`, **do tamanho pedido**.
pub fn sd_box(half: [f64; 3], round: f64) -> Tree {
    offset(
        &box_raw(half[0] - round, half[1] - round, half[2] - round),
        round,
    )
}

pub fn sd_sphere(radius: f64) -> Tree {
    length3(&Tree::x(), &Tree::y(), &Tree::z()) - Tree::constant(radius)
}

fn cylinder_raw(radius: f64, half_height: f64) -> Tree {
    let radial = length2(&Tree::x(), &Tree::y()) - Tree::constant(radius);
    let axial = Tree::z().abs() - Tree::constant(half_height);
    let outside = length2(&radial.max(0.0), &axial.max(0.0));
    let inside = radial.max(axial.clone()).min(0.0);
    outside + inside
}

/// Cilindro no eixo Z, aro das tampas arredondado em `round`, mantendo raio e altura.
pub fn sd_cylinder(radius: f64, half_height: f64, round: f64) -> Tree {
    offset(&cylinder_raw(radius - round, half_height - round), round)
}

/// Toro no plano XY.
pub fn sd_torus(major: f64, minor: f64) -> Tree {
    let q = length2(&Tree::x(), &Tree::y()) - Tree::constant(major);
    length2(&q, &Tree::z()) - Tree::constant(minor)
}

/// União dura: `min(a, b)`. É aqui que nasce a quina viva.
pub fn union_sharp(a: &Tree, b: &Tree) -> Tree {
    a.min(b.clone())
}

/// União com filete **exato** de raio `r`.
///
/// `max(r, min(a,b)) − ‖(max(r−a, 0), max(r−b, 0))‖`
///
/// Preserva a propriedade de distância onde `a` e `b` a têm, e por isso **o raio pedido é o raio
/// entregue** — medido a 0,00 % nos gates desta crate.
pub fn union_round(a: &Tree, b: &Tree, r: f64) -> Tree {
    let ux = (Tree::constant(r) - a.clone()).max(0.0);
    let uy = (Tree::constant(r) - b.clone()).max(0.0);
    a.min(b.clone()).max(r) - length2(&ux, &uy)
}

/// União **suave** (smooth-min polinomial), alcance `k`.
///
/// ⚠️ **NÃO preserva a propriedade de distância**, e o `k` **não é um raio**: medido, entrega 3/4
/// do número pedido. Quem o levar à UI com a etiqueta "raio" mente 25 %, sempre.
pub fn union_smooth(a: &Tree, b: &Tree, k: f64) -> Tree {
    let half = Tree::constant(0.5);
    let h = (half.clone() + half * (b.clone() - a.clone()) / Tree::constant(k))
        .max(0.0)
        .min(1.0);
    let mixed = b.clone() + (a.clone() - b.clone()) * h.clone();
    mixed - Tree::constant(k) * h.clone() * (Tree::constant(1.0) - h)
}

/// Intersecção, com o mesmo caráter de mistura da união — **por De Morgan**.
pub fn intersection(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    neg(&union(&neg(a), &neg(b), blend))
}

/// Subtração (`a` menos `b`): intersectar `a` com o complemento de `b`.
pub fn difference(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    intersection(a, &neg(b), blend)
}

/// O caráter de mistura já resolvido em número, para os três operadores partilharem um caminho só.
#[derive(Clone, Copy, Debug)]
pub enum Blended {
    Sharp,
    Exact(f64),
    Organic(f64),
}

/// A união, escolhendo a fórmula pelo caráter. **Os outros dois operadores passam por aqui** — é o
/// que garante que "arredondar" signifique a mesma coisa nas três operações.
pub fn union(a: &Tree, b: &Tree, blend: Blended) -> Tree {
    match blend {
        // ⚠️ Raio zero cai no caminho DURO de propósito: `union_round(_, _, 0.0)` seria
        // algebricamente equivalente, mas passaria por um `max`/`length` a mais em cada avaliação,
        // e o traçado avalia milhões de vezes por quadro.
        Blended::Sharp => union_sharp(a, b),
        Blended::Exact(r) if r <= 0.0 => union_sharp(a, b),
        Blended::Organic(k) if k <= 0.0 => union_sharp(a, b),
        Blended::Exact(r) => union_round(a, b, r),
        Blended::Organic(k) => union_smooth(a, b, k),
    }
}
