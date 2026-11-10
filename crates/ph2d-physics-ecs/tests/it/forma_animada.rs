//! **A forma animada a meio do Play** (GEF02 onda 7, dono 10/11: «a física tem de ficar fluida com
//! formas de colisão animadas»).
//!
//! Medido no app (cena 66, os portões nos pinos de mundo): o portão afinado repousa no batente e
//! ADORMECE; nada o toca, e o rapier só acorda o dono de um colisor mudado quando esse colisor já
//! tem contactos (`narrow_phase/pair_management.rs`, 0.35.3). A forma mudava e o corpo ficava
//! congelado.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, JointWorldAnchor, PhysicsBridge, PhysicsJoint,
};

/// Um portão (corpo com a forma FILHA, opção B) num pino de mundo na ponta esquerda, batentes
/// `±30°`: cai e repousa em −30°.
fn portao() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let g = sim
        .world_mut()
        .spawn((
            Name::new("Gate"),
            Transform::from_translation(Vec2::new(0.0, 5.0)),
        ))
        .id();
    ph2d_physics_ecs::torna_corpo(sim.world_mut(), g, BodyKind::Dynamic, [0.75, 0.09], false);
    let forma = ph2d_physics_ecs::main_shape(sim.world(), g).expect("a forma filha");
    sim.world_mut().spawn((
        Name::new("Pin"),
        PhysicsJoint {
            body_a: stable_name_id("Gate"),
            body_b: 0,
            kind: JointKind::Pin,
            limits_enabled: true,
            limit_min: (-30f32).to_radians(),
            limit_max: 30f32.to_radians(),
            ..PhysicsJoint::default()
        },
        JointWorldAnchor,
        Transform::from_translation(Vec2::new(-0.75, 5.0)),
    ));
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    (sim, g, forma)
}

fn angulo(bridge: &PhysicsBridge, g: Entity) -> f32 {
    bridge.body_pose(g).expect("corpo").2.to_degrees()
}

/// ⭐ O portão dorme no batente; a forma passa para o OUTRO lado do pino (o peso troca de lado) e
/// ele tem de acordar e virar até ao outro batente (+30°). Controlo: sem mexer na forma ele fica.
#[test]
fn a_forma_que_muda_acorda_o_corpo_que_dorme() {
    let corre = |mexe: bool| {
        let (mut sim, g, forma) = portao();
        let mut bridge = PhysicsBridge::new();
        for t in 1..=300 {
            bridge.dispatch(&mut sim, true, t);
        }
        assert_eq!(
            bridge.body_sleeping(g),
            Some(true),
            "repousou no batente e dormiu"
        );
        assert!((angulo(&bridge, g) + 30.0).abs() < 0.5);
        if mexe {
            let c = &mut *sim.world_mut().get_mut::<Collider>(forma).expect("forma");
            c.shape = ColliderShape::Cuboid {
                half_x: 0.75,
                half_y: 0.09,
            };
            c.offset = [-2.25, 0.0];
        }
        for t in 301..=480 {
            bridge.dispatch(&mut sim, true, t);
        }
        angulo(&bridge, g)
    };
    assert!(
        (corre(false) + 30.0).abs() < 0.5,
        "controlo: parado fica no batente"
    );
    let a = corre(true);
    assert!(
        (a - 30.0).abs() < 1.0,
        "acordou e virou até ao outro batente: {a}"
    );
}

/// Uma mesa: o corpo com a forma principal filha (o tampo) e uma PEÇA, a perna (2.ª forma filha).
fn mesa() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, -0.5)),
        ph2d_physics_ecs::RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 5.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
    ));
    let m = sim
        .world_mut()
        .spawn((
            Name::new("Table"),
            Transform::from_translation(Vec2::new(0.0, 1.0)),
        ))
        .id();
    ph2d_physics_ecs::torna_corpo(sim.world_mut(), m, BodyKind::Dynamic, [0.6, 0.1], false);
    let perna = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.0, -0.5)),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.1,
                    half_y: 0.4,
                },
                ..Collider::default()
            },
            ph2d_ecs::ChildOf(m),
        ))
        .id();
    (sim, m, perna)
}

/// ⭐ A PEÇA animada reafina-se no lugar: o mesmo colisor, e a memória de voltar atrás (o ring)
/// continua a crescer. Antes: arrancada e pendurada de novo a cada quadro, o ring limpo a cada um.
#[test]
fn a_peca_animada_nao_limpa_o_ring() {
    let (mut sim, m, perna) = mesa();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60 {
        bridge.dispatch(&mut sim, true, t);
    }
    let h = bridge.part_handle(perna).expect("a perna pendurada");
    let antes = bridge.ring_stats().0;
    let massa = bridge.body_mass(m).expect("a mesa");
    assert!(antes > 0);
    for t in 61..=120u64 {
        let f = (t - 60) as f32 / 60.0;
        sim.world_mut()
            .get_mut::<Collider>(perna)
            .expect("perna")
            .shape = ColliderShape::Cuboid {
            half_x: 0.1 + 0.1 * f,
            half_y: 0.4,
        };
        bridge.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        bridge.part_handle(perna),
        Some(h),
        "o mesmo colisor, reafinado"
    );
    assert!(
        bridge.ring_stats().0 > antes,
        "o ring cresceu: {} → {}",
        antes,
        bridge.ring_stats().0
    );
    let depois = bridge.body_mass(m).expect("a mesa");
    assert!(
        depois > massa + 0.1,
        "o solver tem a perna nova: {massa} → {depois} kg"
    );
}

