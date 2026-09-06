//! ⭐ **AS EXACTAS DO CATÁLOGO** (W125) — por agora, o cilindro com barriga.
//!
//! # ⛔⛔⛔ E as OUTRAS DUAS deste lote foram construídas e RECUSADAS — ver o doc 06 §126
//!
//! - **O OVO**: a fórmula publicada tem um parâmetro de barriga que **no valor natural degenera num
//!   círculo** (a álgebra dá `x = 0` identicamente), é **não-monótono** de um lado e do outro não há
//!   arco nenhum — com ele fixo em qualquer valor, `28` de `100` combinações de `(raio, topo, vão)`
//!   deixam a peça com um **vinco** entre os dois círculos.
//! - **A ESCADA**: quatro construções, e a última passa a marcha, o chanfro e as arestas — mas o
//!   filete dela é **neutro em volume** (`20 139` amostras dentro com e sem), porque ela tem tantas
//!   quinas côncavas como convexas e do mesmo raio. ⛔ O censo tem um gate com `<` **estrito** que
//!   não distingue *equilibrado* de *inerte*, e o doc dele diz porquê: foi ele que apanhou o `round`
//!   inerte do cone e do prisma em Agosto. *Enfraquecê-lo para caber uma forma é trocar um gate que
//!   apanhou dois defeitos reais por uma escada.*

use fidget::context::Tree;

use crate::ops::length2;

/// ⭐ **CILINDRO COM BARRIGA** (*rounded cylinder*) — o disco de raio `radius` e meia-altura
/// `half_height`, com o bordo inteiro num arco de raio `bulge`.
///
/// ⚠️ **Não é o `round` de um cilindro**: aquele arredonda só o aro, e aqui a **parede** também
/// curva — é a rolha, o botão, o pneu. ⛔ E por isso ela **não tem filete**: o `bulge` já é o
/// arredondamento, e um segundo número para a mesma aresta seriam duas verdades sobre ela.
///
/// ⚠️ **Distância EXACTA** — `‖∇f‖ = 1,0000` medido —, ao contrário da espiral, da mola e da rede,
/// que entregam um minorante.
pub fn sd_rounded_cylinder(radius: f64, bulge: f64, half_height: f64) -> Tree {
    let dx = length2(&Tree::x(), &Tree::y()) - Tree::constant(radius - bulge);
    let dy = Tree::z().abs() - Tree::constant(half_height - bulge);
    dx.clone().max(dy.clone()).min(0.0) + length2(&dx.max(0.0), &dy.max(0.0))
        - Tree::constant(bulge)
}
