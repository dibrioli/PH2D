//! ⭐⭐ **OS BALÕES** (W120) — o retangular, o oval e a nuvem (que também é o balão de pensamento).
//!
//! # ⚠️ O que os três partilham, e é o que os torna uma família
//!
//! Todos são um **corpo** unido a uma **cauda**, e a cauda encosta ao corpo por dentro. A lei da
//! sobreposição que a [`crate::ops_arrows`] escreveu vale aqui inteira: o plano de cima da cauda
//! fica **dentro** do corpo, nunca coincidente com a fronteira dele — duas peças que se tocam sem
//! se sobrepor dão `0` num ponto interior (uma superfície fantasma), e duas cujas fronteiras
//! **coincidem ao longo de uma face** fazem a união arredondada **inchar** para fora dela.

use fidget::context::Tree;

use crate::ops::{length2, slab_and_walls};
use crate::ops_joint::Edge;
use crate::ops_plate2d::{disco_em, rect_round_em};

/// A cauda de um balão: uma cunha que aponta para baixo, com o topo **dentro** do corpo.
///
/// `tx` é onde ela nasce em X, `topo` o `y` do plano que a fecha por cima (dentro do corpo), `base`
/// o `y` da bica, e `meia` a meia-largura dela em `topo`.
fn cauda(tx: f64, topo: f64, base: f64, meia: f64, e: Edge) -> Tree {
    let esquerda = crate::ops::half_plane([tx - meia, topo], [tx, base]);
    let direita = crate::ops::half_plane([tx, base], [tx + meia, topo]);
    let tampa = Tree::y() - Tree::constant(topo);
    // ⭐ A bica arredonda, como o bico de uma seta — ver a nota da [`crate::ops_arrows::sd_arrow`]
    // para a medição que trocou o `max` duro por esta junta.
    let bica = crate::ops_joint::intersection_joint(&esquerda, &direita, e);
    crate::ops_joint::intersection_joint(&bica, &tampa, e)
}

/// Onde a cauda entra no corpo, e quanto ela mede lá — derivado, para as três não divergirem.
fn medidas_da_cauda(half_span: f64, tail: f64, largura: f64) -> (f64, f64, f64) {
    let topo = -half_span + (half_span * 0.5).min(tail * 0.5).max(f64::MIN_POSITIVE);
    let base = -half_span - tail;
    let meia = (tail * 0.45).min(largura * 0.5).max(f64::MIN_POSITIVE);
    (topo, base, meia)
}

/// ⭐ **BALÃO RETANGULAR** — o corpo é um rectângulo de quinas redondas, e a cauda sai da base.
///
/// ⚠️ **A cauda nasce fora do centro** (a `35 %` da meia-largura, à esquerda): centrada ela lê-se
/// como um funil, e é o desenho que toda biblioteca de formas usa.
pub fn sd_speech_rect(
    half_width: f64,
    half_span: f64,
    tail: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let tx = -half_width * 0.35;
    let (topo, base, meia) = medidas_da_cauda(half_span, tail, half_width);
    let rabo = cauda(tx, topo, base, meia, e);
    if chamfer > 0.0 {
        // ⭐ Com chanfro cada peça é uma chapa própria — ver [`crate::ops::plate_joint_n`].
        let g = crate::ops_plate2d::paredes(0.0, 0.0, half_width, half_span);
        let corpo = crate::ops::plate_joint_n(&g, &crate::ops_plate2d::quinas(&g), half_height, e);
        let cauda = crate::ops::plate_joint_n(&[rabo], &[], half_height, e);
        return crate::ops_joint::union_joint(&corpo, &cauda, e);
    }
    let corpo = rect_round_em(0.0, 0.0, half_width, half_span, round);
    let perfil = crate::ops_joint::union_joint(&corpo, &rabo, e);
    slab_and_walls(&perfil, half_height, e)
}

/// Um **OVAL** centrado na origem — a elipse pela mesma receita do [`crate::ops::sd_ellipsoid`].
///
/// ⚠️ **Ela SUBESTIMA a distância** (por `min(a,b)/max(a,b)`), e isso é **seguro** para a marcha de
/// esferas: quem subestima nunca ultrapassa a superfície, e paga passos em vez de correcção.
fn oval(a: f64, b: f64) -> Tree {
    let m = a.min(b);
    let unit = length2(
        &(Tree::x() * Tree::constant(1.0 / a)),
        &(Tree::y() * Tree::constant(1.0 / b)),
    ) - Tree::constant(1.0);
    unit * Tree::constant(m)
}

