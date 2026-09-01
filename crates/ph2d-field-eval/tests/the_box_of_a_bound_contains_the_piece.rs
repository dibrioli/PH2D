//! ⭐⭐⭐ **A CAIXA DE UM BORDO CONTÉM A PEÇA, ATRAVÉS DA PILHA INTEIRA** — o gate que a
//! [`ph2d_field_eval::bounds::Ball::half`] deve, e o único sítio onde ela pode cortar.
//!
//! # ⛔ O modo de falha é SILENCIOSO, e é o mesmo do raio
//!
//! O bordo alimenta a **caixa de recorte** da marcha e as cercas dos deformadores. Uma caixa
//! **pequena demais** não falha: a peça sai cortada e o artista culpa a forma. ⚠️ E a caixa tem uma
//! forma de falhar que a esfera não tinha — **cada lei tem de a manter**, e uma que a esqueça
//! herda a de antes de crescer. *A cerca fica no lado perigoso: quem não sabe os eixos usa o
//! [`ph2d_field_eval::bounds::Ball::new`], que assume o cubo circunscrito.*
//!
//! ⚠️ **A régua é o CAMPO** — bissecta-se a superfície ao longo de muitas direcções e colhe-se a
//! **coordenada** de cada eixo. Comparar a caixa contra o raio seria cego a uma mutação que
//! mexesse nas duas.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::{Field, bounds, hybrid::Registry};

/// Um exemplar **vivo** de cada modificador — ver o irmão dos trios para a razão de não usar o
/// `born` (o `Offset` e o `Taper` nascem neutros, e mediriam o modificador desligado).
fn vivo(k: UnaryKind) -> Unary {
    use ph2d_field::mods::{ARRAY_AXIS, BEND_AXIS, RADIAL_AXIS, TAPER_AXIS, TWIST_AXIS};
    match k {
        UnaryKind::Shell => Unary::Shell { thickness: 0.06 },
        UnaryKind::Offset => Unary::Offset { distance: 0.05 },
        UnaryKind::Mirror => Unary::Mirror,
        UnaryKind::MirrorY => Unary::MirrorY,
        UnaryKind::MirrorZ => Unary::MirrorZ,
        UnaryKind::Array => Unary::Array {
            count: 3,
            spacing: 0.5,
            // ⛔ **Uma junta VIVA, e não a `SHARP`** — ela acrescenta material no vinco entre as
            // cópias, e é o único termo do bordo que uma fixtura de junta zero não pode ver: a
            // mutação que a apagava da lei da matriz **sobreviveu** à 1.ª versão deste gate.
            joint: ph2d_field::Joint {
                chamfer: 0.0,
                fillet: 0.06,
            },
            axis: ARRAY_AXIS,
        },
        UnaryKind::Radial => Unary::Radial {
            count: 6,
            // ⛔ **Uma junta VIVA, e não a `SHARP`** — ela acrescenta material no vinco entre as
            // cópias, e é o único termo do bordo que uma fixtura de junta zero não pode ver: a
            // mutação que a apagava da lei da matriz **sobreviveu** à 1.ª versão deste gate.
            joint: ph2d_field::Joint {
                chamfer: 0.0,
                fillet: 0.06,
            },
            axis: RADIAL_AXIS,
        },
        UnaryKind::Taper => Unary::Taper {
            slope: 0.6,
            axis: TAPER_AXIS,
        },
        UnaryKind::Twist => Unary::Twist {
            turns: 0.35,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: TWIST_AXIS,
        },
        UnaryKind::Bend => Unary::Bend {
            turns: 0.12,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: BEND_AXIS,
        },
    }
}

