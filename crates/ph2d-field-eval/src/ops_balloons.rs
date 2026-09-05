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
    #[allow(clippy::cast_lossless)]
    let n = f64::from(lobes.max(1));
    // ⭐ **A meia-largura de UMA bossa** — o passo entre centros é `2 × base`.
    let base = half_width / n;
    let mut pecas: Vec<Tree> = Vec::with_capacity(lobes as usize + 3);
    // ⚠️ **A mistura sai da menor BOSSA, e não da menor peça** — as bolhas da fieira são disjuntas
    // de tudo, então não há vale nenhum para elas alisarem; deixá-las entrar nesta conta punha o
    // raio a `0,011` e a quebra de curvatura do corpo a **`18,8`** contra uma barra de `2,0`.
    let mut menor_bossa = f64::MAX;
    let mut corpo_topo = f64::MAX;
    let mut bossas: Vec<(f64, f64, f64)> = Vec::with_capacity(lobes as usize);
    for i in 0..lobes.max(1) {
        #[allow(clippy::cast_lossless)]
        let t = (f64::from(i) + 0.5) / n;
        let arco = (t * std::f64::consts::PI).sin();
        // ⛔⛔⛔ **AS BOSSAS NÃO SE TOCAM UMAS ÀS OUTRAS, e é isso que mantém a peça marchável.**
        //
        // Numa união n-ária o tecto de `‖∇f‖` é `√(quantas peças estão ACTIVAS)`. A 1.ª nuvem punha
        // `n` discos livres a sobreporem-se todos, e furava em boa parte do curso dos próprios
        // controlos — medido pela porta do painel: `passo × ‖∇f‖` de `1,06` a **`1,54`** ao arrastar
        // `Span` e `Width`. ⚠️ E a 2.ª tentativa piorou de outra maneira: limitar o raio a
        // `half_width` fazia **todas** as bossas colapsarem no MESMO círculo quando o `Span` passava
        // a largura (o espaço livre `half_width − raio` ia a zero) — `n` superfícies coincidentes.
        //
        // ⭐⭐ A cura é a receita da ESTRELA e da ENGRENAGEM: **um corpo mais uma peça por bossa**,
        // e as bossas separadas entre si. O raio pára a `0,92 × base` e o passo é `2 × base`, logo
        // duas vizinhas nunca se alcançam ⇒ **no máximo DUAS peças activas** (o corpo e uma bossa),
        // e o tecto é `√2`, que é o balde que o módulo já paga.
        let raio = (base * 0.92).min(half_span * 0.5) * 0.22f64.mul_add(arco, 0.78);
        // ⚠️ **Os topos são DIFERENTES** — com todos iguais as bossas ficam tangentes à mesma recta,
        // e duas superfícies tangentes estão a menos de qualquer raio uma da outra.
        let topo = half_span * 0.38f64.mul_add(arco, 0.62);
        let cx = half_width.mul_add(2.0 * t, -half_width)
            * (1.0 - base / half_width.max(f64::MIN_POSITIVE))
            + 0.0;
        let cy = topo - raio;
        menor_bossa = menor_bossa.min(raio);
        corpo_topo = corpo_topo.min(cy);
        bossas.push((cx, cy, raio));
    }
    // ⭐ **O CORPO liga as bossas** — ele sobe até ao centro da bossa mais baixa, então cada uma o
    // cobre pela metade de baixo: a sobreposição tem área, e não há costura a valer zero num ponto
    // interior (a lei que a `sd_star` pagou).
    let corpo_topo = corpo_topo.min(half_span * 0.5).max(-half_span * 0.5);
    // ⚠️ **As quinas de baixo do corpo usam o `round` DO ARTISTA** — a 1.ª redacção punha um raio
    // derivado da bossa, e por isso o controlo *Fillet* não lhes chegava: a sonda leu `2,2 %` da
    // superfície sobre um vinco de `50,0°`. *Uma quina cujo raio não é o do controlo é uma quina que
    // o controlo não arredonda.*
    let meia_altura_corpo = (corpo_topo + half_span) * 0.5;
    pecas.push(crate::ops_plate2d::rect_round_em(
        0.0,
        (corpo_topo - half_span) * 0.5,
        half_width,
        meia_altura_corpo,
        round.min(half_width * 0.9).min(meia_altura_corpo * 0.9),
    ));
    for (cx, cy, raio) in bossas {
        pecas.push(disco_em(cx, cy, raio));
    }
    if tail > 0.0 {
        // A fieira do pensamento: duas bolhas a encolher, para baixo e para o lado. ⚠️ Elas são
        // DISJUNTAS de tudo — é isso que um balão de pensamento é —, então não somam peças activas.
        for (k, f) in [(1.0_f64, 0.30_f64), (1.85, 0.17)] {
            let r = tail * f;
            pecas.push(disco_em(
                -half_width * 0.18f64.mul_add(k, 0.45),
                -half_span - tail * 0.55f64.mul_add(k, 0.30),
                r,
            ));
            let _ = r;
        }
    }
    // ⚠️ **O raio da mistura é uma fracção do MENOR lobo**, e não o `round`: ele é o que dá o vale
    // entre o corpo e uma bossa, e um valor maior que o lobo mais pequeno engolia-o.
    // ⚠️ **A fracção é `0,80`, e ela foi MEDIDA contra a QUEBRA DE CURVATURA** — a régua que vê o
    // risco no sombreado. Uma bossa redonda a encontrar o topo PLANO do corpo é `G1` sem ser `G2`, e
    // com a mistura curta a curvatura salta:
    //
    // | fracção | 0,35 | 0,55 | **0,80** |
    // |---|---:|---:|---:|
    // | quebra de curvatura | `9,89` | `9,57` | **dentro da barra** |
    //
    // ⚠️ E ela **não** custa marcha: uma varredura de `0,50` a `0,10` já tinha mostrado que o raio
    // da mistura move `passo × ‖∇f‖` em `0,05` — o que manda ali é a geometria, não este número.
    let mistura = (menor_bossa * 0.8).min(half_span * 0.25);
    slab_and_walls(&crate::ops::union_round_n(&pecas, mistura), half_height, e)
}
