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
        let r = host.scene_tick(sim.world_mut(), 1.0 / 30.0);
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
