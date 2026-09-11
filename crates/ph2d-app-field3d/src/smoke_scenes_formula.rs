//! ⭐⭐ **AS CENAS DAS FORMAS POR FÓRMULA E DE VÉRTICES AUTORADOS** (W125–W134) — o cilindro com
//! bojo, a superquadrática, a superfórmula, o triângulo, o polígono de `N` e o nó de toro.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::shapes`] responde pelas cenas dos **lotes** (W119–W122: as setas, os sinais, o
//! fluxograma) e passou as `600` linhas do gate de LOC do shell ao receber a cena do polígono.
//! ⛔ *Split, nunca allowlist* — e o corte é por assunto: aqui estão as formas cujo contorno sai de
//! uma **conta** (dois expoentes, oito números de Gielis) ou de **pontos que o artista digita**, e
//! não de uma família de símbolos.

use super::*;

/// ⭐ **A cena `=23`: O CILINDRO COM BOJO** (W125) — a única forma da wave, mostrada pelo que a
/// distingue.
///
/// ⚠️ **Três cópias com o bojo a crescer, e não uma.** A forma nasce da fórmula exacta do Quílez e
/// o bojo é o ÚNICO controle que ela tem; uma cópia só mostraria um cilindro com o aro mole e não
/// diria que o knob percorre de *cilindro* (bojo `0`) a *cápsula* (bojo = `min(raio, meia-altura)`).
/// *Um gate no representante deixa o curso do controle por medir* — e uma cena com uma cópia só faz
/// o mesmo ao olho.
pub fn cena_23() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 23 — O CILINDRO COM BOJO (W125): o mesmo raio e a mesma altura com o \
         bojo a 0,02 / 0,12 / 0,24. A da direita ja' e' quase uma capsula."
    );
    let peca = |bulge: f32, x: f32| {
        leaf(
            Primitive::RoundedCylinder {
                radius: 0.24,
                bulge,
                half_height: 0.30,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            peca(0.02, -0.62),
            peca(0.12, 0.0),
            peca(0.24, 0.62),
            combine(Op::Union(Blend::Sharp), (0..3).map(NodeId).collect()),
        ],
        NodeId(3),
    )
}

/// ⭐⭐⭐ **A cena `=24`: A FAMÍLIA INTEIRA NUM KNOB** (W127) — quatro pontos do mesmo controlo.
///
/// ⚠️ **Quatro peças e não uma**, porque o que esta forma vende não é uma silhueta: é a
/// **travessia**. Uma cópia só mostraria um bloco de cantos moles e não diria que o mesmo número
/// vai do losango à caixa — e a quarta prova que os DOIS expoentes são eixos diferentes.
pub fn cena_24() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 24 — A SUPERQUADRATICA (W127): losango (1,1) · esfera (2,2) · \
         squircle (4,4) · e a ultima com os DOIS expoentes diferentes (16 de cima, 1,6 de lado)."
    );
    let peca = |top: f32, side: f32, x: f32| {
        leaf(
            Primitive::Superquadric {
                half: [0.20, 0.20, 0.20],
                exponent_top: top,
                exponent_side: side,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            peca(1.0, 1.0, -0.72),
            peca(2.0, 2.0, -0.24),
            peca(4.0, 4.0, 0.24),
            peca(16.0, 1.6, 0.72),
            combine(Op::Union(Blend::Sharp), (0..4).map(NodeId).collect()),
        ],
        NodeId(4),
    )
}

/// ⭐⭐⭐ **A cena `=25`: SEIS FORMAS DA MESMA FÓRMULA** (W128) — a superfórmula de Gielis.
///
/// ⚠️ **Seis peças e não uma**: o que esta forma vende é *quantas coisas diferentes ela é*, e uma
/// cópia só mostraria uma estrela do mar. Todas têm a MESMA fórmula e só os oito números mudam.
pub fn cena_25() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 25 — A SUPERFORMULA (W128): esfera · estrela do mar · flor · folha · \
         diamante · e uma com o PERFIL a mexer. A mesma formula nas seis."
    );
    #[allow(clippy::too_many_arguments)]
    let peca = |tm: f32, t1: f32, t2: f32, t3: f32, sm: f32, s1: f32, s2: f32, s3: f32, x: f32| {
        leaf(
            Primitive::Superformula {
                half: [0.17, 0.17, 0.17],
                top_symmetry: tm,
                top_n1: t1,
                top_n2: t2,
                top_n3: t3,
                side_symmetry: sm,
                side_n1: s1,
                side_n2: s2,
                side_n3: s3,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            peca(4.0, 2.0, 2.0, 2.0, 4.0, 2.0, 2.0, 2.0, -1.05),
            peca(5.0, 0.6, 1.7, 1.7, 4.0, 2.0, 2.0, 2.0, -0.63),
            peca(6.0, 1.0, 1.0, 1.0, 4.0, 2.0, 2.0, 2.0, -0.21),
            peca(3.0, 4.0, 4.0, 4.0, 4.0, 2.0, 2.0, 2.0, 0.21),
            peca(4.0, 1.0, 1.0, 1.0, 4.0, 1.0, 1.0, 1.0, 0.63),
            peca(4.0, 2.0, 2.0, 2.0, 2.0, 1.0, 4.0, 1.0, 1.05),
            combine(Op::Union(Blend::Sharp), (0..6).map(NodeId).collect()),
        ],
        NodeId(6),
    )
}

