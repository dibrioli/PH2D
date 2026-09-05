//! ⭐⭐ **AS CENAS DO LOTE DE FORMAS E DA TORÇÃO** (W103–W107) — o catálogo posto lado a lado, e o
//! deformador que só se vê módulo à simetria da secção.
//!
//! # Por que um arquivo irmão
//!
//! O [`super`] é o **roteador**, e ele passou as `600` linhas do gate de LOC do shell. ⛔ *Split,
//! nunca allowlist* — e o corte é por assunto: estas quatro respondem *«o catálogo existe e vê-se»*,
//! e as do irmão [`super::edge`] respondem *«o que os dois recuos de uma aresta fazem»*.
//!
//! ⚠️ **O gate que as apanhou vive em `shells/desktop/tests/`**, e o `cargo test --bins` **não lhe
//! toca** — a mesma cegueira que o §5 do `CLAUDE.md` já nomeia, e que esta linha já pagou uma vez.

use super::*;

/// A cena `=11` — ver o roteador.
pub(crate) fn cena_11() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 11 — O LOTE DA W103: estrela de 5 pontas · gaiola de caixa · \
                 elipsóide, lado a lado"
    );
    let x = |v: f32| Xform {
        translation: [v, 0.0, 0.0],
        ..Xform::IDENTITY
    };
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Star {
                    points: 5,
                    outer: 0.30,
                    inner: 0.12,
                    half_height: 0.10,
                    round: 0.020,
                    chamfer: 0.0,
                },
                x(-0.62),
            ),
            leaf(
                Primitive::BoxFrame {
                    half: [0.26, 0.26, 0.26],
                    thickness: 0.078,
                    round: 0.020,
                    chamfer: 0.0,
                },
                x(0.0),
            ),
            leaf(
                Primitive::Ellipsoid {
                    radii: [0.30, 0.165, 0.24],
                },
                x(0.62),
            ),
            // ⚠️ Aresta viva na junção, como as cenas 9 e 10: elas não se tocam, e um filete
            // de junção seria um número que não faz nada.
            combine(
                Op::Union(Blend::Sharp),
                vec![NodeId(0), NodeId(1), NodeId(2)],
            ),
        ],
        NodeId(3),
    )
}

/// A cena `=12` — ver o roteador.
pub(crate) fn cena_12() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 12 — OS SEIS SÓLIDOS DA W106: octaedro · cone de pontas \
                 arredondadas · esfera cortada · cúpula oca · elo de corrente · ângulo sólido"
    );
    // ⚠️ **Em fileira, com a MESMA escala** — é a disposição que deixa comparar formas, e a
    // razão de ela existir é a mesma da cena 11: um smoke que mostra uma forma de cada vez
    // não responde *«o catálogo está cheio?»*, que é a pergunta desta wave.
    let at = |i: f32| Xform {
        translation: [(i - 2.5) * 0.62, 0.0, 0.0],
        ..Xform::IDENTITY
    };
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Octahedron {
                    radius: 0.28,
                    round: 0.03,
                    chamfer: 0.0,
                },
                at(0.0),
            ),
            leaf(
                Primitive::RoundCone {
                    bottom: 0.20,
                    top: 0.08,
                    half_height: 0.22,
                },
                at(1.0),
            ),
            leaf(
                Primitive::CutSphere {
                    radius: 0.27,
                    cut: 0.10,
                    round: 0.03,
                    chamfer: 0.0,
                },
                at(2.0),
            ),
            leaf(
                Primitive::HollowDome {
                    radius: 0.27,
                    cut: 0.04,
                    thickness: 0.05,
                    round: 0.012,
                    chamfer: 0.0,
                },
                at(3.0),
            ),
            leaf(
                Primitive::Link {
                    major: 0.16,
                    minor: 0.06,
                    length: 0.14,
                },
                at(4.0),
            ),
            leaf(
                Primitive::SolidAngle {
                    radius: 0.30,
                    angle: 0.6,
                    round: 0.03,
                    chamfer: 0.0,
                },
                at(5.0),
            ),
            combine(
                Op::Union(Blend::Sharp),
                vec![
                    NodeId(0),
                    NodeId(1),
                    NodeId(2),
                    NodeId(3),
                    NodeId(4),
                    NodeId(5),
                ],
            ),
        ],
        NodeId(6),
    )
}

