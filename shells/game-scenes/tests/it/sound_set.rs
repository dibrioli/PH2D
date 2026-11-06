//! O sorteio no caminho do produto: `play_target_with` → `SceneAudio::play` → mixer sem dispositivo.
//! A [`sonda_mil_disparos`] é a sonda offline de W1 do SFX Lab; os números saem dela.

use ph2d_app_audio::audio_2d::{play_target_with, update_with};
use ph2d_app_audio::scene::SceneAudio;
use ph2d_audio::{AudioEngine, AudioFormat, AudioRenderer};
use ph2d_ecs::{
    AudioListener2D, AudioSource2D, Entity, Name, PickStrategy, ProjectSounds, SimWorld,
    SoundEntry, SoundSet, Transform, assign_missing_stable_ids,
};

/// Os três sons da cena de smoke — os mesmos bytes que o artista ouve.
const SONS: [&str; 3] = ["coin_low.wav", "coin_high.wav", "coin_noise.wav"];
const BLOCO: usize = 256;

fn sons() -> ProjectSounds {
    let mut l = ProjectSounds::default();
    for n in SONS {
        let p = format!(
            "{}/../../docs/SfxLab/arte/sons/{n}",
            env!("CARGO_MANIFEST_DIR")
        );
        l.add(n, std::fs::read(&p).unwrap_or_else(|e| panic!("{p}: {e}")));
    }
    l
}

fn conjunto(strategy: PickStrategy, no_repeat: bool, pitch: f32, gain: f32) -> SoundSet {
    SoundSet {
        sounds: SONS.iter().map(|n| SoundEntry::new(*n)).collect(),
        strategy,
        no_repeat,
        pitch_jitter_semitones: pitch,
        gain_jitter_db: gain,
    }
}

/// Um objeto com a fonte (e `sound` próprio, para provar quem manda) e o ouvinte em cima dele.
fn cena(set: Option<SoundSet>) -> (SimWorld, Entity) {
    let mut sim = SimWorld::default();
    let w = sim.world_mut();
    w.spawn((Transform::default(), Name::new("Ears"), AudioListener2D));
    let cfg = AudioSource2D {
        sound: "coin_low.wav".into(),
        ..AudioSource2D::default()
    };
    let mut f = w.spawn((Transform::default(), Name::new("Coin Sounds"), cfg));
    if let Some(s) = set {
        f.insert(s);
    }
    let fonte = f.id();
    sim.world_mut().insert_resource(sons());
    assign_missing_stable_ids(sim.world_mut());
    (sim, fonte)
}

fn motor() -> (AudioEngine, AudioRenderer) {
    AudioEngine::new(AudioFormat::stereo(48_000))
}

fn bloco(mixer: &mut AudioRenderer) -> Vec<f32> {
    let mut out = vec![0.0_f32; BLOCO * 2];
    mixer.render(&mut out, BLOCO);
    out
}

/// O que cada som produz no mixer, tocado sozinho.
fn referencias() -> [Vec<f32>; 3] {
    SONS.map(|n| {
        let (mut sim, f) = cena(Some(SoundSet {
            sounds: vec![SoundEntry::new(n)],
            ..SoundSet::default()
        }));
        let (mut livro, (mut eng, mut mixer)) = (SceneAudio::default(), motor());
        assert!(play_target_with(&mut sim, &mut livro, &mut eng, f));
        bloco(&mut mixer)
    })
}

/// O índice da referência mais perto do que saiu. A diferença para a segunda tem de ser clara.
fn qual(saida: &[f32], refs: &[Vec<f32>; 3]) -> usize {
    let dist = |r: &Vec<f32>| -> f32 { saida.iter().zip(r).map(|(a, b)| (a - b) * (a - b)).sum() };
    let d: Vec<f32> = refs.iter().map(dist).collect();
    let i = (0..3).min_by(|a, b| d[*a].total_cmp(&d[*b])).expect("três");
    assert!(d[i] < 1e-3, "o que saiu não é nenhum dos três sons: {d:?}");
    i
}

