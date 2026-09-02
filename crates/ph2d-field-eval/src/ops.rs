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

// ⚠️ Ver a nota do [`crate::ops_norm`]: o corte não pode custar uma reescrita nos chamadores.
pub(crate) use crate::ops_norm::{length2, length3, safe_sqrt};

// ⚠️ **O `pub use` é o que mantém `ops::union` e `ops::Blended`** — cortar um arquivo não pode
// custar uma reescrita em cada sítio que o chamava (a mesma lei do `ph2d_field::Primitive`).
pub use crate::ops_bool::{
    Blended, difference, intersection, intersection_round_at, intersection_round_n, union,
    union_chamfer, union_round, union_round_at, union_round_n, union_sharp, union_smooth,
};

// ⚠️ A mesma lei: a caixa mudou-se para o [`crate::ops_box`] e os chamadores não mexeram.
pub use crate::ops_box::{sd_box, sd_box_frame};

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

pub fn sd_sphere(radius: f64) -> Tree {
    length3(&Tree::x(), &Tree::y(), &Tree::z()) - Tree::constant(radius)
}

pub(crate) fn cylinder_raw(radius: f64, half_height: f64) -> Tree {
    let radial = length2(&Tree::x(), &Tree::y()) - Tree::constant(radius);
    let axial = Tree::z().abs() - Tree::constant(half_height);
    let outside = length2(&radial.max(0.0), &axial.max(0.0));
    let inside = radial.max(axial.clone()).min(0.0);
    outside + inside
}

