//! **A TABELA das alças** — módulo irmão de [`crate::shape_handles`] (a lei lá; aqui, forma a
//! forma, que campo vira alça e em que ponto do contorno ela nasce). Os índices de vértice são
//! os do `cook` de cada forma (`ph2d_vec_scene::{shapes, round, arrows, …}`); o gate
//! `every_handle_sits_on_its_outline` prova que cada um pousa onde diz.
//!
//! Um campo que não está aqui não tem gesto natural (uma ESCOLHA, a suavização do canto): fica
//! só na fila ② — `shape_handles::fields_without_handle` conta-os.

use ph2d_vec_scene::ShapeKind;

use crate::shape_handles::{Alone, HandleDesc, Law, Spot};

const fn h(fields: &'static [usize], spot: Spot, law: Law) -> HandleDesc {
    HandleDesc {
        fields,
        spot,
        law,
        sharp: &[],
        alone: None,
        park: false,
        inset: false,
    }
}

/// A alça mais comum: o vértice `i` que persegue a mão pela projecção no trilho.
const fn at(fields: &'static [usize], i: usize) -> HandleDesc {
    h(fields, Spot::Vertex(i), Law::Nearest)
}

const fn on_sharp(mut d: HandleDesc, sharp: &'static [usize]) -> HandleDesc {
    d.sharp = sharp;
    d
}

const fn parked(mut d: HandleDesc) -> HandleDesc {
    d.park = true;
    d
}

/// Desenhada para DENTRO da caixa: o ponto dela vive sobre uma aresta da caixa (F10).
const fn inset(mut d: HandleDesc) -> HandleDesc {
    d.inset = true;
    d
}

/// Com Alt, esta alça escreve só `fields` e devolve a diferença aos `keep` (um lado sozinho).
const fn alone(mut d: HandleDesc, fields: &'static [usize], keep: &'static [usize]) -> HandleDesc {
    d.alone = Some(Alone { fields, keep });
    d
}

/// O canto do rectângulo arredondado: no lado HORIZONTAL do canto (o `rx` do Inkscape; o
/// Affinity também o põe no lado de cima). Arrastar move os QUATRO (o raio base, cada canto
/// guarda o seu desvio — o *Single radius* de omissão do Affinity, a Selection do Illustrator);
/// com Alt, só este canto.
const fn round_corner(
    corner: usize,
    next: bool,
    alone: &'static [usize],
    keep: &'static [usize],
) -> HandleDesc {
    HandleDesc {
        fields: &[0],
        spot: Spot::Rounding { corner, next },
        law: Law::Nearest,
        sharp: &[0, 1, 2, 3],
        alone: Some(Alone {
            fields: alone,
            keep,
        }),
        park: true,
        inset: false,
    }
}

/// Os vértices do rectângulo vivo (`shapes::rectangle`, Y-para-CIMA): 0 = BL, 1 = BR, 2 = TR,
/// 3 = TL. Os campos do `round_rect_radii`: 0 = raio base (o TL), 1/2/3 = desvio TR/BR/BL.
const ROUND_RECT: &[HandleDesc] = &[
    round_corner(3, false, &[0], &[1, 2, 3]),
    round_corner(2, true, &[1], &[]),
    round_corner(1, false, &[2], &[]),
    round_corner(0, true, &[3], &[]),
];

/// O polígono (`shapes::regular_polygon`, 1.º vértice às 12h, anti-horário): os LADOS pelo
/// ângulo do 2.º vértice; o raio do canto no 1.º; a ENVERGADURA (F10) no meio da 2.ª aresta (a
/// 1.ª é a do arredondamento, que satura no meio dela). O tipo do canto e o liso: o cartão.
const POLYGON: &[HandleDesc] = &[
    on_sharp(h(&[0], Spot::Vertex(1), Law::Count), &[1]),
    on_sharp(h(&[3], Spot::EdgeMid(1), Law::Nearest), &[1]),
    parked(on_sharp(
        h(
            &[1],
            Spot::Rounding {
                corner: 0,
                next: true,
            },
            Law::Nearest,
        ),
        &[1],
    )),
];

