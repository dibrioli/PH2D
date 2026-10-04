//! ⭐⭐⭐ **PRENDER COZE OS EFEITOS** (ordem do dono, 2026-10-03: *«ao aplicar os bones, os efeitos são
//! cozidos antes. E uma vez com bones, o vetor não pode receber efeitos»*) — num irmão pelo tecto de
//! LOC do `skin_live_tests.rs`.

use super::bind;
use super::tests::palco;
use crate::test_support::{pior_desvio_do_desenho, quadro};
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};

fn twist() -> PathEffect {
    PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec { angle: 40.0 })
}

/// ⭐⭐⭐ **GATE — o Bind coze o efeito no desenho, a pilha sai vazia na cena E na fonte guardada, e
/// prender não move um pixel.**
///
/// ⛔ **O CONTROLO:** o efeito muda MESMO o desenho — senão «não moveu» mediria uma forma igual.
#[test]
fn binding_bakes_the_effects_into_the_drawing_and_moves_nothing() {
    let (mut sim, mut scene, map, id, _) = palco();
    let liso = scene.paths()[0].clone();
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(twist())];
    let visto = scene.paths()[0].cooked().into_owned();
    assert!(
        pior_desvio_do_desenho(&liso, &visto) > 0.5,
        "o CONTROLO: o Twist não muda o desenho — a fixtura perdeu o efeito"
    );
    assert_eq!(bind(&mut sim, &mut scene, &map, &[id], None), 1);
    assert!(
        scene.path(id).expect("path").effects.is_empty(),
        "a pilha sobreviveu ao Bind na cena"
    );
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    let fonte = crate::skinned_mesh::le(&sim.world().get::<SkinBind>(e).expect("pele").source)
        .expect("a fonte lê-se");
    assert!(
        fonte.path.effects.is_empty(),
        "a fonte guardada leva a pilha"
    );
    let depois = quadro(&sim, &mut scene, id);
    let pior = pior_desvio_do_desenho(&visto, &depois);
    assert!(pior < 1e-9, "prender com efeito moveu o desenho em {pior}");
}

/// ⭐⭐ **GATE — um efeito DESLIGADO sai com a pilha e não coze nada** (o olho fechado não é desenho).
#[test]
fn binding_drops_a_disabled_effect_without_baking_it() {
    let (mut sim, mut scene, map, id, _) = palco();
    let liso = scene.paths()[0].clone();
    scene.path_mut(id).expect("path").effects = vec![FxEntry {
        enabled: false,
        ..FxEntry::new(twist())
    }];
    assert_eq!(bind(&mut sim, &mut scene, &map, &[id], None), 1);
    assert!(scene.path(id).expect("path").effects.is_empty());
    let depois = quadro(&sim, &mut scene, id);
    let pior = pior_desvio_do_desenho(&liso, &depois);
    assert!(pior < 1e-9, "o efeito desligado foi cozido ({pior})");
}

/// ⭐⭐⭐ **GATE — numa forma de efeito COZIDO no Bind o contacto é só a UNIÃO** (a lei da F50): dobrada
/// sem se cruzar, ela desenha-se igual com e sem a lei do contacto.
///
/// ⛔ **O CONTROLO:** a MESMA fonte sem a marca `efeitos_cozidos` vai à bola, e a bola come os vales
/// entre os dentes do lado de dentro da junta (FOTOGRAFADO na `=5` a `60°`, 2026-10-03).
#[test]
fn a_baked_zigzag_keeps_its_teeth_in_the_bend() {
    use crate::skin_desenho::Leis;
    let monta = |marca: bool| {
        let (mut sim, mut scene, map, id, [_, ponta]) = palco();
        scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::ZigZag(
            ph2d_vec_scene::fx_zigzag::ZigZagSpec {
                amplitude: 6.0,
                ridges: 24.0,
                ..Default::default()
            },
        ))];
        assert_eq!(bind(&mut sim, &mut scene, &map, &[id], None), 1);
        let e = ph2d_ecs::Entity::from_bits(map[&id]);
        let mut skin = sim.world().get::<SkinBind>(e).expect("pele").clone();
        let mut g = crate::skinned_mesh::le(&skin.source).expect("fonte");
        assert!(g.efeitos_cozidos, "o Bind não marcou a fonte cozida");
        g.efeitos_cozidos = marca;
        skin.source = crate::skinned_mesh::grava(&g).expect("grava");
        sim.world_mut().entity_mut(e).insert(skin);
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(ponta)
            .expect("Transform")
            .rotation = 60f32.to_radians();
        let desenho = |contacto: bool| {
            let leis = Leis {
                contacto,
                ..Leis::do_ambiente()
            };
            crate::skin_live::recook_leis(&sim, &mut scene.clone(), leis)
                .remove(&id)
                .expect("desenho")
        };
        pior_desvio_do_desenho(&desenho(true), &desenho(false))
    };
    let (marcada, controlo) = (monta(true), monta(false));
    println!("  contacto contra sem contacto: marcada {marcada:.5} · sem a marca {controlo:.5}");
    assert!(
        marcada < 1e-9,
        "a forma cozida foi reescrita pelo contacto ({marcada})"
    );
    assert!(
        controlo > 1e-3,
        "o CONTROLO: a bola não mexe — a fixtura perdeu o defeito ({controlo})"
    );
}

/// ⭐ **SONDA — o preço do Bind e do 1.º quadro de uma forma com um *Repeater* denso** (A3). Corra em
/// `--release` com o `loadavg` ao lado.
#[test]
#[ignore = "sonda de preço: --release, máquina calma"]
fn diag_o_preco_do_bind_de_um_repeater_denso() {
    use std::time::Instant;
    for copias in [1.0, 13.0, 25.0, 39.0] {
        let (mut sim, mut scene, map, id, _) = palco();
        if copias > 1.0 {
            scene.path_mut(id).expect("path").effects = vec![FxEntry::new(PathEffect::Repeat(
                ph2d_vec_scene::fx_repeat::RepeatSpec {
                    copies_x: copias,
                    move_x: -80.0,
                    copies_y: copias,
                    move_y: -80.0,
                    spin: -72.0,
                    orbit: -72.0,
                },
            ))];
        }
        let t = Instant::now();
        assert_eq!(bind(&mut sim, &mut scene, &map, &[id], None), 1);
        let t_bind = t.elapsed();
        let t = Instant::now();
        let _ = quadro(&sim, &mut scene, id);
        let t_q1 = t.elapsed();
        let t = Instant::now();
        let _ = quadro(&sim, &mut scene, id);
        let t_q2 = t.elapsed();
        println!(
            "  {copias}²: ms: bind {:.1} · 1.º quadro {:.1} · 2.º {:.2} · loadavg {}",
            t_bind.as_secs_f64() * 1e3,
            t_q1.as_secs_f64() * 1e3,
            t_q2.as_secs_f64() * 1e3,
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}