/// ⭐⭐ **A cena `=26`: O TRIÂNGULO QUE O PRISMA NÃO FAZ** (W131) — quatro que só ele alcança.
pub fn cena_26() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 26 — O TRIANGULO de tres vertices quaisquer: escaleno · rectangulo · \
         obtuso · e um com o filete no tecto (o INRAIO). O prisma so' faz regulares."
    );
    let peca = |a: [f32; 2], b: [f32; 2], c: [f32; 2], round: f32, x: f32| {
        leaf(
            Primitive::Triangle {
                a,
                b,
                c,
                half_height: 0.10,
                round,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            peca([-0.20, -0.13], [0.21, -0.06], [-0.03, 0.22], 0.0, -0.72),
            peca([-0.18, -0.16], [0.20, -0.16], [-0.18, 0.20], 0.0, -0.24),
            peca([-0.24, -0.05], [0.24, -0.05], [0.05, 0.11], 0.0, 0.24),
            peca([-0.20, -0.13], [0.21, -0.06], [-0.03, 0.22], 0.055, 0.72),
            combine(Op::Union(Blend::Sharp), (0..4).map(NodeId).collect()),
        ],
        NodeId(4),
    )
}

/// ⭐⭐ **A cena `=27`: O POLÍGONO DE `N` VÉRTICES** (W132) — o que o prisma e o triângulo não fazem.
///
/// As quatro respondem a perguntas diferentes: a **concavidade** (nenhuma outra chapa desta paleta a
/// tem com contorno livre), a **contagem aberta**, o **degrau** e — a última — o **filete no tecto**,
/// que mostra o que ele é: o **aro** (a aresta entre a parede e a tampa) fica um quarto de círculo
/// completo, e as quinas do contorno ficam **vivas**.
///
/// ⛔⛔ **A primeira redacção desta cena mostrava outra coisa e ensinava o CONTRÁRIO do que acontece**
/// — uma peça que se parte em duas por «abertura morfológica», que é o que a nota do `sd_extrude`
/// prometia e a medição da W132 refutou. *Uma cena de smoke que ensina o contrário é pior que uma
/// cena ausente: a ausente não é acreditada.*
pub fn cena_27() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 27 — O POLIGONO de N vertices: (1) concavo de 5 · (2) irregular de 12 \
         · (3) degrau de 6 · (4) a MESMA do (1) com o filete no TECTO: o ARO fica redondo de ponta \
         a ponta e as quinas do contorno ficam VIVAS -- este filete e' da aresta de cima e de baixo."
    );
    let peca = |pontos: Vec<[f32; 2]>, round: f32, x: f32| {
        leaf(
            Primitive::Polygon {
                profile: ph2d_field::polygon_profile(pontos).expect("contorno da cena"),
                half_height: 0.10,
                round,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            // (1) O CÔNCAVO — o entalhe que só esta forma alcança.
            peca(
                vec![
                    [-0.19, -0.11],
                    [0.18, -0.06],
                    [0.07, 0.19],
                    [0.01, 0.02],
                    [-0.11, 0.14],
                ],
                0.012,
                -0.72,
            ),
            // (2) A CONTAGEM ABERTA — doze vértices, três deles reentrantes.
            peca(
                vec![
                    [0.1557, 0.0000],
                    [0.1440, 0.0831],
                    [0.0572, 0.0990],
                    [0.0000, 0.0595],
                    [-0.0974, 0.1688],
                    [-0.1294, 0.0747],
                    [-0.1043, 0.0000],
                    [-0.0812, -0.0469],
                    [-0.0728, -0.1262],
                    [0.0000, -0.2005],
                    [0.0326, -0.0564],
                    [0.0957, -0.0553],
                ],
                0.012,
                -0.24,
            ),
            // (3) O DEGRAU — seis vértices em ângulos rectos, com o filete a arredondar os dois
            // sentidos da quina (o convexo por fora, o reentrante por dentro).
            peca(
                vec![
                    [-0.20, -0.14],
                    [0.20, -0.14],
                    [0.20, 0.00],
                    [0.00, 0.00],
                    [0.00, 0.16],
                    [-0.20, 0.16],
                ],
                0.035,
                0.24,
            ),
            // (4) O FILETE NO TECTO — `0,099` contra uma meia-altura de `0,10`: o aro fica
            // redondo de ponta a ponta (uma almofada) e as quinas do contorno continuam vivas.
            peca(
                vec![
                    [-0.19, -0.11],
                    [0.18, -0.06],
                    [0.07, 0.19],
                    [0.01, 0.02],
                    [-0.11, 0.14],
                ],
                0.099,
                0.72,
            ),
            combine(Op::Union(Blend::Sharp), (0..4).map(NodeId).collect()),
        ],
        NodeId(4),
    )
}