/// **A sonda.** Mil disparos em cada modo, e o que conta é o que saiu do MIXER, comparado com cada som sozinho.
#[test]
fn sonda_mil_disparos() {
    let refs = referencias();
    for (nome, strategy, no_repeat) in [
        ("Shuffle, No repeat", PickStrategy::Shuffle, true),
        ("Random, No repeat", PickStrategy::Random, true),
        ("Random", PickStrategy::Random, false),
        ("Sequence", PickStrategy::Sequence, false),
    ] {
        let (mut sim, f) = cena(Some(conjunto(strategy, no_repeat, 0.0, 0.0)));
        let (mut livro, (mut eng, mut mixer)) = (SceneAudio::default(), motor());
        let (mut conta, mut repetidos, mut antes) = ([0u32; 3], 0, None);
        for _ in 0..1000 {
            assert!(play_target_with(&mut sim, &mut livro, &mut eng, f));
            let i = qual(&bloco(&mut mixer), &refs);
            conta[i] += 1;
            repetidos += u32::from(antes == Some(i));
            antes = Some(i);
        }
        println!(
            "{nome}: {} {} · {} {} · {} {} · repetidos seguidos {repetidos}",
            SONS[0], conta[0], SONS[1], conta[1], SONS[2], conta[2]
        );
        assert_eq!(conta.iter().sum::<u32>(), 1000);
        if no_repeat {
            assert_eq!(repetidos, 0, "{nome} repetiu");
        }
        if strategy != PickStrategy::Random {
            for c in conta {
                assert!(c.abs_diff(333) <= 34, "{nome} desequilibrado: {conta:?}");
            }
        }
    }
}

/// A variação usa a faixa pedida, e o nome sorteado continua a ser um dos três.
#[test]
fn a_variacao_de_altura_e_volume_e_por_disparo() {
    let (mut sim, f) = cena(Some(conjunto(PickStrategy::Shuffle, true, 2.0, 2.0)));
    let (mut livro, (mut eng, mut mixer)) = (SceneAudio::default(), motor());
    let (mut alturas, mut ganhos) = (Vec::new(), Vec::new());
    for _ in 0..1000 {
        assert!(play_target_with(&mut sim, &mut livro, &mut eng, f));
        let _ = bloco(&mut mixer); // o mixer drena os comandos
        let (nome, j) = livro.last_pick().expect("tocou");
        assert!(SONS.contains(&nome));
        alturas.push(j.pitch);
        ganhos.push(j.gain);
    }
    let faixa = |v: &[f32]| {
        (
            v.iter().copied().fold(f32::MAX, f32::min),
            v.iter().copied().fold(0.0, f32::max),
        )
    };
    let ((a0, a1), (g0, g1)) = (faixa(&alturas), faixa(&ganhos));
    println!("altura {a0:.3}..{a1:.3} · ganho {g0:.3}..{g1:.3}");
    let (st, db) = (2.0_f32.powf(2.0 / 12.0), 10.0_f32.powf(2.0 / 20.0));
    assert!(a0 >= 1.0 / st - 1e-3 && a1 <= st + 1e-3, "{a0} {a1}");
    assert!(g0 >= 1.0 / db - 1e-3 && g1 <= db + 1e-3, "{g0} {g1}");
    assert!(
        a1 > 1.1 && a0 < 0.9 && g1 > 1.2 && g0 < 0.85,
        "a faixa não foi usada"
    );
}

/// O passe do quadro reescreve o ganho da voz: sem guardar o sorteio, o volume do disparo sumiria no
/// quadro seguinte. A prova é o que sai do mixer depois do `update`.
#[test]
fn o_volume_do_disparo_sobrevive_ao_quadro_seguinte() {
    let so_um = |gain: f32| {
        let mut s = conjunto(PickStrategy::Sequence, false, 0.0, gain);
        s.sounds.truncate(1);
        s
    };
    let pico = |set: SoundSet, seguir: bool| -> (f32, f32) {
        let set_tem_variacao = set.gain_jitter_db > 0.0;
        let (mut sim, f) = cena(Some(set));
        // Baixo o bastante para o limitador do master não entrar: a régua é a razão entre dois picos.
        sim.world_mut()
            .get_mut::<AudioSource2D>(f)
            .expect("fonte")
            .volume_db = -24.0;
        let (mut livro, (mut eng, mut mixer)) = (SceneAudio::default(), motor());
        // A primeira sorte pode cair perto de 1: dispara até o ganho ser mensurável.
        let j = loop {
            assert!(play_target_with(&mut sim, &mut livro, &mut eng, f));
            let j = livro.last_pick().expect("tocou").1;
            let _ = bloco(&mut mixer);
            if !set_tem_variacao || (j.gain - 1.0).abs() > 0.2 {
                break j;
            }
        };
        if seguir {
            update_with(&mut sim, &mut livro, &mut eng);
        }
        // O som é decrescente: mede-se um bloco fixo depois, nos dois casos.
        for _ in 0..8 {
            let _ = bloco(&mut mixer);
        }
        let p = bloco(&mut mixer)
            .iter()
            .fold(0.0_f32, |m, s| m.max(s.abs()));
        (p, j.gain)
    };
    let (seco, um) = pico(so_um(0.0), true);
    assert_eq!(um, 1.0);
    let (com, g) = pico(so_um(12.0), true);
    let razao = com / seco;
    assert!(
        (razao - g).abs() < 0.03 * g,
        "o ganho sorteado {g} virou {razao} depois do quadro"
    );
}

