//! ⭐ **Os nomes dos clipes publicados no mundo** (`ph2d_ecs::ClipNames`, onda 5 item c) — o que o ▾
//! do *Play Clip* no Inspector oferece. O `upkeep` publica os donos VIVOS, cada nome no dono certo,
//! e não reescreve o recurso num quadro em que nada mudou (HR-3).

use super::*;
use ph2d_ecs::{ClipNames, SimWorld};
use ph2d_timeline::SCENE;

fn objecto(sim: &mut SimWorld, nome: &str) -> Entity {
    sim.world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, Name::new(nome)))
        .id()
}

fn nomes(world: &World, dono: u64) -> Vec<String> {
    world
        .get_resource::<ClipNames>()
        .map(|r| r.of(dono).map(str::to_string).collect())
        .unwrap_or_default()
}

/// A chave da Cena é a MESMA nos dois lados da fronteira (o `ph2d-ecs` não vê a timeline).
#[test]
fn a_chave_da_cena_e_a_da_timeline() {
    assert_eq!(ClipNames::SCENE, SCENE);
}

#[test]
fn o_upkeep_publica_cada_clipe_no_dono_dele_e_so_os_vivos() {
    let mut sim = SimWorld::new();
    let a = objecto(&mut sim, "Herói");
    let b = objecto(&mut sim, "Porta");
    let mut timeline = TimelineState::new();
    timeline
        .doc
        .add_clip_for("andar".into(), a.to_bits())
        .unwrap();
    timeline
        .doc
        .add_clip_for("abrir".into(), b.to_bits())
        .unwrap();
    timeline
        .doc
        .add_clip_for("correr".into(), a.to_bits())
        .unwrap();
    upkeep(&mut timeline, sim.world_mut());

    assert_eq!(nomes(sim.world(), a.to_bits()), ["andar", "correr"]);
    assert_eq!(nomes(sim.world(), b.to_bits()), ["abrir"]);
    let da_cena: Vec<String> = timeline
        .doc
        .clips_of(SCENE)
        .map(|i| timeline.doc.clips()[i].name.clone())
        .collect();
    assert!(!da_cena.is_empty(), "a fixtura tem a Cena com clipe");
    assert_eq!(nomes(sim.world(), SCENE), da_cena, "a Cena com os dela");

    sim.world_mut().despawn(b);
    upkeep(&mut timeline, sim.world_mut());
    assert!(
        nomes(sim.world(), b.to_bits()).is_empty(),
        "um dono que morreu não oferece clipes"
    );
    assert_eq!(nomes(sim.world(), a.to_bits()), ["andar", "correr"]);
}

/// ⚠️ HR-3: num quadro sem mudança o recurso não é reescrito — o tick de mudança dele fica.
#[test]
fn o_upkeep_nao_reescreve_quando_nada_mudou() {
    let mut sim = SimWorld::new();
    let a = objecto(&mut sim, "Herói");
    let mut timeline = TimelineState::new();
    timeline
        .doc
        .add_clip_for("andar".into(), a.to_bits())
        .unwrap();
    upkeep(&mut timeline, sim.world_mut());
    let tick = |w: &World| w.get_resource_change_ticks::<ClipNames>().unwrap().changed;
    let antes = tick(sim.world());

    sim.world_mut().increment_change_tick();
    upkeep(&mut timeline, sim.world_mut());
    assert_eq!(
        tick(sim.world()),
        antes,
        "nada mudou e o recurso foi reescrito"
    );

    timeline
        .doc
        .add_clip_for("correr".into(), a.to_bits())
        .unwrap();
    sim.world_mut().increment_change_tick();
    upkeep(&mut timeline, sim.world_mut());
    assert_ne!(tick(sim.world()), antes, "o clipe novo não foi publicado");
    assert_eq!(nomes(sim.world(), a.to_bits()), ["andar", "correr"]);
}
