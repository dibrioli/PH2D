//! ⭐⭐ **O FLUXOGRAMA** (W122) — o paralelogramo, o atraso, o mostrador e o conector de página.
//!
//! # ⭐ As quatro são INTERSECÇÕES, e é isso que as torna baratas e seguras
//!
//! Nenhuma destas formas é côncava, e por isso nenhuma precisa de união: o `max` de funções
//! 1-Lipschitz é 1-Lipschitz **por definição**, e o perfil entra inteiro na
//! [`crate::ops::plate_joint_n`], que é a porta que serve exactamente perfis de intersecção.
//!
//! # ⚠️ Onde uma DOBRA é segura, e onde ela apaga uma aresta
//!
//! Dobrar duas paredes **paralelas** num `|x| − w` é exacto e não cria canto nenhum — é o que o
//! [`crate::ops_plates::sd_trapezoid`] já faz, e o que o paralelogramo daqui faz duas vezes.
//! ⛔ **Dobrar duas paredes que se ENCONTRAM é outra coisa:** o `|·|` é um `max`, e um `max` cru é
//! uma intersecção **sem raio** — a quina que ele produz fica **viva**, e nem o filete nem o
//! chanfro lá chegam. É por isso que o bico do mostrador e o do conector entram como **dois
//! semiplanos separados**: a mistura tem de os ver como duas peças para arredondar o encontro.
//!
//! # ⭐⭐⭐ E o par do bico entra COM O ÂNGULO DELE, ou a peça rasga onde é mais simples
//!
//! Os dois flancos de um bico deitam-se sobre a **mesma recta** quando ele fecha. Numa mistura
//! n-ária o tecto é `√(quantas peças estão activas)`, e ali estariam **duas cópias da mesma
//! superfície**: medido, `passo × ‖∇f‖ = 1,183` com o bico a zero — o pior campo no ponto mais
//! benigno da forma. ⭐ A cura não é uma cerca no controlo (uma coerção estaciona nela, isto é,
//! entrega sempre o pior caso): é a [`Edge::at`], que já existe desde a W107, com
//! `cos = (altura² − base²)/(altura² + base²)` — em `point = 0` ela vale `1`, que é *duas normais
//! iguais, canto nenhum*. Fica `0,954`, e o zero passa a ser uma **forma** (o atraso, o rectângulo).
//!
//! ⚠️ **E a cura tem METADE:** o perfil composto **não** pode entrar na
//! [`crate::ops::plate_joint_n`], que compõe outra vez e leva a costura dele para o aro — medido,
//! o vinco foi de `5,1°` para `25,3°`. ⇒ **dois caminhos**: sem chanfro o bico entra composto e com
//! o ângulo dentro; com chanfro as peças entram **inteiras**.
//!
//! # ⛔ E a JUNÇÃO do ANSI não está aqui, por ARITMÉTICA
//!
//! O símbolo é *«disco ∪ cruz»* — e a cruz vive **dentro** do disco. A união de um conjunto com um
//! subconjunto dele é o próprio conjunto: `min(disco, cruz) = disco` em todo o ponto, sem medição
//! nenhuma. *Ela é um desenho de TRAÇOS, e o sólido dela é o cilindro que a paleta já tem.* Pelo
//! mesmo argumento ficam de fora o `PredefinedProcess` (duas barras dentro de um rectângulo) e o
//! `NoteBracket` (três segmentos).

use fidget::context::Tree;

use crate::ops::{half_plane, plate_joint_n, slab_and_walls};
use crate::ops_joint::{Edge, intersection_joint};
use crate::ops_plate2d::rect_em;

/// ⭐ **UMA CÁPSULA 2D deitada** — a recta de `(x0, 0)` a `(x1, 0)` engrossada em `r`.
///
/// ⚠️ **Dilatar uma distância EXACTA**, e não `rect_round_em`: aquela prende o raio a `0,999` da
/// meia-altura, e aqui o raio **é** a meia-altura — a tampa tem de ser um semicírculo inteiro, e
/// um resto de recta de `0,001` põe duas quinas onde a forma não tem nenhuma.
fn capsula(x0: f64, x1: f64, r: f64) -> Tree {
    crate::ops::offset(
        &rect_em((x0 + x1) * 0.5, 0.0, (x1 - x0).abs() * 0.5, 0.0),
        r,
    )
}

