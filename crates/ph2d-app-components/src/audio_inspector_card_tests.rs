use ph2d_audio::{AudioFormat, SampleData};

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

// ── A lista de sons (SFX Lab W1). Pelo mundo: `shells/game-scenes/tests/it/a_lista_de_sons.rs` ──

mod a_lista {
    use super::*;

    /// Sem mundo: o que o cartão oferece e quem manda saem só da lista e dos sons do projeto.
    #[test]
    fn the_list_info_offers_what_is_left() {
        let mut l = ProjectSounds::default();
        l.add("a.wav", vec![1]);
        l.add("b.wav", vec![2]);
        let set = SoundSet {
            sounds: vec![ph2d_ecs::SoundEntry::new("a.wav")],
            ..SoundSet::default()
        };
        let i = sound_set_info(Some(&set), Some(&l));
        assert_eq!(i.offer, ["b.wav"]);
        assert!(i.decides);
    }
}