/// A cena de um caixote no chão cuja FORMA tem a escala animada (a curva escreve o `Transform` da
/// filha a cada tique, como o `TimelineScene` do app): altura 1 → 2 entre os tiques 60 e 180.
struct EscalaAnimada(Entity);

impl ph2d_physics_ecs::SceneAtTick for EscalaAnimada {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let f = ((tick as f32 - 60.0) / 120.0).clamp(0.0, 1.0);
        sim.world_mut()
            .get_mut::<Transform>(self.0)
            .expect("a forma")
            .scale
            .y = 1.0 + f;
        true
    }
}

fn caixote() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, -0.5)),
        ph2d_physics_ecs::RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 5.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
    ));
    let c = sim
        .world_mut()
        .spawn((
            Name::new("Crate"),
            Transform::from_translation(Vec2::new(0.0, 0.5)),
        ))
        .id();
    ph2d_physics_ecs::torna_corpo(sim.world_mut(), c, BodyKind::Dynamic, [0.5, 0.5], false);
    let forma = ph2d_physics_ecs::main_shape(sim.world(), c).expect("a forma filha");
    (sim, c, forma)
}

/// ⭐ **Voltar atrás a um tique que não é ponto salvo dá a pose do Play** — o replay corre com as
/// formas daquele tique, não com as do checkpoint. Antes: altura 1,9 m do caixote a meio da curva
/// replayada com a forma do checkpoint.
#[test]
fn o_scrub_fora_do_passo_do_ring_da_a_forma_do_play() {
    let (mut sim, c, forma) = caixote();
    let mut cena = EscalaAnimada(forma);
    let mut bridge = PhysicsBridge::new();
    let mut play = Vec::new();
    for t in 1..=200 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut cena);
        play.push(bridge.body_pose(c).expect("caixote"));
    }
    for alvo in [157u64, 123, 99] {
        bridge.dispatch_with_scene(&mut sim, false, alvo, &mut cena);
        let scrub = bridge.body_pose(c).expect("caixote");
        assert_eq!(
            scrub,
            play[alvo as usize - 1],
            "o salto a {alvo} dá a pose do Play"
        );
    }
}

/// ⭐ **Voltar a zero depois da curva não recria o corpo** — nem com o tique 0 já fora da memória (um
/// Play longo; `forget_checkpoints`): o salto veste as formas do tique 0 no lugar (`drive_shapes`),
/// e o quadro 0 já não vê diferença que o obrigue a arrancar o colisor.
#[test]
fn voltar_a_zero_depois_da_curva_nao_recria_o_corpo() {
    let (mut sim, c, forma) = caixote();
    let mut cena = EscalaAnimada(forma);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=200 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut cena);
    }
    bridge.forget_checkpoints();
    bridge.dispatch_with_scene(&mut sim, false, 0, &mut cena);
    let h = bridge.body_handle_bits(c);
    for t in 0..=30 {
        bridge.dispatch_with_scene(&mut sim, t > 0, t, &mut cena);
        assert_eq!(bridge.body_handle_bits(c), h, "o mesmo corpo no tique {t}");
    }
}

/// Um quadro que deve VÁRIOS tiques (máquina lenta) dá a pose de tique a tique: cada tique corre
/// com a forma dele, não com a do quadro.
#[test]
fn o_play_de_varios_tiques_por_quadro_da_a_forma_de_cada_tique() {
    let corre = |passo: u64| {
        let (mut sim, c, forma) = caixote();
        let mut cena = EscalaAnimada(forma);
        let mut bridge = PhysicsBridge::new();
        let mut t = 0;
        while t < 150 {
            t += passo;
            bridge.dispatch_with_scene(&mut sim, true, t, &mut cena);
        }
        bridge.body_pose(c).expect("caixote")
    };
    assert_eq!(corre(3), corre(1));
}

/// A forma que SALTA no tique logo a seguir a um ponto salvo (escala 1 até 150, 2 desde 151): o
/// replay de 150 a 155 tem de vestir a forma nova já no 151 — o memo diz «já está», o checkpoint diz
/// o contrário, e só o empurrão FORÇADO depois do semeio acerta.
#[test]
fn o_replay_veste_a_forma_que_saltou_depois_do_ponto_salvo() {
    struct Salto(Entity);
    impl ph2d_physics_ecs::SceneAtTick for Salto {
        fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
            let w = sim.world_mut();
            w.get_mut::<Transform>(self.0).expect("forma").scale.y =
                if tick > 150 { 2.0 } else { 1.0 };
            true
        }
    }
    let (mut sim, c, forma) = caixote();
    let mut cena = Salto(forma);
    let mut bridge = PhysicsBridge::new();
    let mut play = Vec::new();
    for t in 1..=170 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut cena);
        play.push(bridge.body_pose(c).expect("caixote"));
    }
    bridge.dispatch_with_scene(&mut sim, false, 155, &mut cena);
    assert_eq!(bridge.body_pose(c).expect("caixote"), play[154]);
}

