//! ⭐⭐ **A CAIXA E A MOLDURA** — a família de paredes ortogonais, num arquivo irmão.
//!
//! # Por que ele saiu do [`crate::ops`]
//!
//! O `ops.rs` passou as `700` linhas do gate de LOC da workspace quando o chanfro deixou de ser um
//! plano dobrado e passou a ser **uma aresta por sinal** (3.º report do Enio sobre a feature,
//! 2026-08-30). ⛔ *Split, nunca allowlist* — e o corte é por assunto: aqui está tudo o que
//! responde *«que forma tem uma caixa»*, e a moldura entra com ela porque **cada viga dela É uma
//! caixa**, pela porta da caixa.

use crate::ops::offset;
use crate::ops_norm::length3;
use fidget::context::Tree;

/// A caixa de meias-extensões `h` **em coordenadas dadas** — a fórmula do IQ, com as três
/// coordenadas como argumento em vez de `x`/`y`/`z`.
///
/// ⭐ É isso que deixa o [`sd_box_frame`] escrever uma viga **no espaço dobrado** (`|y|`, `|z|`) sem
/// uma segunda fórmula de caixa: dobrar por `abs` é uma isometria por peça, então a distância no
/// espaço dobrado É a distância às quatro cópias espelhadas — desde que a caixa fique inteira do
/// lado positivo, que é o que a parede da espessura garante.
pub(crate) fn box_at(px: &Tree, py: &Tree, pz: &Tree, h: [f64; 3]) -> Tree {
    let qx = px.abs() - Tree::constant(h[0]);
    let qy = py.abs() - Tree::constant(h[1]);
    let qz = pz.abs() - Tree::constant(h[2]);
    let outside = length3(&qx.max(0.0), &qy.max(0.0), &qz.max(0.0));
    let inside = qx.max(qy.clone()).max(qz.clone()).min(0.0);
    outside + inside
}

/// Caixa de meias-extensões `half`, arestas **chanfradas** em `chamfer` e depois arredondadas em
/// `round`, **do tamanho pedido**.
///
/// # ⭐⭐⭐ Por que a caixa precisa de conta própria, e as chapas não
///
/// A aresta de uma chapa é a junta de **duas** peças (a laje e a parede), e o
/// [`slab_and_walls`] já as tem à mão. A caixa é a intersecção de **três** lajes, e a fórmula do IQ
/// entrega-a como uma norma só — as peças já não existem lá dentro.
///
/// ⛔ **E o chanfro NÃO é recuperável de uma distância euclidiana**: o filete é a dilatação pela
/// bola de `L²` (`f − r`) e o chanfro é a dilatação pelo octaedro de `L¹`, que a norma não sabe
/// desfazer. ⇒ ele entra como **três planos a 45°**, um por par de eixos, que é o que um chanfro de
/// caixa geometricamente **é**.
///
/// ⭐ O plano do par `(i, j)` é `(q_i + q_j + c)·√½` com `q = |p| − half`. No canto ele vale
/// `c/√2 > 0` (o canto é cortado) e a `c` de distância da quina vale **zero** — logo o recuo
/// entregue é exactamente o pedido, em cada uma das duas faces. Os oito vértices saem como facetas
/// triangulares, onde três planos se encontram, sem uma linha a mais.
pub fn sd_box(half: [f64; 3], round: f64, chamfer: f64) -> Tree {
    box_with_edge(&Tree::x(), &Tree::y(), &Tree::z(), half, round, chamfer)
}

