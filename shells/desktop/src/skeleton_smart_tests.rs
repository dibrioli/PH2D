//! Os gates dos **OSSOS INTELIGENTES**.
//!
//! A LEI do mapeamento (ângulo → tempo de acção) está gateada em `ph2d-skeleton`. Aqui mede-se o que
//! só existe com um mundo ECS **e** um documento de timeline: que girar o controlo move o que a
//! acção anima, que o que ele escreve é **pré-visualização**, e que um controlo sem acção é inerte.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, RootOrder};
use ph2d_timeline::PropKind;

/// A faixa do controlo na cena de teste: de `0` a um quarto de volta.
const ATE: f64 = std::f64::consts::FRAC_PI_2;

/// Um mundo com um osso de CONTROLO e um objecto que a acção desloca em X, de `0` a `10`.
///
/// ⚠️ O clip é montado pela porta do PRODUTO (`add_clip` + `insert_key`), que é quem cria a binding
/// — uma fixtura que montasse as tracks à mão mediria outro programa.
fn cena() -> (SimWorld, TimelineDoc, Entity, Entity) {
    let mut sim = SimWorld::default();
    let controlo = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Control"),
            RootOrder(0),
            ph2d_skeleton_ecs::Bone {
                length: 10.0,
                strength: 1.0,
            },
        ))
        .id();
    let movido = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Driven"), RootOrder(1)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());

    let mut doc = TimelineDoc::default();
    let i = doc.add_clip("Correction".into());
    doc.set_active(i);
    for (t, v) in [(0.0, 0.0_f32), (2.0, 10.0)] {
        doc.insert_key(
            movido.to_bits(),
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    // A binding nasce sem `entity` resolvida (ela é runtime, reconstruída do `wire_id` no load).
    for b in doc.bindings_mut() {
        b.entity = movido.to_bits();
        b.missing = false;
    }
    (sim, doc, controlo, movido)
}

fn liga(sim: &mut SimWorld, e: Entity, clip: &str) {
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_skeleton_ecs::SmartBone {
            clip: clip.into(),
            from: 0.0,
            to: ATE,
        });
}

fn x(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).expect("tem pose").translation.x
}

/// ⭐⭐⭐ **GIRAR O CONTROLO PERCORRE A ACÇÃO** — a razão de existir da coisa.
#[test]
fn turning_the_control_bone_runs_the_action() {
    let (mut sim, doc, controlo, movido) = cena();
    liga(&mut sim, controlo, "Correction");
    let mut pv = PreviewDrive::default();

    // No princípio da faixa, a acção está no princípio.
    drive(&mut sim, &doc, &mut pv);
    assert!((x(&sim, movido) - 0.0).abs() < 1e-4, "no princípio: {}", x(&sim, movido));

    // A meio da faixa, a meio da acção.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
        {
            t.rotation = (ATE / 2.0) as f32;
        }
    }
    drive(&mut sim, &doc, &mut pv);
    assert!((x(&sim, movido) - 5.0).abs() < 1e-3, "a meio: {}", x(&sim, movido));

    // E no fim, no fim.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "idem")]
        {
            t.rotation = ATE as f32;
        }
    }
    drive(&mut sim, &doc, &mut pv);
    assert!((x(&sim, movido) - 10.0).abs() < 1e-3, "no fim: {}", x(&sim, movido));
}

/// ⭐⭐⭐ **O QUE A ACÇÃO ESCREVE É PRÉ-VISUALIZAÇÃO** — vê-se, não se guarda nem se desfaz.
///
/// ⚠️ Sem isto, cada clique com um controlo fora do repouso empilharia um passo de undo cujo
/// conteúdo é *«a acção correu»*. É a mesma lei da âncora de IK, e o ledger é o MESMO.
#[test]
fn what_the_action_writes_is_preview_not_document() {
    let (mut sim, doc, controlo, movido) = cena();
    liga(&mut sim, controlo, "Correction");
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
        #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
        {
            t.rotation = ATE as f32;
        }
    }
    let mut pv = PreviewDrive::default();
    drive(&mut sim, &doc, &mut pv);
    assert!((x(&sim, movido) - 10.0).abs() < 1e-3, "a acção correu");
    // ⭐ O ledger tem de saber repor o AUTORADO (a pose de repouso), que é o que a fotografia usa.
    assert!(
        !pv.is_empty(),
        "o ledger ficou vazio — o que a acção escreveu viraria documento"
    );
}

/// ⛔ **UM CONTROLO SEM ACÇÃO É INERTE, e um que nomeia uma acção que não existe também.**
///
/// ⚠️ A segunda metade é o caso real: o artista renomeia ou apaga o clip. Inventar um índice poria
/// o osso a percorrer a animação do vizinho — **em silêncio**, que é o pior modo de falha.
#[test]
fn a_control_without_an_action_does_nothing_at_all() {
    for nome in ["", "Nao Existe"] {
        let (mut sim, doc, controlo, movido) = cena();
        liga(&mut sim, controlo, nome);
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(controlo) {
            #[expect(clippy::cast_possible_truncation, reason = "o Transform da casa é f32")]
            {
                t.rotation = ATE as f32;
            }
        }
        let mut pv = PreviewDrive::default();
        let feitas = drive(&mut sim, &doc, &mut pv);
        assert_eq!(feitas, 0, "com clip {nome:?} escreveu-se alguma coisa");
        assert!(
            (x(&sim, movido) - 0.0).abs() < 1e-9,
            "com clip {nome:?} o objecto mexeu-se para {}",
            x(&sim, movido)
        );
    }
}

/// ⭐ **SEM CONTROLO NENHUM, O PASSE SAI CEDO** — a guarda que impede toda cena de pagar uma
/// varredura do mundo por quadro. É a mesma lei que o passe da âncora já segue.
#[test]
fn a_scene_without_smart_bones_pays_nothing() {
    let (mut sim, doc, _, movido) = cena();
    let mut pv = PreviewDrive::default();
    assert_eq!(drive(&mut sim, &doc, &mut pv), 0);
    assert!(pv.is_empty(), "sem controlo nenhum o ledger tem de ficar vazio");
    assert!((x(&sim, movido) - 0.0).abs() < 1e-9);
}
