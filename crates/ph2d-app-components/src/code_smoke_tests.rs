//! A cena do editor de scripts ensina o que diz: as três formas do `bob.luau` mexem-se e o quadrado
//! não; mudar a `amplitude` no texto muda as três; o nível 2 abre o script partido.

use super::*;
use ph2d_script::{ScriptHost, ScriptInfo};

fn cena(nivel: u32) -> (SimWorld, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let mut vec = VecScene::new();
    let mut mapa = VecEntityMap::new();
    let e = monta_em(&mut sim, &mut vec, &mut mapa, nivel);
    (sim, e)
}

fn y(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).expect("pose").translation.y - SHAPE_Y
}

/// Corre `n` tiques e devolve a maior distância a `y = 0` de cada forma.
fn passeio(sim: &mut SimWorld, host: &mut ScriptHost, formas: &[Entity], n: usize) -> Vec<f32> {
    let mut max = vec![0.0f32; formas.len()];
    for _ in 0..n {
        host.scene_sync(sim.world_mut());
        let r = host.scene_tick(sim, 1.0 / 30.0);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        for (m, &e) in max.iter_mut().zip(formas) {
            *m = m.max(y(sim, e).abs());
        }
    }
    max
}

#[test]
fn tres_formas_correm_o_mesmo_script_e_o_quadrado_fica_parado() {
    let (mut sim, formas) = cena(1);
    assert_eq!(formas.len(), 4);
    let w = sim.world();
    let bob = w
        .resource::<ProjectScripts>()
        .by_name(BOB_NAME)
        .expect("bob.luau no projeto");
    for &e in &formas[..3] {
        assert_eq!(w.get::<LuauScript>(e), Some(&LuauScript::library(bob)));
    }
    assert!(
        w.get::<LuauScript>(formas[3]).is_none(),
        "o quadrado é o controlo"
    );
    assert!(
        w.get_resource::<ScriptEditRequests>()
            .is_none_or(|r| r.0.is_empty()),
        "nada aberto no nível 1"
    );

    let mut host = ScriptHost::new().expect("vm");
    let antes = passeio(&mut sim, &mut host, &formas, 90);
    for (i, d) in antes.iter().enumerate().take(3) {
        assert!(*d > 0.5 && *d <= 1.0 + 1e-3, "a forma {i} sobe ~1 m: {d}");
    }
    assert_eq!(antes[3], 0.0, "o quadrado não se mexe");

    // O gesto do dono: a amplitude 1.0 → 2 no texto, gravado. As três passam a ~2 m.
    let novo = BOB_LUAU.replace("\"amplitude\", 1.0", "\"amplitude\", 2");
    assert_ne!(
        novo, BOB_LUAU,
        "a linha que o comentário manda mudar existe"
    );
    sim.world_mut()
        .resource_mut::<ProjectScripts>()
        .set_text(bob, &novo);
    let depois = passeio(&mut sim, &mut host, &formas, 90);
    for (i, d) in depois.iter().enumerate().take(3) {
        assert!(*d > 1.5, "a forma {i} sobe mais depois de gravar: {d}");
    }
    assert_eq!(depois[3], 0.0);
}

#[test]
fn o_nivel_2_abre_o_script_partido_com_a_linha_do_erro() {
    let (mut sim, formas) = cena(2);
    assert_eq!(formas.len(), 5);
    let broken = sim
        .world()
        .resource::<ProjectScripts>()
        .by_name(BROKEN_NAME)
        .expect("o script partido no projeto");
    assert_eq!(
        sim.world().resource::<ScriptEditRequests>().0,
        [broken],
        "a aba abre pelo mesmo pedido do «Edit script»"
    );
    let mut host = ScriptHost::new().expect("vm");
    host.scene_sync(sim.world_mut());
    match host.scene_info(broken) {
        Some(ScriptInfo::Broken(d)) => assert_eq!(d.line, Some(3), "{d:?}"),
        other => panic!("o script partido tem de estar partido: {other:?}"),
    }
}