/// ⭐ **BALÃO OVAL** — o mesmo balão com o corpo redondo.
///
/// ⚠️ **A cauda entra MAIS ALTO que a do retangular** (a `55 %` da meia-altura): junto ao fundo o
/// oval é estreito, e uma cauda tão larga como a do rectângulo espreitaria pelos lados dele. *A
/// sobreposição tem de caber na peça, não só existir.*
pub fn sd_speech_oval(
    half_width: f64,
    half_span: f64,
    tail: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let topo = -half_span * 0.55;
    // A meia-largura do oval na altura em que a cauda entra — a cerca que a impede de espreitar.
    let dentro = half_width * (1.0 - (topo / half_span).powi(2)).max(0.0).sqrt();
    let (_, base, meia) = medidas_da_cauda(half_span, tail, dentro * 1.2);
    let tx = -half_width * 0.3;
    let perfil = crate::ops_joint::union_joint(
        &oval(half_width, half_span),
        &cauda(tx, topo, base, meia, e),
        e,
    );
    slab_and_walls(&perfil, half_height, e)
}

/// ⭐⭐ **NUVEM — e o BALÃO DE PENSAMENTO é ela com `tail > 0`.**
///
/// ⚠️ **Uma nuvem e um balão de pensamento são a MESMA forma**, e por isso são a mesma primitiva: o
/// segundo é o primeiro com uma fieira de bolhas a descer. É a lei do [`ph2d_field::Primitive::Cone`]
/// — duas variantes dariam duas fórmulas para a mesma superfície, e a segunda é a que envelhece.
///
/// ⭐ **É a forma que o SDF faz melhor do que o desenho**: uma união arredondada de discos é
/// exactamente o que uma nuvem é, e o [doc 08](../../../docs/3DModeling/08_formas_por_formula.md)
/// já o dizia. ⚠️ A união é **n-ária**, e não uma dobra: dobrar `n` uniões compõe a inflação `n`
/// vezes (a lição da engrenagem), e numa fieira de bolhas nunca há mais de três activas.
///
/// ⚠️ **As bolhas da cauda são DISJUNTAS do corpo e uma da outra** — é isso que um balão de
/// pensamento é. Elas não partilham face com nada, então a união não incha em lado nenhum.
pub fn sd_cloud(
    lobes: u32,
    half_width: f64,
    half_span: f64,
    tail: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let n = lobes.max(1);
    let mut pecas: Vec<Tree> = Vec::with_capacity(n as usize + 2);
    let mut menor = f64::MAX;
    for i in 0..n {
        #[allow(clippy::cast_lossless)]
        let t = (f64::from(i) + 0.5) / f64::from(n);
        let arco = (t * std::f64::consts::PI).sin();
        // ⚠️ Os lobos das pontas são MENORES, e é o que dá a silhueta de nuvem: iguais, a forma
        // lê-se como uma salsicha.
        //
        // ⛔⛔ **E os TOPOS têm de ser DIFERENTES, ou o campo infla** (medido 05/09). A 1.ª redacção
        // punha `cy = half_span − raio`, o que deixa **todas** as bossas tangentes à MESMA recta
        // `y = half_span`. Duas superfícies tangentes estão a menos de qualquer raio uma da outra,
        // então junto ao topo **todas** as peças ficam activas na mistura n-ária — e o tecto dela é
        // `√(activas)`. Medido: `passo × ‖∇f‖ = 1,06` a cinco bossas e `1,31` a doze, acima do `1`
        // em que a marcha atravessa a superfície.
        //
        // ⚠️⚠️ **E o raio da MISTURA não era a alavanca** — uma varredura de `0,50` a `0,10` mudou
        // o número em `0,05` (`measure_cloud_blend`): *duas superfícies tangentes continuam
        // tangentes por mais que se aperte o raio*. A alavanca é a GEOMETRIA.
        //
        // ⭐ E a cura é o que uma nuvem já é: a bossa do meio mais alta, as das pontas mais baixas.
        let topo = half_span * 0.45f64.mul_add(arco, 0.55);
        let raio = half_span * 0.34f64.mul_add(arco, 0.42);
        let cx = half_width * (2.0 * t - 1.0) * (1.0 - raio / half_width.max(f64::MIN_POSITIVE));
        let cy = topo - raio;
        menor = menor.min(raio);
        pecas.push(disco_em(cx, cy, raio));
    }
    if tail > 0.0 {
        // A fieira do pensamento: duas bolhas a encolher, para baixo e para o lado.
        for (k, f) in [(1.0_f64, 0.30_f64), (1.85, 0.17)] {
            let r = tail * f;
            pecas.push(disco_em(
                -half_width * (0.45 + 0.18 * k),
                -half_span - tail * (0.30 + 0.55 * k),
                r,
            ));
            menor = menor.min(r);
        }
    }
    // ⚠️ **O raio da mistura é uma fracção do MENOR lobo**, e não o `round`: ele é o que dá o vale
    // entre duas bossas, e um valor maior que o lobo mais pequeno engolia-o.
    let mistura = (menor * 0.35).min(half_span * 0.25);
    slab_and_walls(&crate::ops::union_round_n(&pecas, mistura), half_height, e)
}