/// Cilindro no eixo Z, aro das tampas arredondado em `round`, mantendo raio e altura.
pub fn sd_cylinder(radius: f64, half_height: f64, round: f64, chamfer: f64) -> Tree {
    if chamfer <= 0.0 {
        // ⭐ **O caminho de sempre, ao bit.**
        return offset(&cylinder_raw(radius - round, half_height - round), round);
    }
    // ⭐ O aro de um cilindro E' a junta de DUAS peças — a parede e a tampa —, e é exactamente a
    // mesma forma que o [`slab_and_walls`] usa para a família das chapas.
    let parede = length2(&Tree::x(), &Tree::y()) - Tree::constant(radius);
    slab_and_walls(
        &parede,
        half_height,
        crate::ops_joint::Edge::square(round, chamfer),
    )
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
/// # ⛔⛔⛔ **E O `round` DESTA FAMÍLIA ERA INERTE — medido na W103, três primitivas depois**
///
/// A receita `offset(max(A, B), r)` com as fontes encolhidas **não arredonda nada**, e a álgebra
/// di-lo numa linha: `{max(A,B) − r < 0}` é `{A < r} ∩ {B < r}` — a interseção das duas peças
/// **dilatadas separadamente**, e não a dilatação da interseção. Cada peça é um semiespaço (uma laje,
/// uma parede): dilatar um semiespaço dá outro semiespaço, **sem canto para arredondar**. O que o
/// recuo tira, o deslocamento repõe — e o aro fica **exatamente tão vivo como estava**.
///
/// ⭐ **Por que funciona na caixa e no cilindro:** ali a fonte é o `box_raw`/`cylinder_raw`, que é a
/// distância **exata** (o termo `length` das partes positivas), e a dilatação de uma distância exata
/// É o corpo com os cantos redondos. *A receita nunca foi «encolher e deslocar»: era «encolher uma
/// distância EXATA e deslocar».*
///
/// `walls` são as meias-fatias já **normalizadas** (gradiente unitário); `half_height` é a laje em Z.
pub(crate) fn slab_and_walls(walls: &Tree, half_height: f64, e: crate::ops_joint::Edge) -> Tree {
    let slab = Tree::z().abs() - Tree::constant(half_height);
    // ⭐⭐⭐ **A PORTA DO CHANFRO DE TODA A FAMÍLIA DAS CHAPAS** (Enio, 2026-08-30: *«em todas as
    // peças temos fillet para as bordas arredondadas mas não temos um slider para chamfer»*).
    //
    // ⭐ O aro de uma chapa é a junta de DUAS peças — a laje e a parede —, e é exactamente aí que o
    // `round` já entrava. Trocar a mistura exacta pela junta composta dá chanfro-e-depois-filete às
    // treze formas desta família **de uma vez**, sem uma linha em cada construtor.
    //
    // ⚠️ **Com `chamfer = 0` isto é o caminho de sempre, ao bit** — ver [`crate::ops_joint`].
    if e.chamfer <= 0.0 {
        return crate::ops_joint::intersection_joint(&slab, walls, e);
    }
    // ⭐ **As DUAS tampas com SINAL, e não o `|z| − h` dobrado** — a dobra tem um vinco em `z = 0`
    // que o plano do chanfro carrega para a superfície quando o filete lá chega: era isso que
    // punha uma costura no EQUADOR de um cilindro (`19,0°`, contra `1,5°` só com filete).
    let tampa = [
        Tree::z() - Tree::constant(half_height),
        -Tree::z() - Tree::constant(half_height),
    ];
    crate::ops_joint::intersection_joint_n(
        &[tampa[0].clone(), tampa[1].clone(), walls.clone()],
        &[
            (tampa[0].clone(), walls.clone()),
            (tampa[1].clone(), walls.clone()),
        ],
        e,
    )
}

/// ⭐⭐⭐ **UMA CHAPA INTEIRA NUMA MISTURA SÓ** — as peças do perfil 2D, as arestas que elas formam,
/// e as duas tampas.
///
/// # ⛔ Ela é a [`slab_and_walls`] sem o encaixe
///
/// A [`slab_and_walls`] recebe o perfil **já composto**, e por isso a mistura do aro herda a costura
/// interna dele e põe-na no aro — é o defeito que o 3.º report do Enio nomeou. Quando o perfil é uma
/// **intersecção** de peças, o chamador tem-nas na mão e pode entregá-las: aí não há composta
/// nenhuma a entrar, e o aro sai tão liso quanto o filete sozinho o faria.
///
/// ⚠️ **Só serve a perfis que são INTERSECÇÃO.** Um perfil feito por **união** (a cruz, a
/// engrenagem, o coração) não é exprimível numa intersecção arredondada, e continua a entrar
/// composto pela [`slab_and_walls`] — está nomeado na catraca do
/// `the_chamfer_never_makes_an_edge_worse_than_the_fillet_alone`.
///
/// ⚠️ **As tampas entram com SINAL**, e cada peça do perfil forma uma aresta com cada uma delas: um
/// perfil de `k` peças dá `k` arestas laterais implícitas mais `2k` de aro.
pub(crate) fn plate_joint_n(
    corpo2d: &[Tree],
    arestas2d: &[(Tree, Tree)],
    half_height: f64,
    e: crate::ops_joint::Edge,
) -> Tree {
    let tampa = [
        Tree::z() - Tree::constant(half_height),
        -Tree::z() - Tree::constant(half_height),
    ];
    let mut corpo = corpo2d.to_vec();
    corpo.extend(tampa.iter().cloned());
    let mut arestas = arestas2d.to_vec();
    for p in corpo2d {
        for t in &tampa {
            arestas.push((p.clone(), t.clone()));
        }
    }
    crate::ops_joint::intersection_joint_n(&corpo, &arestas, e)
}

/// A meia-fatia da parede inclinada de um cone, **normalizada**: `(ρ − a − m·z)/√(1+m²)`.
///
/// `radial` é a coordenada radial da secção (o `length2(x,y)` de um cone, o `|x|` de uma parede
/// plana), e a reta vai de `bottom` em `z = −h` a `top` em `z = +h`.
pub(crate) fn tapered_wall(radial: &Tree, bottom: f64, top: f64, half_height: f64) -> Tree {
    let a = (bottom + top) * 0.5;
    let m = (top - bottom) / (2.0 * half_height);
    (radial.clone() - Tree::constant(a) - Tree::z() * Tree::constant(m))
        / Tree::constant((1.0 + m * m).sqrt())
}

/// Cone reto no eixo Z, de `bottom` a `top`, com o aro arredondado em `round`.
///
/// ⚠️ **As paredes ficam ONDE FORAM AUTORADAS** — nada de encolher e repor. A W101 recuava
/// `a` de `round·√(1+m²)` e deslocava de volta, e a nota dizia que o `max` + `offset` era
/// *«auto-corretivo»*: a silhueta saía exatamente a autorada **para qualquer `round`**. ⛔ Era esse o
/// sintoma de que o filete não fazia nada — ver [`slab_and_walls`].
pub fn sd_cone(bottom: f64, top: f64, half_height: f64, round: f64, chamfer: f64) -> Tree {
    let radial = length2(&Tree::x(), &Tree::y());
    slab_and_walls(
        &tapered_wall(&radial, bottom, top, half_height),
        half_height,
        crate::ops_joint::Edge::square(round, chamfer),
    )
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
pub fn sd_prism(
    sides: u32,
    bottom: f64,
    top: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let n = sides.max(3);
    let beta = std::f64::consts::PI / f64::from(n);
    let k = beta.cos();
    let (b, t) = (bottom * k, top * k);
    let m = (t - b) / (2.0 * half_height);
    // ⭐ **O cosseno entre as normais de duas paredes vizinhas, com a inclinação dentro**: elas são
    // `(cos φ, sin φ, −m)·k`, logo `n_a · n_b = (cos 2β + m²)/(1 + m²)`.
    //
    // ⭐⭐ **É este número que o operador quer, sem passar por ângulo nenhum** — ver
    // [`crate::ops::union_round_at`]. Num hexágono ele dá `−½` (quina obtusa de 120°) e num
    // **triângulo** `+½` (aguda de 60°), e a mesma lei serve os dois.
    let cos_psi = (((2.0 * beta).cos() + m * m) / (1.0 + m * m)).clamp(-1.0, 1.0);
    let parede = |i: u32| {
        let ang = std::f64::consts::TAU * (f64::from(i) + 0.5) / f64::from(n);
        let radial = Tree::x() * Tree::constant(ang.cos()) + Tree::y() * Tree::constant(ang.sin());
        tapered_wall(&radial, b, t, half_height)
    };
    // ⭐⭐⭐ **AS QUINAS LATERAIS ARREDONDAM COM O MESMO RAIO** (W104 — a 2.ª foto do Enio: uma
    // pirâmide com o aro de baixo redondo e as arestas laterais vivas).
    //
    // ⚠️ A W103 fechava as paredes com `max` cru e só arredondava o aro, e o doc chamava-lhe *«a
    // mesma divisão do `Extrude`»*. ⛔ **Não é a mesma:** ali a quina do contorno tem um dono
    // declarado (o editor vetorial, com `Live Corners`), e aqui **ninguém a possui**. *Uma divisão
    // de responsabilidade copiada de outra forma é uma aresta órfã quando o segundo dono não existe.*
    let mut walls: Option<Tree> = None;
    for i in 0..n {
        let d = parede(i);
        walls = Some(walls.map_or_else(
            || d.clone(),
            |w: Tree| {
                crate::ops_joint::intersection_joint(
                    &w,
                    &d,
                    crate::ops_joint::Edge::at(round, chamfer, cos_psi),
                )
            },
        ));
    }
    let walls = walls.unwrap_or_else(|| Tree::constant(0.0));
    if chamfer <= 0.0 {
        return slab_and_walls(
            &walls,
            half_height,
            crate::ops_joint::Edge::square(round, chamfer),
        );
    }
    // ⭐⭐⭐ **AS PAREDES, AS TAMPAS E TODOS OS CHANFROS NUMA MISTURA SÓ** — ver
    // [`crate::ops_joint::intersection_joint_n`]. Fechar as paredes primeiro e o aro depois fazia a
    // mistura do aro receber a composta das paredes, e ela punha a costura lateral **em cima do
    // aro**: medido `21,1°` de giro da normal contra `4,2°` só com filete, todos os pontos duros no
    // aro a `~12°` de uma quina lateral. Com tudo junto: **`4,5°`**.
    let tampa = [
        Tree::z() - Tree::constant(half_height),
        -Tree::z() - Tree::constant(half_height),
    ];
    let paredes: Vec<Tree> = (0..n).map(parede).collect();
    let mut arestas: Vec<(Tree, Tree)> = Vec::new();
    for i in 0..paredes.len() {
        arestas.push((paredes[i].clone(), paredes[(i + 1) % paredes.len()].clone()));
        for t in &tampa {
            arestas.push((paredes[i].clone(), t.clone()));
        }
    }
    let mut corpo = paredes;
    corpo.extend(tampa);
    crate::ops_joint::intersection_joint_n(
        &corpo,
        &arestas,
        crate::ops_joint::Edge::square(round, chamfer),
    )
}

/// Cunha: a caixa de meias-extensões `half` cortada pelo plano que liga `(−hx, +hz)` a `(+hx, −hz)`.
///
/// ⭐ **O plano passa pela ORIGEM** — o ponto médio daqueles dois é o centro do nó —, e por isso ele
/// é `(hz·x + hx·z)/√(hx²+hz²)` sem termo constante.
///
/// ⚠️ **`max` da caixa com o plano**: exacto na superfície e no interior, e um subestimador junto às
/// quinas onde a face recta encontra a inclinada — a mesma troca do [`sd_cone`], com o mesmo
/// `‖∇f‖ ≤ 1` por definição.
pub fn sd_wedge(half: [f64; 3], round: f64, chamfer: f64) -> Tree {
    let (hx, _hy, hz) = (half[0], half[1], half[2]);
    let d = (hx * hx + hz * hz).sqrt();
    // Degenerada (uma meia-extensão a zero) não chega aqui: o documento recusa.
    let (nx, nz) = if d > f64::MIN_POSITIVE {
        (hz / d, hx / d)
    } else {
        (1.0, 0.0)
    };
    // ⛔ **O plano NÃO recua** — ele passa pela origem, e é isso que a primitiva promete. A W102
    // subtraía `round` aqui *«porque a normal já é unitária»*, e subtrair **cresce** um semiespaço:
    // com o `offset` de fora o corte acabava `2·round` fora do sítio, e a peça inteira ficava
    // `+41 %` maior (medido na W103). Ver [`slab_and_walls`], que é o mesmo defeito na outra ponta.
    let plano = Tree::x() * Tree::constant(nx) + Tree::z() * Tree::constant(nz);
    // ⭐ **Os dois números atravessam as DUAS juntas da cunha** — as arestas da caixa e o corte
    // inclinado —, senão chanfrar uma cunha deixaria metade das arestas por chanfrar.
    let e = crate::ops_joint::Edge::square(round, chamfer);
    if chamfer <= 0.0 {
        return crate::ops_joint::intersection_joint(&sd_box(half, round, chamfer), &plano, e);
    }
    // ⭐ **As peças da caixa E o plano do corte, numa mistura só** — ver
    // [`crate::ops_box::box_pieces`]. Passar a caixa já composta punha a costura interna dela na
    // aresta do corte: medido `23,5°` de giro da normal contra `5,9°` só com filete.
    let (mut corpo, mut arestas) =
        crate::ops_box::box_pieces(&Tree::x(), &Tree::y(), &Tree::z(), half, round, chamfer);
    for p in &corpo {
        arestas.push((p.clone(), plano.clone()));
    }
    corpo.push(plano);
    crate::ops_joint::intersection_joint_n(&corpo, &arestas, e)
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
pub fn sd_torus_arc(major: f64, minor: f64, angle: f64, round: f64, chamfer: f64) -> Tree {
    let torus = sd_torus(major, minor);
    if angle >= std::f64::consts::TAU {
        return torus;
    }
    let h = angle * 0.5;
    let (s, c) = (h.sin(), h.cos());
    // Dentro do sector os dois são ≤ 0.
    let n1 = Tree::x() * Tree::constant(-s) + Tree::y() * Tree::constant(c);
    let n2 = Tree::x() * Tree::constant(-s) + Tree::y() * Tree::constant(-c);
    let interseccao = angle <= std::f64::consts::PI;
    let setor = if interseccao {
        n1.clone().max(n2.clone())
    } else {
        n1.clone().min(n2.clone())
    };
    // ⛔⛔ **MEDIDO E RETIRADO (2026-08-30):** dar ao arco as peças em vez do sector composto
    // — os dois semiplanos até meia volta, dois meios-arcos acima dela — foi construído, e a prova
    // de mutação **SOBREVIVEU**: `0,95×` no ponto de trabalho e `5,33×` na saturação, com e sem.
    //
    // ⚠️ A razão é geométrica: a aresta que a composição encaixada estragaria é a do corte contra o
    // **tubo**, e o tubo é uma superfície lisa — não há costura interna para aflorar. *A lei só
    // morde quando a composta tem um vinco a menos de `r` da superfície.* O que sobra na saturação
    // é o filete a ter comido a faceta inteira do chanfro, que é a geometria do pedido.
    // ⭐⭐ **Os DOIS aros do corte arredondam** (W104): a sonda de arestas media `30 %` da superfície
    // deste arco sobre um vinco de `88°`, e ele **não tinha controle de filete nenhum** — era a
    // única forma do catálogo com aresta autorada e sem o slider que a trata.
    crate::ops_joint::intersection_joint(
        &torus,
        &setor,
        crate::ops_joint::Edge::square(round, chamfer),
    )
}

/// União dura: `min(a, b)`. É aqui que nasce a quina viva.
/// O semiplano cuja fronteira passa por `a` e `b`, **negativo do lado esquerdo** de `a → b`.
///
/// ⚠️ A normal é **unitária** — é o que faz o `max` das quatro de um quadrilátero continuar
/// 1-Lipschitz, e é a diferença entre uma distância e um número que só tem o sinal certo.
pub(crate) fn half_plane(a: [f64; 2], b: [f64; 2]) -> Tree {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt();
    // Degenerada não chega aqui: o documento recusa `inner >= outer`, e `points >= 3`.
    let (nx, ny) = if len > f64::MIN_POSITIVE {
        (dy / len, -dx / len)
    } else {
        (1.0, 0.0)
    };
    Tree::x() * Tree::constant(nx) + Tree::y() * Tree::constant(ny)
        - Tree::constant(nx * a[0] + ny * a[1])
}

/// ⭐⭐⭐ **Estrela de `points` pontas puxada em Z** — o **DISCO dos vales unido a uma pipa por
/// ponta**, e cada pipa é um convexo de quatro semiplanos.
///
/// # ⚠️ Por que uma UNIÃO, e por que ela precisa de SOBREPOSIÇÃO
///
/// A lei da W101 (*«sólido de parede reta = laje ∩ meias-fatias»*) constrói **convexos**, e uma
/// estrela não é um. A saída é o `min` — e ela traz uma armadilha que a interseção não tem: `min`
/// de duas peças que se **tocam sem se sobrepor** vale **exatamente zero** na costura, que é um
/// ponto **interior** ao sólido. Um `0` interior é lido como fronteira por quem amostra numa grade,
/// e sai uma superfície fantasma dentro da peça.
///
/// ⛔ A decomposição óbvia — o polígono dos vales **mais** um triângulo por ponta — é exatamente uma
/// **partição**: a base de cada triângulo é uma aresta do polígono, e os dois campos anulam-se lá.
/// ⭐ A cura é a peça de enchimento ser o **disco de raio `inner`**: ele cabe inteiro na estrela (o
/// raio da fronteira nunca desce abaixo do vale) e cobre, com volume, a costura de cada pipa — que
/// é o segmento do centro ao vale. *Duas peças que se sobrepõem num volume não têm costura de
/// campo; duas que se encostam têm-na sempre.*
///
/// # O filete é o do ARO, e o limite dele é a erosão da TAMPA
///
/// Como no prisma, o `round` arredonda a aresta entre a parede e as tampas; as pontas do contorno
/// ficam vivas. A pegada do filete na tampa é exatamente a **erosão 2D** da estrela por `round` — e
/// é por isso que o limite dele é o ponto em que essa erosão deixa de ser uma estrela (a ponta e o
/// vale encontram-se no mesmo raio): ver [`ph2d_field::radius::star_round_limit`].
pub fn sd_star(
    points: u32,
    outer: f64,
    inner: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let n = points.max(3);
    let beta = std::f64::consts::PI / f64::from(n);
    // ⭐ **Os meio-ângulos das duas quinas saem da MESMA aresta** (`lado` é o comprimento dela): na
    // ponta `sin α = inner·sin β/|u|`, que é o mesmo que o
    // [`ph2d_field::radius::star_round_limit`] usa.
    //
    // ⭐⭐ **E o do VALE deriva-se do da ponta, sem um segundo `asin`:** no triângulo
    // `centro–vale–ponta` os ângulos somam `π`, e a metade do entalhe é o suplemento do ângulo em
    // `V` ⇒ `α_vale = α_ponta + β` — ⚠️ a rota pelo `asin(outer·sin β/|u|)` devolve o **suplemento**
    // sempre que o vale passa de `90°`, e uma estrela magra fá-lo.
    //
    // ⚠️ **As duas são quinas de espécie oposta e a lei é UMA:** a ponta é convexa (uma
    // intersecção) e o vale é côncavo (uma união do entalhe), e o que o operador quer nos dois casos
    // é o cosseno das normais exteriores da cunha que está a ser arredondada — ver
    // [`union_round_at`].
    let lado = (outer * outer + inner * inner - 2.0 * outer * inner * beta.cos()).sqrt();
    let alfa_ponta = (inner * beta.sin() / lado).clamp(0.0, 1.0).asin();
    let cos_ponta = -(2.0 * alfa_ponta).cos();
    let cos_vale = -(2.0 * (alfa_ponta + beta)).cos();
    let polar = |r: f64, a: f64| [r * a.cos(), r * a.sin()];
    // ⛔ **E o disco NÃO recua com o filete** — foi tentado e a medição disse que é inerte.
    //
    // A hipótese era boa: a fronteira dele passa exactamente pelos vales, e a curva onde o `min`
    // troca entre ele e as pipas cai dentro do alcance da mistura do aro. ⚠️ Só que, **com a folga
    // do sector no sítio**, recuá-lo `2·round` deixa a leitura da sonda **byte a byte igual**
    // (`1,0 %` · `25,4°` a meio filete, `0,0 %` · `13,6°` no máximo, com e sem). *Uma segunda cura
    // que não move o número é mais uma coisa para manter, não meia cura.*
    let disco = length2(&Tree::x(), &Tree::y()) - Tree::constant(inner);
    let mut pontas: Option<Tree> = None;
    for k in 0..n {
        let phi = std::f64::consts::TAU * f64::from(k) / f64::from(n);
        let tip = polar(outer, phi);
        let (before, after) = (polar(inner, phi - beta), polar(inner, phi + beta));
        // ⭐⭐⭐ **A PONTA é uma quina CONVEXA**, e arredonda-se com o arco exato de raio `round`
        // (W104, a 1.ª foto do Enio).
        let ponta = crate::ops_joint::intersection_joint(
            &half_plane(before, tip),
            &half_plane(tip, after),
            crate::ops_joint::Edge::at(round, chamfer, cos_ponta),
        );
        // ⚠️ E o SECTOR **CORTA A SECO**, de propósito: ele não é uma aresta da peça, é a divisória
        // entre duas pipas vizinhas. Arredondá-lo abriria um sulco **dentro** do sólido.
        //
        // ⭐⭐⭐ **E ELE TEM DE TER FOLGA** (W104-bis). Sem folga, o plano do sector passa **pelo
        // vale**, que é um ponto da SUPERFÍCIE: ali o `max` do sector com a aresta troca de ramo em
        // cima da peça, e as duas pipas que a união vai fundir chegam ao encontro **as duas com um
        // vinco de campo**. Afastando os dois planos de `round`, as pipas passam a **sobrepor-se**
        // no vale em vez de se tocarem, e o único constrangimento activo lá é a aresta a sério.
        //
        // ⚠️ **O tecto da folga é geométrico e a medição bate-o**: os dois planos afastados cruzam-se
        // a `δ/sin β` do centro **do lado oposto** da ponta, e acima de `inner` essa intrusão sai da
        // peça. Com `δ = round` isso está sempre garantido, porque `star_round_limit < inner·sin β`
        // por construção — e a varredura confirma: a `2·round` a forma parte, exactamente onde a
        // conta diz.
        let (a1, a2) = (phi - beta, phi + beta);
        let folga = Tree::constant(round.min(inner * beta.sin()));
        let h1 = Tree::x() * Tree::constant(a1.sin())
            - Tree::y() * Tree::constant(a1.cos())
            - folga.clone();
        let h2 =
            Tree::x() * Tree::constant(-a2.sin()) + Tree::y() * Tree::constant(a2.cos()) - folga;
        let pipa = ponta.max(h1).max(h2);
        // ⭐⭐⭐ **O VALE é uma quina CÔNCAVA**, e uma quina côncava arredonda-se ACRESCENTANDO
        // material no entalhe — o dual de De Morgan do mesmo arco.
        //
        // ⚠️ Ao longo da divisória (do centro ao vale) as duas pipas cobrem os dois lados, então não
        // há para onde acrescentar e nada é acrescentado — o efeito é **só** no vale.
        pontas = Some(pontas.map_or_else(
            || pipa.clone(),
            |w: Tree| {
                crate::ops_joint::union_joint(
                    &w,
                    &pipa,
                    crate::ops_joint::Edge::at(round, chamfer, cos_vale),
                )
            },
        ));
    }
    let pontas = pontas.unwrap_or_else(|| Tree::constant(0.0));
    // ⚠️ **O disco entra por `min` CRU** — ele é enchimento interior e não fronteira, e arredondar
    // contra ele misturaria um raio a mais no vale (a fronteira dele passa exactamente por lá). Ver
    // a nota da costura, acima: com `round = 0` ele continua a ser o que a mata.
    slab_and_walls(
        &pontas.min(disco),
        half_height,
        crate::ops_joint::Edge::square(round, chamfer),
    )
}

/// ⭐⭐⭐ **Elipsóide** — a esfera unitária no espaço escalado, **remedida pelo menor semi-eixo**.
///
/// # ⚠️ A fórmula publicada foi MEDIDA e REJEITADA, e não preterida
///
/// A distância exata a um elipsóide resolve uma **quártica**; por isso a referência (IQ,
/// *distance functions*) publica `k0·(k0−1)/k1`, com `k0 = |p/r|` e `k1 = |p/r²|`. Ela é bem mais
/// **apertada** que esta — logo mais barata de marchar —, e a sonda
/// [`measure_ellipsoid_against_the_published_formula`](../../tests/measure_star_points.rs) mediu as
/// duas onde importa:
///
/// | elipsóide | `f(centro)` correto | a NOSSA | pior `‖∇f‖` | a do IQ | pior `‖∇f‖` |
/// |---|---|---|---|---|---|
/// | 1:1 | −0,450000 | **−0,450000** | 1,0001 | −1,000000 | 1,0002 |
/// | 1:2 | −0,225000 | **−0,225000** | 1,0001 | −1,000000 | **1,3973** |
/// | 1:4 | −0,112500 | **−0,112500** | 1,0001 | −1,000000 | **1,8598** |
/// | 1:8 | −0,056250 | **−0,056250** | 1,0001 | −1,000000 | 1,0001 |
///
/// ⛔ **Duas coisas a reprovam, e as duas são fatais.** O gradiente dela chega a **1,86** — um campo
/// que sobe mais depressa do que a distância faz a marcha de esferas **atravessar a superfície**, que
/// é o único erro que ela não perdoa. E `f(centro)` dá **−1 para qualquer elipsóide**: no centro
/// `k0` e `k1` são os dois zero, o [`LENGTH_FLOOR`] transforma o `0/0` em `1e-15/1e-15`, e o
/// resultado é a constante `−1` — 18× a profundidade real de uma peça de meia-altura `0,056`. *O
/// ponto singular dela é a origem, que é exatamente o ponto que toda grade centrada amostra.*
///
/// ⭐ **Esta é a construção com prova:** `f(p/s)` é `1/min(s)`-Lipschitz, então `f(p/s)·min(s)` é
/// **1-Lipschitz por construção** (medido: `1,0001`, que é o erro da diferença central). O conjunto
/// zero é **exato** (é `Σ(pᵢ/sᵢ)² = 1`, a definição do elipsóide) e a **direção do gradiente também**
/// (`∂ᵢ ∝ pᵢ/sᵢ²` é a normal verdadeira); o que se perde é só a **magnitude**, que fica entre
/// `min(s)/max(s)` e `1`. Subestimar custa **passos de marcha**, nunca correção.
///
/// ⚠️ **E esse é o recurso, medido:** a marcha pelo eixo maior gasta `1` passo numa esfera, `28` a
/// `1:4`, `101` a `1:16`, `324` a `1:64` e `562` a `1:128` — contra o orçamento de `MAX_STEPS = 400`
/// do traçador. ⛔ **Nenhum teto é escrito por causa disto**, e a decisão é deliberada: a forma está
/// **correta** em todos eles (a malha e a exportação não dependem da marcha), e limitar a largura de
/// uma peça porque o previsualizador fica lento é deixar o caminho mais lento definir o produto
/// (`CLAUDE.md` §0). Quem chegar a `1:64` sabe agora que a alavanca é o `MAX_STEPS`.
///
/// ⚠️ Ele **não substitui** a [`ph2d_field::Primitive::Sphere`], que é exata.
pub fn sd_ellipsoid(radii: [f64; 3]) -> Tree {
    let m = radii[0].min(radii[1]).min(radii[2]);
    let over = |t: Tree, r: f64| t * Tree::constant(1.0 / r);
    let unit = length3(
        &over(Tree::x(), radii[0]),
        &over(Tree::y(), radii[1]),
        &over(Tree::z(), radii[2]),
    ) - Tree::constant(1.0);
    unit * Tree::constant(m)
}