/// A estrela (`shapes::star`, pontas nos pares, vales nos ímpares, 1.ª ponta às 12h): as PONTAS
/// pelo ângulo da 2.ª ponta; o raio INTERIOR e a TORÇÃO (F10) no 1.º vale (a distância e o ângulo,
/// como o Inkscape; Ctrl = só o raio); o arredondamento da ponta e do vale nas arestas que saem
/// deles; a ENVERGADURA (F10) no meio da 3.ª aresta.
const STAR: &[HandleDesc] = &[
    on_sharp(h(&[0], Spot::Vertex(2), Law::Count), &[2, 3]),
    on_sharp(h(&[5], Spot::EdgeMid(2), Law::Nearest), &[2, 3]),
    parked(on_sharp(
        h(&[1], Spot::Vertex(1), Law::Polar { angle: 7 }),
        &[2, 3],
    )),
    parked(on_sharp(
        h(
            &[2],
            Spot::Rounding {
                corner: 0,
                next: true,
            },
            Law::Nearest,
        ),
        &[2, 3],
    )),
    parked(on_sharp(
        h(
            &[3],
            Spot::Rounding {
                corner: 1,
                next: true,
            },
            Law::Nearest,
        ),
        &[2, 3],
    )),
];

/// O início e o fim de um arco da família do círculo (0 = abertura, 1 = início).
const ARC_START: HandleDesc = h(
    &[1],
    Spot::Ellipse {
        start: 1,
        sweep: None,
        mid: false,
        scale: None,
    },
    Law::ArcStart { sweep: 0 },
);
const ARC_END: HandleDesc = h(
    &[0],
    Spot::Ellipse {
        start: 1,
        sweep: Some(0),
        mid: false,
        scale: None,
    },
    Law::Sweep { start: 1 },
);

/// A família do círculo (`round::ellipse_sweep`): 0 = abertura, 1 = início, 2 = furo. O furo no
/// meio da fatia, a partir do centro (o Affinity: *Hole radius*, radial); o início e o fim sobre
/// a elipse. Numa elipse INTEIRA o fim está sobre o início e vem por último — puxá-lo abre a pizza
/// (o *pie widget* do Illustrator).
const ROUND: &[HandleDesc] = &[
    parked(h(
        &[2],
        Spot::Ellipse {
            start: 1,
            sweep: Some(0),
            mid: true,
            scale: Some(2),
        },
        Law::Radial,
    )),
    ARC_START,
    ARC_END,
];

/// O arco aberto e o segmento: só o início e o fim.
const ARC: &[HandleDesc] = &[ARC_START, ARC_END];

/// A espiral (`shapes::spiral`): as VOLTAS na ponta de fora, por volta inteira do ponteiro.
/// As voltas na ponta de fora; o RAIO INTERNO (F10) na de dentro — a distância ao centro, como a
/// alça interna do Inkscape (estaciona: no zero ela é o centro). A divergência: o cartão.
const SPIRAL: &[HandleDesc] = &[
    h(&[0], Spot::FromEnd(0), Law::Winding),
    parked(h(&[2], Spot::Vertex(0), Law::Radial)),
];

/// A seta (`arrows::arrow_block`, autorada em `(u, v)` com `v` do topo): 0 = haste, 1 = comprimento
/// da cabeça, 2 = largura da cabeça. A espessura da haste no canto de trás dela (vértice 0); a
/// cabeça pela quina de trás (vértice 2) — um eixo cada.
const ARROW_RIGHT: &[HandleDesc] = &[
    h(&[0], Spot::Vertex(0), Law::Nearest),
    h(&[1, 2], Spot::Vertex(2), Law::Nearest),
];

