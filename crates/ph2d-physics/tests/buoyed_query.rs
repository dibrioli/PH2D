//! **QUANTO DO PESO o fluido carrega** — os gates da consulta `buoyed`
//! (`W-Submerged`).
//!
//! Irmão do `buoyancy.rs` e com a MESMA piscina, de propósito: ele mede a FORÇA
//! e este mede a RAZÃO que sai dela. A pergunta que a consulta responde é a que
//! a lei do platformer faz — *"a gravidade ainda é a única coisa a agir sobre
//! mim?"* — e ela existe porque a modelagem do arco de um pulo vira uma **bomba
//! de energia** quando é o empuxo quem segura o corpo.
//!
//! ⚠️ **O oráculo é o EQUILÍBRIO, nunca um literal:** *boiar em repouso* **é** o
//! empuxo igualar o peso, então um corpo assentado tem de ler exactamente `1` —
//! e é essa propriedade, e não um número escolhido, que torna a lei
//! auto-consistente (o ponto fixo está derivado em `ph2d_platformer::Buoyed`).

use ph2d_physics::{AreaEffect, BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc};

fn desc(body_type: RigidBodyType, x: f32, y: f32, shape: ShapeDesc) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// A piscina do `buoyancy.rs`: sensor 8 × 4 com a superfície em `y = 0`.
fn pool(w: &mut PhysicsWorld, fluid_density: f32) {
    w.spawn_body(BodyDesc {
        is_sensor: true,
        effector: Some(AreaEffect {
            force: [0.0, 0.0],
            drag: 1.5,
            density: fluid_density,
            form_drag: 0.0,
            torque: 0.0,
            world_axes: false,
            falloff: 0.0,
            mirror: [1.0, 1.0],
        }),
        ..desc(
            RigidBodyType::Fixed,
            0.0,
            -2.0,
            ShapeDesc::Cuboid {
                half_x: 4.0,
                half_y: 2.0,
            },
        )
    });
}

fn crate_at(w: &mut PhysicsWorld, x: f32, y: f32, d: f32) -> RigidBodyHandle {
    w.spawn_body(BodyDesc {
        density: d,
        ..desc(
            RigidBodyType::Dynamic,
            x,
            y,
            ShapeDesc::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
        )
    })
}

fn settle(w: &mut PhysicsWorld, n: usize) {
    for _ in 0..n {
        w.step();
    }
}

/// **Boiar em repouso lê `1`, e é a definição, não uma calibração.**
///
/// ⚠️ Nenhum literal de linha d'água aparece aqui: o gate não sabe onde a caixa
/// assenta, só que **onde quer que ela assente** o empuxo iguala o peso. É isso
/// que faz dela um oráculo em vez de um espelho da fórmula.
#[test]
fn a_body_at_rest_on_the_surface_is_carried_whole() {
    let mut w = PhysicsWorld::new();
    pool(&mut w, 4.0);
    let c = crate_at(&mut w, 0.0, -1.0, 1.0);
    settle(&mut w, 1200);
    let s = w.buoyed(c);
    assert!(
        (s - 1.0).abs() < 0.02,
        "uma caixa assentada tem de ler 1,0 (o empuxo IGUALA o peso), e leu {s:.4}"
    );
}

/// **Fora d'água é ZERO** — e o gate mede um corpo que está de facto abaixo do
/// NÍVEL da superfície, só que ao lado da poça.
///
/// ⚠️ **É a metade que uma re-derivação por altura erraria**, e ela é o motivo
/// de a consulta passear pelo grafo de interseção: o recorte de
/// `buoyant_force` conhece o *nível* e mais nada, então um corpo numa vala ao
/// LADO leria `1` sem esta pergunta.
#[test]
fn a_body_beside_the_pool_is_dry_even_below_the_waterline() {
    let mut w = PhysicsWorld::new();
    pool(&mut w, 4.0);
    // x = 6 está fora dos 8 de largura (meia-largura 4), e y = −1 está um metro
    // ABAIXO da superfície.
    let c = crate_at(&mut w, 6.0, -1.0, 1.0);
    settle(&mut w, 5);
    let s = w.buoyed(c);
    assert_eq!(
        s, 0.0,
        "um corpo ao LADO da poça, abaixo do nível dela, tem de ler 0 e leu {s:.4}"
    );
}

/// **Um corpo mais denso que o fluido lê a razão das densidades**, e o número
/// diz quanto da gravidade dele o fluido de facto compensa.
///
/// ⚠️ O oráculo é `ρ_fluido / ρ_corpo` — uma pedra de densidade 4 num fluido de
/// densidade 1 tem um quarto do peso carregado —, e essa é a razão de a
/// grandeza ser PESO e não área submersa: as duas coincidiriam aqui só por
/// acidente de a pedra estar toda dentro.
#[test]
fn a_sinking_body_reads_the_share_of_its_weight_the_fluid_carries() {
    let mut w = PhysicsWorld::new();
    pool(&mut w, 1.0);
    let c = crate_at(&mut w, 0.0, -1.0, 4.0);
    settle(&mut w, 5);
    let s = w.buoyed(c);
    assert!(
        (s - 0.25).abs() < 0.02,
        "densidade 4 em fluido 1 tem de ler 0,25 e leu {s:.4}"
    );
}

/// **A escala tem topo em `1`**, e sem gravidade não há empuxo nenhum.
///
/// As duas metades num gate só porque respondem à MESMA pergunta — *o que este
/// número NUNCA é* — e porque a segunda é a mesma resposta que
/// `buoyant_force` dá (Arquimedes é consequência do peso do fluido).
#[test]
fn the_scale_tops_out_at_one_and_zero_gravity_carries_nothing() {
    let mut w = PhysicsWorld::new();
    pool(&mut w, 20.0);
    let c = crate_at(&mut w, 0.0, -1.5, 1.0);
    settle(&mut w, 3);
    let s = w.buoyed(c);
    assert!(
        (0.0..=1.0).contains(&s),
        "a razão tem de estar em [0, 1] e leu {s:.4}"
    );
    assert!(
        s > 0.9,
        "um fluido 20× mais denso carrega o corpo inteiro, e leu {s:.4}"
    );

    let mut dry = PhysicsWorld::new();
    dry.set_gravity(0.0, 0.0);
    pool(&mut dry, 4.0);
    let d = crate_at(&mut dry, 0.0, -1.0, 1.0);
    settle(&mut dry, 5);
    assert_eq!(
        dry.buoyed(d),
        0.0,
        "sem gravidade não há empuxo, então não há peso carregado"
    );
}

/// **Uma cena SEM poça sai pelo primeiro `if`, e a resposta é zero.**
///
/// ⚠️ Gate de PRESENÇA da rota barata: é o early-out que torna a consulta
/// pagável a cada tique de player, e ele é observável só como este zero.
#[test]
fn a_world_without_a_pool_answers_zero() {
    let mut w = PhysicsWorld::new();
    let c = crate_at(&mut w, 0.0, 2.0, 1.0);
    settle(&mut w, 5);
    assert_eq!(w.buoyed(c), 0.0);
}
