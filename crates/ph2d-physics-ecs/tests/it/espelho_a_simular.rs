//! ⭐ **Um objecto espelhado por escala negativa, a simular** (R4, `docs/CartoesUI/10`): a forma do
//! colisor é a mesma, a queda é a mesma, o offset muda de lado, e a física nunca apaga o sinal da
//! escala que o artista escreveu. O espelho do mundo é a régua: o objecto espelhado tem de fazer o
//! que faz a imagem no espelho do objecto normal.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, RigidBody};

const CAIXA: ColliderShape = ColliderShape::Cuboid {
    half_x: 0.5,
    half_y: 0.25,
};
const BOLA: ColliderShape = ColliderShape::Ball { radius: 0.5 };
const PASSOS: u64 = 300;

fn estatico(sim: &mut SimWorld, x: f32, y: f32, half_x: f32, half_y: f32) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid { half_x, half_y },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(x, y)),
    ));
}

fn corpo(shape: ColliderShape, offset: [f32; 2]) -> (RigidBody, Collider) {
    (
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape,
            density: 1.0,
            offset,
            ..Collider::default()
        },
    )
}

/// Um corpo sem rotação livre, sobre o chão de topo `y = 0.1`.
fn cena(shape: ColliderShape, offset: [f32; 2], t: Transform) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    estatico(&mut sim, 0.0, 0.0, 50.0, 0.1);
    let e = sim
        .world_mut()
        .spawn((corpo(shape, offset), LockRotation, t))
        .id();
    (sim, e)
}

fn pose(t: Vec2, s: Vec2) -> Transform {
    Transform {
        translation: t,
        scale: s,
        ..Transform::default()
    }
}

fn le(sim: &SimWorld, e: Entity) -> Transform {
    *sim.world().get::<Transform>(e).unwrap()
}

/// A trajetória inteira do corpo `e`, tique a tique.
fn trajetoria(sim: &mut SimWorld, e: Entity, passos: u64) -> Vec<(f32, f32)> {
    let mut b = PhysicsBridge::new();
    (1..=passos)
        .map(|tick| {
            b.dispatch(sim, true, tick);
            let t = le(sim, e);
            (t.translation.x, t.translation.y)
        })
        .collect()
}

/// ⭐ Sem offset, o espelho não muda nada da física: a mesma trajetória, em bits, e o sinal da escala
/// sobrevive ao `readback`. *Mutação: o `readback` escrever a escala do corpo, ou `scaled_shape` sem `abs`.*
#[test]
fn a_mirrored_body_falls_and_rests_exactly_like_the_unmirrored_one() {
    for shape in [CAIXA, BOLA] {
        for (sx, sy) in [(2.0_f32, 2.0_f32), (2.0, 3.0)] {
            let origem = Vec2::new(1.0, 6.0);
            let (mut sim, e) = cena(shape, [0.0, 0.0], pose(origem, Vec2::new(sx, sy)));
            let normal = trajetoria(&mut sim, e, PASSOS);
            assert!(
                normal[PASSOS as usize - 1].1 < 3.0,
                "{shape:?}: não caiu: {:?}",
                normal[PASSOS as usize - 1]
            );
            for sinal in [(-1.0_f32, 1.0_f32), (1.0, -1.0), (-1.0, -1.0)] {
                let s = Vec2::new(sx * sinal.0, sy * sinal.1);
                let (mut sim, e) = cena(shape, [0.0, 0.0], pose(origem, s));
                let t = trajetoria(&mut sim, e, PASSOS);
                assert_eq!(t, normal, "{shape:?} sob {s:?} caiu de outro modo");
                assert_eq!(le(&sim, e).scale, s, "a física apagou o espelho");
            }
        }
    }
}

/// ⭐ O offset em Y muda de lado: a bola com o colisor 1 m acima da origem pousa 1 m abaixo do
/// centrado; virada em Y, o colisor fica por baixo e ela pousa 1 m acima dele.
/// *Mutação: `collider_offset` com `abs`.*
#[test]
fn the_offset_goes_to_the_other_side_under_a_vertical_flip() {
    let o = Vec2::new(0.0, 6.0);
    let pouso = |shape, offset, sy: f32| {
        let (mut sim, e) = cena(shape, offset, pose(o, Vec2::new(1.0, sy)));
        *trajetoria(&mut sim, e, PASSOS).last().unwrap()
    };
    for (shape, h) in [(CAIXA, 0.25_f32), (BOLA, 0.5)] {
        let centro = 0.1 + h;
        for (sy, esperado) in [(1.0_f32, centro - 1.0), (-1.0, centro + 1.0)] {
            let (x, y) = pouso(shape, [0.0, 1.0], sy);
            assert!(
                (y - esperado).abs() < 0.03,
                "{shape:?} sy={sy}: y={y}, esperado {esperado}"
            );
            assert!(x.abs() < 1e-4, "{shape:?} sy={sy}: x={x}");
        }
    }
}

