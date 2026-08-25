//! ⭐⭐ **O QUE O LAÇO APANHA** (W58b) — os gates da lei de captura, separados dos do GESTO.
//!
//! ⚠️ O irmão [`super::lasso_tests`] pergunta *o gesto nasce, desenha e vira pedido?*; este pergunta
//! *o pedido tem lá dentro o que devia?*. Duas leis, dois arquivos — e o corte foi forçado pelo teto
//! de LOC do shell (HR-18, 600), que é o **terceiro** portão de LOC desta linha e o que uma corrida
//! com filtro de nome não alcança.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

use super::lasso_tests::{AREA, armed_with, begin, pixel_of, win};
use crate::field3d_scene::SelectRequest;
use crate::field3d_smoke::Drag;

/// ⛔⛔ **O LAÇO APANHA O QUE ESTÁ TAPADO** (W58b) — o defeito que o Enio reportou.
///
/// Enio, 2026-08-24: *"o retângulo de seleção não seleciona mais de 2 objetos ao mesmo tempo"*.
///
/// ⚠️ **A causa não era um teto: era a PERGUNTA.** A 1.ª versão perguntava só *"o que se vê"*, e
/// `+ Box`/`+ Sphere` nascem no **alvo da câmera** — um artista que acrescenta três formas antes de
/// as mexer tem três **no mesmo sítio**, e só a da frente ganha o `min |f|`. Medido antes da cura:
/// um laço sobre **cinco** empilhadas apanhava **uma**.
///
/// ⭐ A cura é a lei do modo de **objeto** de todo modelador: apanha-se também por **origem**.
///
/// ⚠️ **E os três gates anteriores não podiam ver isto** — as bolas deles estão em **fila**, cada
/// uma com o seu pedaço de silhueta. *Uma fixtura sem oclusão não mede um defeito de oclusão.*
#[test]
fn a_lasso_catches_the_shapes_that_hide_behind_each_other() {
    for n in [2usize, 3, 4, 5] {
        let stacked = {
            let mut nodes: Vec<Node> = (0..n)
                .map(|_| ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.25 }, Xform::IDENTITY))
                .collect();
            nodes.push(Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: (0..n as u32).map(NodeId).collect(),
                },
            ));
            FieldDoc::new(nodes, NodeId(n as u32)).expect("a peça")
        };
        let got = armed_with(&stacked, |sim| {
            let (a, b) = ([4.0f32, 4.0], [396.0f32, 296.0]);
            crate::field3d_smoke::with_smoke(|s| {
                begin(
                    s,
                    winit::event::MouseButton::Left,
                    Drag::Orbit,
                    true,
                    win(a),
                );
                s.last_pointer = win(b);
                s.lasso = s.lasso.map(|(from, _)| (from, b));
                crate::field3d_input::finish_for_test(s);
            });
            match crate::field3d_scene::ecs_bridge(
                sim,
                None,
                &[],
                &crate::field3d_scene::no_drawing(),
            ) {
                Some(SelectRequest::ToggleMany(bits)) => bits.len(),
                other => panic!("{n} formas empilhadas: o laço não pediu seleção: {other:?}"),
            }
        });
        assert_eq!(
            got, n,
            "{n} formas no MESMO sítio: o laço apanhou {got} — tudo o que está atrás continua \
             inalcançável por gesto de canvas"
        );
    }
}

/// ⭐⭐⭐ **A METADE DA SUPERFÍCIE APANHA O QUE A DA ORIGEM NÃO VÊ** — e é por isso que são duas.
///
/// ⚠️ **Este gate nasceu de três mutações que SOBREVIVERAM.** Ao acrescentar a metade da origem
/// (W58b), os gates que mediam o laço com bolas **em fila** deixaram de prender a metade da
/// superfície: sabotá-la ficava verde, porque a origem apanhava tudo na mesma. *Uma metade nova
/// pode tornar a antiga inobservável — e uma metade que nenhum gate observa é uma metade que se
/// apaga sem ninguém reparar.*
///
/// A fixtura separa-as, e é **exigente de propósito**:
///
/// - **duas** esferas, para que atribuir o dono errado colapse a contagem;
/// - as **origens de ambas** ficam à ESQUERDA do rectângulo — só o corpo entra;
/// - o **canto de partida** do rectângulo é fundo, e os corpos entram em **alturas diferentes**,
///   para que amostrar um ponto só não chegue.
#[test]
fn the_surface_half_catches_what_the_origin_half_cannot_see() {
    let ball = |y: f32| {
        ph2d_field_eval::leaf(
            Primitive::Sphere { radius: 0.25 },
            Xform {
                translation: [-0.55, y, 0.0],
                ..Xform::IDENTITY
            },
        )
    };
    let doc = FieldDoc::new(
        vec![
            ball(0.35),
            ball(-0.35),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("a peça");
    armed_with(&doc, |sim| {
        // O rectângulo começa à DIREITA das duas origens e vai até à quina de baixo.
        let origin_px = pixel_of([-0.55, 0.35, 0.0]);
        let a = [origin_px[0] + 20.0, 4.0];
        let b = [AREA.w - 4.0, AREA.h - 4.0];
        for y in [0.35f32, -0.35] {
            let o = pixel_of([-0.55, y, 0.0]);
            assert!(
                o[0] < a[0],
                "a fixtura não separa as duas metades: uma origem caiu DENTRO do rectângulo"
            );
        }
        crate::field3d_smoke::with_smoke(|s| {
            begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                true,
                win(a),
            );
            s.last_pointer = win(b);
            s.lasso = s.lasso.map(|(from, _)| (from, b));
            crate::field3d_input::finish_for_test(s);
        });
        let req =
            crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing());
        let Some(SelectRequest::ToggleMany(v)) = req else {
            panic!("o laço não apanhou os corpos que entram nele: {req:?}");
        };
        assert_eq!(
            v.len(),
            2,
            "o laço apanhou {} das DUAS esferas cujo corpo entra nele e cuja origem fica de fora",
            v.len()
        );
    });
}

