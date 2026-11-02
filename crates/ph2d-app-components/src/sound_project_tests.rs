use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_audio::{AudioFormat, SampleData};
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components, snapshot_to_world};
use ph2d_ecs::{SimWorld, StableId, Transform};

use super::*;

fn reg() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    r
}

/// Uma pasta só deste teste, com um `.wav` de verdade e um ficheiro que não é som.
fn pasta() -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = std::env::temp_dir().join(format!(
        "ph2d_proj_sons_{}_{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("temp");
    let data = SampleData::from_interleaved(vec![0.0_f32; 960], AudioFormat::stereo(48_000));
    ph2d_audio_encode::write_wav(
        &dir.join("moeda.wav"),
        &data,
        ph2d_audio_encode::BitDepth::Pcm16,
    )
    .expect("wav");
    std::fs::write(dir.join("nota.wav"), "isto não é som").expect("escreve");
    dir
}

fn com_som(sim: &mut SimWorld, id: u64, sound: &str) {
    sim.world_mut().spawn((
        Transform::IDENTITY,
        StableId(id),
        AudioSource2D {
            sound: sound.into(),
            ..AudioSource2D::default()
        },
    ));
}

fn snapshot(sim: &mut SimWorld) -> WorldSnapshot {
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::new();
    ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &reg(), &mut snap)
        .expect("captura");
    snap
}

fn sons_das_fontes(snap: &WorldSnapshot) -> Vec<String> {
    let mut sim = SimWorld::new();
    let es = snapshot_to_world(sim.world_mut(), snap, &reg()).expect("restaura");
    let mut v: Vec<String> = es
        .into_iter()
        .filter_map(|e| sim.world().get::<AudioSource2D>(e).map(|s| s.sound.clone()))
        .collect();
    v.sort();
    v
}

/// ⭐ O projeto antigo (o `sound` era um caminho): o load copia o ficheiro UMA vez para dentro do
/// projeto e as fontes passam a nomeá-lo; o que não se acha fica e é contado; um nome que já é som
/// do projeto não se toca.
#[test]
fn an_old_path_is_copied_into_the_project_once_and_renamed() {
    let dir = pasta();
    let wav = dir.join("moeda.wav").to_string_lossy().into_owned();
    let mut sim = SimWorld::new();
    com_som(&mut sim, 1, &wav);
    com_som(&mut sim, 2, "moeda.wav"); // relativo à pasta do projeto
    com_som(&mut sim, 3, "/nao/existe.ogg");
    com_som(&mut sim, 4, "");
    let mut snap = snapshot(&mut sim);
    let mut lib = ProjectSounds::default();
    let (copiados, faltam) = import_paths(&mut snap, &mut lib, Some(&dir), None);
    assert_eq!((copiados, faltam), (1, 1));
    assert_eq!(lib.len(), 1);
    assert_eq!(
        sons_das_fontes(&snap),
        ["", "/nao/existe.ogg", "moeda.wav", "moeda.wav"]
    );
    // Segunda passagem: nada a importar.
    assert_eq!(import_paths(&mut snap, &mut lib, Some(&dir), None), (0, 1));
}

/// ⛔ Um ficheiro que não é som não entra na biblioteca.
#[test]
fn a_file_that_is_not_sound_is_refused() {
    let dir = pasta();
    let mut lib = ProjectSounds::default();
    assert!(matches!(
        import_into(&mut lib, &dir.join("nota.wav")),
        Err(ImportError::NotAudio(_))
    ));
    assert!(lib.is_empty());
    assert_eq!(
        import_into(&mut lib, &dir.join("moeda.wav")).unwrap(),
        "moeda.wav"
    );
    assert!(is_sound_file(Path::new("a/b.OGG")) && !is_sound_file(Path::new("b.png")));
}

/// O blob do mundo grava e reabre os mesmos sons, e o load diz o que copiou.
#[test]
fn the_project_sounds_round_trip_and_the_load_says_what_it_copied() {
    let dir = pasta();
    let mut sim = SimWorld::new();
    import(sim.world_mut(), &dir.join("moeda.wav")).unwrap();
    let blob = save(sim.world());
    let mut vazio = SimWorld::new();
    com_som(&mut vazio, 1, &dir.join("moeda.wav").to_string_lossy());
    let mut snap = snapshot(&mut vazio);
    let aberto = open(&mut snap, &blob, "/x/proj.ph2dproj").unwrap();
    assert_eq!(&aberto.lib, sim.world().resource::<ProjectSounds>());
    assert_eq!(
        aberto.notice, None,
        "o mesmo conteúdo já estava: nada copiado"
    );
    assert!(open(&mut snap, b"lixo", "/x/p.ph2dproj").is_err());
    let mut outro = SimWorld::new();
    com_som(&mut outro, 1, "moeda.wav");
    let mut snap = snapshot(&mut outro);
    let proj = dir.join("p.ph2dproj").to_string_lossy().into_owned();
    let n = open(&mut snap, &[], &proj).unwrap().notice.expect("frase");
    assert!(n.starts_with("Copied 1 sound file"), "{n}");
}