/// ⭐ **PARALELOGRAMO** (*Data*) — o rectângulo inclinado: duas bases horizontais e dois flancos.
///
/// `skew` é o quanto a base de cima escorrega em `+X` face ao centro; `0` dá o rectângulo **ao
/// bit**, porque a inclinação entra como `k = 0` e a normalização vale `1`.
///
/// ⭐⭐ **As duas peças são LAJES, e as duas são exactas:** o conjunto é
/// `{|y| ≤ s} ∩ {|x − k·y| ≤ w}`, e a distância a uma laje obliqua é a forma linear **normalizada**
/// (`√(1 + k²)`). ⚠️ É a única razão de o `max` das duas continuar a ser uma distância — sem a
/// normalização o campo subiria com a inclinação e a marcha atravessaria a peça.
pub fn sd_parallelogram(
    half_width: f64,
    half_span: f64,
    skew: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let k = skew / half_span.max(f64::MIN_POSITIVE);
    let norm = 1.0 / (1.0 + k * k).sqrt();
    let bases = Tree::y().abs() - Tree::constant(half_span);
    let flancos = ((Tree::x() - Tree::y() * Tree::constant(k)).abs() - Tree::constant(half_width))
        * Tree::constant(norm);
    if chamfer <= 0.0 {
        return slab_and_walls(&intersection_joint(&flancos, &bases, e), half_height, e);
    }
    plate_joint_n(
        &[flancos.clone(), bases.clone()],
        &[(flancos, bases)],
        half_height,
        e,
    )
}

/// ⭐ **ATRASO** (*Delay*) — a face esquerda reta e a direita num semicírculo inteiro.
///
/// ⭐⭐ **É uma cápsula CORTADA, e não um rectângulo unido a um meio-disco.** As duas construções
/// dão a mesma silhueta e só uma sobrevive ao filete: no rectângulo mais meio-disco a tampa é
/// **tangente** às faces de cima e de baixo, e uma união arredondada sobre um contacto tangente
/// incha para fora delas (a família do `TANGENT_JOIN_EXCEPTION`). Aqui a parede corta a cápsula
/// pelo **centro** da tampa esquerda — o corte mais largo que existe.
pub fn sd_delay(
    half_width: f64,
    half_span: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s) = (half_width, half_span);
    // A recta vai da parede (`−w`) ao centro da tampa direita (`w − s`); a tampa esquerda sobra `s`
    // para lá da parede, e é essa sobra que faz o corte ter área.
    let corpo = capsula(-w, w - s, s);
    let parede = Tree::constant(-w) - Tree::x();
    if chamfer <= 0.0 {
        return slab_and_walls(&intersection_joint(&corpo, &parede, e), half_height, e);
    }
    plate_joint_n(
        &[corpo.clone(), parede.clone()],
        &[(corpo, parede)],
        half_height,
        e,
    )
}

/// ⭐ **MOSTRADOR** (*Display*) — o atraso com a esquerda a fechar num BICO.
///
/// ⚠️ **Os dois flancos do bico entram SEPARADOS** — ver o cabeçalho do módulo: dobrados num
/// `|y|` a ponta seria um `max` cru, e ficaria viva.
pub fn sd_display(
    half_width: f64,
    half_span: f64,
    point: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s) = (half_width, half_span);
    let corpo = capsula(-w, w - s, s);
    // ⚠️ **Anti-horário**: o [`half_plane`] é negativo à esquerda de `a → b`, e o interior fica à
    // esquerda de quem percorre a fronteira no sentido positivo.
    let cima = half_plane([-w + point, s], [-w, 0.0]);
    let baixo = half_plane([-w, 0.0], [-w + point, -s]);
    if chamfer <= 0.0 {
        // ⭐⭐⭐ **O BICO entra COMPOSTO, e com o ÂNGULO dele dentro** — ver o cabeçalho do módulo.
        let cunha = intersection_joint(&cima, &baixo, Edge::at(round, chamfer, cos_bico(point, s)));
        return slab_and_walls(&intersection_joint(&cunha, &corpo, e), half_height, e);
    }
    // ⛔ **Com chanfro as três peças entram INTEIRAS** — um perfil já composto leva a costura dele
    // para o aro, e o gate `the_chamfer_never_makes_an_edge_worse_than_the_fillet_alone` mediu-o:
    // com a cunha composta aqui, o vinco do aro ia de `5,1°` para `25,3°`.
    plate_joint_n(
        &[corpo.clone(), cima.clone(), baixo.clone()],
        &[
            (corpo.clone(), cima.clone()),
            (cima, baixo.clone()),
            (baixo, corpo),
        ],
        half_height,
        e,
    )
}

