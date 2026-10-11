//! ⭐⭐ **As formas do Motion bloqueiam a luz como a imagem** (W5e; plano §2.4: a C0 captura o
//! Motion). Um objeto Motion com «bloqueia a luz» e a placa das formas dele (discos — as MESMAS cópias
//! que a placa desenha no ecrã, num passe irmão com o afim da grade): cada disco tem as FACES e a luz
//! dele bate com a da IMAGEM gêmea no espelho em relação à lâmpada — a régua das cenas 6 e 7. E a cena
//! 8 do smoke: a lâmpada sobre os objetos do smoke do Motion, que ela marca quando eles nascem.
//!
//! Controlo na MESMA corrida: a mesma cena sem a família do Motion na luz — as formas não bloqueiam.

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_app_motion::motion_shape_gen::VecPathStore;
use ph2d_app_motion::motion_shape_placa::luz::OclusaoDoMotion;
use ph2d_app_motion::motion_shape_placa::{GeometriasDaPlaca, PlacaDeFormas};
use ph2d_ecs::{GlobalTransform, MotionGenerator, Name, PresentWorld, SimRef, Transform};
use ph2d_eval_motion::VectorInstance;
use ph2d_light2d_ecs::BloqueiaLuz;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_render::{DISC_TILE_PX, Sprite, disc_tile_pixels};

use crate::o_traco_do_flip::{CONTRASTE_DAS_FACES, espelho};

/// A face da lâmpada das FORMAS contra a da IMAGEM, relativo — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`, 30
/// quadros com o rebote): o disco grande −0,49 %, o pequeno +0,06 %; as faces 45 × e 37 ×. A tolerância
/// é a das cenas 6 e 7.
const TOL_FORMA_IMAGEM: f64 = 0.01;
/// A variação dos anéis das formas acima da da imagem — MEDIDO: +6,0 % e +22,3 %. ⚠️ NÃO é a captura:
/// a camada deslocada ±½ px dá 4,4–12,5 % (grande) e 13,3–22,7 % (pequeno) — o interior depende da FASE
/// da grade (o F5 da fila, `docs/Luz2D/01_plano.md` §7.1, o último por ordem do dono). As facetas da
/// placa também não: a tolerância do nível de detalhe a 0,125 e a 0,01 px dá 19,2 e 22,1 %. Recusas no
/// handoff W5e. O pior medido + 10 %.
const TOL_TV_FORMA_IMAGEM: f64 = 0.25;

fn linear(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[test]
fn as_formas_do_motion_bloqueiam_como_a_imagem() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    // O mundo da cena 7 (o chão, a lâmpada, as imagens gêmeas) com um objeto Motion no lugar do Flip.
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 7));
    let objeto = w
        .spawn((
            Name::new("Formas"),
            Transform::IDENTITY,
            MotionGenerator,
            BloqueiaLuz { opacidade: 1.0 },
        ))
        .id();
    let cena = cena_de_luz(&w);
    assert_eq!(cena.outros.len(), 1, "o objeto Motion bloqueia");

    // Sem preenchimento explícito: a cor de uma forma da placa é o tint da cópia.
    let mut store = VecPathStore::default();
    let h = store.push(ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5));
    let insts: Vec<VectorInstance> = smoke::DISCOS_DE_TINTA
        .iter()
        .map(|d| VectorInstance {
            geometry_id: h,
            texture_id: 0,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            premultiplied: 0.0,
            world_pos: [d.centro[0] as f32, d.centro[1] as f32],
            size: [(2.0 * d.raio) as f32; 2],
            basis: [1.0, 0.0, 0.0, 1.0],
            tint: [linear(d.cor[0]), linear(d.cor[1]), linear(d.cor[2]), 1.0],
            anchor: [0.0, 0.0],
            sampling: 0,
            blend_linha: 0,
            mistura: Default::default(),
        })
        .collect();
    let mut geos = GeometriasDaPlaca::default();
    let mut placa = PlacaDeFormas::default();
    assert!(
        placa.decide(true, &insts, &store, &mut geos, kurbo::Affine::IDENTITY),
        "a placa aceita os discos"
    );

    let mut r = crate::comum::renderer(&gpu);
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = r
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = crate::comum::quad(disco, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        present.world_mut().spawn((i, SimRef(e)));
    }
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let mut medir = |com_motion: bool| {
        let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
        for _ in 0..30 {
            assert!(l.preparar(&w, afim, (1920, 1080)));
            if com_motion {
                let mut m = OclusaoDoMotion {
                    placa: &mut placa,
                    geometrias: &geos,
                    dispositivo: None,
                    objetos: vec![objeto.to_bits()],
                };
                l.gravar_com(&gpu, &mut present, &r, &mut [&mut m]);
            } else {
                l.gravar(&gpu, &mut present, &r);
            }
        }
        crate::comum::fluencia_lida(&gpu, l.solver().expect("solver"))
    };
    let sem = medir(false);
    let f = medir(true);
    let mut falhas = Vec::new();
    for d in &smoke::DISCOS_DE_TINTA {
        let semi = [d.raio; 2];
        let controlo = espelho(&sem, d.centro, semi);
        assert!(
            controlo.frente[0] < CONTRASTE_DAS_FACES * controlo.tras[0],
            "controlo: sem o Motion na luz o {} não devia ter faces",
            d.nome
        );
        let m = espelho(&f, d.centro, semi);
        let paridade = (m.frente[0] - m.frente[1]) / m.frente[1];
        eprintln!(
            "formas do Motion · {}: faces forma {:.3} × {:.3} · imagem {:.3} × {:.3} · forma × imagem {:+.2} % · anéis {:+.1} %",
            d.nome,
            m.frente[0],
            m.tras[0],
            m.frente[1],
            m.tras[1],
            100.0 * paridade,
            100.0 * m.variacao
        );
        if m.frente[0] < CONTRASTE_DAS_FACES * m.tras[0] {
            falhas.push(format!("{} sem faces", d.nome));
        }
        if paridade.abs() > TOL_FORMA_IMAGEM {
            falhas.push(format!("{}: forma × imagem {paridade:+.3}", d.nome));
        }
        if m.variacao > TOL_TV_FORMA_IMAGEM {
            falhas.push(format!("{}: anéis {:+.3}", d.nome, m.variacao));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join(" · "));
}

/// ⭐ A cena 8 ensina as formas do Motion: a lâmpada e o chão; os objetos Motion (os do smoke do Motion)
/// recebem o «bloqueia a luz» quando nascem — antes, nada a marcar. O censo do roteador mora aqui (a
/// ÚLTIMA cena).
#[test]
fn a_cena_8_ensina_as_formas_do_motion() {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 8));
    assert!(!smoke::encenar(&mut World::new(), 9), "não há cena 9");
    assert_eq!(smoke::ROUTERS[0].max_level, 8, "o maior nível do `match`");
    assert_eq!(cena_de_luz(&w).luzes, 2, "a lâmpada e a luz ambiente");
    assert_eq!(smoke::marcar_motion(&mut w), 0, "antes do smoke do Motion");
    let pilha = w
        .spawn((Name::new("Pile"), Transform::IDENTITY, MotionGenerator))
        .id();
    assert_eq!(smoke::marcar_motion(&mut w), 1);
    assert!(w.get::<BloqueiaLuz>(pilha).is_some(), "a Pile bloqueia");
    assert_eq!(smoke::marcar_motion(&mut w), 0, "uma vez só");
    assert_eq!(
        cena_de_luz(&w).outros.len(),
        1,
        "a coleta vê o objeto Motion"
    );
}