/// A seta dupla (`arrows::arrow_double`): a haste pelo vértice 2, a cabeça da direita pelo 4.
const ARROW_DOUBLE: &[HandleDesc] = &[
    h(&[0], Spot::Vertex(2), Law::Nearest),
    h(&[1, 2], Spot::Vertex(4), Law::Nearest),
];

/// A seta dobrada (`arrows::arrow_bent`: a cauda sai do canto BL, sobe pela haste e a ponta aponta
/// para cima). 0 = espessura, 1 = comprimento da cabeça, 2 = largura da cabeça, 3 = raio do
/// cotovelo. A espessura no ÚLTIMO vértice (`(0, 1 − bv)`, a face de cima da cauda — só ela o
/// move); a cabeça pela quina ESQUERDA dela (o 5 do contorno com o cotovelo vivo: os arcos dos
/// cotovelos mudam a contagem antes e depois dela) — comprimento num eixo, largura no outro, como a
/// seta recta; o cotovelo no 1.º vértice, onde o arco de fora começa na aresta de baixo (com raio
/// zero é a própria quina — o mesmo ponto, contínuo).
const ARROW_BENT: &[HandleDesc] = &[
    h(&[0], Spot::FromEnd(0), Law::Nearest),
    on_sharp(at(&[1, 2], 5), &[3]),
    at(&[3], 1),
];

/// A divisa (`arrows::chevron`): 0 = profundidade da ponta (o ombro de cima, vértice 1), 1 = o
/// entalhe (vértice 5, no eixo — no mínimo cai no meio da aresta esquerda, a alça de escala).
const CHEVRON: &[HandleDesc] = &[parked(at(&[0], 1)), parked(at(&[1], 5))];

/// O paralelogramo (`flow::parallelogram`): a inclinação na quina de cima à esquerda (vértice 0 —
/// com zero é o canto da caixa).
const PARALLELOGRAM: &[HandleDesc] = &[parked(at(&[0], 0))];

/// O trapézio (`flow::trapezoid`): as DUAS quinas de cima (vértices 0 e 1) andam na horizontal —
/// o *Trapezoid* do Affinity; sem Alt as duas juntas (simétrico), com Alt só a agarrada (o
/// *Left/Right point*, F10): a esquerda escreve o recuo e devolve a diferença ao desvio da direita.
const TRAPEZOID: &[HandleDesc] = &[
    alone(parked(at(&[0], 0)), &[0], &[1]),
    alone(parked(at(&[0], 1)), &[1], &[]),
];
/// O trapézio invertido: as quinas de BAIXO (vértices 2 = direita e 3 = esquerda).
const TRAPEZOID_FLIP: &[HandleDesc] = &[
    alone(parked(at(&[0], 2)), &[1], &[]),
    alone(parked(at(&[0], 3)), &[0], &[1]),
];

/// O MEIO do losango (F10, o *Mid point* do Affinity) no vértice da direita — que vive sobre a
/// aresta da caixa, onde mora a alça de escala: desenha-se para dentro.
const DIAMOND: &[HandleDesc] = &[inset(at(&[0], 1))];

/// O hexágono achatado (`flow::hexagon_flat`): o corte na quina de cima à direita (vértice 1).
const HEXAGON_FLAT: &[HandleDesc] = &[parked(at(&[0], 1))];

/// O cilindro (`flow::cylinder`, a tampa de cima é o arco 0..2): a altura da tampa no fim dela, à
/// direita (vértice 2), na vertical.
const CYLINDER: &[HandleDesc] = &[parked(at(&[0], 2))];

/// O documento (`flow::document`): a onda na ponta direita dela (vértice 2, `(1, 1 − wave)`).
const DOCUMENT: &[HandleDesc] = &[parked(at(&[0], 2))];

/// O ecrã (`flow::display`, o bico à esquerda): o nariz no início da aresta de cima (vértice 0).
const DISPLAY: &[HandleDesc] = &[parked(at(&[0], 0))];

