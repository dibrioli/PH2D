//! **A forma «Outline» pela ponte ECS** (W2b do plano f2): o `ColliderOutline` da FILHA vira peças
//! convexas no solver pelo `PhysicsBridge::dispatch`, com a escala COM SINAL, e sobrevive ao rewind e
//! ao snapshot (o caminho do save e do undo).
//!
//! Imagem 32×32 px, `half` 2×2 m: 8 px = 1 m (o Detail de 2 px fica bem abaixo dos vãos), o centro
//! da imagem na origem do corpo, y para cima. As figuras escrevem-se em unidades de 8 px.

use std::sync::Arc;

use ph2d_core::Vec2;
use ph2d_ecs::scene::{
    ComponentRegistry, WorldSnapshot, register_ecs_components, snapshot_to_world, world_to_snapshot,
};
use ph2d_ecs::{ChildOf, SimWorld, Transform, TransformPropagationState, WorklistBuf};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderOutline, ColliderShape, PhysicsBridge, RigidBody,
    register_physics_components,
};

const Q: i32 = 16;

/// Um anel em unidades de 8 px (y para baixo) → Q4.
fn ring(u: &[[i32; 2]]) -> Vec<[i32; 2]> {
    u.iter().map(|&[x, y]| [x * 8 * Q, y * 8 * Q]).collect()
}

/// U aberto para cima: postes x∈[0,1] e x∈[3,4], fundo y∈[3,4] (px). Em metros: postes
/// x∈[−2,−1] e x∈[1,2], o fundo do lado de dentro em y = −1.
fn u_ring() -> Vec<[i32; 2]> {
    ring(&[
        [0, 0],
        [1, 0],
        [1, 3],
        [3, 3],
        [3, 0],
        [4, 0],
        [4, 4],
        [0, 4],
    ])
}

/// L: poste x∈[0,1] em toda a altura, pé y∈[3,4] em toda a largura (px). Em metros: poste
/// x∈[−2,−1] até y = 2; pé até y = −1.
fn l_ring() -> Vec<[i32; 2]> {
    ring(&[[0, 0], [1, 0], [1, 3], [4, 3], [4, 4], [0, 4]])
}

fn outline(r: Vec<[i32; 2]>) -> ColliderOutline {
    ColliderOutline {
        rings: Arc::from(vec![r]),
        picture_px: [32, 32],
        ..ColliderOutline::default()
    }
}

fn scaled(x: f32, y: f32, s: Vec2) -> Transform {
    Transform {
        translation: Vec2::new(x, y),
        rotation: 0.0,
        scale: s,
        skew_x: 0.0,
        skew_y: 0.0,
    }
}

/// Corpo Static na origem, com a forma Outline numa FILHA (opção B), de escala `s`.
fn static_outline(sim: &mut SimWorld, r: Vec<[i32; 2]>, s: Vec2) {
    let body = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    sim.world_mut().spawn((
        Collider {
            shape: ColliderShape::Outline {
                half_x: 2.0,
                half_y: 2.0,
            },
            ..Collider::default()
        },
        outline(r),
        scaled(0.0, 0.0, s),
        ChildOf(body),
    ));
}

fn ball(sim: &mut SimWorld, x: f32, y: f32) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ))
        .id()
}

fn run(bridge: &mut PhysicsBridge, sim: &mut SimWorld, ticks: std::ops::RangeInclusive<u64>) {
    for tick in ticks {
        bridge.dispatch(sim, true, tick);
    }
}

fn rest_y(r: Vec<[i32; 2]>, s: Vec2, at: [f32; 2]) -> f32 {
    let mut sim = SimWorld::new();
    static_outline(&mut sim, r, s);
    let b = ball(&mut sim, at[0], at[1]);
    let mut bridge = PhysicsBridge::new();
    run(&mut bridge, &mut sim, 1..=240);
    bridge.body_pose(b).expect("a bola tem corpo").1
}

/// A bola cai DENTRO do U (fundo em y = −1 ⇒ centro ≈ −0,7), não em cima da caixa (≈ 2,3).
/// *Mutação: a ponte devolver a caixa (`resolve` sem as peças) ⇒ ≈ 2,3, vermelho.*
#[test]
fn uma_bola_cai_dentro_do_u_e_nao_na_caixa() {
    let y = rest_y(u_ring(), Vec2::new(1.0, 1.0), [0.0, 5.0]);
    assert!(
        (y + 0.7).abs() < 0.05,
        "a bola parou em y = {y}; dentro do U é ≈ −0,7"
    );
}

/// O L espelhado (`scale.x = −1`) tem o poste à DIREITA: a sonda em x = −1,5 cai no pé (≈ −0,7);
/// sem espelho cai no topo do poste (≈ 2,3). *Mutação: a escala em módulo (`abs`) ⇒ ≈ 2,3.*
#[test]
fn o_l_espelhado_tem_o_poste_do_outro_lado() {
    let reto = rest_y(l_ring(), Vec2::new(1.0, 1.0), [-1.5, 5.0]);
    let espelho = rest_y(l_ring(), Vec2::new(-1.0, 1.0), [-1.5, 5.0]);
    assert!(
        (reto - 2.3).abs() < 0.05,
        "sem espelho: topo do poste, y = {reto}"
    );
    assert!(
        (espelho + 0.7).abs() < 0.05,
        "espelhado: o pé, y = {espelho}"
    );
}

