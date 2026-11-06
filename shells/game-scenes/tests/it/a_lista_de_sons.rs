//! A lista de sons do cartão Audio pelo mundo: a edição que o painel manda vira commit, e o snapshot lê de volta.
//! Saiu de `ph2d-app-components` pela catraca da crate; as edições puras estão em `ph2d-sound-set`.

use ph2d_app_components::audio_inspector::{apply_audio_edit, build_audio_info};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommandQueue, apply_editor_commands, register_ecs_components,
};
use ph2d_ecs::{
    AudioSource2D, Entity, PickStrategy, ProjectSounds, SimWorld, SoundEntry, SoundSet, Transform,
    World,
};
use ph2d_editor_core::AudioFieldEdit;

/// O cartão diz «falta» só para um nome que os sons do projeto não têm.
#[test]
fn the_card_says_missing_only_for_a_name_outside_the_project() {
    let mut w = World::new();
    w.get_resource_or_init::<ProjectSounds>()
        .add("plim.wav", vec![1]);
    let falta = |w: &mut World, sound: &str| {
        let e = w
            .spawn((
                Transform::IDENTITY,
                AudioSource2D {
                    sound: sound.into(),
                    ..AudioSource2D::default()
                },
            ))
            .id();
        build_audio_info(w, e.to_bits(), 1)
            .and_then(|i| i.source)
            .map(|s| s.file_missing)
    };
    assert_eq!(falta(&mut w, "plim.wav"), Some(false));
    assert_eq!(falta(&mut w, ""), Some(false));
    assert_eq!(falta(&mut w, "/velho/caminho.wav"), Some(true));
}

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    r
}

/// Um objeto com o `sound` de sempre e um projeto com três sons.
fn cena(sound: &str) -> (SimWorld, Entity) {
    let mut sim = SimWorld::default();
    let mut l = sim.world_mut().get_resource_or_init::<ProjectSounds>();
    l.add("a.wav", vec![1]);
    l.add("b.wav", vec![2]);
    l.add("c.wav", vec![3]);
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            AudioSource2D {
                sound: sound.into(),
                ..AudioSource2D::default()
            },
        ))
        .id();
    (sim, e)
}

fn edit(sim: &mut SimWorld, e: Entity, r: &ComponentRegistry, ed: AudioFieldEdit) {
    let q = EditorCommandQueue::new();
    assert!(apply_audio_edit(sim, e.to_bits(), &ed, &q, r).is_none());
    apply_editor_commands(sim.world_mut(), &q, r).expect("o commit aplica");
}

fn nomes(sim: &SimWorld, e: Entity) -> Vec<String> {
    sim.world()
        .get::<SoundSet>(e)
        .map(|s| s.sounds.iter().map(|x| x.name.clone()).collect())
        .unwrap_or_default()
}

/// *Mutação: tirar o `push(antigo)` do `SoundSet::add_keeping` ⇒ vermelho.*
#[test]
fn pick_on_an_object_without_a_set_creates_it_and_keeps_the_old_sound() {
    let r = registry();
    let (mut sim, e) = cena("a.wav");
    assert!(sim.world().get::<SoundSet>(e).is_none());
    edit(&mut sim, e, &r, AudioFieldEdit::SetAdd("b.wav".into()));
    assert_eq!(nomes(&sim, e), ["a.wav", "b.wav"]);
    let src = sim.world().get::<AudioSource2D>(e).expect("fonte");
    assert_eq!(
        src.sound, "a.wav",
        "o `sound` é o som de quando a lista esvazia"
    );
    // O que já está na lista não entra duas vezes.
    edit(&mut sim, e, &r, AudioFieldEdit::SetAdd("b.wav".into()));
    assert_eq!(nomes(&sim, e), ["a.wav", "b.wav"]);
    // Sem `sound`, só o escolhido.
    let (mut sim, e) = cena("");
    edit(&mut sim, e, &r, AudioFieldEdit::SetAdd("c.wav".into()));
    assert_eq!(nomes(&sim, e), ["c.wav"]);
}

