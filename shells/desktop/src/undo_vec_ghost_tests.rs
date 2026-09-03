//! ⭐⭐⭐ **O FANTASMA NA ORIGEM** — report do Enio, 2026-08-27: *«as peças apagadas não voltaram
//! em seus lugares corretos e como filhos de seus pais, mas voltaram sem pais e na posição (0,0)
//! do mundo»*.
//!
//! # O mecanismo, reproduzido por sonda antes de qualquer linha de cura
//!
//! A reconciliação `path ⟺ entidade` corre **cedo** no quadro e o *Delete* da Hierarquia corre
//! **tarde**. ⇒ o quadro em que uma peça vetorial é apagada **termina inconsistente**: a entidade
//! morreu, o `VecPath` dela ficou. A captura do undo fotografava esse instante; o Ctrl+Z repunha-o;
//! e a reconciliação do quadro seguinte via *«um path sem entidade»* e **cunhava** uma —
//! `Transform::default()`, sem `ChildOf`, chamada `Path N`:
//!
//! ```text
//! UNDO+1: Path 0<-None@Vec2(0.0, 0.0) | Path 1<-None@Vec2(0.0, 0.0)
//! ```
//!
//! ⚠️ **É PRÉ-EXISTENTE** — todo `despawn` de uma forma vetorial o produz. O que a F5.1 fez foi
//! multiplicá-lo por instância (apagar UMA peça do mestre mata N peças de cópia) e torná-lo
//! gritante: um FILHO que volta como raiz na origem é impossível de não ver.

use ph2d_ecs::{ChildOf, Entity, Name, SimWorld, Transform, VecPathRef};
use ph2d_vec_scene::{VecScene, rectangle};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Um pai com uma peça VETORIAL — a forma que o report tem.
fn scene_with_a_vector_child() -> (
    SimWorld,
    VecScene,
    crate::vec_entities::VecEntityMap,
    Entity,
    Entity,
) {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let parent = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(5.0, 3.0)),
            Name::new("Group"),
        ))
        .id();
    let id = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
    let piece = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Plate"),
            VecPathRef(id),
            ChildOf(parent),
        ))
        .id();
    map.insert(id, piece.to_bits());
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_ecs::assign_missing_root_order(sim.world_mut());
    (sim, scene, map, parent, piece)
}

fn take(
    sim: &mut SimWorld,
    scene: &VecScene,
    r: &ph2d_ecs::scene::ComponentRegistry,
) -> crate::undo::ProjectState {
    crate::undo::ProjectState::capture(
        &crate::preview_drive::PreviewDrive::default(),
        sim,
        scene,
        &ph2d_flip::FlipDoc::new(),
        &ph2d_guides::GuideSet::default(),
        &ph2d_ui_state::StateSets::default(),
        &crate::project_library::LibraryDoc::default(),
        r,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

/// Quantas entidades com `Name` NÃO têm pai — o número que o report leu na tela.
fn parentless(sim: &mut SimWorld) -> Vec<String> {
    let all: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world()).map(|(e, _)| e).collect()
    };
    let mut out: Vec<String> = all
        .into_iter()
        .filter(|&e| sim.world().get::<ChildOf>(e).is_none())
        .filter_map(|e| sim.world().get::<Name>(e).map(|n| n.0.clone()))
        .collect();
    out.sort();
    out
}

/// ⭐⭐⭐ **Uma captura tirada no quadro do Delete não pode ressuscitar um fantasma.**
///
/// ⚠️ **O CONTROLE POSITIVO é metade do gate:** ele monta a inconsistência à mão (a entidade morre
/// e o path fica) e prova que, sem a reconciliação, o `sync` de facto cunha `Path N` na origem.
/// Sem esse lado, um gate verde não distinguiria *«a cura funciona»* de *«a cena nunca teve o
/// fenómeno»* — e foi exactamente assim que este defeito viveu.
///
/// (Mutação: tirar o `vec_entities::sync` do `capture_project` ⇒ o produto volta a produzi-lo; o
/// arch-gate `the_photograph_reconciles_the_document_first` é quem o mata.)
#[test]
fn a_deleted_vector_child_does_not_come_back_as_a_ghost_at_the_origin() {
    let r = reg();

    // ── CONTROLE POSITIVO: o estado inconsistente PRODUZ o fantasma ──────────────────────────
    {
        let (mut sim, scene, _map0, _parent, piece) = scene_with_a_vector_child();
        let mut scene = scene;
        let mut map;
        sim.world_mut().despawn(piece); // o Delete, e o `sync` do quadro já correu
        let snap = take(&mut sim, &scene, &r);
        let (s2, m2, _f, _fm) = snap.restore(&mut sim, &r);
        scene = s2;
        map = m2;
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        assert!(
            parentless(&mut sim).iter().any(|n| n.starts_with("Path ")),
            "o controle nao produziu o fantasma — a fixtura nao contem o fenomeno, e o gate \
             abaixo nao mede nada"
        );
    }

    // ── A CURA: reconciliar ANTES da fotografia ─────────────────────────────────────────────
    let (mut sim, mut scene, mut map, _parent, piece) = scene_with_a_vector_child();
    sim.world_mut().despawn(piece);
    // É isto que o `App::capture_project` faz hoje, pela mesma porta.
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let snap = take(&mut sim, &scene, &r);
    let (s2, m2, _f, _fm) = snap.restore(&mut sim, &r);
    scene = s2;
    map = m2;
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert_eq!(
        parentless(&mut sim),
        vec!["Group".to_string()],
        "o Ctrl+Z ressuscitou um fantasma sem pai na origem"
    );
}

/// ⭐⭐ **E um filho vetorial que NÃO foi apagado volta com o pai dele** — a metade que impede a
/// cura de comer o caso normal.
///
/// ⚠️ A pose é `(5,3)` no pai e identidade no filho, então um filho que perdesse o pai leria
/// `(0,0)` — o número exacto do report. *A régua é o MUNDO, e não o `ChildOf`: um pai reposto e
/// uma pose perdida dão o mesmo sintoma.*
#[test]
fn an_untouched_vector_child_survives_the_round_trip_with_its_parent() {
    let r = reg();
    let (mut sim, mut scene, mut map, _parent, piece) = scene_with_a_vector_child();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let snap = take(&mut sim, &scene, &r);
    let _ = piece;
    let (s2, m2, _f, _fm) = snap.restore(&mut sim, &r);
    scene = s2;
    map = m2;
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert_eq!(parentless(&mut sim), vec!["Group".to_string()]);
    let plate = {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, n)| n.0 == "Plate")
            .map(|(e, _)| e)
            .expect("a peca voltou")
    };
    assert_eq!(
        ph2d_ecs::world_transform(sim.world(), plate)
            .expect("pose")
            .translation,
        ph2d_core::Vec2::new(5.0, 3.0),
        "a peca voltou na origem em vez de no sitio do pai"
    );
}
