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
    assert!(loaded.doc.clip_belongs_to(andar, h2), "o clip reencontra o Herói desta sessão");
    assert_eq!(loaded.doc.clips_of(SCENE).count(), 1, "a Cena continua com a dela");
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
    assert!(!upkeep(&mut timeline, sim.world_mut()), "sem chaves, a Cena não reinicia");
    assert_eq!(timeline.doc.clips().len(), 1, "a animação saiu com o objecto");
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
    assert!(timeline.doc.has_animation(volta), "a animação voltou com o objecto");
}