/// ⭐⭐⭐ **A cena `=28`: O NÓ DE TORO `(p, q)`** (W134) — quatro pares, e nenhum é o vizinho do outro.
///
/// ⚠️ **As quatro respondem a perguntas diferentes**, e é isso que faz uma cena valer mais do que um
/// gate: o **trevo** `(2,3)` é a forma que dá nome à família; o `(3,2)` é **o mesmo par ao
/// contrário** e desenha outra peça, que é o que prova que os dois números não são intermutáveis; o
/// `(2,5)` mostra que **`q` aperta a corda ao tubo** sem tocar na árvore; e o `(5,2)` mostra que
/// **`p` a espalha em torno do eixo** e, com ela, o vazio no meio.
///
/// ⚠️ **A corda é sempre uma fracção do TECTO, nunca um número fixo** — o tecto depende de `p` e de
/// `q` ([`ph2d_field::knot_cord_ceiling`]), então uma corda literal desenharia quatro peças com
/// folgas diferentes e a cena leria como ruído.
pub fn cena_28() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 28 — O NO DE TORO (p,q): (1) trevo 2,3 · (2) 3,2 -- o MESMO par ao \
         contrario da outra peca · (3) 2,5 -- o q aperta a corda ao tubo · (4) 5,2 -- o p espalha \
         em volta do eixo. A corda e sempre 85% do tecto, que depende de p e de q."
    );
    let peca = |winds: u32, loops: u32, x: f32| {
        let (radius, tube) = (0.20_f32, 0.085_f32);
        leaf(
            Primitive::TorusKnot {
                radius,
                tube,
                cord: ph2d_field::knot_cord_ceiling(radius, tube, winds, loops) * 0.85,
                winds,
                loops,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            peca(2, 3, -0.72),
            peca(3, 2, -0.24),
            peca(2, 5, 0.24),
            peca(5, 2, 0.72),
            combine(Op::Union(Blend::Sharp), (0..4).map(NodeId).collect()),
        ],
        NodeId(4),
    )
}

/// ⭐⭐⭐ **A CENA 29 — a ROSCA e o SERRILHADO** (W135).
///
/// ⚠️ **As quatro têm o MESMO cilindro** e mudam só o que a rosca é: o passo, o flanco, as entradas
/// e as mãos. *A peça de fora não se mexe; o que muda é o filete que corre nela.*
///
/// # Errors
/// Só se uma das quatro violar uma cerca do documento — o que é o gate a fazer o trabalho dele.
pub fn cena_29() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 29 — A ROSCA: (1) parafuso fino · (2) passo grosso, mesmo flanco · \
         (3) QUATRO entradas -- o filete sobe quatro vezes mais depressa · (4) SERRILHADO, as duas \
         maos cruzadas. A profundidade e sempre uma fraccao do tecto, que depende do passo E do \
         flanco."
    );
    let peca = |pitch: f32, flank: f32, starts: u32, hands: u32, fraccao: f32, x: f32| {
        let radius = 0.17_f32;
        leaf(
            Primitive::Thread {
                radius,
                half_height: 0.24,
                pitch,
                depth: ph2d_field::thread_depth_ceiling(radius, pitch, flank) * fraccao,
                flank,
                starts,
                hands,
                round: 0.0,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            peca(0.045, 30.0, 1, 1, 0.75, -0.72),
            peca(0.090, 30.0, 1, 1, 0.75, -0.24),
            peca(0.090, 30.0, 4, 1, 0.75, 0.24),
            // ⚠️ **`10` entradas, e o número é o ÂNGULO DE CRUZAMENTO** — ver [`a_knurl`].
            peca(0.075, 45.0, 10, 2, 0.70, 0.72),
            combine(Op::Union(Blend::Sharp), (0..4).map(NodeId).collect()),
        ],
        NodeId(4),
    )
}

