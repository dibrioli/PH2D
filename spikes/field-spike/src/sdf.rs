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

/// `−a` — o **complemento** do sólido. Dentro vira fora, e a distância troca de sinal.
fn neg(a: &Tree) -> Tree {
    Tree::constant(0.0) - a.clone()
}

/// **Deslocamento** (offset) da superfície por `r`. É o operador mais barato que existe: uma
/// subtração.
///
/// ⚠️ **É ele que arredonda ARESTA EXTERNA (convexa)** — e a razão é geométrica, não algébrica:
/// deslocar a superfície para fora por `r` faz cada quina convexa virar um arco de raio
/// **exatamente** `r`, enquanto uma quina côncava permanece viva. Deslocar para dentro faz o
/// inverso. *É por isso que arredondar por fora e por dentro são operações diferentes, e não a
/// mesma com sinal trocado.*
///
/// Ele **cresce** a peça por `r`. Quem quiser manter o tamanho encolhe a fonte antes — é o que
/// [`sd_round_box`] faz, e é a receita canônica.
pub fn offset(a: &Tree, r: f64) -> Tree {
    a.clone() - Tree::constant(r)
}

/// Caixa de meia-aresta `half` com as arestas arredondadas em raio `r`, **do tamanho pedido**.
///
/// A receita é `caixa(half − r)` deslocada de `r`: encolhe-se a fonte exatamente o quanto o
/// deslocamento vai crescer. O resultado é **distância exata**, e o raio entregue é o raio pedido —
/// o que [`crate::probe::outer_radius_error`] verifica.
pub fn sd_round_box(half: f64, r: f64) -> Tree {
    offset(&sd_box(half - r, half - r, half - r), r)
}

/// Cilindro em Z com o **aro das tampas** arredondado em `rr`, mantendo raio e altura.
pub fn sd_round_cylinder_z(radius: f64, h: f64, rr: f64) -> Tree {
    offset(&sd_capped_cylinder_z(radius - rr, h - rr), rr)
}
/// Idem, eixo X.
pub fn sd_round_cylinder_x(radius: f64, h: f64, rr: f64) -> Tree {
    offset(&sd_capped_cylinder_x(radius - rr, h - rr), rr)
}
/// Idem, eixo Y.
pub fn sd_round_cylinder_y(radius: f64, h: f64, rr: f64) -> Tree {
    offset(&sd_capped_cylinder_y(radius - rr, h - rr), rr)
}

/// Intersecção com a aresta (convexa) arredondada em `r`.
///
/// ⚠️ **Derivada por De Morgan, não copiada:** intersectar é unir os complementos e complementar o
/// resultado — `A ∩ B = ¬(¬A ∪ ¬B)`. Logo `intersection_round(a,b,r) = −union_round(−a,−b,r)`, e o
/// arredondamento acompanha a dualidade sem precisar de fórmula nova. Uma fórmula a mais seria uma
/// segunda resposta à mesma pergunta, com uma chance a mais de divergir.
pub fn intersection_round(a: &Tree, b: &Tree, r: f64) -> Tree {
    neg(&union_round(&neg(a), &neg(b), r))
}

/// Subtração (`a` menos `b`) com a **aresta do corte** arredondada em `r`.
///
/// Retirar `b` de `a` é intersectar `a` com o complemento de `b` — daí sair de graça da mesma
/// dualidade acima. É a operação que dá a boca arredondada de um furo, e é das mais usadas em arte
/// hard-surface.
pub fn difference_round(a: &Tree, b: &Tree, r: f64) -> Tree {
    intersection_round(a, &neg(b), r)
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
