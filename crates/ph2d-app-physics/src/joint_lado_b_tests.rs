//! F27 Bloco A — o lado B de uma junta (mundo ↔ corpo) e o motor com faixa própria, pelos DOIS
//! escritores: o painel (`anchor_to_world` / `set_joint_body`) e a curva (`canais_junta::aplicar`).

use crate::canais_junta::aplicar;
use crate::joint::{anchor_to_world, set_joint_body};
use ph2d_core::Vec2;
use ph2d_ecs::canal::familia::JUNTA;
use ph2d_ecs::canal_junta as j;
use ph2d_ecs::{CanaisEscritos, Entity, FieldKey, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, CustomAxis, JointKind, JointWorldAnchor, PhysicsBridge,
    PhysicsJoint, RigidBody,
};
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::{PropKind, TimelineDoc};

const PIVO: [f32; 2] = [1.0, 5.0];
const CENTRO_B: [f32; 2] = [3.0, 5.0];

fn corpo(sim: &mut SimWorld, nome: &str, p: [f32; 2]) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(p[0], p[1])),
        ))
        .id()
}

/// Dois corpos e um Pin no `PIVO`; `com_b` nomeia o corpo B, `mundo` põe o marcador.
fn cena(com_b: bool, mundo: bool) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let a = corpo(&mut sim, "A", [0.0, 5.0]);
    let b = corpo(&mut sim, "B", CENTRO_B);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let sid = |e| ph2d_ecs::stable_id_of(sim.world(), e).expect("stable id").0;
    let pj = PhysicsJoint {
        kind: JointKind::Pin,
        body_a: sid(a),
        body_b: if com_b { sid(b) } else { 0 },
        ..PhysicsJoint::default()
    };
    let xf = Transform::from_translation(Vec2::new(PIVO[0], PIVO[1]));
    let junta = sim.world_mut().spawn((pj, xf)).id();
    if mundo {
        sim.world_mut().entity_mut(junta).insert(JointWorldAnchor);
    }
    (sim, junta, b)
}

fn semear(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> PhysicsJoint {
    bridge.dispatch(sim, false, 0);
    bridge.dispatch(sim, false, 0);
    let e = bridge_e(sim);
    *sim.world().get::<PhysicsJoint>(e).expect("junta")
}

fn bridge_e(sim: &mut SimWorld) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &PhysicsJoint)>();
    q.iter(sim.world()).next().expect("junta").0
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
}

fn curva_ponta(sim: &mut SimWorld, junta: Entity, v: f32) {
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(
            junta.to_bits(),
            FieldKey::new(JUNTA, j::PONTA_NO_MUNDO, 0, 0),
            v,
        );
}

/// ⛔ **Soltar do mundo um pino SEM corpo B é recusado**, pelo painel e pela curva — a junta
/// ficaria dormente e o corpo cairia. Controlo: com corpo B o mesmo pedido passa.
#[test]
fn soltar_do_mundo_sem_corpo_b_e_recusado_pelos_dois_escritores() {
    let (mut sim, junta, _) = cena(false, true);
    assert!(
        !anchor_to_world(sim.world_mut(), junta, false),
        "o painel soltou um pino sem corpo B"
    );
    assert!(sim.world().get::<JointWorldAnchor>(junta).is_some());
    curva_ponta(&mut sim, junta, 0.0);
    aplicar(
        &mut sim,
        &mut PreviewDrive::default(),
        &TimelineDoc::default(),
    );
    assert!(
        sim.world().get::<JointWorldAnchor>(junta).is_some(),
        "a curva soltou um pino sem corpo B — a junta ficou dormente"
    );

    let (mut sim, junta, _) = cena(true, true);
    curva_ponta(&mut sim, junta, 0.0);
    aplicar(
        &mut sim,
        &mut PreviewDrive::default(),
        &TimelineDoc::default(),
    );
    assert!(
        sim.world().get::<JointWorldAnchor>(junta).is_none(),
        "controlo: com corpo B a curva tem de soltar"
    );
}

/// **Escolher o corpo B tira o pino de mundo** — é o fim do clique em «Body» recusado acima.
#[test]
fn escolher_o_corpo_b_tira_o_pino_de_mundo() {
    let (mut sim, junta, b) = cena(false, true);
    assert!(set_joint_body(&mut sim, junta.to_bits(), true, b));
    assert!(sim.world().get::<JointWorldAnchor>(junta).is_none());
    let pj = sim.world().get::<PhysicsJoint>(junta).expect("junta");
    assert!(pj.names_two_bodies());
}

