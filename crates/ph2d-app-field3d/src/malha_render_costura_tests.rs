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

/// ⭐⭐⭐ **O BRILHO NO RENDER POR MALHA, pela rota do produto** — no Render por malha o painel
/// oferece as fileiras do brilho (onde a placa o tem) e as do Estilo; e ligar o brilho PELO PAINEL acende o fundo à volta das bolas no quadro do desenhista.
///
/// ⚠️ A metade da placa só corre com aparelho; a do painel corre em todo lado.
#[test]
fn in_the_mesh_render_the_panel_offers_the_bloom_and_switching_it_on_lights_the_background() {
    armed_with(&two_balls(), |sim| {
        liga_o_render(sim);
        crate::scene::ecs_bridge(sim, None, &[], &crate::scene::no_drawing());
        let rows = ph2d_panel_model3d::state::current().rows;
        let conta = |f: fn(&ph2d_field::Param) -> bool| rows.iter().filter(|r| f(&r.param)).count();
        let brilho = conta(|p| matches!(p, ph2d_field::Param::Bloom(_)));
        let estilo = conta(|p| matches!(p, ph2d_field::Param::Style(_)));
        assert!(
            estilo > 0,
            "o Estilo corre no desenhista de jogo: as fileiras têm de estar no painel"
        );
        let tem = crate::malha_render_quadro::tem_brilho();
        if !tem {
            assert_eq!(brilho, 0, "sem brilho nesta placa, nada de fileiras mortas");
            println!("sem brilho nesta placa — a metade da placa saltada");
            return;
        }
        assert!(
            brilho > 0,
            "o brilho corre no desenhista: as fileiras têm de estar no painel"
        );

        let doc = crate::smoke::with_smoke(|s| s.doc.clone())
            .flatten()
            .expect("o documento");
        let tamanho = (AREA.w.round() as u32, AREA.h.round() as u32);
        let quadro = || {
            let feito = crate::smoke::with_smoke(|s| {
                crate::malha_render_quadro::desenha(s, s.active, tamanho, &doc, false)
            })
            .expect("armado");
            match feito {
                crate::malha_render_quadro::Feito::Novo(rgba) => rgba,
                outro => panic!("o desenhista existe e não desenhou: {outro:?}"),
            }
        };
        let sem = quadro();
        // ⭐ Pelo painel: o interruptor (posição 0) e o limiar (posição 1) — a arrumação do `pack`.
        for (slot, value) in [(0u8, 1.0f32), (1, 0.2)] {
            ph2d_panel_model3d::state::push_intent_for_test(
                ph2d_panel_model3d::ModelIntent::SetParam {
                    entity: 0,
                    param: ph2d_field::Param::Bloom(slot),
                    value,
                },
            );
        }
        crate::scene::apply_intents_for_test(sim.world_mut(), &[]);
        assert!(
            crate::smoke::with_smoke(|s| s.bloom.enabled).unwrap_or(false),
            "o intent do painel não chegou ao brilho da cena"
        );
        let com = quadro();
        let acesos = sem
            .as_chunks::<4>()
            .0
            .iter()
            .zip(com.as_chunks::<4>().0)
            .filter(|(a, b)| a[3] == 0 && b[3] > 0)
            .count();
        assert!(
            acesos > 500,
            "o halo não chegou ao fundo à volta das bolas: {acesos} píxeis"
        );
    });
}

/// ⭐⭐⭐ **O ESTILO NO RENDER POR MALHA, pela rota do produto** — numa peça com ARESTAS (a cena dos
/// nós), a tinta de aresta mudada PELO PAINEL muda o quadro do desenhista; e mexer na SUAVIDADE
/// (a escala a que a curvatura é medida) muda-o outra vez, depois de a curvatura ser re-assada por
/// vértice noutra thread — sem extrair a malha de novo.
#[test]
fn in_the_mesh_render_the_style_tint_and_its_softness_reach_the_frame() {
    armed_with(&crate::smoke::scenes::scene(28), |sim| {
        liga_o_render(sim);
        let doc = crate::smoke::with_smoke(|s| s.doc.clone())
            .flatten()
            .expect("o documento");
        let tamanho = (AREA.w.round() as u32, AREA.h.round() as u32);
        // Desenha até sair um quadro NOVO (a 1.ª curvatura vem de outra thread: até lá, espera).
        let quadro = || {
            let t = std::time::Instant::now();
            loop {
                let feito = crate::smoke::with_smoke(|s| {
                    crate::malha_render_quadro::desenha(s, s.active, tamanho, &doc, false)
                })
                .expect("armado");
                match feito {
                    crate::malha_render_quadro::Feito::Novo(rgba) => return Some(rgba),
                    crate::malha_render_quadro::Feito::SemAparelho => return None,
                    _ if t.elapsed().as_secs() > 30 => panic!("o quadro nunca chegou"),
                    _ => std::thread::sleep(std::time::Duration::from_millis(5)),
                }
            }
        };
        let geracao = crate::malha_render_estado::com(|e| e.geracao);
        let Some(liso) = quadro() else {
            println!("sem aparelho — saltado");
            return;
        };
        // A tinta de ARESTA (a cor da posição 4 da arrumação) e a nitidez dela, pelo painel.
        ph2d_panel_model3d::state::push_intent_for_test(
            ph2d_panel_model3d::ModelIntent::SetColor {
                entity: 0,
                anchor: ph2d_field::Param::Style(4),
                srgb: [255, 40, 20],
            },
        );
        crate::scene::apply_intents_for_test(sim.world_mut(), &[]);
        let tingido = quadro().expect("quadro");
        let muda = |a: &[u8], b: &[u8]| a.iter().zip(b).filter(|(x, y)| x != y).count();
        assert!(
            muda(&liso, &tingido) > 500,
            "a tinta de aresta não chegou ao quadro: {} canais",
            muda(&liso, &tingido)
        );
        // A SUAVIDADE (posição 21): a curvatura re-assa, e o quadro muda quando ela chega.
        ph2d_panel_model3d::state::push_intent_for_test(
            ph2d_panel_model3d::ModelIntent::SetParam {
                entity: 0,
                param: ph2d_field::Param::Style(21),
                value: ph2d_style::Curvature::MAX_SOFTNESS,
            },
        );
        crate::scene::apply_intents_for_test(sim.world_mut(), &[]);
        let t = std::time::Instant::now();
        let suave = loop {
            let q = quadro().expect("quadro");
            if q != tingido || t.elapsed().as_secs() > 30 {
                break q;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        assert!(
            muda(&tingido, &suave) > 500,
            "a suavidade não mudou a tinta: {} canais",
            muda(&tingido, &suave)
        );
        assert_eq!(
            crate::malha_render_estado::com(|e| e.geracao),
            geracao,
            "o estilo não pode re-extrair a malha"
        );
    });
}