/// ⭐ O offset em X também: há um poste só à ESQUERDA. O objecto de colisor à direita falha-o e cai
/// no chão; espelhado em X, o colisor passa para a esquerda e pousa no poste.
#[test]
fn the_offset_goes_to_the_other_side_under_a_horizontal_flip() {
    let pouso = |shape, sx: f32| {
        let (mut sim, e) = cena(
            shape,
            [0.8, 0.0],
            pose(Vec2::new(0.0, 6.0), Vec2::new(sx, 1.0)),
        );
        estatico(&mut sim, -0.8, 1.0, 0.3, 1.0);
        trajetoria(&mut sim, e, PASSOS).last().unwrap().1
    };
    for (shape, h) in [(CAIXA, 0.25_f32), (BOLA, 0.5)] {
        let (fora, sobre) = (pouso(shape, 1.0), pouso(shape, -1.0));
        assert!((fora - (0.1 + h)).abs() < 0.03, "{shape:?}: {fora}");
        assert!((sobre - (2.0 + h)).abs() < 0.03, "{shape:?}: {sobre}");
    }
}

/// ⭐ A régua do espelho, com rotação LIVRE e offset: um corpo inclinado, de colisor fora do centro,
/// vira de lado ao bater no chão. Espelhado em X, faz o que faz a imagem no espelho do normal.
/// *Mutação: o offset sem sinal (a queda vira para o lado errado).*
#[test]
fn a_tumbling_mirrored_body_is_the_mirror_image_of_the_normal_one() {
    let queda = |x: f32, rot: f32, sx: f32| {
        let mut sim = SimWorld::new();
        estatico(&mut sim, 0.0, 0.0, 50.0, 0.1);
        let t = Transform {
            rotation: rot,
            ..pose(Vec2::new(x, 4.0), Vec2::new(sx, 1.0))
        };
        let e = sim.world_mut().spawn((corpo(CAIXA, [0.6, 0.2]), t)).id();
        let mut b = PhysicsBridge::new();
        let mut linha = Vec::new();
        for tick in 1..=PASSOS {
            b.dispatch(&mut sim, true, tick);
            let t = le(&sim, e);
            linha.push((t.translation.x, t.translation.y, t.rotation));
        }
        linha
    };
    let (normal, espelhado) = (queda(-2.0, -0.3, 1.0), queda(2.0, 0.3, -1.0));
    let mut pior = 0.0_f32;
    for (n, m) in normal.iter().zip(&espelhado) {
        pior = pior
            .max((n.0 + m.0).abs())
            .max((n.1 - m.1).abs())
            .max((n.2 + m.2).abs());
    }
    let fim = normal[PASSOS as usize - 1];
    assert!(
        (fim.0 + 2.0).abs() > 0.01 && fim.2.abs() < 0.01,
        "a caixa inclinada não tombou até deitar: {fim:?}"
    );
    assert!(pior < 0.02, "o espelhado afastou-se do espelho em {pior}");
}

/// O reset devolve a pose autorada, com o sinal. *Mutação: o `readback` apagar o sinal da escala.*
#[test]
fn a_rewind_gives_back_the_pose_and_the_flip() {
    let autorado = pose(Vec2::new(1.0, 6.0), Vec2::new(-2.0, 3.0));
    let (mut sim, e) = cena(CAIXA, [0.5, 0.2], autorado);
    let mut b = PhysicsBridge::new();
    for tick in 1..=PASSOS {
        b.dispatch(&mut sim, true, tick);
    }
    let caiu = le(&sim, e);
    assert_ne!(caiu.translation.y, autorado.translation.y);
    assert_eq!(caiu.scale, autorado.scale);
    b.dispatch(&mut sim, false, 0);
    let voltou = le(&sim, e);
    assert_eq!(voltou.translation, autorado.translation);
    assert_eq!(voltou.scale, autorado.scale);
}

/// ⭐ Filho de um pai espelhado: o corpo simula onde se desenha, o `Transform` local não acumula o
/// espelho quadro a quadro, e o colisor herda o sinal do MUNDO.
/// *Mutação: `write_world_pose` sem `inverse_compose`.*
#[test]
fn a_body_under_a_mirrored_parent_simulates_where_it_is_drawn() {
    let mut sim = SimWorld::new();
    estatico(&mut sim, 0.0, 0.0, 50.0, 0.1);
    let pai = sim
        .world_mut()
        .spawn(pose(Vec2::new(5.0, 0.0), Vec2::new(-1.0, 1.0)))
        .id();
    let filho = sim
        .world_mut()
        .spawn((
            corpo(BOLA, [0.0, 0.0]),
            LockRotation,
            Transform::from_translation(Vec2::new(2.0, 6.0)),
            ChildOf(pai),
        ))
        .id();
    let mut b = PhysicsBridge::new();
    for tick in 1..=PASSOS {
        b.dispatch(&mut sim, true, tick);
        let local = le(&sim, filho);
        let (x, _, _) = b.body_pose(filho).unwrap();
        assert!((x - 3.0).abs() < 1e-3, "tique {tick}: simula em x={x}");
        assert!(
            (local.translation.x - 2.0).abs() < 1e-3,
            "tique {tick}: o local andou para {}",
            local.translation.x
        );
    }
    assert!((le(&sim, filho).translation.y - 0.6).abs() < 0.03);
    assert_eq!(le(&sim, filho).scale, Vec2::new(1.0, 1.0));
}