/// Sem sons ligados no conjunto, vale o `sound` de sempre; com, o conjunto manda.
#[test]
fn o_conjunto_manda_so_quando_tem_som_ligado() {
    let (mut sim, f) = cena(None);
    let (mut livro, (mut eng, _m)) = (SceneAudio::default(), motor());
    assert!(play_target_with(&mut sim, &mut livro, &mut eng, f));
    assert_eq!(livro.last_pick().expect("tocou").0, "coin_low.wav");

    let mut desligado = conjunto(PickStrategy::Random, false, 0.0, 0.0);
    for s in &mut desligado.sounds {
        s.enabled = false;
    }
    let (mut sim, f) = cena(Some(desligado));
    assert!(play_target_with(&mut sim, &mut livro, &mut eng, f));
    assert_eq!(livro.last_pick().expect("tocou").0, "coin_low.wav");

    let mut um = conjunto(PickStrategy::Random, false, 0.0, 0.0);
    um.sounds.remove(0);
    um.sounds.remove(0);
    let (mut sim, f) = cena(Some(um));
    assert!(play_target_with(&mut sim, &mut livro, &mut eng, f));
    assert_eq!(livro.last_pick().expect("tocou").0, "coin_noise.wav");
}

/// O Autoplay arranca pelo sorteio também: uma porta só.
#[test]
fn o_autoplay_passa_pelo_sorteio() {
    let (mut sim, f) = cena(Some(conjunto(PickStrategy::Sequence, false, 0.0, 0.0)));
    sim.world_mut()
        .get_mut::<AudioSource2D>(f)
        .expect("fonte")
        .autoplay = true;
    sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
    let (mut livro, (mut eng, _m)) = (SceneAudio::default(), motor());
    let r = update_with(&mut sim, &mut livro, &mut eng);
    assert_eq!(r.started, 1);
    assert_eq!(livro.last_pick().expect("tocou").0, SONS[0]);
}

/// ⛔ Uma porta só: fora de `SceneAudio::play`, nenhum código do produto chama o motor para tocar um som de
/// cena, e ninguém além dele sorteia. Lê o fonte, porque a segunda porta compila.
#[test]
fn o_sorteio_tem_uma_porta_so() {
    let fonte = |n: &str| {
        let p = format!(
            "{}/../../crates/ph2d-app-audio/src/{n}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e}"))
    };
    let audio_2d = fonte("audio_2d.rs");
    assert!(!audio_2d.contains(".pick("));
    assert!(!audio_2d.contains("eng.play(") && !audio_2d.contains("motor.play("));
    let cena = fonte("scene.rs");
    assert_eq!(cena.matches(".pick(").count(), 1);
    assert_eq!(cena.matches("eng.play(").count(), 1);
}

/// **O Autoplay espera o Play**: nada soa enquanto se edita, o Play arranca, o Stop cala e o Play seguinte
/// arranca de novo. A voz é contada no mixer, e o som fica em ciclo para continuar vivo entre as bordas.
#[test]
fn autoplay_espera_o_play() {
    let vozes = |mixer: &mut AudioRenderer| {
        let _ = bloco(mixer);
        mixer.active_voices()
    };
    let (mut sim, f) = cena(None);
    {
        let mut cfg = sim.world_mut().get_mut::<AudioSource2D>(f).expect("fonte");
        cfg.autoplay = true;
        cfg.looping = true;
    }
    let mut livro = SceneAudio::default();
    let (mut eng, mut mixer) = motor();

    for _ in 0..3 {
        let r = update_with(&mut sim, &mut livro, &mut eng);
        assert_eq!(r.started, 0, "o Autoplay tocou sem Play");
    }
    assert_eq!(vozes(&mut mixer), 0, "há voz sem Play");

    for rodada in ["primeiro", "segundo"] {
        sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
        let r = update_with(&mut sim, &mut livro, &mut eng);
        assert_eq!(r.started, 1, "o {rodada} Play não arrancou o Autoplay");
        assert_eq!(vozes(&mut mixer), 1, "{rodada} Play sem voz");
        assert_eq!(
            update_with(&mut sim, &mut livro, &mut eng).started,
            0,
            "o Autoplay renasceu no quadro seguinte"
        );
        sim.world_mut().remove_resource::<ph2d_ecs::RunOnCopy>();
        update_with(&mut sim, &mut livro, &mut eng);
        assert_eq!(vozes(&mut mixer), 0, "o Stop não calou o som");
    }
}
