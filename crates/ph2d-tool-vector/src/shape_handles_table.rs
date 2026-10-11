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
    }
}

const fn on_sharp(mut d: HandleDesc, sharp: &'static [usize]) -> HandleDesc {
    d.sharp = sharp;
    d
}

const fn parked(mut d: HandleDesc) -> HandleDesc {
    d.park = true;
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
/// ângulo do 2.º vértice; o raio do canto no 1.º.
const POLYGON: &[HandleDesc] = &[
    on_sharp(h(&[0], Spot::Vertex(1), Law::Count), &[1]),
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
/// pelo ângulo da 2.ª ponta; o raio INTERIOR no 1.º vale (distância ao centro); o arredondamento
/// da ponta e do vale nas arestas que saem deles.
const STAR: &[HandleDesc] = &[
    on_sharp(h(&[0], Spot::Vertex(2), Law::Count), &[2, 3]),
    parked(on_sharp(h(&[1], Spot::Vertex(1), Law::Radial), &[2, 3])),
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
const SPIRAL: &[HandleDesc] = &[h(&[0], Spot::FromEnd(0), Law::Winding)];

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

/// As alças de `kind` (vazio = a forma não tem parâmetro com gesto natural).
#[must_use]
pub(crate) fn of(kind: ShapeKind) -> &'static [HandleDesc] {
    match kind {
        ShapeKind::RoundRect => ROUND_RECT,
        ShapeKind::Polygon => POLYGON,
        ShapeKind::Star => STAR,
        ShapeKind::Ellipse | ShapeKind::Pie => ROUND,
        ShapeKind::Arc | ShapeKind::Segment => ARC,
        ShapeKind::Spiral => SPIRAL,
        ShapeKind::ArrowRight => ARROW_RIGHT,
        ShapeKind::ArrowDouble => ARROW_DOUBLE,
        _ => &[],
    }
}