/// O conector de página (`flow::offpage`): a ponta no ombro direito (vértice 2, na vertical).
const OFF_PAGE: &[HandleDesc] = &[parked(at(&[0], 2))];

/// O colchete de nota (`flow::note_bracket`, aberto): a aba no fim de cima (vértice 0).
const NOTE_BRACKET: &[HandleDesc] = &[parked(at(&[0], 0))];

/// O balão rectangular (`bubbles::speech_rect`, lido no contorno com os cantos VIVOS — os arcos
/// mudam a contagem: 0 = canto, 1 = canto oposto, 2 = flanco `pa` da base, 3 = BICO, 4 = `pb`; o
/// canto rodado por `place` segue o sector do bico). O raio no canto 0 (o mais longe do rabo); o
/// flanco da base leva a largura dela ao longo da aresta e a faixa do rabo (`room`) através dela —
/// eixos independentes.
///
/// O BICO (vértice 3) é um ponto 2-D (x e y, o *Callout* do Affinity), por último (por cima), pela
/// lei `BoxPoint`: os dois campos SÃO a posição da mão na caixa (o `fit` cola o bico à aresta e
/// troca de sector num salto que nenhuma busca sobre o vértice atravessava).
const SPEECH_RECT: &[HandleDesc] = &[
    parked(on_sharp(
        h(
            &[0],
            Spot::Rounding {
                corner: 0,
                next: true,
            },
            Law::Nearest,
        ),
        &[0],
    )),
    on_sharp(at(&[3, 4], 2), &[0]),
    on_sharp(h(&[1, 2], Spot::Vertex(3), Law::BoxPoint), &[0]),
];

/// O balão oval (`bubbles::speech_oval`): a meia-abertura no flanco `P(tb)` (vértice 0, desliza
/// pela elipse); o BICO 2-D no último vértice (o arco muda de contagem com a abertura), por cima. A
/// faixa do rabo NÃO tem alça: encolhe a elipse inteira e move o flanco junto com a abertura (o
/// par no flanco é acoplado: 1,3e-4 de erro nas 3 rondas) — nenhum vértice anda só com ela.
const SPEECH_OVAL: &[HandleDesc] = &[at(&[2], 0), h(&[0, 1], Spot::FromEnd(0), Law::BoxPoint)];

/// O pensamento (`bubbles::thought`): o corpo (a nuvem) é o contorno principal, as bolhas os
/// sub-contornos (a 0 é a menor, centrada no bico). Os gomos pelo ângulo do vértice 6 (medido
/// 5..24: o ângulo do 4, 6 e 8 é MONÓTONO na contagem, o do 10 recua; o 6 cobre 136°, o 4 só 56°);
/// o BICO 2-D no 1.º vértice da bolha 0. O número de bolhas NÃO tem alça: a bolha 0 não muda com
/// ele, e qualquer outra deixa de existir com uma bolha só.
const THOUGHT: &[HandleDesc] = &[
    h(&[3], Spot::Vertex(6), Law::Count),
    h(&[0, 1], Spot::Sub(0, 0), Law::BoxPoint),
];

/// A explosão (`bubbles::burst`, pontas nos pares, vales nos ímpares): as PONTAS pelo ângulo do
/// vértice 8 (a 5.ª ponta — medido: com o jitter de fábrica o ângulo do 2 recua sete vezes na
/// faixa 5..30, o do 8 duas, de < 1°, e cobre 235° em vez de 61°); o raio interior no 1.º vale
/// (distância ao centro, como a estrela).
const BURST: &[HandleDesc] = &[
    h(&[0], Spot::Vertex(8), Law::Count),
    parked(h(&[1], Spot::Vertex(1), Law::Radial)),
];

/// A chaveta (`bubbles::brace`, aberta; o BICO à esquerda): o aperto no bico (vértice 3, na
/// vertical); o talo e o braço no fim do arco de baixo (vértice 1 = `(talo, 1 − braço)`) — um
/// eixo cada.
const BRACE: &[HandleDesc] = &[at(&[1], 3), at(&[2, 0], 1)];

