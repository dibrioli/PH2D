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
