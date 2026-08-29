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
use std::f64::consts::FRAC_1_SQRT_2;

/// Piso do argumento do `sqrt`, para que o **gradiente** exista em zero.
///
/// ⭐ **`sqrt` tem derivada INFINITA em zero**, e as normas abaixo são somas de `max(q, 0)`: dentro
/// de uma caixa ou de um cilindro **todos** os termos são exatamente zero, então o argumento é zero
/// no interior INTEIRO da peça, e não num ponto isolado. A diferenciação automática devolve ali
/// `0/0`, e quem consome esse `NaN` é a extração da malha: sem normal não há QEF, a célula cai no
/// baricentro das travessias, e a **quina viva** sai serrilhada.
///
/// ⚠️ **Este é o mecanismo por trás do achado da W0** (*"o desvio é igual à fração de célula em que
/// a face cai"*), e ele estava atribuído ao extrator. Duas hipóteses foram medidas e **refutadas**
/// antes desta: o leque da `fidget` e a interpolação linear da travessia. O que a medição fechou foi
/// a aritmética: o desvio era `0,72 × fração`, que é literalmente o baricentro. §21 do doc de
/// resultados.
///
/// ⚠️ **O número é de REPRESENTAÇÃO.** `sqrt(1e-30) = 1e-15`, que é 8 ordens de grandeza abaixo do
/// ULP de um `f32` de ordem 1 (1,19e-7) — logo o VALOR não muda em nenhum bit —, e `1e-30` é normal
/// em `f32` (o mínimo é 1,18e-38), então o piso não vira zero na fita. Abaixo do piso a derivada é a
/// de uma constante, que é **zero e finita**; é isso que se queria.
///
/// ⛔ Não vale trocar por `sqrt(s + ε)`: isso muda o valor em `√ε` em **toda parte**, e um raio de
/// filete deixaria de ser o pedido.
const LENGTH_FLOOR: f64 = 1.0e-30;

/// A raiz com o piso acima. ⚠️ **Toda raiz de uma soma de quadrados desta crate passa por aqui** —
/// uma que não passe reintroduz o `NaN` no gradiente, e o sintoma aparece na malha, três camadas
/// abaixo, como uma quina serrilhada.
pub(crate) fn safe_sqrt(s: Tree) -> Tree {
    s.max(LENGTH_FLOOR).sqrt()
}

fn length2(x: &Tree, y: &Tree) -> Tree {
    safe_sqrt(x.square() + y.square())
}