/// O coração (`symbols::heart`, dois vértices): a fenda no vértice 0, na vertical (o *Heart* do
/// Affinity).
const HEART: &[HandleDesc] = &[at(&[0], 0)];

/// A nuvem (`bubbles::cloud`): os gomos pelo ângulo do vértice 6 (medido 5..24 com a
/// irregularidade de fábrica: o único de 4/6/8/10 sem recuo). ⛔ O fofo NÃO tem alça: o `refit`
/// normaliza a nuvem à caixa, e o único vértice de índice estável (o 0, o vale antes do 1.º gomo —
/// os arcos mudam de contagem com o fofo) anda 1,5 % do raio na faixa inteira, nada abaixo de
/// ~0,66, e deixa de ser monótono com a suavização (0,90 → 0,907 → 0,899).
const CLOUD: &[HandleDesc] = &[h(&[0], Spot::Vertex(6), Law::Count)];

/// O raio (`symbols::bolt`): a cintura pelo degrau do meio (vértice 2, o par 2–8 abre com ela).
const BOLT: &[HandleDesc] = &[at(&[0], 2)];

/// A lua (`symbols::moon`): a fase no meio do terminador (vértice 3), no eixo horizontal — o
/// *Crescent* do Affinity.
const MOON: &[HandleDesc] = &[at(&[0], 3)];

/// A gota (`symbols::drop`): o ápice é fixo; a ponta é o ponto de tangência T₊ (vértice 1), que
/// desliza pela barriga com ela (o arco muda de contagem; o 1 é sempre o início dele).
const DROP: &[HandleDesc] = &[at(&[0], 1)];

/// O escudo (`symbols::shield`): a curva no fim do ombro direito (vértice 2), na vertical.
const SHIELD: &[HandleDesc] = &[at(&[0], 2)];

/// A etiqueta (`symbols::tag`): a ponta no início da aresta de cima (vértice 1); o furo no 1.º
/// vértice do olho (sub-contorno 0, às 3h dele — anda na horizontal; o olho não é centrado na
/// caixa). Com furo zero o olho não existe e a alça dele some.
const TAG: &[HandleDesc] = &[parked(at(&[0], 1)), h(&[1], Spot::Sub(0, 0), Law::Nearest)];

/// A engrenagem (`symbols::gear`, 1.º dente às 12h): os DENTES pelo ângulo da 2.ª ponta (vértice 4
/// — monótono 4..30); a profundidade no 1.º vale (vértice 2, distância ao centro); o furo no 1.º
/// vértice do eixo (sub-contorno 0, às 3h — o *Hole radius* do Affinity, distância ao centro). Sem
/// `park`: com furo zero o eixo não existe e não há ponto onde estacionar.
const GEAR: &[HandleDesc] = &[
    h(&[0], Spot::Vertex(4), Law::Count),
    h(&[1], Spot::Vertex(2), Law::Radial),
    h(&[2], Spot::Sub(0, 0), Law::Radial),
    // F10: a LARGURA do dente no fim do topo do 1.º (a envergadura dos flancos: o cartão).
    at(&[3], 1),
];

/// A cruz (`symbols::cross`): o braço na quina INTERIOR de cima à direita (vértice 4), na diagonal
/// — no mínimo cai junto do centro (o pivô).
const CROSS: &[HandleDesc] = &[parked(at(&[0], 4))];

/// O visto (`symbols::check`): a espessura no cotovelo de dentro (vértice 1).
const CHECK: &[HandleDesc] = &[at(&[0], 1)];

/// A faixa (`symbols::banner`): o entalhe no «V» da cauda direita (vértice 8), na horizontal.
const BANNER: &[HandleDesc] = &[at(&[0], 8)];

