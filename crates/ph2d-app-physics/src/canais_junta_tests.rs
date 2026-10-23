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
    assert_eq!(todos.len(), 16 + 9);
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
    for kk in campos()
        .into_iter()
        .chain(campos_roda())
        .chain([k(j::PONTA_NO_MUNDO)])
    {
        let info = kk.info().expect("campo publicado sem vocabulário");
        let nome = ph2d_i18n::tr(info.rotulo);
        assert_ne!(nome, info.rotulo, "{kk:?}");
        assert!(nomes.insert(nome), "dois grupos com o nome {nome}");
    }
}

/// Uma roldana de corda (e a corda) para os testes da §13.
fn roldana(sim: &mut SimWorld) -> (Entity, Entity) {
    let corda = sim
        .world_mut()
        .spawn((
            Name::new("Rope"),
            PhysicsJoint::of_kind(JointKind::Pulley),
            Transform::default(),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let rope = ph2d_ecs::stable_id_of(sim.world(), corda).map_or(0, |s| s.0);
    assert_ne!(rope, 0, "controlo: a corda sem identidade não é apontável");
    let roda = sim
        .world_mut()
        .spawn(PulleyWheel {
            rope,
            ..PulleyWheel::default()
        })
        .id();
    (corda, roda)
}

/// ⭐⭐ **O tipo, o Active, o Collide e o lado B no mundo** chegam ao componente pela curva — o
/// tipo PRIMEIRO (ele re-semeia os limites), e o autorado é o de antes, nos dois factos.
#[test]
fn o_tipo_o_active_o_collide_e_o_mundo_chegam_pela_curva() {
    let mut sim = SimWorld::new();
    let antes = PhysicsJoint::default();
    let e = sim.world_mut().spawn(antes).id();
    let b = e.to_bits();
    {
        let mut w = sim
            .world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default);
        // O limite chega ANTES do tipo na fila: o tipo não pode apagá-lo.
        w.escrever(b, k(j::LIMITE_MAX), 0.3);
        w.escrever(
            b,
            k(j::TIPO),
            f32::from(crate::joint::tag_of(JointKind::Slider)),
        );
        w.escrever(b, k(j::ATIVA), 0.0);
        w.escrever(b, k(j::COLIDE_PAR), 1.0);
        w.escrever(b, k(j::PONTA_NO_MUNDO), 1.0);
    }
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &mut drive));
    let pj = *sim.world().get::<PhysicsJoint>(e).unwrap();
    assert_eq!(pj.kind, JointKind::Slider);
    assert!(
        perto(pj.limit_max, 0.3),
        "o tipo apagou o limite: {}",
        pj.limit_max
    );
    assert!(!pj.active && pj.collide_connected);
    assert!(sim.world().get::<JointWorldAnchor>(e).is_some());
    assert_eq!(
        drive.authored(b, Driver::JointParams),
        Some(Driven::JointParams(antes))
    );
    assert_eq!(
        drive.authored(b, Driver::PontaNoMundo),
        Some(Driven::PontaNoMundo(false))
    );
    publicar(&mut sim, &[b]);
    let l = sim.world().resource::<CanaisLidos>();
    assert_eq!(l.valor(b, k(j::PONTA_NO_MUNDO)), Some(1.0));
    assert_eq!(l.valor(b, k(j::ATIVA)), Some(0.0));
}

/// ⭐⭐ **A roldana pela curva**, campo a campo pelo funil do painel; a talha de Weston é o
/// marcador; mexer na rota re-abre o comprimento da corda (`anchored = false`), e a corda declara-se.
#[test]
fn a_roldana_chega_pela_curva_e_a_rota_reabre_a_corda() {
    let mut sim = SimWorld::new();
    let (corda, roda) = roldana(&mut sim);
    sim.world_mut()
        .get_mut::<PhysicsJoint>(corda)
        .unwrap()
        .anchored = true;
    let antes = *sim.world().get::<PulleyWheel>(roda).unwrap();
    let b = roda.to_bits();
    let alvos = [
        (j::RODA_RAIO, 0.8),
        (j::RODA_RAIO_SAIDA, 0.3),
        (j::RODA_WESTON, 1.0),
        (j::RODA_ORDEM, 2.0),
        (j::RODA_LADO, 2.0),
        (j::RODA_MOTOR, 1.5),
        (j::RODA_QUEBRA_LIGADA, 1.0),
        (j::RODA_QUEBRA_FORCA, 250.0),
    ];
    {
        let mut w = sim
            .world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default);
        for (c, v) in alvos {
            w.escrever(b, k(c), v);
        }
    }
    let mut drive = PreviewDrive::default();
    assert!(aplicar(&mut sim, &mut drive));
    for (c, v) in alvos {
        let lido = ler_mundo(sim.world(), roda, k(c)).expect("campo da roldana");
        assert!(perto(lido, v), "campo {c}: escreveu {v}, leu {lido}");
    }
    assert!(sim.world().get::<WestonAxle>(roda).is_some());
    assert!(
        !sim.world().get::<PhysicsJoint>(corda).unwrap().anchored,
        "a rota mudou e o comprimento da corda não re-abriu"
    );
    assert_eq!(
        drive.authored(b, Driver::Roldana),
        Some(Driven::Roldana {
            roda: antes,
            weston: false
        })
    );
    assert!(
        drive.drives(corda.to_bits()),
        "a escrita na corda não se declarou"
    );
}

