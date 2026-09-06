//! ⭐⭐ **AS CENAS DOS LOTES DE FORMAS** (W119–W122) — as setas, os sinais e o fluxograma postos
//! lado a lado.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::lote`] responde pelas cenas do catálogo e da torção (W103–W107) e passou as `600`
//! linhas do gate de LOC do shell ao receber a cena do fluxograma. ⛔ *Split, nunca allowlist* — e
//! o corte é por assunto: aqui estão as cenas cujo trabalho é **mostrar as portas novas de um lote**.

use super::*;

/// A cena `=18` — ver o roteador.
///
/// ⭐⭐ **AS NOVE PORTAS DA W119, lado a lado** — e a cena responde a uma pergunta que a paleta não
/// responde: *elas parecem-se com o que o nome promete?* ⚠️ A seta e a seta dupla ficam **coladas**,
/// e o tubo, a anilha e o arco também: são a mesma primitiva, e vê-las juntas é o que mostra que a
/// diferença está nos números.
pub(crate) fn cena_18() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 18 — AS NOVE PORTAS DA W119: seta · seta dupla · seta dobrada · \
                 chevron · losango · segmento · tubo · anilha · arco de anel"
    );
    // Três colunas por fileira, três fileiras.
    let at = |i: f32| Xform {
        translation: [
            (i % 3.0 - 1.0) * 0.72,
            (1.0 - (i / 3.0).floor()) * 0.62,
            0.0,
        ],
        ..Xform::IDENTITY
    };
    let seta = |heads: u32| Primitive::Arrow {
        heads,
        half_length: 0.30,
        shaft: 0.066,
        head: 0.15,
        head_length: 0.165,
        half_height: 0.075,
        round: 0.024,
        chamfer: 0.0,
    };
    let anel = |inner: f32, angle: f32, half_height: f32| Primitive::Tube {
        outer: 0.30,
        inner,
        angle,
        half_height,
        round: 0.022,
        chamfer: 0.0,
    };
    FieldDoc::new(
        vec![
            leaf(seta(1), at(0.0)),
            leaf(seta(2), at(1.0)),
            leaf(
                Primitive::BentArrow {
                    run: 0.28,
                    rise: 0.28,
                    shaft: 0.058,
                    head: 0.13,
                    head_length: 0.145,
                    half_height: 0.075,
                    round: 0.020,
                    chamfer: 0.0,
                },
                at(2.0),
            ),
            leaf(
                Primitive::Chevron {
                    half_length: 0.28,
                    half_span: 0.21,
                    thickness: 0.084,
                    half_height: 0.075,
                    round: 0.020,
                    chamfer: 0.0,
                },
                at(3.0),
            ),
            leaf(
                Primitive::Rhombus {
                    half_width: 0.30,
                    half_span: 0.186,
                    half_height: 0.075,
                    round: 0.024,
                    chamfer: 0.0,
                },
                at(4.0),
            ),
            leaf(
                Primitive::CircleSegment {
                    radius: 0.30,
                    cut: -0.075,
                    half_height: 0.075,
                    round: 0.024,
                    chamfer: 0.0,
                },
                at(5.0),
            ),
            // ⚠️ **O tubo mostra-se ALTO** — chato ele lê-se como a anilha ao lado, e a cena
            // deixaria de responder o que existe para responder.
            leaf(anel(0.186, std::f32::consts::PI, 0.36), at(6.0)),
            leaf(anel(0.165, std::f32::consts::PI, 0.042), at(7.0)),
            leaf(anel(0.186, 1.0, 0.075), at(8.0)),
            combine(Op::Union(Blend::Sharp), (0..9).map(NodeId).collect()),
        ],
        NodeId(9),
    )
}

