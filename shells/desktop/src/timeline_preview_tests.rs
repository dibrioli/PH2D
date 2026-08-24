//! **O TERCEIRO membro da família pré-visualização↔documento** — a timeline.
//!
//! ⚠️ A auditoria [21 §4](../../../docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md)
//! curou dois casos (a §11 Animation e o solver) e **mediu** um terceiro: enquanto o playhead toca,
//! a timeline escreve `Transform` a partir das curvas, e um clique naquele quadro empilha um passo
//! de undo cujo conteúdo é só a pose que o documento já sabe calcular.
//!
//! ⚠️ **E a nota que a deixou de fora estava ERRADA num ponto que decide o preço.** Ela dizia que a
//! alternativa do lado do shell era *«um censo de TODAS as poses por quadro de reprodução, e esse
//! custo é um número que ninguém mediu»*. Não é: o `TimelineDoc` **nomeia** as entidades que anima
//! (`doc.bindings()`), então o censo é `O(bindings)` — a dúzia de objetos que o artista keyou — e
//! não `O(mundo)`. *Uma ausência afirmada sem olhar a API é um palpite com cara de medição.*

use crate::preview_drive::PreviewDrive;
use crate::undo::ProjectState;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::{Name, SimWorld, Transform, TransformPropagationState, WorklistBuf};
use ph2d_timeline::{PropKind, TimelineDoc};

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    reg
}

fn capture(drive: &PreviewDrive, sim: &mut SimWorld, reg: &ComponentRegistry) -> ProjectState {
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::new();
    ProjectState::capture(
        drive,
        sim,
        &ph2d_vec_scene::VecScene::new(),
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        reg,
        &mut prop,
        &mut wl,
    )
}

/// Um objeto keyado: `x` vai de 0 a 10 em 4 s.
fn rig() -> (SimWorld, ph2d_ecs::Entity, TimelineDoc) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Keyed")))
        .id();
    let mut doc = TimelineDoc::new();
    for (t, v) in [(0.0, 0.0_f32), (4.0, 10.0)] {
        doc.insert_key(
            e.to_bits(),
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    (sim, e, doc)
}

/// **UM QUADRO DA TIMELINE A TOCAR NÃO É UMA EDIÇÃO** — o terceiro caso da família, curado pelo
/// MESMO ledger que serve a §11 e o solver.
///
/// ⚠️ **Ele corre o `apply` REAL do `ph2d-timeline`**, que é o produtor do produto — o
/// `timeline_bridge::run` é que não é alcançável (nove argumentos de estado do shell), e o que ele
/// faz de relevante para este gate é chamar esta função.
///
/// **Mutação que deve sangrar:** tirar o `declare_timeline_writes`.
#[test]
fn a_playing_timeline_is_not_an_edit() {
    let reg = registry();
    let (mut sim, e, mut doc) = rig();
    let mut drive = PreviewDrive::default();
    let at_rest = capture(&drive, &mut sim, &reg);

    let mut t = 0.0;
    for _ in 0..24 {
        t += 1.0 / 60.0;
        let before = crate::timeline_preview::poses_of_bindings(sim.world(), &doc);
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut doc, t);
        crate::timeline_preview::declare_timeline_writes(sim.world(), &before, &mut drive);
        drive.settle();
        assert_eq!(
            capture(&drive, &mut sim, &reg),
            at_rest,
            "a curva a tocar mudou a captura — cada clique viraria um passo de Ctrl+Z vazio"
        );
    }
    // CONTROLO POSITIVO: o objeto ANDOU de facto.
    assert!(
        sim.world().get::<Transform>(e).unwrap().translation.x > 0.9,
        "a curva nao moveu o objeto — o gate acima nao mediu nada"
    );

    // E PARAR deixa UM passo: a pose passa a ser documento, e o Ctrl+Z desfaz a reprodução.
    drive.settle();
    assert_ne!(
        capture(&drive, &mut sim, &reg),
        at_rest,
        "parar a reproducao tem de deixar UM passo"
    );
}

/// **O CENSO É `O(bindings)`, e não `O(mundo)`** — a medição que desbloqueou esta wave.
///
/// ⛔ A nota que deixou a timeline de fora dizia que o custo era desconhecido porque a alternativa
/// era varrer todas as poses. O documento **nomeia** quem ele anima.
///
/// **Mutação que deve sangrar:** o censo varrer o mundo em vez das bindings.
#[test]
fn the_census_only_looks_at_what_the_document_animates() {
    let (mut sim, _e, doc) = rig();
    // Mais cem objetos que a timeline não conhece.
    for i in 0..100 {
        sim.world_mut()
            .spawn((Transform::default(), Name::new(format!("Extra {i}"))));
    }
    let poses = crate::timeline_preview::poses_of_bindings(sim.world(), &doc);
    assert_eq!(
        poses.len(),
        1,
        "o censo apanhou {} entidades num mundo de 101 — ele tem de olhar so' as keyadas",
        poses.len()
    );
}

/// **Uma entidade com VÁRIAS props keyadas entra UMA vez** — `TranslationX` e `TranslationY` são
/// duas bindings do mesmo objeto, e declarar a mesma pose duas vezes é trabalho a dobrar por nada.
#[test]
fn an_entity_keyed_on_two_axes_is_censused_once() {
    let (sim, e, mut doc) = rig();
    doc.insert_key(
        e.to_bits(),
        PropKind::TranslationY,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    assert!(
        doc.bindings().len() >= 2,
        "a fixtura precisa de duas bindings"
    );
    assert_eq!(
        crate::timeline_preview::poses_of_bindings(sim.world(), &doc).len(),
        1
    );
}

/// **Uma binding cuja entidade já não existe não derruba o censo** — um objeto apagado deixa a
/// binding pendurada, e o documento mostra-a a vermelho de propósito (a §12 tomou a mesma decisão).
#[test]
fn a_dangling_binding_does_not_break_the_census() {
    let (mut sim, e, doc) = rig();
    sim.world_mut().despawn(e);
    assert!(crate::timeline_preview::poses_of_bindings(sim.world(), &doc).is_empty());
}
