//! Os gates da [`super`] — a família JUNTA: o funil do painel, e a curva que chega ao solver.

use super::*;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::JointKind;
use ph2d_preview_drive::Driver;

fn custom() -> PhysicsJoint {
    PhysicsJoint {
        kind: JointKind::Custom,
        ..PhysicsJoint::default()
    }
}

fn perto(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-5 * a.abs().max(1.0)
}

/// Todo campo publicado lê de volta o que se escreveu, pelo funil (os ângulos dão a volta pelos
/// graus do painel — o eco esconde o ulp).
#[test]
fn cada_campo_le_o_que_escreveu() {
    let base = custom();
    let todos = campos();
    assert_eq!(todos.len(), 13 + 9);
    for kk in todos {
        let discreto = kk.info().is_some_and(|i| i.discreto);
        let alvo = if discreto { 1.0 } else { 0.75 };
        let pj = escrever(base, kk, alvo);
        let lido = ler(&pj, kk).expect("campo publicado");
        assert!(perto(lido, alvo), "{kk:?}: escreveu {alvo}, leu {lido}");
    }
}

/// As escolhas arredondam e prendem-se à tabela; os números passam pelos pisos do funil.
#[test]
fn a_escrita_segue_o_dominio_do_painel() {
    let pj = escrever(custom(), k(j::MOTOR_MODO), 0.7);
    assert_eq!(pj.motor_mode, MotorMode::Position);
    let pj = escrever(custom(), k(j::MOTOR_EIXO), 7.0);
    assert_eq!(pj.custom.motor_axis, CustomAxis::Rotation);
    let pj = escrever(custom(), k(j::do_eixo(j::EIXO_MODO, 1)), 1.4);
    assert_eq!(pj.custom.axis(CustomAxis::Y).mode, AxisMode::Limited);
    let pj = escrever(custom(), k(j::RIGIDEZ), -3.0);
    assert_eq!(pj.stiffness, 0.0, "o piso do funil, o mesmo do painel");
    assert!(ler(&custom(), FieldKey::new(JUNTA, j::RIGIDEZ, 1, 0)).is_none());
}

/// ⭐ **O MESMO funil que o painel**: o limite de uma dobradiça escrito em radianos pelo canal dá
/// a junta que o Inspector dá ao escrever os graus.
#[test]
fn o_canal_e_a_edicao_do_painel_dao_a_mesma_junta() {
    let pin = PhysicsJoint::default();
    assert_eq!(pin.kind, JointKind::Pin);
    let pelo_canal = escrever(pin, k(j::LIMITE_MIN), -std::f32::consts::FRAC_PI_4);
    let pelo_painel =
        crate::joint::joint_with_edit(pin, JointFieldEdit::LimitMin(-45.0)).expect("edição");
    assert!(perto(pelo_canal.limit_min, pelo_painel.limit_min));
    let rot = j::do_eixo(j::EIXO_MAX, 2);
    let pelo_canal = escrever(custom(), k(rot), 1.0);
    let pelo_painel =
        crate::joint::joint_with_edit(custom(), JointFieldEdit::AxisMax(2, 1.0_f32.to_degrees()))
            .expect("edição");
    assert!(perto(
        pelo_canal.custom.axes[2].max,
        pelo_painel.custom.axes[2].max
    ));
}

/// ⭐⭐ **Ponta-a-ponta no documento**: a curva escreve em `CanaisEscritos` → `aplicar` muda o
/// componente, o eco fica, e o autorado (o que o undo e o save fotografam) é o de antes.
#[test]
fn a_curva_chega_ao_componente_e_nao_ao_documento() {
    let mut sim = SimWorld::new();
    let antes = custom();
    let e = sim.world_mut().spawn(antes).id();
    let b = e.to_bits();
    {
        let mut w = sim
            .world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default);
        w.escrever(b, k(j::QUEBRA_LIGADA), 1.0);
        w.escrever(b, k(j::QUEBRA_FORCA), 42.0);
    }
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &mut drive));
    let pj = *sim.world().get::<PhysicsJoint>(e).unwrap();
    assert!(pj.break_enabled);
    assert_eq!(pj.break_force, 42.0);
    assert_eq!(
        drive.authored(b, Driver::JointParams),
        Some(Driven::JointParams(antes))
    );
    publicar(&mut sim, &[b]);
    let l = sim.world().resource::<CanaisLidos>();
    assert_eq!(l.valor(b, k(j::QUEBRA_FORCA)), Some(42.0));
}

/// ⭐⭐⭐ **E chega ao SOLVER**: uma corda de 10 kg que não quebra; a curva liga a quebra a 50 N e
/// a corda parte — pela ponte, como a edição do painel (controlo: sem a curva ela segura).
#[test]
fn a_curva_que_liga_a_quebra_parte_a_corda() {
    use ph2d_core::Vec2;
    use ph2d_physics_ecs::{
        BodyKind, Collider, ColliderShape, MassOverride, PhysicsBridge, RigidBody,
    };
    let quebras = |com_curva: bool| {
        let mut sim = SimWorld::new();
        for (nome, kind, y, r) in [
            ("Hook", BodyKind::Static, 6.0, 0.05),
            ("Load", BodyKind::Dynamic, 5.0, 0.2),
        ] {
            let e = sim
                .world_mut()
                .spawn((
                    Name::new(nome),
                    RigidBody { kind },
                    Collider {
                        shape: ColliderShape::Ball { radius: r },
                        ..Collider::default()
                    },
                    Transform::from_translation(Vec2::new(0.0, y)),
                ))
                .id();
            if kind == BodyKind::Dynamic {
                sim.world_mut().entity_mut(e).insert(MassOverride(10.0));
            }
        }
        let corda = sim
            .world_mut()
            .spawn((
                Name::new("Rope"),
                PhysicsJoint {
                    body_a: ph2d_ecs::stable_name_id("Hook"),
                    body_b: ph2d_ecs::stable_name_id("Load"),
                    kind: JointKind::Rope,
                    max_length: 1.0,
                    ..PhysicsJoint::default()
                },
                Transform::from_translation(Vec2::new(0.0, 6.0)),
            ))
            .id();
        let mut bridge = PhysicsBridge::default();
        let mut drive = PreviewDrive::default();
        let mut partiu = 0;
        for t in 1..=90 {
            if com_curva && t == 10 {
                let mut w = sim
                    .world_mut()
                    .get_resource_or_insert_with(CanaisEscritos::default);
                w.escrever(corda.to_bits(), k(j::QUEBRA_LIGADA), 1.0);
                w.escrever(corda.to_bits(), k(j::QUEBRA_FORCA), 50.0);
            }
            aplicar(&mut sim, &mut drive);
            ph2d_physics_ecs::resolve_body_names(sim.world_mut());
            bridge.dispatch(&mut sim, true, t);
            partiu += crate::bridge::dispatch::break_reports(&bridge, &sim).len();
        }
        partiu
    };
    assert_eq!(quebras(false), 0, "controlo: sem a curva a corda segura");
    assert_eq!(
        quebras(true),
        1,
        "a curva ligou a quebra e o solver partiu-a"
    );
}

/// Todo campo publicado tem rótulo traduzido, e os três eixos dizem-se diferentes.
#[test]
fn todo_campo_tem_nome_no_dope_sheet() {
    let mut nomes = std::collections::BTreeSet::new();
    for kk in campos() {
        let info = kk.info().expect("campo publicado sem vocabulário");
        let nome = ph2d_i18n::tr(info.rotulo);
        assert_ne!(nome, info.rotulo, "{kk:?}");
        assert!(nomes.insert(nome), "dois grupos com o nome {nome}");
    }
}