/// A PEÇA que muda acorda o dono adormecido: o portão do pino de mundo leva o peso numa peça; ela
/// passa para o outro lado do pino e ele tem de virar até ao outro batente.
#[test]
fn a_peca_que_muda_acorda_o_dono_que_dorme() {
    let (mut sim, g, _) = portao();
    let peso = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(0.5, 0.0)),
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                ..Collider::default()
            },
            ph2d_ecs::ChildOf(g),
        ))
        .id();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=300 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert_eq!(bridge.body_sleeping(g), Some(true), "dormiu no batente");
    sim.world_mut()
        .get_mut::<Transform>(peso)
        .expect("peso")
        .translation
        .x = -4.0;
    for t in 301..=480 {
        bridge.dispatch(&mut sim, true, t);
    }
    let a = angulo(&bridge, g);
    assert!((a - 30.0).abs() < 1.0, "acordou e virou: {a}");
}

/// A cena da PEÇA com a escala animada (a perna da mesa cresce de 0,8 m a 1,6 m de altura).
struct PernaAnimada(Entity);

impl ph2d_physics_ecs::SceneAtTick for PernaAnimada {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        let f = ((tick as f32 - 30.0) / 120.0).clamp(0.0, 1.0);
        sim.world_mut()
            .get_mut::<Transform>(self.0)
            .expect("a perna")
            .scale
            .y = 1.0 + f;
        true
    }
}

/// Voltar atrás com uma PEÇA animada dá a pose do Play (o replay veste a peça de cada tique).
#[test]
fn o_scrub_da_peca_animada_da_a_pose_do_play() {
    let (mut sim, m, perna) = mesa();
    let mut cena = PernaAnimada(perna);
    let mut bridge = PhysicsBridge::new();
    let mut play = Vec::new();
    for t in 1..=200 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut cena);
        play.push(bridge.body_pose(m).expect("mesa"));
    }
    for alvo in [157u64, 93] {
        bridge.dispatch_with_scene(&mut sim, false, alvo, &mut cena);
        assert_eq!(
            bridge.body_pose(m).expect("mesa"),
            play[alvo as usize - 1],
            "o salto a {alvo}"
        );
    }
}

/// Parado EXACTAMENTE num ponto salvo depois de o artista mudar a curva: o checkpoint guardou a
/// forma velha, e o solver tem de ficar com a da curva nova.
#[test]
fn parar_num_ponto_salvo_depois_de_mudar_a_curva_da_a_forma_nova() {
    struct Fixa(Entity, f32);
    impl ph2d_physics_ecs::SceneAtTick for Fixa {
        fn put(&mut self, sim: &mut SimWorld, _: u64) -> bool {
            sim.world_mut()
                .get_mut::<Transform>(self.0)
                .expect("forma")
                .scale
                .y = self.1;
            true
        }
    }
    let (mut sim, c, forma) = caixote();
    let mut bridge = PhysicsBridge::new();
    for t in 1..=170 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut Fixa(forma, 1.0));
    }
    bridge.dispatch_with_scene(&mut sim, false, 150, &mut Fixa(forma, 2.0));
    let (mn, mx) = bridge.body_aabb(c).expect("caixote");
    assert!(
        (mx[1] - mn[1] - 2.0).abs() < 1e-3,
        "a altura da curva nova, 2 m: {}",
        mx[1] - mn[1]
    );
}

/// A PEÇA que salta logo a seguir a um ponto salvo (a perna 1× até 150, 2× desde 151): o replay de
/// 150 a 155 veste-a já no 151, porque o memo das peças volta com o checkpoint.
#[test]
fn o_replay_veste_a_peca_que_saltou_depois_do_ponto_salvo() {
    struct Salto(Entity);
    impl ph2d_physics_ecs::SceneAtTick for Salto {
        fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
            let w = sim.world_mut();
            w.get_mut::<Transform>(self.0).expect("perna").scale.y =
                if tick > 150 { 2.0 } else { 1.0 };
            true
        }
    }
    let (mut sim, m, perna) = mesa();
    let mut cena = Salto(perna);
    let mut bridge = PhysicsBridge::new();
    let mut play = Vec::new();
    for t in 1..=170 {
        bridge.dispatch_with_scene(&mut sim, true, t, &mut cena);
        play.push(bridge.body_pose(m).expect("mesa"));
    }
    bridge.dispatch_with_scene(&mut sim, false, 155, &mut cena);
    assert_eq!(bridge.body_pose(m).expect("mesa"), play[154]);
}