/// A caixa **em coordenadas dadas**, com os dois recuos da aresta — a irmã do [`box_at`] que sabe
/// chanfrar.
///
/// ⭐ **Uma porta, dois leitores**: a [`sd_box`] e as três vigas do [`sd_box_frame`]. ⛔ Uma segunda
/// cópia desta receita divergiria no dia em que o chanfro mudasse, e só uma das duas formas o notaria
/// — que é exactamente o que a nota do [`crate::ops_plates::rect_round`] já descreve para o filete.
pub(crate) fn box_with_edge(
    px: &Tree,
    py: &Tree,
    pz: &Tree,
    half: [f64; 3],
    round: f64,
    chamfer: f64,
) -> Tree {
    if chamfer <= 0.0 {
        // ⭐ **O caminho de sempre, ao bit** — encolher a fonte e deslocar a superfície.
        return offset(
            &box_at(
                px,
                py,
                pz,
                [half[0] - round, half[1] - round, half[2] - round],
            ),
            round,
        );
    }
    // ⭐⭐⭐ **A CAIXA E OS TRÊS PLANOS, MISTURADOS DE UMA VEZ.**
    //
    // ⛔⛔ **Esta nota já descreveu «encolher, chanfrar, deslocar», e essa construção foi RETIRADA
    // pelo report do Enio de 2026-08-30** (*«não funcionou, o fillet só muda a posição do
    // chamfer»*): deslocar um semiespaço dá **outro semiespaço**, sem canto para arredondar — a lei
    // que a W104 já tinha medido neste mesmo módulo. Medido: com o chanfro em `0,12`, o giro da
    // normal na quina ficava **cravado em `45,000°`** para qualquer filete, e só a posição
    // deslizava.
    //
    // ⭐ A mistura é a única forma de arredondar as quinas que o chanfro cria, e as quatro
    // superfícies entram **ao mesmo tempo** (ver [`crate::ops_bool::intersection_round_n`]) em vez
    // de duas a duas encaixadas — o que mantém o tecto de Cauchy–Schwarz em `√(activas)` em vez de
    // o fazer crescer com o comprimento da corrente.
    //
    // ⚠️ **E ela INFLA o campo**, que é o preço declarado: quem paga é o divisor da aresta, dentro
    // do [`crate::primitive_tree::primitive`], com o orçamento da marcha a subir pelo
    // [`crate::field_shrink`]. ⛔ *O divisor tem de estar na porta ÚNICA* — quando ele viveu numa
    // das duas rotas de compilação, o traçado marchava o campo cru e a peça saía com facetas
    // escuras que mudavam ao rodar.
    let q = [
        px.abs() - Tree::constant(half[0]),
        py.abs() - Tree::constant(half[1]),
        pz.abs() - Tree::constant(half[2]),
    ];
    // ⭐ **A caixa e os TRÊS planos, arredondados de uma vez** — ver a nota da
    // [`crate::ops_bool::intersection_round_n`] para as duas construções recusadas antes desta.
    // As SEIS meias-faces com sinal — `+x`, `−x`, `+y`, … — e não as três dobradas.
    // ⚠️ **As COORDENADAS que a função recebe**, e não `Tree::x/y/z`: a moldura
    // ([`sd_box_frame`]) passa expressões já dobradas, e ler o eixo global fazia a peça
    // **desaparecer** (medido: `0` de `64 000` células dentro).
    let eixo = [px.clone(), py.clone(), pz.clone()];
    let face: Vec<Tree> = (0..3)
        .flat_map(|i| {
            [
                eixo[i].clone() - Tree::constant(half[i]),
                -eixo[i].clone() - Tree::constant(half[i]),
            ]
        })
        .collect();
    let mut arestas: Vec<(Tree, Tree)> = Vec::new();
    for (i, j) in [(0usize, 1usize), (1, 2), (0, 2)] {
        // ⚠️ **Separar a dobra só quando os DOIS lados dela não podem estar activos ao mesmo
        // tempo** — ver o ponto 1 do doc da [`crate::ops_joint::intersection_joint_n`]: numa viga
        // fina a separação come material a dobrar e a peça desaparece.
        if chamfer + round < 2.0 * half[i].min(half[j]) {
            for si in 0..2 {
                for sj in 0..2 {
                    arestas.push((face[2 * i + si].clone(), face[2 * j + sj].clone()));
                }
            }
        } else {
            arestas.push((q[i].clone(), q[j].clone()));
        }
    }
    crate::ops_joint::intersection_joint_n(
        &[q[0].clone(), q[1].clone(), q[2].clone()],
        &arestas,
        crate::ops_joint::Edge::square(round, chamfer),
    )
}