fn length3(x: &Tree, y: &Tree, z: &Tree) -> Tree {
    safe_sqrt(x.square() + y.square() + z.square())
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

/// ⭐⭐⭐ **A LEI DAS TRÊS FORMAS DA W101, numa frase:** *um sólido de parede reta é a interseção de
/// uma laje com meias-fatias, e `max` de funções 1-Lipschitz é 1-Lipschitz.*
///
/// # ⚠️ Por que ela existe, e por que NÃO é a fórmula da referência
///
/// O `sdCappedCone` publicado é **exato em toda parte**, e paga por isso com **ramificações**
/// (`(q.y<0)?r1:r2`, e o sinal `(cb.x<0 && ca.y<0)?-1:1`). Esta crate compila para uma fita da
/// `fidget`, e as ramificações que ela tem — `compare`/`and`/`or` — produzem funções
/// **descontínuas**: o gradiente por diferenciação automática deixa de existir na fronteira delas, e
/// quem consome esse gradiente é a extração da malha (sem normal não há QEF) e a marcha. É a mesma
/// razão pela qual o [`LENGTH_FLOOR`] existe, um nível acima.
///
/// ⭐ **O que se perde e o que se ganha, dito com precisão.** `max(a, b)` de duas distâncias exatas
/// é:
/// - **exato na superfície** — o zero de `max` é exatamente a fronteira da interseção;
/// - **exato no interior** — a distância à parede mais próxima é o `max` das perpendiculares;
/// - **um SUBESTIMADOR no exterior**, junto às quinas onde duas paredes não são ortogonais.
///
/// Subestimar é **seguro** para a marcha de esferas (nunca ultrapassa) e custa passos, não
/// correção. E `‖∇f‖ ≤ 1` não é esperança: o máximo de funções 1-Lipschitz é 1-Lipschitz, por
/// definição. ⇒ *o passo da marcha não muda por causa destas formas* — e o gate
/// `every_primitive_honours_the_march` mede-o, forma a forma, derivado de `PrimitiveKind::ALL`.
///
/// ⚠️ **É a MESMA aritmética que o `box_raw` faz**, com uma diferença: ali as três paredes são
/// ortogonais, então o termo exterior (`length` das partes positivas) é exato pelo Pitágoras. Aqui
/// a parede inclina, o Pitágoras deixaria de valer, e por isso o exterior fica no `max`.
///
/// `walls` são as meias-fatias já **normalizadas** (gradiente unitário); `half_height` é a laje em Z.
fn slab_and_walls(walls: Tree, half_height: f64) -> Tree {
    (Tree::z().abs() - Tree::constant(half_height)).max(walls)
}

/// A meia-fatia da parede inclinada de um cone, **normalizada**: `(ρ − a − m·z)/√(1+m²)`.
///
/// `radial` é a coordenada radial da secção (o `length2(x,y)` de um cone, o `|x|` de uma parede
/// plana), e a reta vai de `bottom` em `z = −h` a `top` em `z = +h`.
fn tapered_wall(radial: &Tree, bottom: f64, top: f64, half_height: f64) -> Tree {
    let a = (bottom + top) * 0.5;
    let m = (top - bottom) / (2.0 * half_height);
    (radial.clone() - Tree::constant(a) - Tree::z() * Tree::constant(m))
        / Tree::constant((1.0 + m * m).sqrt())
}

/// Cone reto no eixo Z, de `bottom` a `top`, com o aro arredondado em `round`.
///
/// ⚠️ **O recuo da parede NÃO é `bottom − round`** — ele é perpendicular à parede inclinada, e por
/// isso desce `a` de `round·√(1+m²)` mantendo a **mesma inclinação**. Escrever `bottom − round`
/// devolveria um cone raso maior do que o pedido, e o erro cresce com a inclinação. A conta vive
/// numa porta só ([`ph2d_field::radius::cone_round_limit`] é a irmã que a valida).
pub fn sd_cone(bottom: f64, top: f64, half_height: f64, round: f64) -> Tree {
    let h = half_height - round;
    let a = (bottom + top) * 0.5;
    let m = (top - bottom) / (2.0 * half_height);
    let a2 = a - round * (1.0 + m * m).sqrt();
    // A inclinação é preservada, então as pontas da reta recuada saem dela e da altura nova.
    let (b2, t2) = (a2 - m * h, a2 + m * h);
    let radial = length2(&Tree::x(), &Tree::y());
    offset(&slab_and_walls(tapered_wall(&radial, b2, t2, h), h), round)
}

/// Cápsula no eixo Z: o segmento de `−half_height` a `+half_height`, engrossado em `radius`.
///
/// ⭐ **Exata em toda parte, e sem uma ramificação** — a distância a um segmento é a distância ao
/// ponto dele mais próximo, e «o mais próximo» é o `z` **preso** ao intervalo, que é
/// `min(max(z, −h), h)`. É a única das três da W101 que não perde nada.
pub fn sd_capsule(radius: f64, half_height: f64) -> Tree {
    let clamped = Tree::z().max(-half_height).min(half_height);
    length3(&Tree::x(), &Tree::y(), &(Tree::z() - clamped)) - Tree::constant(radius)
}

/// Prisma regular de `sides` lados no eixo Z, possivelmente **ESTREITADO** — `bottom` e `top` são os
/// **circunraios** das duas pontas.
///
/// ⭐ **`sides` meias-fatias, uma por parede**, e cada uma é a **mesma reta inclinada** do cone
/// ([`tapered_wall`]) medida numa direção fixa em vez do raio. ⚠️ A quina fica na direção de `2πk/n`
/// e a parede é perpendicular a `π/n + 2πk/n`, à distância **apótema** = `raio·cos(π/n)`: uma parede
/// na direção da quina daria um polígono rodado de meio setor.
///
/// ⭐⭐ **Com `top == bottom` é o prisma; com `top == 0` é uma PIRÂMIDE** — a mesma fórmula, e é
/// isso que faz a pirâmide não ser uma segunda resposta à mesma pergunta.
pub fn sd_prism(sides: u32, bottom: f64, top: f64, half_height: f64, round: f64) -> Tree {
    let n = sides.max(3);
    let k = f64::from(std::f32::consts::PI / n as f32).cos();
    // O recuo do filete é perpendicular à parede inclinada — a conta do [`sd_cone`], no apótema.
    let (a, m) = (
        (bottom + top) * 0.5 * k,
        (top - bottom) * k / (2.0 * half_height),
    );
    let h = half_height - round;
    let a2 = a - round * (1.0 + m * m).sqrt();
    let (b2, t2) = (a2 - m * h, a2 + m * h);
    let mut walls: Option<Tree> = None;
    for i in 0..n {
        let ang = std::f64::consts::TAU * (f64::from(i) + 0.5) / f64::from(n);
        let radial = Tree::x() * Tree::constant(ang.cos()) + Tree::y() * Tree::constant(ang.sin());
        let d = tapered_wall(&radial, b2, t2, h);
        walls = Some(walls.map_or_else(|| d.clone(), |w: Tree| w.max(d.clone())));
    }
    let walls = walls.unwrap_or_else(|| Tree::constant(0.0));
    offset(&slab_and_walls(walls, h), round)
}

/// Cunha: a caixa de meias-extensões `half` cortada pelo plano que liga `(−hx, +hz)` a `(+hx, −hz)`.
///
/// ⭐ **O plano passa pela ORIGEM** — o ponto médio daqueles dois é o centro do nó —, e por isso ele
/// é `(hz·x + hx·z)/√(hx²+hz²)` sem termo constante.
///
/// ⚠️ **`max` da caixa com o plano**: exacto na superfície e no interior, e um subestimador junto às
/// quinas onde a face recta encontra a inclinada — a mesma troca do [`sd_cone`], com o mesmo
/// `‖∇f‖ ≤ 1` por definição.
pub fn sd_wedge(half: [f64; 3], round: f64) -> Tree {
    let (hx, hy, hz) = (half[0], half[1], half[2]);
    let d = (hx * hx + hz * hz).sqrt();
    // Degenerada (uma meia-extensão a zero) não chega aqui: o documento recusa.
    let (nx, nz) = if d > f64::MIN_POSITIVE {
        (hz / d, hx / d)
    } else {
        (1.0, 0.0)
    };
    // ⚠️ A normal já é unitária, então recuar o plano de `round` é subtrair `round` — e não
    // `round·√(1+m²)`: aquele factor é o do cone, onde a coordenada não estava normalizada.
    let plano =
        Tree::x() * Tree::constant(nx) + Tree::z() * Tree::constant(nz) - Tree::constant(round);
    offset(
        &box_raw(hx - round, hy - round, hz - round).max(plano),
        round,
    )
}

/// Arco de toro no plano XY, centrado no `+X`, com abertura total `angle`.
///
/// ⭐⭐ **A escolha entre interseção e união é feita ao MONTAR a árvore**, em Rust — não com uma
/// ramificação dentro do campo. Até meia volta o sector é a interseção de dois semiplanos; acima
/// dela é a **união** deles (o complemento de um sector estreito). ⚠️ Uma `compare` da `fidget` daria
/// uma função **descontínua**, e o gradiente por diferenciação automática deixaria de existir na
/// fronteira dela — é a razão pela qual toda esta crate evita ramificar.
///
/// ⚠️ `angle >= 2π` devolve o toro inteiro: não há dois semiplanos que exprimam «tudo», e inventar
/// um corte de `2π − ε` deixaria uma fenda invisível que o artista descobria ao exportar.
pub fn sd_torus_arc(major: f64, minor: f64, angle: f64) -> Tree {
    let torus = sd_torus(major, minor);
    if angle >= std::f64::consts::TAU {
        return torus;
    }
    let h = angle * 0.5;
    let (s, c) = (h.sin(), h.cos());
    // Dentro do sector os dois são ≤ 0.
    let n1 = Tree::x() * Tree::constant(-s) + Tree::y() * Tree::constant(c);
    let n2 = Tree::x() * Tree::constant(-s) + Tree::y() * Tree::constant(-c);
    let setor = if angle <= std::f64::consts::PI {
        n1.max(n2)
    } else {
        n1.min(n2)
    };
    torus.max(setor)
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

/// ⭐⭐⭐ **União com CHANFRO de alcance `r`** (W99) — o corte reto a 45°, em vez do arco.
///
/// `min(min(a, b), (a + b − r) · √½)`
///
/// # ⚠️ Por que ela é UMA linha, e por que isso não é sorte
///
/// O plano do chanfro num canto de 90° é `a + b = r`, e a distância de um ponto a ele é
/// `(a + b − r)/√2` — **exacta**, não aproximada. O `min` com o canto vivo é o que a limita à região
/// onde ela de facto é a superfície mais próxima. *No CAD, filete e chanfro são duas máquinas com
/// modos de falha diferentes; aqui são a mesma conta com um termo trocado, e nenhuma pode falhar.*
///
/// # ⚠️ Ela é sempre um MINORANTE, e a marcha depende disso
///
/// O resultado é `min(min(a,b), …)`, logo **nunca maior** que `min(a, b)` — que já é um minorante
/// da distância à união. ⇒ andar o valor do campo continua a ser seguro, mesmo onde o termo do
/// chanfro tem gradiente acima de `1` (ele tem, quando as duas normais se alinham). *Um passo é
/// seguro porque o valor é menor que a distância, não porque o gradiente é unitário.*
pub fn union_chamfer(a: &Tree, b: &Tree, r: f64) -> Tree {
    let corte = (a.clone() + b.clone() - Tree::constant(r)) * Tree::constant(FRAC_1_SQRT_2);
    a.min(b.clone()).min(corte)
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
    /// ⭐⭐⭐ **O CHANFRO** (W99) — o corte reto a 45º. Ver [`union_chamfer`].
    Chamfer(f64),
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
        Blended::Chamfer(r) if r <= 0.0 => union_sharp(a, b),
        Blended::Organic(k) if k <= 0.0 => union_sharp(a, b),
        Blended::Exact(r) => union_round(a, b, r),
        Blended::Chamfer(r) => union_chamfer(a, b, r),
        Blended::Organic(k) => union_smooth(a, b, k),
    }
}
