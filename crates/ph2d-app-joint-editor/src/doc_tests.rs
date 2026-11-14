use super::*;
use crate::model::{AssemblyRow, JointRow, Part};
use ph2d_joint_templates::by_name;
use ph2d_physics_ecs::PhysicsJoint;

fn car_row(linked: u32) -> AssemblyRow {
    let t = by_name(JointKind::Wheel, "Car").expect("car");
    let part = |role, sid, name: &str, x| Part {
        role,
        sid,
        bits: sid + 100,
        name: name.to_owned(),
        at: [x, 0.3],
    };
    let joint = |rule, bits| {
        let mut j = PhysicsJoint::of_kind(JointKind::Wheel);
        j.role = rule;
        j.assembly = 9;
        JointRow {
            bits,
            rule,
            joint: j,
            info: None,
        }
    };
    AssemblyRow {
        sid: 9,
        bits: 900,
        name: "Car 1".to_owned(),
        template: t,
        parts: vec![
            part(0, 1, "Car Body", 0.0),
            part(1, 2, "Back", -0.7),
            part(2, 3, "Front", 0.7),
        ],
        joints: vec![joint(0, 50), joint(1, 51)],
        linked,
    }
}

fn doc_with(row: AssemblyRow) -> JointEditorDoc {
    let mut d = JointEditorDoc::default();
    d.set_snapshot(Snapshot {
        assemblies: vec![row],
        bodies: vec![
            "Back".into(),
            "Car Body".into(),
            "Crate".into(),
            "Front".into(),
        ],
    });
    d.open_on(9);
    d
}

/// ⭐ «Both wheels · Rear wheel · Front wheel»: o grupo vem primeiro e liga «Same as».
#[test]
fn o_carro_oferece_as_duas_rodas_e_cada_uma() {
    let t = by_name(JointKind::Wheel, "Car").expect("car");
    let o = scope_options(t);
    let labels: Vec<&str> = o.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(labels.len(), 3);
    assert_eq!(o[0].rules, vec![0, 1]);
    assert_eq!(o[0].group, Some(0));
    assert_eq!(&labels[1..], ["Rear wheel", "Front wheel"]);
}

/// ⭐ **O alcance lê o «Same as» da montagem**: ligado = as duas juntas; desligado = a escolhida.
///
/// Mutação que deve sangrar: devolver sempre a opção escolhida em [`JointEditorDoc::scope_index`].
#[test]
fn o_alcance_segue_o_same_as_da_montagem() {
    let mut d = doc_with(car_row(u32::MAX));
    d.scope.insert(9, 2);
    let t: Vec<u64> = d.targets().iter().map(|j| j.bits).collect();
    assert_eq!(
        t,
        [50, 51],
        "ligado: as duas rodas, seja qual for a escolha"
    );
    let mut d = doc_with(car_row(0));
    d.scope.insert(9, 2);
    let t: Vec<u64> = d.targets().iter().map(|j| j.bits).collect();
    assert_eq!(t, [51], "desligado: só a da frente");
}

/// Escolher uma roda só desliga o grupo (um pedido); escolher o grupo volta a ligá-lo.
#[test]
fn escolher_uma_roda_desliga_o_same_as() {
    let mut d = doc_with(car_row(u32::MAX));
    d.set_scope(1);
    assert_eq!(
        d.take_pending(),
        [Edit::Linked {
            joint: 50,
            on: false
        }]
    );
    let mut d = doc_with(car_row(0));
    d.set_scope(0);
    assert_eq!(
        d.take_pending(),
        [Edit::Linked {
            joint: 50,
            on: true
        }]
    );
}

/// ⭐ **A queda diz o que vai acontecer**, e recusa o que não pode: um objeto sem corpo, um papel já
/// ocupado, uma peça que já está noutro papel.
#[test]
fn a_queda_diz_antes_o_que_faz_e_recusa_o_que_nao_pode() {
    let d = doc_with(car_row(u32::MAX));
    assert!(
        !d.verdict(2, &["Crate".into()]).0,
        "o papel da frente já tem a Front"
    );
    assert!(!d.verdict(0, &["Ghost".into()]).0, "sem corpo");
    assert!(
        !d.verdict(1, &["Front".into()]).0,
        "a Front já é a roda da frente"
    );
    let mut fresh = JointEditorDoc::default();
    fresh.set_snapshot(Snapshot {
        assemblies: vec![],
        bodies: vec!["Crate".into()],
    });
    fresh.choose_template(1);
    let (ok, line) = fresh.verdict(2, &["Crate".into()]);
    assert!(ok, "{line}");
    assert!(
        line.contains("Crate") && line.contains("Front wheel"),
        "{line}"
    );
    assert!(fresh.drop_into(2, &["Crate".into()]));
    assert_eq!(
        fresh.take_pending(),
        [Edit::Place {
            assembly: 0,
            template: 1,
            role: 2,
            bodies: vec!["Crate".into()],
        }]
    );
}