/// ⭐ Espelhar a meio da simulação (o personagem que vira de lado com o relógio a andar): o colisor
/// muda de lado no corpo vivo, sem o recriar. Vira-se ainda no ar: virado já pousado, o colisor salta
/// para baixo do chão e o corpo cai (medido: `y = -123`), a lei do teste seguinte.
/// *Mutação: o `retune_body` sem `set_position_wrt_parent`.*
#[test]
fn flipping_with_the_clock_running_moves_the_collider_at_once() {
    let (mut sim, e) = cena(
        BOLA,
        [0.0, 1.0],
        pose(Vec2::new(0.0, 6.0), Vec2::new(1.0, 1.0)),
    );
    let mut b = PhysicsBridge::new();
    for tick in 1..=5 {
        b.dispatch(&mut sim, true, tick);
    }
    let no_ar = le(&sim, e).translation.y;
    assert!(no_ar > 5.0, "{no_ar}");
    sim.world_mut().get_mut::<Transform>(e).unwrap().scale = Vec2::new(1.0, -1.0);
    for tick in 6..=PASSOS {
        b.dispatch(&mut sim, true, tick);
    }
    let depois = le(&sim, e);
    assert_eq!(
        depois.scale,
        Vec2::new(1.0, -1.0),
        "a física apagou o espelho"
    );
    assert!(
        (depois.translation.y - 1.6).abs() < 0.05,
        "o colisor ficou do lado de antes: y={}",
        depois.translation.y
    );
}

/// ⭐ R5, o limite medido contra o Godot 4.7.2 (`godot_espelho_em_repouso.txt`): pousado sobre chão fino,
/// mudar o colisor para o outro lado da origem atravessa o chão, cá e lá. Virar em Y é esse mesmo salto.
/// A diferença é da casa: o Godot apaga a escala do corpo (`corpo_scale_y` volta a 1 e o colisor fica),
/// o PH2D guarda o sinal. *Mutação: curar o salto com um «levanta» automático ⇒ vermelho.*
#[test]
fn flipping_at_rest_over_a_thin_floor_falls_like_the_oracles_moved_collider() {
    let oraculo = include_str!("godot_espelho_em_repouso.txt");
    let origem_y = |caso: &str| -> f32 {
        let linha = oraculo
            .lines()
            .find(|l| l.starts_with(caso) && l.contains("t+300"))
            .expect("a linha do oráculo");
        let valor = linha.split("origem_y=").nth(1).expect("origem_y");
        valor.split_whitespace().next().unwrap().parse().unwrap()
    };
    assert!(origem_y("forma_pos_y") > 1000.0, "o Godot atravessa o chão");
    assert!(
        origem_y("corpo_scale_y") < 41.0,
        "o Godot não mexe o colisor"
    );
    assert!(
        oraculo.contains("corpo_scale_y  t+300 origem_y=40.300 forma_y=-59.700 scale=(1.0, 1.0)")
    );

    let (mut sim, e) = cena(
        BOLA,
        [0.0, 1.0],
        pose(Vec2::new(0.0, 6.0), Vec2::new(1.0, 1.0)),
    );
    let mut b = PhysicsBridge::new();
    for tick in 1..=PASSOS {
        b.dispatch(&mut sim, true, tick);
    }
    let pousado = le(&sim, e).translation.y;
    assert!((pousado + 0.4).abs() < 0.05, "pousado em y={pousado}");
    sim.world_mut().get_mut::<Transform>(e).unwrap().scale = Vec2::new(1.0, -1.0);
    for tick in PASSOS + 1..=PASSOS * 2 {
        b.dispatch(&mut sim, true, tick);
    }
    let depois = le(&sim, e);
    assert!(
        depois.translation.y < -10.0,
        "atravessou? y={}",
        depois.translation.y
    );
    assert_eq!(
        depois.scale,
        Vec2::new(1.0, -1.0),
        "a física apagou o espelho"
    );
}

/// Espelhar parado, em repouso, é o caso do Inspector: o corpo é re-descrito.
/// *Mutação: `collider_offset` com `abs` (o `b.rest != desc` não dispara).*
#[test]
fn flipping_at_rest_redescribes_the_body() {
    let (mut sim, e) = cena(
        BOLA,
        [0.0, 1.0],
        pose(Vec2::new(0.0, 6.0), Vec2::new(1.0, 1.0)),
    );
    let mut b = PhysicsBridge::new();
    b.dispatch(&mut sim, false, 0);
    sim.world_mut().get_mut::<Transform>(e).unwrap().scale = Vec2::new(1.0, -1.0);
    b.dispatch(&mut sim, false, 0);
    for tick in 1..=PASSOS {
        b.dispatch(&mut sim, true, tick);
    }
    let y = le(&sim, e).translation.y;
    assert!((y - 1.6).abs() < 0.03, "y={y}");
}