/// ⭐ **CONECTOR DE PÁGINA** (*Off-page*) — o rectângulo que fecha num bico em baixo.
///
/// ⛔ **Sem parede de baixo, e a razão é a mesma da etiqueta:** os dois flancos do bico cortam-na
/// **sempre** — ela nunca está na fronteira —, e uma parede a mais na mistura é uma quase-duplicada
/// que faz o campo subir quando o bico fica raso.
pub fn sd_offpage(
    half_width: f64,
    half_span: f64,
    point: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s) = (half_width, half_span);
    let topo = Tree::y() - Tree::constant(s);
    let esquerda = Tree::constant(-w) - Tree::x();
    let direita = Tree::x() - Tree::constant(w);
    let bico_esq = half_plane([-w, -s + point], [0.0, -s]);
    let bico_dir = half_plane([0.0, -s], [w, -s + point]);
    if chamfer <= 0.0 {
        // ⭐⭐⭐ **O BICO entra COMPOSTO, e com o ÂNGULO dele dentro** — ver o cabeçalho do módulo.
        let cunha = intersection_joint(
            &bico_esq,
            &bico_dir,
            Edge::at(round, chamfer, cos_bico(point, w)),
        );
        return slab_and_walls(
            &crate::ops::intersection_round_n(&[topo, esquerda, cunha, direita], round),
            half_height,
            e,
        );
    }
    // ⛔ **Com chanfro as cinco peças entram INTEIRAS** — ver a nota gémea na [`sd_display`].
    let corpo = [topo, esquerda, bico_esq, bico_dir, direita];
    let arestas = vec![
        (corpo[0].clone(), corpo[1].clone()),
        (corpo[1].clone(), corpo[2].clone()),
        (corpo[2].clone(), corpo[3].clone()),
        (corpo[3].clone(), corpo[4].clone()),
        (corpo[4].clone(), corpo[0].clone()),
    ];
    plate_joint_n(&corpo, &arestas, half_height, e)
}

/// ⭐⭐⭐ **O COSSENO DAS NORMAIS EXTERIORES dos dois flancos de um bico** de base `base` (a meia-base
/// do triângulo) e altura `altura`.
///
/// As normais são `(±base, altura)/√(base² + altura²)` num referencial e `(altura, ±base)` no
/// outro; em qualquer deles o produto interno vale `(altura² − base²)/(altura² + base²)`.
///
/// ⚠️ **Com o bico a ZERO ele vale `+1`** — as duas normais são a mesma —, e é exactamente isso que
/// a [`crate::ops::union_round_at`] precisa de saber para não arredondar um canto que não existe.
fn cos_bico(base: f64, altura: f64) -> f64 {
    let (b2, a2) = (base * base, altura * altura);
    ((a2 - b2) / (a2 + b2).max(f64::MIN_POSITIVE)).clamp(-1.0, 1.0)
}

/// ⭐⭐⭐ **DOCUMENTO** (*Document*) — o retângulo cuja base é uma ONDA.
///
/// # ⛔⛔ Ela estava declarada «fica desenhada», e a recusa respondia a OUTRA pergunta
///
/// O [doc 08](../../../docs/3DModeling/08_formas_por_formula.md) dá-a como classe **C/D** porque
/// *«a distância a uma senóide não é fechada»* — o que é **verdade** e **não é o que o módulo
/// pede**. Uma marcha de esferas precisa de um **minorante** da distância, nunca do valor exacto:
/// andar a menos custa passos, andar a mais atravessa a superfície
/// ([`crate::safe_march_step`]).
///
/// ⭐ E o minorante de uma curva implícita é uma linha de álgebra: para `g(x,y) = base(x) − y`,
/// `|g| / max‖∇g‖ ≤ dist`. Aqui `‖∇g‖ = √(1 + base'(x)²) ≤ √(1 + (a·π/w)²)`, que é uma **constante**
/// — logo a divisão é rigorosa em todo o plano, e não uma aproximação com erro.
///
/// ⚠️ **A SUPERFÍCIE é exacta**: o zero de `g` é a senóide, ao bit. O que é conservador é só a
/// *distância* longe dela. *Uma inexactidão que subestima é folga, não perigo.*
///
/// ⚠️ **Meia onda ao longo da peça** (`k = π/w`): em `x = ±w` o seno vale zero, então a base
/// encontra os dois flancos **exactamente** em `−half_span` — a onda não muda a altura das quinas.
pub fn sd_document(
    half_width: f64,
    half_span: f64,
    wave: f64,
    half_height: f64,
    round: f64,
    chamfer: f64,
) -> Tree {
    let e = Edge::square(round, chamfer);
    let (w, s) = (half_width, half_span);
    let k = std::f64::consts::PI / w.max(f64::MIN_POSITIVE);
    // ⭐ O MAIOR `‖∇g‖` possível — `cos` vale no máximo `1`.
    let lip = (wave * k).mul_add(wave * k, 1.0).sqrt();
    let base = Tree::constant(wave) * (Tree::x() * Tree::constant(k)).sin() - Tree::constant(s);
    let onda = (base - Tree::y()) / Tree::constant(lip);
    let lados = Tree::x().abs() - Tree::constant(w);
    let topo = Tree::y() - Tree::constant(s);
    if chamfer <= 0.0 {
        return slab_and_walls(
            &crate::ops::intersection_round_n(&[lados, topo, onda], round),
            half_height,
            e,
        );
    }
    // ⚠️ **Só as DUAS arestas que existem**: o topo e a onda nunca se encontram, e declarar um par
    // que não forma quina põe um plano de corte que não corta nada e ainda conta no tecto da
    // mistura — a lei que a nuvem pagou.
    plate_joint_n(
        &[lados.clone(), topo.clone(), onda.clone()],
        &[(lados.clone(), topo), (lados, onda)],
        half_height,
        e,
    )
}
