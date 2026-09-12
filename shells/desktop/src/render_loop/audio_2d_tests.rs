//! Os gates da PONTE do som de cena — o que ela DECIDE, medido sem placa de som.
//!
//! ⚠️ **Nenhum deles ouve nada**, e é a fronteira: o que sai pela coluna é do `cpal`. O que se prova
//! aqui é o livro — quem arranca, quem é calado, quantas vozes ficam.
//!
//! ⚠️ **Eles correm o caminho do PRODUTO** (`update_with`, `play_target_with`), e não uma cópia
//! dele: um gate que reimplementasse o passe mediria o teste.

use ph2d_audio::{AudioEngine, AudioFormat, AudioRenderer, SampleData};
use ph2d_core::Vec2;
use ph2d_ecs::{
    AudioListener2D, AudioSource2D, Entity, Name, SimWorld, Transform, assign_missing_stable_ids,
};

use super::{play_target_with, stop_target_with, update_with};
use ph2d_audio_desktop::scene::SceneAudio;

/// Um motor sem dispositivo — o `AudioEngine::new` devolve o par controlo/renderer sem tocar no
/// `cpal`.
///
/// ⭐⭐⭐ **O renderer VOLTA com ele, e isso é o que separa dois gates de um.** Sem correr o mixer, o
/// único oráculo disponível é o LIVRO — e o livro não distingue *«a voz foi parada»* de *«a voz foi
/// esquecida»*, que é exactamente a mutação que sobreviveu à primeira redacção deste ficheiro. Com
/// o renderer, a pergunta passa a ser feita a quem tem a resposta: **o mixer**.
fn motor() -> (AudioEngine, AudioRenderer) {
    AudioEngine::new(AudioFormat::stereo(48_000))
}

/// Corre o mixer o suficiente para ele **drenar os comandos** e devolve quantas vozes soam.
///
/// ⚠️ **Os comandos não têm efeito nenhum até alguém renderizar**: o `play`/`stop` do lado de
/// controlo só escreve num ring. Um gate que perguntasse `active_voices()` sem isto leria sempre
/// zero — e passaria sobre tudo.
fn vozes_no_mixer(r: &mut AudioRenderer) -> usize {
    let mut out = vec![0.0_f32; 256];
    r.render(&mut out, 128);
    r.active_voices()
}

/// Um `.wav` de verdade na pasta temporária — o `AudioSource2D` nomeia um CAMINHO, então um gate
/// que lhe desse um nome inventado mediria só o braço da falha.
fn som(nome: &str) -> String {
    let path = std::env::temp_dir().join(nome);
    if !path.exists() {
        let data = SampleData::from_interleaved(
            vec![0.0_f32; 4_800], // 0,1 s de silêncio: o gate mede o LIVRO, não o que se ouve
            AudioFormat::stereo(48_000),
        );
        ph2d_audio_encode::write_wav(&path, &data, ph2d_audio_encode::BitDepth::Pcm16)
            .expect("o gate tem de conseguir escrever a fixtura");
    }
    path.to_string_lossy().into_owned()
}