/// O Drive «Front» liga o motor da frente e desliga o de trás; a da frente herda a velocidade de fábrica.
#[test]
fn o_drive_da_frente_troca_o_motor_de_roda() {
    let mut row = car_row(u32::MAX);
    row.joints[0].joint.motor_enabled = true;
    let mut d = doc_with(row);
    assert_eq!(d.drive(), Some(Drive::Rule(0)));
    d.set_drive(Drive::Rule(1));
    let p = d.take_pending();
    assert!(p.contains(&Edit::Joints {
        joints: vec![50],
        edit: JointFieldEdit::MotorEnabled(false)
    }));
    assert!(p.contains(&Edit::Joints {
        joints: vec![51],
        edit: JointFieldEdit::MotorEnabled(true)
    }));
    assert!(p.iter().any(|e| matches!(e, Edit::Joints { joints, edit: JointFieldEdit::MotorSpeed(_) } if joints == &[51])));
}

/// O entre-eixos é medido nas peças da cena (1,4 m entre −0,7 e 0,7).
#[test]
fn o_entre_eixos_e_medido_na_cena() {
    let d = doc_with(car_row(u32::MAX));
    assert!((d.wheelbase().expect("duas rodas") - 1.4).abs() < 1e-5);
}

/// ⭐ **Um papel em fileira leva todos os objetos largados; um papel único só o primeiro.**
///
/// Mutações que devem sangrar: `drop_into` levar sempre um corpo / sempre todos / inverter `many`.
#[test]
fn a_queda_na_fileira_leva_todos_e_no_papel_unico_so_o_primeiro() {
    let mut d = JointEditorDoc::default();
    d.set_snapshot(Snapshot {
        assemblies: vec![],
        bodies: vec!["Hook".into(), "A".into(), "B".into(), "C".into()],
    });
    d.choose_template(7);
    let names: Vec<String> = ["A", "B", "C"].map(String::from).into();
    assert!(d.drop_into(1, &names), "o Link é uma fileira");
    assert_eq!(
        d.take_pending(),
        [Edit::Place {
            assembly: 0,
            template: 7,
            role: 1,
            bodies: names.clone()
        }]
    );
    assert!(d.drop_into(0, &names), "o Hook é um só");
    assert_eq!(
        d.take_pending(),
        [Edit::Place {
            assembly: 0,
            template: 7,
            role: 0,
            bodies: vec!["A".into()]
        }]
    );
}

/// ⭐ **A queda recusada não deixa pedido nenhum no quadro.**
///
/// Mutação que deve sangrar: `drop_into` ignorar o veredito de [`JointEditorDoc::verdict`].
#[test]
fn a_queda_recusada_nao_deixa_pedido() {
    let mut d = doc_with(car_row(u32::MAX));
    assert!(
        !d.drop_into(2, &["Crate".into()]),
        "o papel da frente já tem a Front"
    );
    assert!(!d.drop_into(0, &["Ghost".into()]), "sem corpo");
    assert!(!d.drop_into(9, &["Crate".into()]), "papel que não existe");
    assert!(d.take_pending().is_empty());
}

/// O alcance escolhido fora das opções cai na última (nunca fora do vetor), com o grupo desligado.
///
/// Mutação que deve sangrar: o recurso de `scope_index` ser sempre a primeira opção.
#[test]
fn o_alcance_fora_das_opcoes_fica_na_ultima() {
    let mut d = doc_with(car_row(0));
    d.scope.insert(9, 7);
    let t: Vec<u64> = d.targets().iter().map(|j| j.bits).collect();
    assert_eq!(t, [51], "a última opção é a roda da frente");
}

/// A queda numa montagem aberta leva o `sid` dela (não cria uma nova).
///
/// Mutação que deve sangrar: `drop_into` pôr sempre `assembly: 0`.
#[test]
fn a_queda_na_montagem_aberta_leva_o_sid_dela() {
    let mut row = car_row(u32::MAX);
    row.parts.retain(|p| p.role != 2);
    let mut d = doc_with(row);
    assert!(d.drop_into(2, &["Crate".into()]));
    assert_eq!(
        d.take_pending(),
        [Edit::Place {
            assembly: 9,
            template: d.template().id.0,
            role: 2,
            bodies: vec!["Crate".into()]
        }]
    );
}
