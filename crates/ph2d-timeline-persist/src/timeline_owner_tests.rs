//! O DONO de cada clip (doc ObjectsAndComponents 01 §6) tem a vida de uma binding: atravessa o
//! ficheiro pelo `StableId`, sai com o objecto num passo do undo da timeline, e volta com ele.

use super::*;
use ph2d_ecs::{SimWorld, StableId};
use ph2d_timeline::{SCENE, TimelineIntent as I, apply_intent};

fn heroi(sim: &mut SimWorld) -> Entity {
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("Herói")))
        .id()
}

#[test]
fn a_animacao_de_um_objecto_atravessa_o_ficheiro_e_reencontra_o_dono() {
    let mut sim = SimWorld::new();
    let h = heroi(&mut sim).to_bits();
    let mut timeline = TimelineState::new();
    let andar = timeline.doc.add_clip_for("andar".into(), h).unwrap();
    let bytes = serialize(&mut timeline, sim.world_mut()).unwrap();

    let mut sim2 = SimWorld::new();
    for _ in 0..3 {
        sim2.world_mut().spawn(());
    }
    let h2 = heroi(&mut sim2).to_bits();
    assert_ne!(h, h2);
    let mut loaded = install_from_project(&bytes).unwrap();
    assert!(!loaded.doc.has_animation(h2), "chega destacado");
    assert!(!upkeep(&mut loaded, sim2.world_mut()), "curar não é reset");
    assert!(
        loaded.doc.clip_belongs_to(andar, h2),
        "o clip reencontra o Herói desta sessão"
    );
    assert_eq!(
        loaded.doc.clips_of(SCENE).count(),
        1,
        "a Cena continua com a dela"
    );
}

#[test]
fn apagar_o_objecto_leva_a_animacao_dele_e_o_undo_da_timeline_a_devolve() {
    let mut sim = SimWorld::new();
    let e = heroi(&mut sim);
    let h = e.to_bits();
    let mut timeline = TimelineState::new();
    timeline.doc.add_clip_for("andar".into(), h).unwrap();
    upkeep(&mut timeline, sim.world_mut()); // carimba a identidade em vida
    let id = *sim.world().get::<StableId>(e).unwrap();

    sim.world_mut().despawn(e);
    assert!(
        !upkeep(&mut timeline, sim.world_mut()),
        "sem chaves, a Cena não reinicia"
    );
    assert_eq!(
        timeline.doc.clips().len(),
        1,
        "a animação saiu com o objecto"
    );
    assert!(timeline.history.can_undo(), "num passo do undo da timeline");

    // O caminho de volta é o das tracks: Ctrl+Z global (o objecto volta, o MESMO `StableId`,
    // bits novos) + Ctrl+Z da timeline (o documento volta, o dono dormente) — e a cura recola.
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut timeline, &mut ph, I::Undo);
    let volta = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("Herói"), id))
        .id()
        .to_bits();
    assert_ne!(volta, h);
    assert!(!upkeep(&mut timeline, sim.world_mut()));
    assert!(
        timeline.doc.has_animation(volta),
        "a animação voltou com o objecto"
    );
}

#[test]
fn duplicar_da_a_copia_clips_proprios_num_passo_do_undo() {
    let mut timeline = TimelineState::new();
    let (heroi, braco, copia, braco2) = (7_u64, 8_u64, 17_u64, 18_u64);
    timeline.doc.add_clip_for("andar".into(), heroi).unwrap();
    timeline.doc.add_clip_for("acenar".into(), braco).unwrap();
    let antes = timeline.doc.clips().len();

    let n = copy_animations(&mut timeline, &[(heroi, copia), (braco, braco2)]);
    assert_eq!(n, 2, "o Herói e o braço — cada um a sua");
    assert!(timeline.doc.has_animation(copia) && timeline.doc.has_animation(braco2));
    assert_eq!(
        timeline.doc.clips_of(heroi).count(),
        1,
        "o original fica como estava"
    );

    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut timeline, &mut ph, I::Undo);
    assert_eq!(
        timeline.doc.clips().len(),
        antes,
        "um Ctrl+Z da timeline desfaz a cópia inteira"
    );

    assert_eq!(
        copy_animations(&mut timeline, &[(99, 100)]),
        0,
        "sem Animação, nada"
    );
}

#[test]
fn a_copia_que_o_editor_anuncia_leva_a_animacao_no_quadro_seguinte() {
    let mut sim = SimWorld::new();
    let e = heroi(&mut sim);
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    let mut timeline = TimelineState::new();
    timeline
        .doc
        .add_clip_for("andar".into(), e.to_bits())
        .unwrap();
    let copia = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &reg, e, None).unwrap();
    ph2d_ecs::announce_copy(sim.world_mut(), &copia);

    assert!(!upkeep(&mut timeline, sim.world_mut()));
    assert!(
        timeline.doc.has_animation(copia.root.to_bits()),
        "a cópia ganhou clips próprios"
    );
    assert!(
        ph2d_ecs::take_copies(sim.world_mut(), ph2d_ecs::CopyReader::Timeline).is_empty(),
        "o anúncio foi drenado"
    );
    assert_eq!(
        timeline.doc.clips_of(e.to_bits()).count(),
        1,
        "o original fica como estava"
    );
}

/// O Ctrl+Z global: o objecto volta com o MESMO `StableId` e bits novos (o snapshot v2 reinsere-o).
fn renasce(sim: &mut SimWorld, id: StableId) -> u64 {
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new("Herói"), id))
        .id()
        .to_bits()
}

