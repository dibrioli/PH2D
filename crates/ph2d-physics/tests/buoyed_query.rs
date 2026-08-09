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

/// **A escala NÃO tem topo — ela é a razão das densidades**, e sem gravidade não
/// há empuxo nenhum.
///
/// # ⚠️ Este gate afirmava o CONTRÁRIO até 2026-08-09, e a mudança é deliberada
///
/// Ele chamava-se `the_scale_tops_out_at_one_…` e pinava um `clamp(0.0, 1.0)`.
/// O teto era honesto enquanto o único consumidor perguntava `> 0` (a trava do
/// fluido do W-Submerged) — *"o fluido carrega-te inteiro"* era de facto o fim da
/// escala **para ele**. A lei cinemática (W-KinFluid) lê a MAGNITUDE para derivar
/// a gravidade efetiva, e ali o teto era fatal: medido, uma cápsula 4× menos
/// densa que a água satura em `1` já a **`y = 0,2`**, com mais de metade do corpo
/// fora, e `g·(1 − 1)` é **zero** — o personagem pararia no meio da poça e nunca
/// subiria. *Quem acrescenta o consumidor que precisa do número reconfere a nota
/// que o capava.*
///
/// ⚠️ **O oráculo é a razão das DENSIDADES**, que é o que Arquimedes dá para um
/// corpo todo submerso — e é ela que diz quantas vezes a gravidade dele o fluido
/// mais que compensa. Nenhum literal de profundidade aparece: o gate não sabe
/// onde a caixa está, só que **ela está toda dentro**.
///
/// A segunda metade fica: as duas respondem à MESMA pergunta (*o que este número
/// nunca é*), e a de gravidade zero é a mesma resposta que `buoyant_force` dá.
#[test]
fn the_scale_is_the_density_ratio_and_zero_gravity_carries_nothing() {
    let mut w = PhysicsWorld::new();
    pool(&mut w, 20.0);
    let c = crate_at(&mut w, 0.0, -1.5, 1.0);
    settle(&mut w, 3);
    let s = w.buoyed(c);
    assert!(
        (s - 20.0).abs() < 0.5,
        "um fluido 20× mais denso, com o corpo TODO dentro, carrega 20 pesos — e leu {s:.4}"
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

// ── O CORPO CINEMÁTICO (W-KinFluid) ──────────────────────────────────────────

/// Uma caixa **cinemática** — o corpo do player em modo Snap.
fn kinematic_crate(w: &mut PhysicsWorld, x: f32, y: f32, d: f32) -> RigidBodyHandle {
    w.spawn_body(BodyDesc {
        density: d,
        ..desc(
            RigidBodyType::KinematicPositionBased,
            x,
            y,
            ShapeDesc::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
        )
    })
}

/// **A ÁGUA EXISTE PARA UM CORPO CINEMÁTICO, e ele lê o MESMO número que o
/// dinâmico** — o gate que a wave inteira serve.
///
/// ⚠️ **Nasceu VERMELHO com `0,0000` contra `4,0`, e por DUAS causas em série:**
/// o par sensor-estático × corpo-cinemático não existia no grafo de interseção
/// (o `ActiveCollisionTypes::default()` do rapier só liga pares que começam em
/// DYNAMIC) e, mesmo existindo, `RigidBody::mass()` devolve zero para massa
/// infinita, então a consulta saía pelo `weight <= 0`. Cada uma sozinha bastava
/// para a resposta ser muda.
///
/// ⚠️ **O oráculo é o CONTROLE dinâmico, não um literal:** a mesma caixa, na
/// mesma poça, no mesmo tique — o que se afirma é que a espécie do corpo **não
/// é uma pergunta que a água faça**.
#[test]
fn the_water_exists_for_a_kinematic_body_too() {
    let mut w = PhysicsWorld::new();
    pool(&mut w, 4.0);
    let dynamic = crate_at(&mut w, -1.0, -1.0, 1.0);
    let kinematic = kinematic_crate(&mut w, 1.0, -1.0, 1.0);
    settle(&mut w, 3);

    let d = w.buoyed(dynamic);
    let k = w.buoyed(kinematic);
    assert!(
        d > 3.0,
        "o CONTROLE dinâmico tem de ler a razão 4 e leu {d:.4}"
    );
    assert!(
        (k - d).abs() < 0.01,
        "o cinemático tem de ler o MESMO que o dinâmico: {k:.4} contra {d:.4}"
    );
}

/// **E o ARRASTO do meio sai da MESMA varredura.**
///
/// ⚠️ **Ele é o MÁXIMO sobre as zonas, e é isso que apaga a dedup:** um corpo
/// composto sobrepõe a mesma poça com CADA forma dele (a lição da
/// W-CompoundZone), e uma soma sobre pares faria a água resistir `N` vezes mais
/// a uma jangada de `N` peças. O gate mede as duas coisas: o número certo, e a
/// **INVARIÂNCIA a quantas formas o corpo tem**.
#[test]
fn the_medium_reports_its_drag_once_however_many_shapes_i_have() {
    let mut w = PhysicsWorld::new();
    pool(&mut w, 4.0);
    let one = crate_at(&mut w, -1.0, -1.0, 1.0);
    // O mesmo corpo, com uma SEGUNDA forma dentro da mesma poça (W-Compound).
    let two = crate_at(&mut w, 1.0, -1.0, 1.0);
    w.attach_part(
        two,
        &desc(
            RigidBodyType::Dynamic,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
        ),
        [0.6, 0.0, 0.0],
    )
    .expect("a peça tem de prender num corpo que existe");
    settle(&mut w, 3);

    let a = w.fluid_at(one);
    let b = w.fluid_at(two);
    assert!(
        (a.drag - 1.5).abs() < 1.0e-6,
        "o arrasto tem de ser o da poça (1,5) e leu {:.4}",
        a.drag
    );
    assert!(
        (b.drag - a.drag).abs() < 1.0e-6,
        "duas formas na mesma poça não dobram a viscosidade dela: {:.4} contra {:.4}",
        b.drag,
        a.drag
    );
    // CONTROLE: o corpo seco não lê meio nenhum.
    let dry = crate_at(&mut w, 6.0, 2.0, 1.0);
    settle(&mut w, 3);
    assert_eq!(w.fluid_at(dry), ph2d_physics::FluidAt::DRY);
}
