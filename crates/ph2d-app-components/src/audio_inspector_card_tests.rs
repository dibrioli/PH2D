use ph2d_audio::{AudioFormat, SampleData};
use ph2d_ecs::Transform;

use super::*;

fn pasta() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("ph2d_cartao_som_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp");
    let data = SampleData::from_interleaved(vec![0.0_f32; 960], AudioFormat::stereo(48_000));
    ph2d_audio_encode::write_wav(
        &dir.join("plim.wav"),
        &data,
        ph2d_audio_encode::BitDepth::Pcm16,
    )
    .expect("wav");
    std::fs::write(dir.join("texto.wav"), "não é som").expect("escreve");
    dir
}

/// ⭐ GEF 21 E2 — o campo Sound: um caminho que existe é IMPORTADO e o campo fica com o NOME; o
/// nome fica; um ficheiro que não é som é recusado com voz; o resto fica escrito.
#[test]
fn the_sound_field_imports_a_file_and_keeps_its_name() {
    let dir = pasta();
    let mut w = World::new();
    let plim = dir.join("plim.wav").to_string_lossy().into_owned();
    assert_eq!(sound_name(&mut w, &plim).unwrap(), "plim.wav");
    assert!(w.resource::<ProjectSounds>().get("plim.wav").is_some());
    assert_eq!(sound_name(&mut w, " plim.wav ").unwrap(), "plim.wav");
    assert_eq!(sound_name(&mut w, "").unwrap(), "");
    assert_eq!(sound_name(&mut w, "/nao/ha.ogg").unwrap(), "/nao/ha.ogg");
    let texto = dir.join("texto.wav").to_string_lossy().into_owned();
    assert!(sound_name(&mut w, &texto).is_err());
    assert_eq!(w.resource::<ProjectSounds>().len(), 1);
}

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