/// ⭐ **O laço NÃO apanha o que está atrás do olho.** Um ponto atrás da câmera projecta-se num sítio
/// qualquer do ecrã; sem a guarda do `project`, um laço na quina apanharia o que está às costas.
#[test]
fn a_lasso_does_not_catch_what_is_behind_the_camera() {
    // Uma bola no enquadramento e outra **muito atrás** do olho.
    let doc = FieldDoc::new(
        vec![
            ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.2 }, Xform::IDENTITY),
            ph2d_field_eval::leaf(
                Primitive::Sphere { radius: 0.2 },
                Xform {
                    translation: [0.0, 0.0, 40.0],
                    ..Xform::IDENTITY
                },
            ),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("a peça");
    let got = armed_with(&doc, |sim| {
        let (a, b) = ([4.0f32, 4.0], [396.0f32, 296.0]);
        crate::field3d_smoke::with_smoke(|s| {
            begin(
                s,
                winit::event::MouseButton::Left,
                Drag::Orbit,
                true,
                win(a),
            );
            s.last_pointer = win(b);
            s.lasso = s.lasso.map(|(from, _)| (from, b));
            crate::field3d_input::finish_for_test(s);
        });
        match crate::field3d_scene::ecs_bridge(sim, None, &[], &crate::field3d_scene::no_drawing())
        {
            Some(SelectRequest::ToggleMany(bits)) => bits.len(),
            other => panic!("o laço não pediu seleção: {other:?}"),
        }
    });
    assert_eq!(
        got, 1,
        "o laço apanhou {got} — a bola que está ÀS COSTAS do artista entrou na seleção"
    );
}

/// A sonda que mediu o defeito e a cura — ver o gate acima.
///
/// ⚠️ **Ela também mede o que o laço NÃO promete**: com afastamento grande as formas saem do
/// **enquadramento**, e um rectângulo de ecrã não pode apanhar o que não está no ecrã. Isso não é
/// defeito — é a definição de um laço de viewport.
#[test]
#[ignore = "sonda de diagnóstico — corre com --ignored --nocapture"]
fn the_probe_of_how_many_a_lasso_catches_when_they_overlap() {
    let stacked = |n: usize, spread: f32| {
        let mut nodes: Vec<Node> = (0..n)
            .map(|i| {
                ph2d_field_eval::leaf(
                    Primitive::Sphere { radius: 0.25 },
                    Xform {
                        translation: [spread * i as f32, 0.0, 0.0],
                        ..Xform::IDENTITY
                    },
                )
            })
            .collect();
        nodes.push(Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: (0..n as u32).map(NodeId).collect(),
            },
        ));
        FieldDoc::new(nodes, NodeId(n as u32)).expect("a peça")
    };
    println!("formas | afastamento | apanhadas");
    for n in [3usize, 4, 5] {
        for spread in [0.0f32, 0.05, 0.15, 0.3, 0.5] {
            let got = armed_with(&stacked(n, spread), |sim| {
                let (a, b) = ([4.0f32, 4.0], [396.0f32, 296.0]);
                crate::field3d_smoke::with_smoke(|s| {
                    begin(
                        s,
                        winit::event::MouseButton::Left,
                        Drag::Orbit,
                        true,
                        win(a),
                    );
                    s.last_pointer = win(b);
                    s.lasso = s.lasso.map(|(from, _)| (from, b));
                    crate::field3d_input::finish_for_test(s);
                });
                match crate::field3d_scene::ecs_bridge(
                    sim,
                    None,
                    &[],
                    &crate::field3d_scene::no_drawing(),
                ) {
                    Some(SelectRequest::ToggleMany(bits)) => bits.len(),
                    _ => 0,
                }
            });
            println!("{n:>6} | {spread:>11.2} | {got:>9}");
        }
    }
}