/// ⭐⭐ **AS PEÇAS DE UMA CAIXA CHANFRADA, para quem precisa de as MISTURAR com mais alguma coisa.**
///
/// ⚠️ Ela existe porque a [`crate::ops::sd_wedge`] é *«uma caixa cortada por um plano»*, e passar-lhe
/// a caixa **já composta** punha a costura interna da caixa na aresta do corte — o defeito que o 3.º
/// report do Enio nomeou. *Quem mistura precisa das peças, não do resultado.*
pub(crate) fn box_pieces(
    px: &Tree,
    py: &Tree,
    pz: &Tree,
    half: [f64; 3],
    round: f64,
    chamfer: f64,
) -> (Vec<Tree>, Vec<(Tree, Tree)>) {
    let q = [
        px.abs() - Tree::constant(half[0]),
        py.abs() - Tree::constant(half[1]),
        pz.abs() - Tree::constant(half[2]),
    ];
    let eixo = [px.clone(), py.clone(), pz.clone()];
    let face: Vec<Tree> = (0..3)
        .flat_map(|i| {
            [
                eixo[i].clone() - Tree::constant(half[i]),
                -eixo[i].clone() - Tree::constant(half[i]),
            ]
        })
        .collect();
    let mut arestas: Vec<(Tree, Tree)> = Vec::new();
    for (i, j) in [(0usize, 1usize), (1, 2), (0, 2)] {
        if chamfer + round < 2.0 * half[i].min(half[j]) {
            for si in 0..2 {
                for sj in 0..2 {
                    arestas.push((face[2 * i + si].clone(), face[2 * j + sj].clone()));
                }
            }
        } else {
            arestas.push((q[i].clone(), q[j].clone()));
        }
    }
    (vec![q[0].clone(), q[1].clone(), q[2].clone()], arestas)
}

/// ⭐⭐ **A gaiola de uma caixa** — as 12 arestas de secção quadrada, em **três** vigas dobradas.
///
/// ⭐ Cada família de quatro vigas paralelas é **uma** caixa no espaço dobrado por `abs` nos dois
/// eixos perpendiculares a ela ([`box_at`]) — 3 caixas em vez de 12, e a distância é a mesma porque
/// o reflexo é uma isometria. ⚠️ A dobra só é exata com a caixa inteira do lado positivo, o que
/// pede `thickness <= min(half)`; o documento recusa acima disso.
///
/// ⚠️ **As vigas SOBREPÕEM-SE nos oito cantos** (um cubo de lado `thickness − 2·round`), então o
/// `min` não tem costura de campo — ver a nota do [`sd_star`], que teve de a construir de propósito.
pub fn sd_box_frame(half: [f64; 3], thickness: f64, round: f64, chamfer: f64) -> Tree {
    // ⚠️ **A LINHA DE CENTRO da viga não se mexe com o filete**: encolher a moldura de `round` baixa
    // a meia-extensão e a meia-espessura na mesma medida, e os dois `round` cancelam-se em
    // `half − thickness/2`. É isso que faz a viga dilatada voltar à espessura pedida.
    let e = thickness * 0.5;
    let c = [
        half[0] - thickness * 0.5,
        half[1] - thickness * 0.5,
        half[2] - thickness * 0.5,
    ];
    let (ax, ay, az) = (Tree::x().abs(), Tree::y().abs(), Tree::z().abs());
    let fold = |a: &Tree, k: f64| a.clone() - Tree::constant(k);
    // ⭐ **As três vigas são CAIXAS, e passam pela porta delas** ([`box_with_edge`]) — é isso que dá
    // chanfro à moldura sem uma segunda cópia da receita. ⚠️ Com `chamfer = 0` cada viga recai no
    // caminho de sempre (encolher e deslocar), e a moldura sai **ao bit** como antes.
    let viga =
        |px: &Tree, py: &Tree, pz: &Tree, h: [f64; 3]| box_with_edge(px, py, pz, h, round, chamfer);
    let beams = [
        viga(
            &Tree::x(),
            &fold(&ay, c[1]),
            &fold(&az, c[2]),
            [half[0], e, e],
        ),
        viga(
            &fold(&ax, c[0]),
            &Tree::y(),
            &fold(&az, c[2]),
            [e, half[1], e],
        ),
        viga(
            &fold(&ax, c[0]),
            &fold(&ay, c[1]),
            &Tree::z(),
            [e, e, half[2]],
        ),
    ];
    beams[0].clone().min(beams[1].clone()).min(beams[2].clone())
}