/// ⛔⛔ **Desligar o pino não pode atirar a ponta para o CENTRO do corpo B**: a âncora local de B
/// era `[0,0]` em relação ao prego, e passava a ler-se em relação ao corpo. E o inverso: ligar o
/// pino não pode deixar o prego deslocado pela âncora antiga de B.
#[test]
fn trocar_o_lado_b_re_deriva_a_ancora_de_b() {
    let (mut sim, junta, _) = cena(true, true);
    let mut bridge = PhysicsBridge::new();
    let antes = semear(&mut sim, &mut bridge);
    assert!(
        antes.anchored && perto(antes.local_b, [0.0, 0.0]),
        "pré: {antes:?}"
    );
    assert!(anchor_to_world(sim.world_mut(), junta, false));
    let depois = semear(&mut sim, &mut bridge);
    let quer = [PIVO[0] - CENTRO_B[0], PIVO[1] - CENTRO_B[1]];
    assert!(
        depois.anchored && perto(depois.local_b, quer),
        "a ponta saltou: local_b {:?}, o pivô no corpo B é {quer:?}",
        depois.local_b
    );

    assert!(anchor_to_world(sim.world_mut(), junta, true));
    let de_novo = semear(&mut sim, &mut bridge);
    assert!(
        de_novo.anchored && perto(de_novo.local_b, [0.0, 0.0]),
        "o prego ficou deslocado pela âncora antiga de B: {:?}",
        de_novo.local_b
    );
}

fn motor(sim: &mut SimWorld, kk: u16, v: f32, doc: &TimelineDoc) -> PhysicsJoint {
    let e = bridge_e(sim);
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), FieldKey::new(JUNTA, kk, 0, 0), v);
    aplicar(sim, &mut PreviewDrive::default(), doc);
    *sim.world().get::<PhysicsJoint>(e).expect("junta")
}

/// ⛔⛔ **A troca de eixo ou de tipo pela curva não apaga a chave do motor do mesmo quadro**: a
/// timeline escreve o alvo e a velocidade ANTES do funil, e o re-semear zerava-os. Controlo: sem
/// faixa o re-semear continua (a unidade mudou e o número antigo mentiria).
#[test]
fn a_faixa_do_motor_ganha_do_re_semear_da_curva() {
    for com_faixa in [true, false] {
        let mut sim = SimWorld::new();
        let pj = PhysicsJoint {
            kind: JointKind::Custom,
            motor_target: 1.25,
            ..PhysicsJoint::default()
        };
        let mut pj = pj;
        pj.custom.motor_axis = CustomAxis::Rotation;
        let e = sim.world_mut().spawn(pj).id();
        let mut doc = TimelineDoc::default();
        if com_faixa {
            doc.bind(e.to_bits(), PropKind::JointMotorTarget);
        }
        let x = f32::from(CustomAxis::X.tag());
        let depois = motor(&mut sim, j::MOTOR_EIXO, x, &doc);
        assert_eq!(
            depois.custom.motor_axis,
            CustomAxis::X,
            "o eixo tinha de mudar"
        );
        let quer = if com_faixa { 1.25 } else { 0.0 };
        assert_eq!(depois.motor_target, quer, "com faixa = {com_faixa}");
    }

    for com_faixa in [true, false] {
        let mut sim = SimWorld::new();
        let pj = PhysicsJoint {
            kind: JointKind::Pin,
            motor_target: 0.5,
            motor_speed: 3.0,
            ..PhysicsJoint::default()
        };
        let e = sim.world_mut().spawn(pj).id();
        let mut doc = TimelineDoc::default();
        if com_faixa {
            doc.bind(e.to_bits(), PropKind::JointMotorTarget);
            doc.bind(e.to_bits(), PropKind::JointMotorSpeed);
        }
        let slider = f32::from(crate::joint::tag_of(JointKind::Slider));
        let depois = motor(&mut sim, j::TIPO, slider, &doc);
        assert_eq!(depois.kind, JointKind::Slider);
        if com_faixa {
            assert_eq!([depois.motor_target, depois.motor_speed], [0.5, 3.0]);
        } else {
            assert_eq!(
                depois.motor_target, 0.0,
                "controlo: sem faixa o alvo re-semeia"
            );
            assert_ne!(
                depois.motor_speed, 3.0,
                "controlo: sem faixa a velocidade re-semeia"
            );
        }
    }
}
