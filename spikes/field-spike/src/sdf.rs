//! As primitivas e os operadores — **portados de referência publicada**, nunca inventados.
//!
//! ⚠️ `DIRETIVA_IMPLEMENTACAO` §1: *"existe algoritmo de referência publicado? **Porte-o** antes de
//! escrever a sua versão. Constante de magia inventada = PARE e ache a fonte."* Não há uma única
//! constante escolhida a olho neste arquivo: cada forma abaixo é a distância **exata** à superfície,
//! e cada uma tem a fonte ao lado.
//!
//! # As fontes
//!
//! - **Primitivas** (`sd_box`, `sd_capped_cylinder_z`): Inigo Quilez,
//!   [*3D SDFs / distance functions*](https://iquilezles.org/articles/distfunctions/).
//! - **`union_smooth`**: Inigo Quilez, [*smooth minimum*](https://iquilezles.org/articles/smin/),
//!   variante **polinomial** (a `quadratic`), que é a que a indústria usa por ser barata e C¹.
//! - **`union_round`**: o operador de **união redonda**, que é identidade geométrica e não truque:
//!   a superfície resultante é o lugar dos pontos a distância `r` do conjunto
//!   `{p : a(p) ≥ r} ∩ {p : b(p) ≥ r}`. Daí a forma fechada
//!   `max(r, min(a,b)) − ‖max(r−a, 0), max(r−b, 0)‖`. Aparece assim no `hg_sdf` (Mercury) e no
//!   material do Quilez; ⚠️ **é derivada aqui, não copiada**, e o teste
//!   [`probe::radius_error`](crate::probe::radius_error) a verifica contra o valor analítico —
//!   um operador que se diz exato e não é, este spike reprova.
//!
//! # A diferença que este arquivo existe para tornar mensurável
//!
//! `union_round` **preserva** a propriedade de distância (‖∇f‖ = 1) quando as entradas a têm;
//! `union_smooth` **não preserva** — e é isso que o `probe::eikonal` mede.

use fidget::context::Tree;

/// `‖(x, y, z)‖` — a raiz da soma dos quadrados, sem atalho.
fn length3(x: &Tree, y: &Tree, z: &Tree) -> Tree {
    (x.square() + y.square() + z.square()).sqrt()
}

/// `‖(x, y)‖`.
fn length2(x: &Tree, y: &Tree) -> Tree {
    (x.square() + y.square()).sqrt()
}

/// Caixa centrada na origem, meias-extensões `(hx, hy, hz)`. Distância **exata**, dentro e fora.
///
/// Fonte: Quilez, `sdBox`:
/// ```text
/// vec3 q = abs(p) - b;
/// return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
/// ```
/// O primeiro termo mede a distância por fora (só as componentes positivas contam); o segundo,
/// negativo apenas no interior, mede a distância à face mais próxima.
pub fn sd_box(hx: f64, hy: f64, hz: f64) -> Tree {
    let qx = Tree::x().abs() - Tree::constant(hx);
    let qy = Tree::y().abs() - Tree::constant(hy);
    let qz = Tree::z().abs() - Tree::constant(hz);

    let outside = length3(&qx.max(0.0), &qy.max(0.0), &qz.max(0.0));
    let inside = qx.max(qy.clone()).max(qz.clone()).min(0.0);
    outside + inside
}

/// Cilindro com tampas, eixo em **Z**, raio `r`, meia-altura `h`. Distância exata.
///
/// Fonte: Quilez, `sdCappedCylinder` (aqui com o eixo em Z em vez de Y):
/// ```text
/// vec2 d = abs(vec2(length(p.xy), p.z)) - vec2(r, h);
/// return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
/// ```
pub fn sd_capped_cylinder_z(r: f64, h: f64) -> Tree {
    let radial = length2(&Tree::x(), &Tree::y()) - Tree::constant(r);
    let axial = Tree::z().abs() - Tree::constant(h);

    let outside = length2(&radial.max(0.0), &axial.max(0.0));
    let inside = radial.max(axial.clone()).min(0.0);
    outside + inside
}

/// O mesmo cilindro, eixo em **X**.
pub fn sd_capped_cylinder_x(r: f64, h: f64) -> Tree {
    let radial = length2(&Tree::y(), &Tree::z()) - Tree::constant(r);
    let axial = Tree::x().abs() - Tree::constant(h);

    let outside = length2(&radial.max(0.0), &axial.max(0.0));
    let inside = radial.max(axial.clone()).min(0.0);
    outside + inside
}

/// O mesmo cilindro, eixo em **Y**.
pub fn sd_capped_cylinder_y(r: f64, h: f64) -> Tree {
    let radial = length2(&Tree::x(), &Tree::z()) - Tree::constant(r);
    let axial = Tree::y().abs() - Tree::constant(h);

    let outside = length2(&radial.max(0.0), &axial.max(0.0));
    let inside = radial.max(axial.clone()).min(0.0);
    outside + inside
}

/// Meio-espaço `{p : p·n ≤ d}` sobre o eixo X — usado só pela sonda analítica de raio.
/// A distância a um plano é a coordenada, e ela é exata por definição.
pub fn sd_half_space_x() -> Tree {
    Tree::x()
}

/// Idem, sobre Y.
pub fn sd_half_space_y() -> Tree {
    Tree::y()
}

/// União **dura**: `min(a, b)`. Exata, e é aqui que nasce a quina viva.
pub fn union_sharp(a: &Tree, b: &Tree) -> Tree {
    a.min(b.clone())
}

/// União **redonda exata** de raio `r` — o operador que dá o *look* de produto.
///
/// `max(r, min(a,b)) − ‖(max(r−a, 0), max(r−b, 0))‖`
///
/// ⚠️ Preserva a propriedade de distância quando `a` e `b` a têm: fora da zona de mistura ele é
/// literalmente `min(a,b)`, e dentro dela é a distância ao eixo do filete menos `r`. É por isso que
/// o raio pedido **é** o raio entregue — o que a [`crate::probe::radius_error`] verifica.
pub fn union_round(a: &Tree, b: &Tree, r: f64) -> Tree {
    let ux = (Tree::constant(r) - a.clone()).max(0.0);
    let uy = (Tree::constant(r) - b.clone()).max(0.0);
    a.min(b.clone()).max(r) - length2(&ux, &uy)
}

/// União **suave** (smooth-min polinomial de Quilez), parâmetro `k`.
///
/// ```text
/// h = clamp(0.5 + 0.5*(b − a)/k, 0, 1)
/// mix(b, a, h) − k*h*(1 − h)
/// ```
///
/// ⚠️ **NÃO preserva a propriedade de distância** — o termo `−k·h·(1−h)` afunda o campo dentro da
/// zona de mistura, e o gradiente deixa de ter norma 1. É o caráter "orgânico", e é exatamente a
/// degradação que a [`crate::probe::eikonal`] existe para quantificar.
pub fn union_smooth(a: &Tree, b: &Tree, k: f64) -> Tree {
    let half = Tree::constant(0.5);
    let h = (half.clone() + half * (b.clone() - a.clone()) / Tree::constant(k))
        .max(0.0)
        .min(1.0);
    // mix(b, a, h) = b + (a − b)·h
    let mixed = b.clone() + (a.clone() - b.clone()) * h.clone();
    mixed - Tree::constant(k) * h.clone() * (Tree::constant(1.0) - h)
}