/// Uma cena com um ouvinte na origem e uma fonte, com a config dada.
fn cena(cfg: AudioSource2D) -> (SimWorld, Entity) {
    let mut sim = SimWorld::default();
    let w = sim.world_mut();
    w.spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Name::new("Ouvinte"),
        AudioListener2D,
    ));
    let fonte = w
        .spawn((
            Transform::from_translation(Vec2::new(1.0, 0.0)),
            Name::new("Fonte"),
            cfg,
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    (sim, fonte)
}

fn com_autoplay(looping: bool) -> AudioSource2D {
    AudioSource2D {
        sound: som("ph2d_gate_audio2d.wav"),
        autoplay: true,
        looping,
        max_distance: 10.0,
        ..AudioSource2D::default()
    }
}

/// ⭐⭐⭐ **`autoplay` ARRANCA no nascimento — e NÃO renasce a cada quadro.**
///
/// # ⚠️ Este gate existe porque a casa já pagou o defeito ao contrário
///
/// O `Timer` shipou com a condição *«não está a correr»* e o report do dono foi *«nada do smoke
/// funciona»*: um *one-shot* que acaba satisfaz aquilo, e renasceria para sempre. A lei é
/// **«começar é uma ARESTA, e a aresta é o nascimento»** — e um som de uma vez só é exactamente a
/// mesma forma.
///
/// **Mutação que deve sangrar:** trocar o `take_birth` por `livro.live_voices() == 0`.
#[test]
fn autoplay_starts_once_at_birth_and_never_again() {
    let (mut sim, _) = cena(com_autoplay(false));
    let mut livro = SceneAudio::default();
    // ⚠️ **Aqui o oráculo é o LIVRO, e não o mixer** — e a diferença é de RELÓGIO: o livro retira
    // uma voz pelo relógio de parede, o mixer só avança com os quadros que alguém lhe renderiza.
    // Num gate sem dispositivo ninguém os renderiza, então o mixer nunca veria a voz acabar. *No
    // app os dois andam juntos porque a placa de som renderiza em tempo real.*
    let (mut eng, _mixer) = motor();

    let r1 = update_with(&mut sim, &mut livro, &mut eng);
    assert_eq!(
        r1.started, 1,
        "a fonte com autoplay nao arrancou no nascimento"
    );
    assert_eq!(r1.sources, 1);
    assert_eq!(r1.listeners, 1);

    // ⭐⭐⭐ **A fixtura tem de deixar a voz ACABAR**, senão ela não produz o fenómeno: a primeira
    // redacção corria os nove quadros de seguida, a amostra de 0,1 s ainda soava, e a mutação
    // «lê `live_voices() == 0`» **SOBREVIVEU**. *Um som de uma vez só só renasce depois de morrer.*
    std::thread::sleep(std::time::Duration::from_millis(160));
    update_with(&mut sim, &mut livro, &mut eng);
    assert_eq!(
        livro.live_voices(),
        0,
        "a fixtura NAO produz o fenomeno: a voz ainda esta' viva, entao «renascer» nao e' testavel"
    );
    let mut extra = 0;
    for _ in 0..9 {
        extra += update_with(&mut sim, &mut livro, &mut eng).started;
    }
    assert_eq!(
        extra, 0,
        "a fonte RENASCEU: o autoplay esta' a ler «nao esta' a tocar» em vez do nascimento"
    );
}

/// ⛔ **Sem `autoplay` ninguém arranca sozinho** — anexar o componente não faz barulho.
///
/// **Mutação que deve sangrar:** tirar o `cfg.autoplay` da guarda.
#[test]
fn a_source_without_autoplay_never_starts_by_itself() {
    let (mut sim, _) = cena(AudioSource2D {
        sound: som("ph2d_gate_audio2d.wav"),
        max_distance: 10.0,
        ..AudioSource2D::default()
    });
    let mut livro = SceneAudio::default();
    let (mut eng, mut mixer) = motor();
    for _ in 0..10 {
        assert_eq!(
            update_with(&mut sim, &mut livro, &mut eng).started,
            0,
            "arrancou sem ninguem mandar"
        );
    }
    assert_eq!(
        livro.live_voices(),
        0,
        "ficou uma voz viva sem ninguem a pedir"
    );
    assert_eq!(
        vozes_no_mixer(&mut mixer),
        0,
        "o MIXER recebeu uma voz que ninguem mandou tocar"
    );
}

/// ⭐⭐ **O verbo `Play Sound` faz o som soar — e a POLIFONIA corta pela ponta velha.**
///
/// ⚠️ **As duas metades juntas** porque a segunda é o que separa *«um passo»* de *«uma parede de
/// som»*: com `max_polyphony = 1` (o default) o segundo disparo **substitui** o primeiro. Sem esta
/// metade, um sinal repetido empilharia vozes até o pool encher e calar a cena inteira.
///
/// **Mutação que deve sangrar:** tirar o laço que corta pela ponta velha no `SceneAudio::play`.
#[test]
fn the_play_verb_sounds_and_polyphony_replaces_the_oldest() {
    let (mut sim, fonte) = cena(AudioSource2D {
        sound: som("ph2d_gate_audio2d.wav"),
        looping: true, // ⚠️ em ciclo, senão a conta do fim retirava a voz e o gate media outra coisa
        max_distance: 10.0,
        ..AudioSource2D::default()
    });
    let mut livro = SceneAudio::default();
    let (mut eng, mut mixer) = motor();
    assert!(
        play_target_with(&mut sim, &mut livro, &mut eng, fonte),
        "o verbo nao chegou a tocar nada"
    );
    assert_eq!(livro.live_voices(), 1);
    for _ in 0..5 {
        play_target_with(&mut sim, &mut livro, &mut eng, fonte);
    }
    assert_eq!(
        livro.live_voices(),
        1,
        "com max_polyphony = 1, seis disparos tinham de deixar UMA voz"
    );
    assert_eq!(
        vozes_no_mixer(&mut mixer),
        1,
        "o LIVRO diz uma e o MIXER diz outra — as vozes cortadas nao foram paradas"
    );
    assert!(
        stop_target_with(&sim, &mut livro, &eng, fonte),
        "o verbo de calar nao achou nada para calar"
    );
    assert_eq!(livro.live_voices(), 0, "calar nao calou");
    assert_eq!(
        vozes_no_mixer(&mut mixer),
        0,
        "o MIXER continua a tocar depois do stop"
    );
}

/// **Uma fonte com mais polifonia ACUMULA até ao teto dela.**
///
/// ⚠️ O irmão do gate acima, e a razão de ele existir: sem este, pôr o corte a `1` fixo passaria os
/// dois — *uma cerca que corta sempre não é a cerca que se pediu*.
#[test]
fn a_polyphonic_source_stacks_up_to_its_own_ceiling() {
    let (mut sim, fonte) = cena(AudioSource2D {
        sound: som("ph2d_gate_audio2d.wav"),
        looping: true,
        max_distance: 10.0,
        max_polyphony: 3,
        ..AudioSource2D::default()
    });
    let mut livro = SceneAudio::default();
    let (mut eng, mut mixer) = motor();
    for _ in 0..8 {
        play_target_with(&mut sim, &mut livro, &mut eng, fonte);
    }
    assert_eq!(
        livro.live_voices(),
        3,
        "o teto de polifonia da FONTE nao foi honrado"
    );
    assert_eq!(
        vozes_no_mixer(&mut mixer),
        3,
        "o MIXER discorda do livro: as cinco cortadas nao foram paradas"
    );
}

/// ⭐⭐ **Um objecto que SAI da cena é calado** — e não deixa um som órfão.
///
/// ⚠️ **É o modo de falha de um `Ctrl+Z`** sobre um objecto que estava a soar: apagar do livro sem
/// parar as vozes deixaria um zumbido a tocar para sempre, sem dono que o pudesse calar.
///
/// **Mutação que deve sangrar:** no `forget_absent`, esquecer sem chamar `stop`.
#[test]
fn an_object_that_leaves_the_scene_is_silenced() {
    let (mut sim, fonte) = cena(com_autoplay(true));
    let mut livro = SceneAudio::default();
    let (mut eng, mut mixer) = motor();
    assert_eq!(update_with(&mut sim, &mut livro, &mut eng).live, 1);
    assert_eq!(
        vozes_no_mixer(&mut mixer),
        1,
        "o mixer nao chegou a tocar nada"
    );

    sim.world_mut().despawn(fonte);
    let r = update_with(&mut sim, &mut livro, &mut eng);
    assert_eq!(r.sources, 0, "a fonte ainda esta' na cena");
    assert_eq!(r.live, 0, "o livro ainda a conta");
    // ⭐⭐⭐ **A pergunta é ao MIXER, e não ao livro.** Esquecer sem parar deixa o livro a zero e a
    // voz a soar para sempre — e foi essa a mutação que sobreviveu à primeira redacção deste gate.
    assert_eq!(
        vozes_no_mixer(&mut mixer),
        0,
        "o som ficou ORFAO: a fonte saiu da cena e o MIXER continua a toca-la"
    );
}

/// **Uma cena sem ouvinte ainda toca** — e o relatório di-lo.
///
/// ⚠️ ⛔ A alternativa recusada é calar: uma cena a meio de ser montada não tem ouvinte, e um
/// produto que se cala sem explicar lê-se como avariado.
#[test]
fn a_scene_with_no_listener_still_plays_and_says_so() {
    let mut sim = SimWorld::default();
    let _ = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(7.0, 0.0)),
            Name::new("Fonte"),
            com_autoplay(true),
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    let mut livro = SceneAudio::default();
    let (mut eng, mut mixer) = motor();
    let r = update_with(&mut sim, &mut livro, &mut eng);
    assert_eq!(r.listeners, 0, "o relatorio tem de NOMEAR a ausencia");
    assert_eq!(r.started, 1, "sem ouvinte o som ainda toca, sem posicao");
    assert_eq!(r.live, 1);
    assert_eq!(vozes_no_mixer(&mut mixer), 1, "o MIXER nao chegou a tocar");
}