/// A cena `=13` — ver o roteador.
pub(crate) fn cena_13() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 13 — AS OITO CHAPAS DA W106: engrenagem · cruz · coração · \
                 lua · gota · fatia · trapézio · vesica"
    );
    let at = |i: f32| Xform {
        translation: [
            (i % 4.0 - 1.5) * 0.62,
            (if i < 4.0 { 0.34 } else { -0.34 }),
            0.0,
        ],
        ..Xform::IDENTITY
    };
    FieldDoc::new(
        vec![
            leaf(
                Primitive::Gear {
                    teeth: 12,
                    root: 0.19,
                    outer: 0.27,
                    tooth: 0.45,
                    half_height: 0.07,
                    round: 0.012,
                    chamfer: 0.0,
                },
                at(0.0),
            ),
            leaf(
                Primitive::Cross {
                    arm: 0.27,
                    width: 0.08,
                    half_height: 0.07,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(1.0),
            ),
            leaf(
                Primitive::Heart {
                    size: 0.17,
                    half_height: 0.07,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(2.0),
            ),
            leaf(
                Primitive::Moon {
                    radius: 0.27,
                    bite: 0.24,
                    offset: 0.13,
                    half_height: 0.07,
                    round: 0.012,
                    chamfer: 0.0,
                },
                at(3.0),
            ),
            leaf(
                Primitive::Drop {
                    radius: 0.14,
                    height: 0.36,
                    half_height: 0.07,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(4.0),
            ),
            leaf(
                Primitive::Pie {
                    radius: 0.27,
                    angle: 1.0,
                    half_height: 0.07,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(5.0),
            ),
            leaf(
                Primitive::Trapezoid {
                    bottom: 0.27,
                    top: 0.12,
                    half_width: 0.17,
                    half_height: 0.07,
                    round: 0.02,
                    chamfer: 0.0,
                },
                at(6.0),
            ),
            leaf(
                Primitive::Vesica {
                    radius: 0.28,
                    offset: 0.15,
                    half_height: 0.07,
                    round: 0.012,
                    chamfer: 0.0,
                },
                at(7.0),
            ),
            combine(
                Op::Union(Blend::Sharp),
                vec![
                    NodeId(0),
                    NodeId(1),
                    NodeId(2),
                    NodeId(3),
                    NodeId(4),
                    NodeId(5),
                    NodeId(6),
                    NodeId(7),
                ],
            ),
        ],
        NodeId(8),
    )
}

/// A cena `=14` — ver o roteador.
pub(crate) fn cena_14() -> Result<FieldDoc, ph2d_field::FieldError> {
    println!(
        "[field-smoke] cena 14 — A TORÇÃO (W107): barra CHATA reta · torcida inteira · \
                 torcida só do meio para cima (a BANDA)"
    );
    // ⚠️ **Três colunas IGUAIS**, e é isso que faz a cena responder: uma torção mostrada
    // sozinha não diz se ela torceu — diz que a forma é assim. A da esquerda é a régua.
    //
    // ⛔⛔ **A SECÇÃO É CHATA, e a 1.ª versão desta cena usava uma QUADRADA — o report do
    // Enio foi *«nada aparece torcido»*.** Uma torção só se vê **módulo a simetria da
    // secção**: um quadrado repete-se a cada 90°, um hexágono a cada 60°, e um cilindro é
    // invisível a qualquer ângulo. Medido a 0,25 voltas/un (112° no total):
    //
    // | secção | simetria | variação da silhueta |
    // |---|---|---:|
    // | cilindro | contínua | **`+0,0 %`** |
    // | prisma 6 | 60° | `+11,9 %` |
    // | caixa quadrada | 90° | `+37,3 %` |
    // | **caixa 3:1** | **180°** | **`+146,0 %`** |
    //
    // *Uma cena de smoke que demonstra a feature na forma que a esconde é pior que nenhuma.*
    let coluna = |x: f32, mods: Vec<ph2d_field::Unary>| {
        let mut n = leaf(
            Primitive::Box {
                half: [0.34, 0.11, 0.62],
                round: 0.02,
                chamfer: 0.0,
            },
            Xform {
                translation: [x, 0.0, 0.0],
                ..Xform::IDENTITY
            },
        );
        n.mods = mods;
        n
    };
    FieldDoc::new(
        vec![
            coluna(-0.85, Vec::new()),
            // ⚠️ **`0,35` voltas/un = 156° no total, e o número tem razão:** a secção repete-se
            // a cada 180°, então uma torção que passe disso volta a ler-se como um ângulo
            // pequeno. *O valor mais legível é o maior que ainda cabe numa meia-volta.*
            coluna(
                0.0,
                vec![ph2d_field::Unary::Twist {
                    turns: 0.35,
                    lower: -9.0,
                    upper: 9.0,
                    // A banda cobre a peça inteira: não há ombro dentro dela para amaciar.
                    falloff: 0.0,
                    axis: ph2d_field::mods::TWIST_AXIS,
                }],
            ),
            // ⭐ A BANDA: abaixo de `z = 0` a coluna fica intacta, e acima do topo dela o
            // ângulo **congela** — a ponta roda como corpo rígido, que é o que as quatro
            // referências fazem.
            coluna(
                0.85,
                vec![ph2d_field::Unary::Twist {
                    turns: 0.35,
                    lower: 0.0,
                    upper: 9.0,
                    // ⭐ **O OMBRO** — o report do Enio (*«muito dura a transição»*): sem
                    // ele o giro da normal salta de `0,0` para `157,3 °/un` no fim da banda.
                    falloff: 0.22,
                    axis: ph2d_field::mods::TWIST_AXIS,
                }],
            ),
            combine(
                Op::Union(Blend::Sharp),
                vec![NodeId(0), NodeId(1), NodeId(2)],
            ),
        ],
        NodeId(3),
    )
}

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