/// ⭐⭐ **UM Ctrl+Z** (doc ObjectsAndComponents 01 §14, item 3): o undo global devolve o objecto e,
/// no mesmo quadro, o cemitério devolve a animação dele — sem o Ctrl+Z da timeline, e o passo da
/// purga sai da fila (um Ctrl+Z a seguir não o «desfaz» outra vez). (Mutação: `ressuscita` sem a
/// chamada no `upkeep` ⇒ RED.)
#[test]
fn um_ctrl_z_global_devolve_o_objecto_com_a_animacao() {
    let mut sim = SimWorld::new();
    let e = heroi(&mut sim);
    let mut timeline = TimelineState::new();
    timeline
        .doc
        .add_clip_for("andar".into(), e.to_bits())
        .unwrap();
    upkeep(&mut timeline, sim.world_mut());
    let id = *sim.world().get::<StableId>(e).unwrap();
    let passos = timeline.history.can_undo();

    sim.world_mut().despawn(e);
    upkeep(&mut timeline, sim.world_mut());
    assert_eq!(
        timeline.doc.clips().len(),
        1,
        "a animação saiu com o objecto"
    );

    let volta = renasce(&mut sim, id);
    upkeep(&mut timeline, sim.world_mut());
    assert!(
        timeline.doc.has_animation(volta),
        "voltou com o objecto, sem 2.º Ctrl+Z"
    );
    assert_eq!(
        timeline.history.can_undo(),
        passos,
        "o passo da purga saiu da fila"
    );
}

/// ⭐ As PISTAS do objecto (as chaves dele nos clipes da Cena) também voltam, e o documento que a
/// purga do último objecto animado reiniciou volta inteiro.
#[test]
fn as_pistas_e_o_reset_tambem_voltam_num_ctrl_z() {
    let mut sim = SimWorld::new();
    let e = heroi(&mut sim);
    let mut timeline = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut timeline,
        &mut ph,
        I::AddKey {
            entity: e.to_bits(),
            prop: ph2d_timeline::PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(0.0),
            value: ph2d_anim::AnimValue::Float(1.0),
            interp: ph2d_anim::Interp::Linear,
        },
    );
    upkeep(&mut timeline, sim.world_mut());
    let id = *sim.world().get::<StableId>(e).unwrap();
    let antes = timeline.doc.clone();

    sim.world_mut().despawn(e);
    ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, 0.0);
    assert!(
        upkeep(&mut timeline, sim.world_mut()),
        "o último objecto animado reinicia"
    );

    let volta = renasce(&mut sim, id);
    upkeep(&mut timeline, sim.world_mut());
    assert_eq!(timeline.doc.bindings().len(), 1, "a pista voltou");
    assert_eq!(
        timeline.doc.bindings()[0].entity,
        volta,
        "recolada no objecto renascido"
    );
    assert_eq!(timeline.doc.clips(), antes.clips(), "o documento inteiro");
}

/// ⛔ Editado DEPOIS de apagar ⇒ o trabalho novo não é apagado: a animação volta pelo caminho de
/// sempre (o Ctrl+Z da timeline), em dois passos.
#[test]
fn editado_depois_de_apagar_volta_em_dois_passos() {
    let mut sim = SimWorld::new();
    let e = heroi(&mut sim);
    let mut timeline = TimelineState::new();
    timeline
        .doc
        .add_clip_for("andar".into(), e.to_bits())
        .unwrap();
    upkeep(&mut timeline, sim.world_mut());
    let id = *sim.world().get::<StableId>(e).unwrap();
    sim.world_mut().despawn(e);
    upkeep(&mut timeline, sim.world_mut());

    let pre = timeline.doc.clone();
    timeline.doc.add_clip_for("novo".into(), SCENE).unwrap();
    timeline.history.push(pre);
    let volta = renasce(&mut sim, id);
    upkeep(&mut timeline, sim.world_mut());
    assert!(
        !timeline.doc.has_animation(volta),
        "nada sobrescreve a edição"
    );
    assert!(
        timeline.doc.clips().iter().any(|c| c.name == "novo"),
        "a edição fica"
    );
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut timeline, &mut ph, I::Undo);
    apply_intent(&mut timeline, &mut ph, I::Undo);
    upkeep(&mut timeline, sim.world_mut());
    assert!(
        timeline.doc.has_animation(volta),
        "o caminho de sempre continua lá"
    );
}

/// ⭐ As fontes do Arrange da Cena levam o NOME de cada dono (onda 4, item 5) — o documento não o
/// conhece; o cabeçalho dos grupos do ▾ lê-o daqui.
#[test]
fn as_fontes_do_arrange_levam_o_nome_do_dono() {
    let mut sim = SimWorld::new();
    let h = heroi(&mut sim).to_bits();
    let mut timeline = TimelineState::new();
    timeline.doc.add_clip_for("andar".into(), h).unwrap();
    let mut view = ph2d_timeline::TimelineViewSnapshot::default();
    view.rebuild(&mut timeline, &ph2d_core::Playhead::new(1.0 / 60.0), false);
    crate::publish_animation_header(&mut view, &timeline, sim.world());
    let nomes: Vec<(u64, &str)> = view
        .animation
        .sources
        .iter()
        .map(|g| (g.owner, g.name.as_str()))
        .collect();
    assert_eq!(nomes, [(SCENE, ""), (h, "Herói")]);
}
