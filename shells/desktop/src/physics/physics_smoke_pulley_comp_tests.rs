//! **A metade da fronteira**: os gates da cena `physics_smoke_pulley_comp` que exercitam um
//! GESTO DA SHELL (`crate::physics::…`) sobre uma cena que vive na crate da família.
//!
//! ⚠️ **Eles estão aqui porque um `#[cfg(test)]` é INVISÍVEL do outro lado da
//! fronteira de crate** (HOWTO §2): o sujeito é da crate, o gesto é da shell, e um
//! teste não pode ter um pé em cada. Os outros gates da mesma cena ficaram **com o
//! sujeito**, na crate — *o corte é por quem o teste EXERCITA, não por quem ele nomeia.*

use ph2d_app_physics::physics_smoke_pulley_comp::*;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let mut q = sim.world_mut().query::<(&Name, &Transform)>();
    q.iter(sim.world())
        .find(|(n, _)| n.as_str() == name)
        .map(|(_, t)| t.translation.y)
        .expect("corpo vivo")
}

/// **Os dois gestos do PISO, na cena 63 de verdade** — pelo caminho do produto
/// (`add_pulley_wheel` é a função que o botão chama).
///
/// A medição na bancada (o elevador de `measure_pulley_route_gestures`) prova o
/// mecanismo; esta prova que a CENA que o artista abre sobrevive aos dois gestos,
/// porque uma mensagem de smoke que promete algo não medido é como duas cenas desta
/// linha já afirmaram o que a medição desmentiu.
///
/// `cargo test -p ph2d-host-desktop --bins probe_smoke_63_floor -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate"]
fn probe_smoke_63_floor() {
    /// O maior passo que a carga engrenada dá num tique só, em 2 s.
    fn worst(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> f32 {
        let mut prev = y_of(sim, "Geared Load");
        let mut worst = 0.0f32;
        for t in 1..=120 {
            bridge.dispatch(sim, true, t);
            let now = y_of(sim, "Geared Load");
            worst = worst.max((now - prev).abs());
            prev = now;
        }
        worst
    }
    fn scene() -> (SimWorld, PhysicsBridge) {
        let mut sim = SimWorld::new();
        build_composition(sim.world_mut());
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        (sim, bridge)
    }
    fn rope_bits(sim: &mut SimWorld) -> u64 {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Geared Rope")
            .map(|(e, _)| e.to_bits())
            .expect("a corda engrenada")
    }

    // ⚠️ **O CONTROLE POSITIVO vem antes do número:** um salto igual ao do
    // controle só significa "não deu tranco" se o gesto de fato POUSOU. Sem isto,
    // um `add_pulley_wheel` que desiste em silêncio mede a cena intocada e o
    // resultado é verde sobre nada.
    fn state(sim: &mut SimWorld, bridge: &PhysicsBridge) -> (usize, f32) {
        let e = ph2d_ecs::Entity::from_bits(rope_bits(sim));
        let l0 = sim
            .world()
            .get::<ph2d_physics_ecs::PhysicsJoint>(e)
            .map_or(f32::NAN, |j| j.max_length);
        (bridge.rope_wheels(e).count(), l0)
    }

    println!("\n=== CENA 63 — os dois gestos que o piso cobre (2 s) ===");
    let (mut sim, mut bridge) = scene();
    let base = state(&mut sim, &bridge);
    let control = worst(&mut sim, &mut bridge);
    println!(
        "  controle (ninguem tocou)         roldanas={} L0={:.4}  maior salto = {control:.4} m",
        base.0, base.1
    );

    let (mut sim, mut bridge) = scene();
    let bits = rope_bits(&mut sim);
    crate::physics::joint_wheel::add_pulley_wheel(&mut sim, &bridge, bits);
    bridge.dispatch(&mut sim, false, 0);
    let after = state(&mut sim, &bridge);
    println!(
        "  + Add Wheel                      roldanas={} L0={:.4}  maior salto = {:.4} m{}",
        after.0,
        after.1,
        worst(&mut sim, &mut bridge),
        if after.0 == base.0 {
            "   <-- O GESTO NAO POUSOU"
        } else {
            ""
        }
    );

    let (mut sim, mut bridge) = scene();
    let e = ph2d_ecs::Entity::from_bits(rope_bits(&mut sim));
    if let Some(mut j) = sim.world_mut().get_mut::<ph2d_physics_ecs::PhysicsJoint>(e) {
        j.max_length = 2.0;
    }
    bridge.dispatch(&mut sim, false, 0);
    let after = state(&mut sim, &bridge);
    println!(
        "  + Rope Length = 2.0 (absurdo)    roldanas={} L0={:.4}  maior salto = {:.4} m{}",
        after.0,
        after.1,
        worst(&mut sim, &mut bridge),
        if (after.1 - 2.0).abs() < 1.0e-3 {
            "   <-- O PISO NAO SUBIU"
        } else {
            ""
        }
    );
    println!();
}

/// **E a cena 63 sobrevive aos dois gestos do PISO** — a afirmação que a mensagem
/// faz, no rig que o artista abre.
///
/// A bancada (`ph2d-physics-ecs/tests/it/pulley_route_floor.rs`) prova o mecanismo com
/// um elevador de 12 m; esta prova a CENA, cuja corda mede 80,9 m e cujo tambor é
/// diferencial — geometria que nenhuma fixture de bancada tem.
///
/// ⚠️ **O controle positivo vem PRIMEIRO, e é o que torna o número legível:** um
/// salto igual ao do controle só significa *não trancou* se o gesto POUSOU. Sem a
/// contagem de roldanas e sem o `L0`, um `add_pulley_wheel` que desistisse em
/// silêncio mediria a cena intocada — verde sobre nada.
#[test]
fn the_composition_scene_survives_the_floors_two_gestures() {
    fn scene() -> (SimWorld, PhysicsBridge) {
        let mut sim = SimWorld::new();
        build_composition(sim.world_mut());
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        (sim, bridge)
    }
    fn rope(sim: &mut SimWorld) -> ph2d_ecs::Entity {
        let mut q = sim.world_mut().query::<(ph2d_ecs::Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.as_str() == "Geared Rope")
            .map(|(e, _)| e)
            .expect("a corda engrenada")
    }
    fn l0(sim: &mut SimWorld) -> f32 {
        let e = rope(sim);
        sim.world()
            .get::<ph2d_physics_ecs::PhysicsJoint>(e)
            .map_or(f32::NAN, |j| j.max_length)
    }
    fn worst(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> f32 {
        let mut prev = y_of(sim, "Geared Load");
        let mut worst = 0.0f32;
        for t in 1..=120 {
            bridge.dispatch(sim, true, t);
            let now = y_of(sim, "Geared Load");
            worst = worst.max((now - prev).abs());
            prev = now;
        }
        worst
    }

    let (mut sim, mut bridge) = scene();
    let wheels_before = bridge.rope_wheels(rope(&mut sim)).count();
    let control = worst(&mut sim, &mut bridge);
    assert!(
        control < 0.05,
        "o CONTROLE ja tranca ({control:.4} m) — a fixture nao mede o defeito"
    );

    // (a) Acrescentar uma roldana pelo caminho do PRODUTO.
    let (mut sim, mut bridge) = scene();
    let bits = rope(&mut sim).to_bits();
    crate::physics::joint_wheel::add_pulley_wheel(&mut sim, &bridge, bits);
    bridge.dispatch(&mut sim, false, 0);
    let wheels_after = bridge.rope_wheels(rope(&mut sim)).count();
    assert!(
        wheels_after == wheels_before + 1,
        "o gesto nao pousou: {wheels_before} roldanas antes, {wheels_after} depois — \
         o numero abaixo mediria a cena intocada"
    );
    let added = worst(&mut sim, &mut bridge);
    assert!(
        added < control * 1.5,
        "Add Wheel trancou a cena: maior salto {added:.4} m contra {control:.4} do controle"
    );

    // (b) E um comprimento impossivel digitado na row.
    let (mut sim, mut bridge) = scene();
    let e = rope(&mut sim);
    if let Some(mut j) = sim.world_mut().get_mut::<ph2d_physics_ecs::PhysicsJoint>(e) {
        j.max_length = 2.0;
    }
    bridge.dispatch(&mut sim, false, 0);
    let floored = l0(&mut sim);
    assert!(
        floored > 10.0,
        "o piso nao subiu: a row ficou em {floored:.4} m sobre uma corda de \
         {MEASURED_GEARED_ROPE_LENGTH:.1} m"
    );
    let typed = worst(&mut sim, &mut bridge);
    assert!(
        typed < control * 1.5,
        "um comprimento impossivel trancou a cena: maior salto {typed:.4} m contra \
         {control:.4} do controle"
    );
}