/// ⭐⭐⭐ **A CENA 30 — as CURVAS COM ESPESSURA** (W136).
///
/// ⚠️ **As duas primeiras são a MESMA primitiva** — a parábola é a Bezier com os três pontos no
/// sítio que a torna `y = k·x²`, e a cena põe-nas lado a lado para o olho confirmar o que a sonda
/// mediu a `5,5e-17`.
///
/// # Errors
/// Só se uma das quatro violar uma cerca do documento — o que é o gate a fazer o trabalho dele.
pub fn cena_30() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 30 — AS CURVAS: (1) bezier, um arco · (2) a PARABOLA, que e' a MESMA \
         primitiva com os pontos no sitio de y = k x^2 · (3) a bezier com os tres pontos EM LINHA, \
         que degenera num segmento e o campo resolve exacto · (4) a onda em anel, 8 lobulos."
    );
    let curva = |a: [f32; 2], b: [f32; 2], c: [f32; 2], x: f32| {
        leaf(
            Primitive::Bezier {
                a,
                b,
                c,
                thickness: 0.035,
                half_height: 0.06,
                round: 0.012,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    let (w, k) = (0.20_f32, 4.0_f32);
    let (radius, amplitude) = (0.20_f32, 0.055_f32);
    FieldDoc::new(
        vec![
            curva([-0.20, -0.10], [0.0, 0.26], [0.20, -0.10], -0.72),
            curva([-w, k * w * w], [0.0, -k * w * w], [w, k * w * w], -0.24),
            curva([-0.20, -0.08], [0.0, 0.0], [0.20, 0.08], 0.24),
            leaf(
                Primitive::CircleWave {
                    radius,
                    amplitude,
                    lobes: 8,
                    thickness: ph2d_field::wave_thickness_ceiling(radius, amplitude) * 0.16,
                    half_height: 0.06,
                    round: 0.012,
                    chamfer: 0.0,
                },
                Xform {
                    translation: [0.72, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            combine(Op::Union(Blend::Sharp), (0..4).map(NodeId).collect()),
        ],
        NodeId(4),
    )
}

/// ⭐⭐⭐ **A cena `=31`: AS DUAS ÚLTIMAS FORMAS DO CATÁLOGO** (W139) — e cada uma mostrada pelo
/// controlo que a define.
///
/// ⚠️ **Três cópias de cada, e não uma.** A cratera é o `depth` (raso · fundo · quase a atravessar)
/// e a lente é o `offset` (gorda · canónica · fina), que são os únicos números que as separam de
/// uma esfera. *Um gate no representante deixa o CURSO do controlo por medir, e uma cena com uma
/// cópia só faz o mesmo ao olho.*
pub fn cena_31() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 31 — AS DUAS ULTIMAS: em cima, a ESFERA COM CRATERA com a mordida a \
         0,25 / 0,50 / 0,80 do raio; em baixo, a LENTE com os centros a 0,25 / 0,50 / 0,80 do raio \
         (mais afastados = mais fina). Cada uma e' UMA peca na Hierarquia."
    );
    let r = 0.22_f32;
    let em = |p: Primitive, x: f32, y: f32| {
        leaf(
            p,
            Xform {
                translation: [x, y, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    let cratera = |k: f32, x: f32| {
        em(
            Primitive::CrateredSphere {
                radius: r,
                crater: r * 0.75,
                depth: r * k,
                round: r * 0.04,
                chamfer: 0.0,
            },
            x,
            0.32,
        )
    };
    let lente = |k: f32, x: f32| {
        em(
            Primitive::Lens {
                radius: r,
                offset: r * k,
                round: r * 0.04,
                chamfer: 0.0,
            },
            x,
            -0.32,
        )
    };
    FieldDoc::new(
        vec![
            cratera(0.25, -0.62),
            cratera(0.50, 0.0),
            cratera(0.80, 0.62),
            lente(0.25, -0.62),
            lente(0.50, 0.0),
            lente(0.80, 0.62),
            combine(Op::Union(Blend::Sharp), (0..6).map(NodeId).collect()),
        ],
        NodeId(6),
    )
}