/// ⛔ **Um caminho que não abre não arranca — e não tenta outra vez.**
///
/// ⚠️ **A segunda metade não é performance:** uma tentativa por quadro daria uma linha de erro 60
/// vezes por segundo sobre um facto que não muda, e o terminal do dono deixaria de servir para
/// diagnosticar seja o que for.
///
/// **Mutação que deve sangrar:** não guardar a falha (`decoded.insert(path, None)`).
#[test]
fn a_path_that_does_not_open_never_starts_and_is_not_retried() {
    let (mut sim, _) = cena(AudioSource2D {
        sound: "/nao/existe/isto.wav".into(),
        autoplay: true,
        max_distance: 10.0,
        ..AudioSource2D::default()
    });
    let mut livro = SceneAudio::default();
    let (mut eng, mut mixer) = motor();
    for _ in 0..5 {
        assert_eq!(update_with(&mut sim, &mut livro, &mut eng).started, 0);
    }
    assert_eq!(livro.live_voices(), 0);
    assert_eq!(
        livro.decoded_ok(),
        0,
        "nada devia ter sido descodificado com sucesso"
    );
    assert_eq!(
        vozes_no_mixer(&mut mixer),
        0,
        "o MIXER recebeu uma voz de um ficheiro que nao abre"
    );
}