#[test]
fn removing_the_last_sound_gives_the_object_its_old_sound_back() {
    let r = registry();
    let (mut sim, e) = cena("a.wav");
    edit(&mut sim, e, &r, AudioFieldEdit::SetAdd("a.wav".into()));
    let i = |sim: &mut SimWorld| {
        build_audio_info(sim.world_mut(), e.to_bits(), 1)
            .and_then(|i| i.source)
            .expect("fonte")
    };
    assert!(i(&mut sim).set.decides);
    edit(&mut sim, e, &r, AudioFieldEdit::SetRemove(0));
    assert!(nomes(&sim, e).is_empty(), "a lista fica vazia");
    let s = i(&mut sim);
    assert!(!s.set.decides, "sem som ligado o `sound` volta a mandar");
    assert_eq!(s.sound, "a.wav");
    assert!(!s.is_mute());
    // Índice fora não grava nem entra em pânico.
    edit(&mut sim, e, &r, AudioFieldEdit::SetRemove(9));
}

#[test]
fn mode_no_repeat_and_the_variations_write_the_component_through_the_law() {
    let r = registry();
    let (mut sim, e) = cena("");
    edit(&mut sim, e, &r, AudioFieldEdit::SetAdd("a.wav".into()));
    edit(&mut sim, e, &r, AudioFieldEdit::SetStrategy(1));
    edit(&mut sim, e, &r, AudioFieldEdit::SetNoRepeat(false));
    edit(&mut sim, e, &r, AudioFieldEdit::SetPitchJitter(3.0));
    // O `sanitize` manda: acima da faixa satura, e o peso também.
    edit(&mut sim, e, &r, AudioFieldEdit::SetGainJitter(99.0));
    edit(&mut sim, e, &r, AudioFieldEdit::SetWeight(0, 99.0));
    edit(&mut sim, e, &r, AudioFieldEdit::SetEnabled(0, false));
    let s = sim.world().get::<SoundSet>(e).expect("conjunto");
    assert_eq!(s.strategy, PickStrategy::Sequence);
    assert!(!s.no_repeat);
    assert!((s.pitch_jitter_semitones - 3.0).abs() < 1e-6);
    assert!((s.gain_jitter_db - 24.0).abs() < 1e-6);
    assert!((s.sounds[0].weight - 16.0).abs() < 1e-6);
    assert!(!s.sounds[0].enabled);
    // Estratégia fora da tabela não muda nada.
    edit(&mut sim, e, &r, AudioFieldEdit::SetStrategy(200));
    let s = sim.world().get::<SoundSet>(e).expect("conjunto");
    assert_eq!(s.strategy, PickStrategy::Sequence);
}

/// O cartão oferece o que falta, nomeia o que não existe e diz quem manda.
#[test]
fn the_snapshot_offers_what_is_left_and_names_what_is_gone() {
    let (mut sim, e) = cena("");
    sim.world_mut().entity_mut(e).insert(SoundSet {
        sounds: vec![SoundEntry::new("a.wav"), SoundEntry::new("sumiu.wav")],
        ..SoundSet::default()
    });
    let s = build_audio_info(sim.world_mut(), e.to_bits(), 1)
        .and_then(|i| i.source)
        .expect("fonte");
    assert_eq!(s.set.offer, ["b.wav", "c.wav"]);
    assert_eq!(s.set.project_sounds, 3);
    assert_eq!(s.set.missing_names().collect::<Vec<_>>(), ["sumiu.wav"]);
    assert!(
        s.file_missing,
        "um nome da lista fora do projeto também falta"
    );
    assert!(s.set.decides);
    assert!(!s.no_sound_to_play());
    assert_eq!(s.set.mode_labels.len(), PickStrategy::ALL.len());
    assert_eq!(s.set.sequence_tag, 1);
}

/// Com a lista a mandar, o som largado ou importado vai para ela; no campo ele não tocaria.
#[test]
fn a_dropped_sound_goes_to_the_list_when_the_list_decides() {
    let r = registry();
    let (mut sim, e) = cena("");
    edit(&mut sim, e, &r, AudioFieldEdit::SetAdd("a.wav".into()));
    edit(&mut sim, e, &r, AudioFieldEdit::Sound("b.wav".into()));
    assert_eq!(nomes(&sim, e), ["a.wav", "b.wav"]);
    let src = sim.world().get::<AudioSource2D>(e).expect("fonte");
    assert_eq!(src.sound, "");
    // Sem lista ligada, o campo de sempre.
    edit(&mut sim, e, &r, AudioFieldEdit::SetEnabled(0, false));
    edit(&mut sim, e, &r, AudioFieldEdit::SetEnabled(1, false));
    edit(&mut sim, e, &r, AudioFieldEdit::Sound("c.wav".into()));
    let src = sim.world().get::<AudioSource2D>(e).expect("fonte");
    assert_eq!(src.sound, "c.wav");
}