/// ⚠️ **Uma caixa ASSIMÉTRICA nos três eixos**, e com uma pose **rodada e escalada**: é sob rotação
/// que a caixa deixa de ser invariante, e é aí que uma lei preguiçosa corta.
fn peca(mods: Vec<Unary>, rodada: bool) -> FieldDoc {
    let mut n = Node::new(
        if rodada {
            Xform {
                translation: [0.11, -0.07, 0.05],
                // ~37° em torno de um eixo oblíquo.
                rotation: [0.2, 0.3, 0.1, 0.925],
                scale: 1.3,
            }
        } else {
            Xform::IDENTITY
        },
        NodeKind::Leaf(Primitive::Box {
            half: [0.14, 0.31, 0.22],
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// Até onde a peça chega em cada eixo, medido no CAMPO por bissecção.
fn alcance_por_eixo(doc: &FieldDoc, longe: f64) -> [f64; 3] {
    let f = Field::new(doc);
    let mut a = [0.0f64; 3];
    const DIRS: usize = 48;
    for i in 0..DIRS {
        for j in 0..(DIRS * 2) {
            #[allow(clippy::cast_precision_loss)]
            let theta = std::f64::consts::PI * (i as f64 + 0.5) / DIRS as f64;
            #[allow(clippy::cast_precision_loss)]
            let phi = std::f64::consts::TAU * (j as f64 + 0.5) / (DIRS * 2) as f64;
            let d = [
                theta.sin() * phi.cos(),
                theta.sin() * phi.sin(),
                theta.cos(),
            ];
            let at = |t: f64| f.at(d[0] * t, d[1] * t, d[2] * t);
            let (mut lo, mut hi) = (0.0f64, longe);
            for _ in 0..40 {
                let m = 0.5 * (lo + hi);
                if at(m) < 0.0 { lo = m } else { hi = m }
            }
            if lo > 0.0 {
                for e in 0..3 {
                    a[e] = a[e].max((d[e] * hi).abs());
                }
            }
        }
    }
    a
}

/// ⭐⭐⭐ **CADA MODIFICADOR, SOZINHO E EM PAR, COM E SEM POSE RODADA.**
///
/// ⛔⛔ **Prova de mutação (2026-08-31):** fazer a lei da matriz (`bounds::canonical_step`) crescer a
/// caixa **só** no eixo em que ela anda mas esquecer a junta reprova aqui; e trocar o
/// `Ball::of` do `place` por um que **não** re-envolva sob rotação reprova em todas as fixturas
/// rodadas. *É a rotação que separa uma caixa honesta de uma herdada.*
#[test]
fn the_box_of_a_bound_contains_the_piece_through_the_whole_stack() {
    let reg = Registry::default();
    let mut maus = Vec::new();
    let mut medidos = 0usize;
    for rodada in [false, true] {
        for a in UnaryKind::ALL {
            for b in [None, Some(UnaryKind::Twist), Some(UnaryKind::Array)] {
                let mut mods = vec![vivo(a)];
                if let Some(b) = b {
                    mods.push(vivo(b));
                }
                let nome = format!(
                    "[{a:?}{}]{}",
                    b.map_or(String::new(), |b| format!(", {b:?}")),
                    if rodada { " rodada" } else { "" }
                );
                let doc = peca(mods, rodada);
                let Some(bola) = bounds::bounding_ball(&doc, &reg) else {
                    continue;
                };
                medidos += 1;
                let h = bola.half();
                let c = bola.center;
                let alcance = alcance_por_eixo(&doc, f64::from(bola.radius) * 4.0);
                for e in 0..3 {
                    // ⚠️ **A caixa é centrada no centro da bola**, então o alcance medido a partir da
                    // ORIGEM tem de caber em `|c| + h`.
                    let cabe = f64::from(c[e].abs() + h[e]) * 1.002;
                    if alcance[e] > cabe {
                        maus.push(format!(
                            "{nome}: eixo {e} chega a {:.4} e a caixa diz {cabe:.4}",
                            alcance[e]
                        ));
                    }
                    // ⛔ **E a caixa nunca passa a esfera** — a invariante da estrutura.
                    assert!(
                        h[e] <= bola.radius * 1.0001,
                        "{nome}: a caixa do eixo {e} ({}) passou o raio ({})",
                        h[e],
                        bola.radius
                    );
                }
            }
        }
    }
    assert!(medidos >= 50, "só {medidos} fixturas — a lista partiu-se");
    assert!(
        maus.is_empty(),
        "{} bordo(s) CORTAM a peça — e um bordo pequeno demais não falha, ele corta e não diz \
         nada: {}",
        maus.len(),
        maus.join(" · ")
    );
}

/// ⭐⭐⭐ **A CAIXA DE UMA UNIÃO CONTÉM TODOS OS FILHOS** — report do Enio com foto, 2026-09-01:
/// *«Bug no render. Os 3 cilindros cruzados viraram isso.»*
///
/// # ⛔⛔⛔ O ponto cego que este gate fecha
///
/// O irmão acima varre **um nó folha** com pilha de modificadores, com e sem pose rodada — e nunca
/// um `Combine`. A fusão de duas bolas (`Ball::merge`) tinha um atalho *«uma contém a outra, fica a
/// maior»* que comparava só as **esferas** e devolvia a vencedora **inteira, com o `half` dela**.
///
/// ⚠️ **Três cilindros cruzados são o caso exacto em que ele morde**: mesmo centro, mesmo raio, e
/// caixas em eixos diferentes. O teste dispara à primeira, a união fica com a caixa de UM cilindro
/// (`0,18 × 0,18 × 0,60`), e o recorte da marcha corta os outros dois braços — `754` de `2 576`
/// pixels do interior com a normal a `172,7°` do oráculo.
///
/// ⚠️ **E o defeito só passou a doer quando o `Ball::aabb` deixou de ser o cubo do raio**: as três
/// bolas têm o mesmo raio, então a esfera nunca cortou nada. *Uma resposta pode estar errada há
/// semanas e só doer no dia em que alguém a lê.*
///
/// ⛔⛔ **Prova de mutação:** devolver `self`/`other` inteiros no atalho do `merge` reprova aqui em
/// todas as rotações menos a identidade.
#[test]
fn the_box_of_a_union_contains_every_child() {
    use ph2d_field::{Blend, Op};
    let reg = Registry::default();
    let q = std::f32::consts::FRAC_1_SQRT_2;
    let mut maus = Vec::new();
    let mut medidos = 0usize;
    for junta in [0.0f32, 0.10] {
        for poses in [
            // ⭐ A cruz da foto: os três no MESMO centro e com o MESMO raio, que é o que faz o
            // atalho da esfera disparar.
            vec![
                Xform::IDENTITY,
                Xform {
                    rotation: [q, 0.0, 0.0, q],
                    ..Xform::IDENTITY
                },
                Xform {
                    rotation: [0.0, q, 0.0, q],
                    ..Xform::IDENTITY
                },
            ],
            // ⚠️ E um par DESLOCADO, para o ramo geral do `merge` não ficar sem população.
            vec![
                Xform::IDENTITY,
                Xform {
                    translation: [0.4, 0.0, 0.0],
                    rotation: [0.0, q, 0.0, q],
                    ..Xform::IDENTITY
                },
            ],
        ] {
            let cil = Primitive::Cylinder {
                radius: 0.18,
                half_height: 0.6,
                round: 0.0,
                chamfer: 0.0,
            };
            let n = poses.len();
            let mut nodes: Vec<ph2d_field::Node> = poses
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    let mut no = ph2d_field::Node::new(*x, NodeKind::Leaf(cil.clone()));
                    if i > 0 {
                        no.verb = Some(Op::Union(Blend::Exact { radius: junta }));
                    }
                    no
                })
                .collect();
            nodes.push(ph2d_field::Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: (0..n).map(|i| NodeId(i as u32)).collect(),
                },
            ));
            #[allow(clippy::cast_possible_truncation)]
            let doc = FieldDoc::new(nodes, NodeId(n as u32)).expect("a união");
            let Some(bola) = bounds::bounding_ball(&doc, &reg) else {
                continue;
            };
            medidos += 1;
            let h = bola.half();
            let c = bola.center;
            let alcance = alcance_por_eixo(&doc, f64::from(bola.radius) * 4.0);
            for e in 0..3 {
                let cabe = f64::from(c[e].abs() + h[e]) * 1.002;
                if alcance[e] > cabe {
                    maus.push(format!(
                        "{n} cilindros junta {junta}: eixo {e} chega a {:.4} e a caixa diz {cabe:.4}",
                        alcance[e]
                    ));
                }
            }
        }
    }
    assert!(medidos >= 4, "só {medidos} uniões — a lista partiu-se");
    assert!(
        maus.is_empty(),
        "{} união(ões) CORTAM um filho — e um recorte que corta não falha, ele desenha outra peça: \
         {}",
        maus.len(),
        maus.join(" · ")
    );
}