/// A cena `=19` — ver o roteador.
///
/// ⭐⭐ **AS DEZ PORTAS DA W120**, lado a lado — e a cena responde à pergunta que a paleta não
/// responde: *elas parecem-se com o que o nome promete?* ⚠️ A nuvem e o balão de pensamento ficam
/// **colados**: são a mesma primitiva, e vê-las juntas é o que mostra que a diferença é a fieira.
pub(crate) fn cena_19() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 19 — AS DEZ PORTAS DA W120: balão · oval · pensamento · nuvem · \
                 raio · escudo · etiqueta · visto · faixa · chave"
    );
    // Cinco colunas por fileira, duas fileiras.
    let at = |i: f32| Xform {
        translation: [
            (i % 5.0 - 2.0) * 0.60,
            if i < 5.0 { 0.34 } else { -0.34 },
            0.0,
        ],
        ..Xform::IDENTITY
    };
    let nuvem = |tail: f32| Primitive::Cloud {
        lobes: 5,
        half_width: 0.25,
        half_span: 0.125,
        tail,
        half_height: 0.06,
        round: 0.025,
        chamfer: 0.0,
    };
    FieldDoc::new(
        vec![
            leaf(
                Primitive::SpeechRect {
                    half_width: 0.25,
                    half_span: 0.165,
                    tail: 0.11,
                    half_height: 0.06,
                    round: 0.025,
                    chamfer: 0.0,
                },
                at(0.0),
            ),
            leaf(
                Primitive::SpeechOval {
                    half_width: 0.25,
                    half_span: 0.155,
                    tail: 0.11,
                    half_height: 0.06,
                    round: 0.025,
                    chamfer: 0.0,
                },
                at(1.0),
            ),
            leaf(nuvem(0.10), at(2.0)),
            leaf(nuvem(0.0), at(3.0)),
            leaf(
                Primitive::Bolt {
                    half_width: 0.155,
                    half_span: 0.25,
                    half_height: 0.06,
                    round: 0.012,
                    chamfer: 0.0,
                },
                at(4.0),
            ),
            leaf(
                Primitive::Shield {
                    half_width: 0.195,
                    half_span: 0.25,
                    half_height: 0.06,
                    round: 0.025,
                    chamfer: 0.0,
                },
                at(5.0),
            ),
            leaf(
                Primitive::Tag {
                    half_width: 0.25,
                    half_span: 0.145,
                    point: 0.14,
                    hole: 0.038,
                    half_height: 0.06,
                    round: 0.025,
                    chamfer: 0.0,
                },
                at(6.0),
            ),
            leaf(
                Primitive::Check {
                    half_width: 0.25,
                    half_span: 0.18,
                    thickness: 0.065,
                    half_height: 0.06,
                    round: 0.025,
                    chamfer: 0.0,
                },
                at(7.0),
            ),
            leaf(
                Primitive::Banner {
                    half_width: 0.25,
                    half_span: 0.125,
                    notch: 0.08,
                    half_height: 0.06,
                    round: 0.025,
                    chamfer: 0.0,
                },
                at(8.0),
            ),
            leaf(
                Primitive::Brace {
                    half_span: 0.25,
                    thickness: 0.06,
                    half_height: 0.06,
                    round: 0.025,
                    chamfer: 0.0,
                },
                at(9.0),
            ),
            combine(Op::Union(Blend::Sharp), (0..10).map(NodeId).collect()),
        ],
        NodeId(10),
    )
}

/// ⭐⭐ **A cena `=20`: AS QUATRO PORTAS DA W122** — o lote do fluxograma, lado a lado.
///
/// ⚠️ **Cada uma no ponto que a EXERCITA**, e não no de nascimento: o paralelogramo inclinado (a
/// zero ele é o retângulo), o mostrador com bico (a zero ele é o atraso) e o conector com bico (a
/// zero ele é o retângulo). *Uma cena que mostra três formas no ponto em que elas são outra coisa
/// ensina o contrário do que diz.*
pub(crate) fn cena_20() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 20 — AS QUATRO PORTAS DA W122: paralelogramo · atraso · mostrador · \
                 conector de página"
    );
    let at = |i: f32| Xform {
        translation: [(i - 1.5) * 0.62, 0.0, 0.0],
        ..Xform::IDENTITY
    };
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Parallelogram {
                    half_width: 0.20,
                    half_span: 0.14,
                    skew: 0.075,
                    half_height: 0.06,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(0.0),
            ),
            leaf(
                Primitive::Delay {
                    half_width: 0.25,
                    half_span: 0.14,
                    half_height: 0.06,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(1.0),
            ),
            leaf(
                Primitive::Display {
                    half_width: 0.25,
                    half_span: 0.14,
                    point: 0.145,
                    half_height: 0.06,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(2.0),
            ),
            leaf(
                Primitive::OffPage {
                    half_width: 0.20,
                    half_span: 0.175,
                    point: 0.11,
                    half_height: 0.06,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(3.0),
            ),
            combine(Op::Union(Blend::Sharp), (0..4).map(NodeId).collect()),
        ],
        NodeId(4),
    )
}