/// ⭐⭐⭐ **O Auto Key vê a mão em CADA campo novo**: o painel muda o componente (pela mesma fila),
/// a família publica, e o diff dos campos pede a chave do campo mexido — e só dele.
#[test]
fn cada_campo_novo_mexido_pelo_painel_pede_a_sua_chave() {
    use ph2d_timeline::{PropKind, TimelineDoc, autokey_campos};
    let doc = TimelineDoc::new();
    let lidos = |sim: &SimWorld, b: u64| -> Vec<(FieldKey, f32)> {
        sim.world().resource::<CanaisLidos>().campos(b).collect()
    };
    let painel_junta: [(u16, JointFieldEdit); 4] = [
        (
            j::TIPO,
            JointFieldEdit::Kind(crate::joint::tag_of(JointKind::Rod)),
        ),
        (j::ATIVA, JointFieldEdit::Active(false)),
        (j::COLIDE_PAR, JointFieldEdit::CollideConnected(true)),
        (j::PONTA_NO_MUNDO, JointFieldEdit::AnchorToWorld(true)),
    ];
    for (c, ed) in painel_junta {
        let mut sim = SimWorld::new();
        let e = sim.world_mut().spawn(PhysicsJoint::default()).id();
        let b = e.to_bits();
        publicar(&mut sim, &[b]);
        let base = lidos(&sim, b).into_iter().collect();
        match ed {
            JointFieldEdit::AnchorToWorld(on) => {
                crate::joint::anchor_to_world(sim.world_mut(), e, on)
            }
            _ => {
                let pj = *sim.world().get::<PhysicsJoint>(e).unwrap();
                *sim.world_mut().get_mut::<PhysicsJoint>(e).unwrap() =
                    crate::joint::joint_with_edit(pj, ed).unwrap();
            }
        }
        publicar(&mut sim, &[b]);
        let plan = autokey_campos(&doc, b, 0.5, &lidos(&sim, b), &base, true, false);
        assert!(
            plan.keys.iter().any(|&(p, _)| p == PropKind::Field(k(c))),
            "campo {c}: o Auto Key não pediu a chave ({:?})",
            plan.keys
        );
    }
    let painel_roda = [
        (j::RODA_RAIO, WheelFieldEdit::Radius(0.7)),
        (j::RODA_RAIO_SAIDA, WheelFieldEdit::RadiusOut(0.2)),
        (j::RODA_ORDEM, WheelFieldEdit::Order(3)),
        (j::RODA_LADO, WheelFieldEdit::Wrap(1)),
        (j::RODA_MOTOR, WheelFieldEdit::MotorDegPerS(90.0)),
        (j::RODA_QUEBRA_LIGADA, WheelFieldEdit::BreakEnabled(true)),
        (j::RODA_QUEBRA_FORCA, WheelFieldEdit::BreakForce(99.0)),
    ];
    for (c, ed) in painel_roda {
        let mut sim = SimWorld::new();
        let (_, roda) = roldana(&mut sim);
        let b = roda.to_bits();
        publicar(&mut sim, &[b]);
        let base = lidos(&sim, b).into_iter().collect();
        let w = *sim.world().get::<PulleyWheel>(roda).unwrap();
        *sim.world_mut().get_mut::<PulleyWheel>(roda).unwrap() =
            crate::joint_wheel::wheel_with_edit(w, ed).unwrap();
        publicar(&mut sim, &[b]);
        let plan = autokey_campos(&doc, b, 0.5, &lidos(&sim, b), &base, true, false);
        assert_eq!(
            plan.keys.iter().map(|&(p, _)| p).collect::<Vec<_>>(),
            vec![PropKind::Field(k(c))],
            "campo {c}"
        );
    }
}

/// ⛔⛔ **Escrever o tipo que a junta JÁ TEM não a muda** (F26): o funil re-semeava as âncoras
/// (`anchored = false`) em toda escrita do tipo, a curva do tipo declarava a junta conduzida em
/// todo quadro, e o Auto Key — que salta o conduzido — deixava de a ver (a volta ao Pin no quadro
/// 24 da cena `=26` não gravava). E clicar no chip do tipo já escolhido largava as âncoras.
#[test]
fn escrever_o_mesmo_tipo_nao_mexe_na_junta() {
    let mola = PhysicsJoint {
        kind: JointKind::Spring,
        anchored: true,
        local_b: [0.3, -0.2],
        ..PhysicsJoint::default()
    };
    let tag = f32::from(crate::joint::tag_of(JointKind::Spring));
    assert_eq!(escrever(mola, k(j::TIPO), tag), mola.clamped());
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(mola.clamped()).id();
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k(j::TIPO), tag);
    let mut drive = PreviewDrive::default();
    assert!(
        !aplicar(&mut sim, &mut drive),
        "a curva parada no mesmo tipo mexeu na junta"
    );
    assert!(
        !drive.drives(e.to_bits()),
        "e declarou-a conduzida — o Auto Key deixa de a ver"
    );
}

/// O comprimento de uma POLIA é da rota (o Auto Key não o amostra); o de uma corda é do artista.
#[test]
fn so_a_corda_da_polia_tem_o_comprimento_da_rota() {
    let mut sim = SimWorld::new();
    let polia = sim
        .world_mut()
        .spawn(PhysicsJoint::of_kind(JointKind::Pulley))
        .id();
    let corda = sim
        .world_mut()
        .spawn(PhysicsJoint::of_kind(JointKind::Rope))
        .id();
    assert!(e_polia(sim.world(), polia.to_bits()));
    assert!(!e_polia(sim.world(), corda.to_bits()));
}
