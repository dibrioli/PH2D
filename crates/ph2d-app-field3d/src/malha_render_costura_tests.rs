//! ⭐⭐⭐ **A COSTURA DO RENDER POR MALHA** — o caminho do produto, ponta a ponta: o chip «Render» do
//! painel → a ponte do mundo → os objetos → o clique escolhe o objeto INTEIRO → (na placa) o quadro
//! tem a peça onde a câmara a projeta.

use crate::scene::lasso_tests::{AREA, armed_with, pixel_of, two_balls};

fn liga_o_render(sim: &mut ph2d_ecs::SimWorld) {
    // ⭐ O slot do chip «Render» é o índice dele na fileira — a mesma lista que o painel oferece.
    let slot = crate::shading::Shading::ALL
        .iter()
        .position(|s| *s == crate::shading::Shading::Render)
        .expect("o Render está na fileira");
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetShading {
        slot,
    });
    crate::scene::apply_intents_for_test(sim.world_mut(), &[]);
    let t = std::time::Instant::now();
    loop {
        crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
        let n = crate::malha_render_estado::com(|e| e.objetos.len()).unwrap_or(0);
        if n > 0 || t.elapsed().as_secs() > 20 {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn the_render_chip_turns_the_part_into_objects_and_a_click_picks_a_whole_object() {
    armed_with(&two_balls(), |sim| {
        assert!(
            crate::malha_render_estado::com(|_| ()).is_none(),
            "em Matcap não há estado"
        );
        liga_o_render(sim);
        let n = crate::malha_render_estado::com(|e| e.objetos.len()).unwrap_or(0);
        assert_eq!(n, 2, "as duas bolas soltas são dois objetos");

        // O clique em cima da bola da esquerda escolhe o objeto dela — por inteiro.
        let alvo = pixel_of([-0.6, 0.0, 0.0]);
        crate::smoke::with_smoke(|s| s.pending_pick = Some((alvo, false)));
        let pedido = crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
        let Some(crate::scene::SelectRequest::Many(bits)) = pedido else {
            panic!("o clique no Render tem de escolher o objeto inteiro: {pedido:?}");
        };
        assert_eq!(bits.len(), 1);
        let (us, movel) = crate::malha_render_estado::objeto_de(
            sim.world(),
            bevy_ecs::entity::Entity::from_bits(bits[0]),
        )
        .expect("é um objeto do Render");
        assert!(movel);
        assert_eq!(us.len(), 1);
    });
}

/// ⭐⭐ Na placa: o quadro do Render tem a bola no pixel onde a câmara a projeta, opaca, e fundo
/// transparente entre as duas.
#[test]
#[ignore = "precisa de aparelho"]
fn the_render_frame_has_the_ball_where_the_camera_puts_it() {
    armed_with(&two_balls(), |sim| {
        liga_o_render(sim);
        let doc = crate::smoke::with_smoke(|s| s.doc.clone())
            .flatten()
            .expect("o documento");
        let tamanho = (AREA.w.round() as u32, AREA.h.round() as u32);
        let feito = crate::smoke::with_smoke(|s| {
            crate::malha_render_quadro::desenha(s, s.active, tamanho, &doc, false)
        })
        .expect("armado");
        let crate::malha_render_quadro::Feito::Novo(rgba) = feito else {
            println!("sem aparelho — saltado ({feito:?})");
            return;
        };
        let alfa =
            |p: [f32; 2]| rgba[((p[1] as usize) * tamanho.0 as usize + p[0] as usize) * 4 + 3];
        assert_eq!(alfa(pixel_of([-0.6, 0.0, 0.0])), 255, "a bola da esquerda");
        assert_eq!(alfa(pixel_of([0.6, 0.0, 0.0])), 255, "a bola da direita");
        // Entre as duas, acima do chão: só fundo (ou a sombra, que é preta e meio transparente).
        let meio = pixel_of([0.0, 0.3, 0.0]);
        assert!(
            alfa(meio) < 255,
            "o meio tem de ser fundo: alfa {}",
            alfa(meio)
        );
    });
}

/// ⛔⛔ **A frase da trava é dita UMA vez, não a cada quadro** (report do dono, 02/10, foto: dezenas de
/// avisos «In Render you move whole objects…» empilhados). O canal não repete a última frase, mas
/// é LIMPO a cada quadro em que a peça coze — logo uma frase dita por quadro voltava como nova.
#[test]
fn a_locked_selection_is_announced_once_not_every_frame() {
    // Uma bola mordida por outra: a mordida é UM objeto; seleccionar só a bola de dentro trava.
    let doc = ph2d_field::FieldDoc::new(
        vec![
            ph2d_field::Node::new(
                ph2d_field::Xform::at(-0.3, 0.0, 0.0),
                ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Sphere { radius: 0.3 }),
            ),
            ph2d_field::Node::new(
                ph2d_field::Xform::at(-0.1, 0.0, 0.0),
                ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Sphere { radius: 0.15 }),
            ),
            ph2d_field::Node::new(
                ph2d_field::Xform::IDENTITY,
                ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Difference(ph2d_field::Blend::Sharp),
                    children: vec![ph2d_field::NodeId(0), ph2d_field::NodeId(1)],
                },
            ),
            ph2d_field::Node::new(
                ph2d_field::Xform::at(0.6, 0.0, 0.0),
                ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Sphere { radius: 0.2 }),
            ),
            ph2d_field::Node::new(
                ph2d_field::Xform::IDENTITY,
                ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![ph2d_field::NodeId(2), ph2d_field::NodeId(3)],
                },
            ),
        ],
        ph2d_field::NodeId(4),
    )
    .expect("doc");
    armed_with(&doc, |sim| {
        liga_o_render(sim);
        let world = sim.world_mut();
        let mut q = world.query::<(bevy_ecs::entity::Entity, &ph2d_field_ecs::FieldObject)>();
        let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
        // A bola de DENTRO da mordida (a 2.ª folha): não é um objeto inteiro.
        let folha = crate::materials::folhas(sim.world(), root)[1].0;
        let _ = crate::notice::drain();
        crate::notice::forget_last();
        let mut ditas = 0;
        for _ in 0..5 {
            crate::scene::ecs_bridge(sim, Some(folha.to_bits()), &[], &crate::scene::no_drawing());
            ditas += crate::notice::drain()
                .iter()
                .filter(|m| m.contains("whole objects"))
                .count();
        }
        assert_eq!(ditas, 1, "a trava foi dita {ditas} vezes em 5 quadros");
        assert!(
            crate::smoke::with_smoke(|s| s.gizmo.is_none()).unwrap_or(false),
            "o gizmo trava"
        );
    });
}