/// ⭐⭐ **A cena `=21`: AS DUAS QUE SAÍRAM DO DESENHO** (W123) — a espiral e o documento.
///
/// ⚠️ **A espiral com TRÊS voltas e o documento COM onda**: as duas no ponto que as exercita, e não
/// no de nascimento — a zero, uma é um arco e o outro é um retângulo.
pub(crate) fn cena_21() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 21 — AS DUAS DA W123: espiral (3 voltas) · documento (base ondulada). \
         Nenhuma delas tem um segmento desenhado."
    );
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Spiral {
                    radius: 0.06,
                    pitch: 0.09,
                    turns: 3.0,
                    thickness: 0.025,
                    half_height: 0.06,
                    round: 0.012,
                    chamfer: 0.0,
                },
                Xform {
                    translation: [-0.40, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            leaf(
                Primitive::Document {
                    half_width: 0.30,
                    half_span: 0.20,
                    wave: 0.07,
                    half_height: 0.06,
                    round: 0.02,
                    chamfer: 0.0,
                },
                Xform {
                    translation: [0.42, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            combine(Op::Union(Blend::Sharp), (0..2).map(NodeId).collect()),
        ],
        NodeId(2),
    )
}

/// ⭐⭐ **A cena `=22`: A MOLA E A REDE** (W124) — as duas de maior alcance do levantamento.
pub(crate) fn cena_22() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 22 — A MOLA (3 voltas) e a REDE GYROID (4 células). Nenhuma delas tem \
         um segmento desenhado, e a rede é UMA LINHA de fórmula."
    );
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Helix {
                    radius: 0.20,
                    pitch: 0.10,
                    turns: 3.0,
                    thickness: 0.03,
                    round: 0.0,
                    chamfer: 0.0,
                },
                Xform {
                    translation: [-0.38, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            leaf(
                Primitive::Gyroid {
                    half: [0.24; 3],
                    cell: 0.12,
                    thickness: 0.012,
                    round: 0.004,
                    chamfer: 0.0,
                },
                Xform {
                    translation: [0.38, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
            ),
            combine(Op::Union(Blend::Sharp), (0..2).map(NodeId).collect()),
        ],
        NodeId(2),
    )
}

/// ⭐ **A cena `=23`: O CILINDRO COM BOJO** (W125) — a única forma da wave, mostrada pelo que a
/// distingue.
///
/// ⚠️ **Três cópias com o bojo a crescer, e não uma.** A forma nasce da fórmula exacta do Quílez e
/// o bojo é o ÚNICO controle que ela tem; uma cópia só mostraria um cilindro com o aro mole e não
/// diria que o knob percorre de *cilindro* (bojo `0`) a *cápsula* (bojo = `min(raio, meia-altura)`).
/// *Um gate no representante deixa o curso do controle por medir* — e uma cena com uma cópia só faz
/// o mesmo ao olho.
pub(crate) fn cena_23() -> Result<FieldDoc, ph2d_field::FieldError> {
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
pub(crate) fn cena_24() -> Result<FieldDoc, ph2d_field::FieldError> {
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
pub(crate) fn cena_25() -> Result<FieldDoc, ph2d_field::FieldError> {
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
