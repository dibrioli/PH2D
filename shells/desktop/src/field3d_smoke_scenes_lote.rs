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