/// O cubo (`iso::iso_cube`, o hexágono 0 = topo, 1 = ombro direito): a inclinação no topo (na
/// horizontal), a altura no ombro (na vertical). O ponto de vista é uma escolha.
const ISO_CUBE: &[HandleDesc] = &[at(&[1], 0), at(&[0], 1)];

/// O cone (`iso::iso_cone`): o ápice é fixo; a boca é o ponto de tangência do lado direito
/// (vértice 1, o início do arco da base).
const ISO_CONE: &[HandleDesc] = &[at(&[0], 1)];

/// A pirâmide (`iso::iso_pyramid`: ápice, direita, frente, esquerda): a altura no lado direito, a
/// inclinação na frente (na horizontal).
const ISO_PYRAMID: &[HandleDesc] = &[at(&[0], 1), at(&[1], 2)];

/// O processo pré-definido (`flow::predefined_process`): as barras são sub-contornos (rectas); a
/// largura no topo da barra esquerda (sub-contorno 0, vértice 0), na horizontal.
const PREDEFINED_PROCESS: &[HandleDesc] = &[parked(h(&[0], Spot::Sub(0, 0), Law::Nearest))];

/// O osso (`symbols_rig::bone`): o olho no 1.º vértice dele (sub-contorno 0, às 3h — o olho está no
/// ombro, não no centro da caixa).
const BONE: &[HandleDesc] = &[h(&[0], Spot::Sub(0, 0), Law::Nearest)];

/// O segmento de corda (`symbols_rig::rope_segment`): a quina interior de cima à esquerda (vértice
/// 2 = `(nó, 0,5 − corda/2)`) — a corda num eixo, o nó no outro.
const ROPE_SEGMENT: &[HandleDesc] = &[at(&[0, 1], 2)];

/// As alças de `kind` (vazio = a forma não tem parâmetro com gesto natural).
#[must_use]
pub(crate) fn of(kind: ShapeKind) -> &'static [HandleDesc] {
    use ShapeKind as K;
    match kind {
        K::RoundRect => ROUND_RECT,
        K::Polygon => POLYGON,
        K::Star => STAR,
        K::Ellipse | K::Pie => ROUND,
        K::Arc | K::Segment => ARC,
        K::Spiral => SPIRAL,
        K::ArrowRight => ARROW_RIGHT,
        K::ArrowDouble => ARROW_DOUBLE,
        K::ArrowBent => ARROW_BENT,
        K::Chevron => CHEVRON,
        K::Parallelogram => PARALLELOGRAM,
        K::Trapezoid => TRAPEZOID,
        K::Diamond => DIAMOND,
        K::TrapezoidFlip => TRAPEZOID_FLIP,
        K::HexagonFlat => HEXAGON_FLAT,
        K::Cylinder => CYLINDER,
        K::Document => DOCUMENT,
        K::Display => DISPLAY,
        K::OffPage => OFF_PAGE,
        K::NoteBracket => NOTE_BRACKET,
        K::SpeechRect => SPEECH_RECT,
        K::SpeechOval => SPEECH_OVAL,
        K::Thought => THOUGHT,
        K::Burst => BURST,
        K::Brace => BRACE,
        K::Heart => HEART,
        K::Cloud => CLOUD,
        K::Bolt => BOLT,
        K::Moon => MOON,
        K::Drop => DROP,
        K::Shield => SHIELD,
        K::Tag => TAG,
        K::Gear => GEAR,
        K::Cross => CROSS,
        K::Check => CHECK,
        K::Banner => BANNER,
        K::IsoCube => ISO_CUBE,
        K::IsoCone => ISO_CONE,
        K::IsoPyramid => ISO_PYRAMID,
        K::RopeSegment => ROPE_SEGMENT,
        K::PredefinedProcess => PREDEFINED_PROCESS,
        K::Bone => BONE,
        // Sem campo: Rectangle, Line, Diamond, Pill, Delay, Junction.
        _ => &[],
    }
}