/// Escala 2× dobra a extensão: o poste do L passa a x∈[−4,−2] e a y = 4 ⇒ a sonda em x = −3 pára
/// em ≈ 4,3. *Mutação: a meia-caixa sem a escala ⇒ a sonda cai no pé.*
#[test]
fn escala_dois_dobra_o_contorno() {
    let y = rest_y(l_ring(), Vec2::new(2.0, 2.0), [-3.0, 8.0]);
    assert!((y - 4.3).abs() < 0.05, "o topo do poste escalado: y = {y}");
}

/// O rewind do ECS troca o `PhysicsWorld` e re-spawna por `BodyDesc`: as peças vão junto, e a
/// segunda corrida acaba onde a primeira acabou. *Mutação: sem `carry_outlines_from` ⇒ caixa.*
#[test]
fn o_rewind_do_ecs_guarda_o_contorno() {
    let mut sim = SimWorld::new();
    static_outline(&mut sim, u_ring(), Vec2::new(1.0, 1.0));
    let b = ball(&mut sim, 0.0, 5.0);
    let mut bridge = PhysicsBridge::new();
    run(&mut bridge, &mut sim, 1..=240);
    let primeira = bridge.body_pose(b).expect("corpo").1;
    bridge.dispatch(&mut sim, false, 0);
    run(&mut bridge, &mut sim, 1..=240);
    let segunda = bridge.body_pose(b).expect("corpo").1;
    assert!((primeira + 0.7).abs() < 0.05, "1.ª corrida: y = {primeira}");
    assert_eq!(
        primeira.to_bits(),
        segunda.to_bits(),
        "o replay é outro mundo"
    );
}

/// O `ColliderOutline` atravessa o snapshot (o fio do save e do undo) inteiro.
#[test]
fn o_contorno_vai_e_volta_pelo_snapshot() {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_physics_components(&mut reg);
    let mut src = SimWorld::new();
    let o = ColliderOutline {
        detail_q4: 7,
        alpha_cutoff: 9,
        min_island_px: 11,
        picture_px: [4, 5],
        ..outline(l_ring())
    };
    src.world_mut().spawn((Transform::default(), o.clone()));
    let mut state = TransformPropagationState::new(src.world_mut());
    let mut worklist = WorklistBuf::new();
    let mut snap = WorldSnapshot::new();
    world_to_snapshot(src.world_mut(), &mut state, &mut worklist, &reg, &mut snap)
        .expect("snapshot");
    let mut dst = SimWorld::new();
    let spawned = snapshot_to_world(dst.world_mut(), &snap, &reg).expect("restore");
    let back = dst.world().get::<ColliderOutline>(spawned[0]).cloned();
    assert_eq!(back, Some(o));
}

/// **Nunca um buraco onde a arte é sólida**, também na imagem PEQUENA espelhada (4×4 px, Detail
/// 2 px = metade da imagem). Aí o Detail funde o L numa peça só, com o canto de dentro em rampa de
/// 45° (1,5 px para fora, dentro do Detail): a sonda do lado do pé escorrega e sai pela ponta — não
/// é buraco (era o «y = −50» de 09/11). A lei medida: enquanto a bola está sobre a arte, nunca
/// afunda abaixo do topo dela (poste y = 2, pé y = −1), nas duas mãos e nos dois lados.
/// *Mutação: a escala em módulo no `resolve` ⇒ o poste espelhado fica do outro lado e a sonda
/// sobre ele afunda até a rampa, vermelho.*
#[test]
fn o_l_pequeno_espelhado_nao_tem_buraco_onde_a_arte_e_solida() {
    let px: Vec<[i32; 2]> = [[0, 0], [1, 0], [1, 3], [4, 3], [4, 4], [0, 4]]
        .iter()
        .map(|&[x, y]| [x * Q, y * Q])
        .collect();
    for sx in [1.0f32, -1.0] {
        for x0 in [-1.5f32, 1.5] {
            let mut sim = SimWorld::new();
            static_outline(&mut sim, px.clone(), Vec2::new(sx, 1.0));
            let mut o = sim.world_mut().query::<&mut ColliderOutline>();
            o.single_mut(sim.world_mut()).expect("a forma").picture_px = [4, 4];
            let b = ball(&mut sim, x0, 5.0);
            let mut bridge = PhysicsBridge::new();
            for tick in 1..=240 {
                bridge.dispatch(&mut sim, true, tick);
                let (x, y, _) = bridge.body_pose(b).expect("corpo");
                let topo = if x * sx < -1.0 { 2.0 } else { -1.0 };
                assert!(
                    x.abs() > 1.7 || y > topo + 0.25,
                    "sx = {sx}, sonda em {x0}: no tique {tick} a bola está em ({x}, {y}), \
                     dentro da arte (topo {topo})"
                );
            }
        }
    }
}