/// ⭐⭐ **A cena 6 ensina o que diz** (CodeEditor item 6): pelo caminho do produto — a ponte com as
/// portas da fábrica, o dreno das mortes e o rebobinar —, com o PILOTO das fotos a dar o teclado e o
/// rato. O círculo anda para a direita, três estrelas nascem da receita, sobem e somem; o clique leva
/// o quadrado sem script ao rato; rebobinar devolve tudo.
#[test]
fn a_cena_6_e_um_jogo_que_responde_ao_teclado_e_ao_rato() {
    let mut sim = SimWorld::new();
    let mut vec = VecScene::new();
    let mut mapa = VecEntityMap::new();
    let pecas = monta_em(&mut sim, &mut vec, &mut mapa, 6);
    let [nave, alvo, receita] = [pecas[0], pecas[1], pecas[2]];
    assert!(
        sim.world()
            .get_resource::<ScriptEditRequests>()
            .is_none_or(|r| r.0.is_empty()),
        "nenhuma aba abre"
    );
    // O registo do PRODUTO (o script incluído) e o sync das instâncias — a 1.ª versão deste teste
    // não os tinha, e a foto mostrou o que ele não via: a 1.ª estrela perdia o script.
    let reg = crate::component_registry_for_tests::registo();
    let fisica = ph2d_physics_ecs::PhysicsBridge::new();
    let mut eco = crate::instance_sync::MasterEcho::default();
    let mut host = ScriptHost::new().expect("vm");
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let mut entrada = ph2d_input::InputState::new();
    let x = |sim: &SimWorld, e: Entity| sim.world().get::<Transform>(e).expect("pose").translation;
    let copias = |sim: &mut SimWorld| {
        let w = sim.world_mut();
        w.query_filtered::<Entity, bevy_ecs::query::With<ph2d_ecs::Spawned>>()
            .iter(w)
            .count()
    };
    let mut nasceram = 0;
    let mut morreram = 0;
    for n in 0..260u64 {
        // O caminho do APP: os eventos no estado de entrada, a porta entrega e limpa.
        let onde = piloto(&mut entrada, n);
        crate::script_input::frame(Some(&host), &mut entrada, Some(onde));
        let antes = copias(&mut sim);
        let mut portas = crate::script_bridge::ScriptDoors {
            registry: &reg,
            tags: &ph2d_tags::TagTree::new(),
            docs: crate::instance_docs::OwnedDocs {
                vec_scene: &mut vec,
                vec_entities: &mut mapa,
            },
            tick: n,
        };
        let f = crate::script_bridge::frame(
            &mut host,
            &mut sim,
            &mut drive,
            true,
            1,
            1.0 / 60.0,
            &[],
            &mut portas,
        );
        assert!(f.failed.is_empty(), "quadro {n}: {:?}", f.failed);
        nasceram += copias(&mut sim).saturating_sub(antes);
        let mortes: Vec<ph2d_ecs::Death> = f
            .despawned
            .into_iter()
            .map(|entity| ph2d_ecs::Death {
                entity,
                signal: String::new(),
                why: ph2d_ecs::DeathCause::Killed,
            })
            .collect();
        morreram += crate::factory_bridge::apply_deaths(&mut sim, &mortes);
        drive.settle();
        // O que a shell faz a cada quadro: o sync mestre → instância e o documento vectorial.
        crate::instance_sync::sync_instances(
            &mut sim,
            &reg,
            &fisica,
            &mut eco,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut vec,
                vec_entities: &mut mapa,
            },
        );
        ph2d_vec_entities::entities::sync(&mut sim, &mut vec, &mut mapa);
    }
    assert!(
        x(&sim, nave).x > -1.0,
        "as setas levaram o círculo para a direita: {:?}",
        x(&sim, nave)
    );
    assert_eq!(nasceram, 3, "três toques no espaço, três estrelas");
    assert_eq!(morreram, 3, "cada uma apagou-se ao fim de 1,5 s");
    assert_eq!(copias(&mut sim), 0, "nenhuma estrela ficou parada");
    assert_eq!(
        vec.paths().len(),
        3,
        "o documento vectorial volta às três peças"
    );
    assert_eq!(
        (x(&sim, alvo).x, x(&sim, alvo).y),
        (-3.0, 2.0),
        "o clique levou o quadrado ao rato"
    );
    assert!(
        sim.world().get::<Transform>(receita).is_some(),
        "a receita fica"
    );
    let linhas: Vec<String> = host.drain_console().into_iter().map(|l| l.text).collect();
    assert_eq!(linhas, vec!["tiro\t1", "tiro\t2", "tiro\t3"]);
    crate::script_bridge::rewind(&mut host, &mut sim, &mut drive);
    assert_eq!(
        (x(&sim, alvo).x, x(&sim, alvo).y),
        (JOGO_AT[1][0], JOGO_AT[1][1]),
        "rebobinar devolve o quadrado"
    );
    assert_eq!(
        (x(&sim, nave).x, x(&sim, nave).y),
        (JOGO_AT[0][0], JOGO_AT[0][1]),
        "e o círculo"
    );
}
