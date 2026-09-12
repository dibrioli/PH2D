//! Gates da membrana do `audio.bands`.
//!
//! ⚠️ O crate do nó não alcança um arquivo de som (a cerca do doc 63 §6), então é
//! **aqui** que a cadeia inteira é afirmada: o arquivo entra, a transformada roda,
//! as bandas saem, e o nó lê pela MESMA chave que o shell escreveu.

use super::*;
use crate::motion_state::MotionState;
use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_node_audio_bands::{Scale, Weighting, param};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// Uma senoide de `hz`, mono a 48 kHz.
fn tone(hz: f32, secs: f32) -> SampleData {
    let sr = 48_000.0f32;
    let n = (secs * sr) as usize;
    let s: Vec<f32> = (0..n)
        .map(|i| (std::f32::consts::TAU * hz * (i as f32 / sr)).sin() * 0.9)
        .collect();
    SampleData::from_interleaved(
        s,
        AudioFormat {
            sample_rate: 48_000,
            channels: ChannelLayout::Mono,
        },
    )
}

fn spec(count: usize) -> BandSpec {
    BandSpec {
        count,
        min_hz: 40.0,
        max_hz: 16_000.0,
        scale: Scale::Log,
        weighting: Weighting::None,
        floor_db: -60.0,
        gain: 1.0,
        smoothing: 0.0,
    }
}

/// **A banda que contém o tom acende, e as outras não.** É a cadeia inteira numa
/// asserção: transformada → eixo de frequência → corte em bandas → normalização.
#[test]
fn the_band_that_holds_the_tone_is_the_one_that_lights_up() {
    let s = spec(8);
    let track = BandTrack::build(&tone(1000.0, 0.4), &s);
    let levels = track.at(0.2);
    assert_eq!(levels.len(), 8);

    // Qual banda DEVIA acender, perguntado ao corte — nunca a um índice escrito
    // à mão, que envelheceria com a escala.
    let edges = s.edges();
    let want = (0..8)
        .find(|k| (edges[*k]..edges[k + 1]).contains(&1000.0))
        .expect("1 kHz cai numa das bandas");

    let (loud, level) = levels
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, v)| (i, *v))
        .unwrap();
    assert_eq!(loud, want, "a banda de 1 kHz e' a que acende: {levels:?}");
    assert!(level > 0.8, "e ela chega perto do topo: {level}");
    // O CONTROLE: as vizinhas ficam LONGE — sem isto, um fold que devolvesse o
    // mesmo número em toda banda passaria (o `max` escolheria a primeira).
    let others: f32 = levels
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != loud)
        .map(|(_, v)| *v)
        .fold(0.0, f32::max);
    assert!(
        others < level * 0.6,
        "vizinhas quietas: {others} vs {level}"
    );
}

/// **Silêncio é zero em toda banda** — o piso mapeia para 0, e um arquivo mudo não
/// pode dirigir nada.
#[test]
fn silence_reads_as_zero_everywhere() {
    let quiet = SampleData::from_interleaved(
        vec![0.0; 24_000],
        AudioFormat {
            sample_rate: 48_000,
            channels: ChannelLayout::Mono,
        },
    );
    let track = BandTrack::build(&quiet, &spec(6));
    assert!(
        track.at(0.25).iter().all(|v| *v == 0.0),
        "{:?}",
        track.at(0.25)
    );
}

/// **As bandas são função do INSTANTE**, e a leitura fora do clipe não entra em
/// pânico nem inventa: ela devolve a coluna de borda.
#[test]
fn the_levels_follow_the_playhead_and_the_ends_are_clamped() {
    // Meio segundo de grave seguido de meio segundo de agudo.
    let sr = 48_000usize;
    let mut s: Vec<f32> = (0..sr / 2)
        .map(|i| (std::f32::consts::TAU * 100.0 * (i as f32 / sr as f32)).sin() * 0.9)
        .collect();
    s.extend(
        (0..sr / 2).map(|i| (std::f32::consts::TAU * 6000.0 * (i as f32 / sr as f32)).sin() * 0.9),
    );
    let clip = SampleData::from_interleaved(
        s,
        AudioFormat {
            sample_rate: 48_000,
            channels: ChannelLayout::Mono,
        },
    );
    let track = BandTrack::build(&clip, &spec(8));
    let low_at = |t: f64| track.at(t)[..4].iter().fold(0.0f32, |a, b| a.max(*b));
    let high_at = |t: f64| track.at(t)[4..].iter().fold(0.0f32, |a, b| a.max(*b));

    assert!(low_at(0.25) > high_at(0.25), "no comeco manda o grave");
    assert!(high_at(0.75) > low_at(0.75), "no fim manda o agudo");
    // Fora do clipe: sem pânico, e SEMPRE a mesma coluna de borda.
    //
    // ⚠️ A 1ª versão disto comparava `at(99.0)` com `at(0.9999)` e nasceu VERMELHA
    // sobre produto correto: a última coluna da análise olha para o **zero-padding**
    // que a transformada põe no fim, então ela é silêncio enquanto `0,9999 s` ainda
    // pega o tom. *A borda de uma análise não é o último instante do áudio*, e o que
    // se afirma é o CLAMP, não que ele caia sobre som.
    assert_eq!(track.at(99.0).len(), 8);
    assert_eq!(track.at(99.0), track.at(1e9));
    assert_eq!(track.at(-5.0), track.at(0.0));
}

/// ⚠️ **O suavizado é um filtro sobre a GRAVAÇÃO, então o scrub é EXATO.** Ler o
/// mesmo instante duas vezes, com um salto pelo meio, dá o mesmo número **ao bit**
/// — a propriedade que um one-pole por-quadro destruiria, e a razão de o
/// suavizado morar na construção da matriz em vez do laço de publicação.
#[test]
fn scrubbing_back_gives_the_same_numbers_to_the_bit() {
    let s = BandSpec {
        smoothing: 0.8,
        ..spec(8)
    };
    let track = BandTrack::build(&tone(1000.0, 0.6), &s);
    let first: Vec<f32> = track.at(0.30).to_vec();
    let _ = track.at(0.55);
    let _ = track.at(0.05);
    let again: Vec<f32> = track.at(0.30).to_vec();
    assert_eq!(first, again, "o scrub e' exato");
    // E o suavizado de facto AGE (senão o gate acima seria verde por vacuidade).
    let sharp = BandTrack::build(&tone(1000.0, 0.6), &spec(8));
    assert_ne!(
        sharp.at(0.30),
        track.at(0.30),
        "smoothing 0.8 muda os numeros"
    );
}

/// **A análise é construída UMA vez por `(arquivo, params)`**, e trocar um param
/// constrói outra. É o que impede a transformada de correr por quadro.
#[test]
fn the_analysis_is_built_once_and_a_param_change_builds_another() {
    let dir = std::env::temp_dir().join("ph2d_audio_bands_gate");
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("tone.wav");
    write_wav(&path, &tone(1000.0, 0.3));
    let p = path.to_string_lossy().to_string();

    let mut cache = BandCache::default();
    let a = spec(8);
    let ka = a.key(&p);
    assert!(!cache.track(&ka, &p, &a).at(0.1).is_empty());
    assert_eq!(cache.len(), 1);
    cache.track(&ka, &p, &a);
    assert_eq!(cache.len(), 1, "a mesma analise nao e' reconstruida");

    let b = BandSpec { count: 16, ..a };
    cache.track(&b.key(&p), &p, &b);
    assert_eq!(cache.len(), 2, "outros params, outra analise");
}

/// **Arquivo ausente NÃO entra em pânico e NÃO adivinha** — o nó fica em silêncio,
/// a política do `source.object` para um objeto que não existe.
#[test]
fn a_missing_file_is_silence_not_a_panic() {
    let mut cache = BandCache::default();
    let s = spec(8);
    let track = cache.track(&s.key("/nao/existe.wav"), "/nao/existe.wav", &s);
    assert!(track.at(0.0).is_empty());
    // ...e a ausência é memoizada (senão todo quadro tentaria abrir o mesmo nada).
    cache.track(&s.key("/nao/existe.wav"), "/nao/existe.wav", &s);
    assert_eq!(cache.len(), 1);
}

/// **A porta CRUZADA: o que o shell publica é o que o nó lê.**
///
/// ⚠️ É o gate que a chave existe para permitir — as duas metades derivam o mesmo
/// nome da MESMA função, então um param que uma delas esquecesse deixaria o cook
/// a ler um externo que ninguém escreveu (stream vazio, barras paradas).
#[test]
fn publish_then_cook_the_node_reads_its_own_bands() {
    let dir = std::env::temp_dir().join("ph2d_audio_bands_gate");
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("cross.wav");
    write_wav(&path, &tone(1000.0, 0.4));

    let reg = registry();
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("audio.bands");
    state
        .doc
        .graph
        .set_text_param(n, FILE_KEY, path.to_string_lossy().as_ref());
    state.doc.graph.set_param(n, param::COUNT, 8.0);

    publish(&mut state, 0.2);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &reg, n, 0.2)
        .expect("coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e' um stream")
    };
    // Solto, a cardinalidade e' o numero de BANDAS — *as bandas elas mesmas*.
    assert_eq!(s.count(), 8);
    let Some(Column::Scalar(v)) = Stream::get(s, VALUE_COL) else {
        panic!("emite a coluna de valor")
    };
    assert!(v.iter().any(|x| *x > 0.5), "o tom chega ao no: {v:?}");
}

/// **Com entrada ligada, o elemento `i` toma a banda `i % count`** — a *Use Index
/// Context* da referência, e o que faz a biblioteca `motion.*` agir por elemento.
#[test]
fn with_geometry_connected_each_element_takes_its_band() {
    let dir = std::env::temp_dir().join("ph2d_audio_bands_gate");
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("idx.wav");
    write_wav(&path, &tone(1000.0, 0.4));

    let reg = registry();
    let mut state = MotionState::new();
    let src = state.doc.graph.add_node("motion.scatter");
    state.doc.graph.set_param(src, "count", 20.0);
    let n = state.doc.graph.add_node("audio.bands");
    state
        .doc
        .graph
        .set_text_param(n, FILE_KEY, path.to_string_lossy().as_ref());
    state.doc.graph.set_param(n, param::COUNT, 8.0);
    state
        .doc
        .graph
        .connect(ph2d_nodegraph::graph::Edge {
            from: (src, 0),
            to: (n, 0),
            delayed: false,
        })
        .expect("liga");

    publish(&mut state, 0.2);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &reg, n, 0.2)
        .expect("coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    assert_eq!(s.count(), 20, "a cardinalidade vem da GEOMETRIA");
    let Some(Column::Scalar(v)) = Stream::get(s, VALUE_COL) else {
        panic!("coluna")
    };
    // ⚠️ **A metade que faltava, achada por MUTAÇÃO:** `v[i] == v[i % 8]` é
    // satisfeito por QUALQUER campo constante, então um nó que devolvesse sempre a
    // banda 0 passava aqui. O ciclo só significa alguma coisa se as oito bandas
    // forem de facto distintas.
    let distinct = v[..8]
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), x| (l.min(*x), h.max(*x)));
    assert!(
        distinct.1 - distinct.0 > 0.2,
        "as 8 bandas diferem entre si: {distinct:?}"
    );
    for i in 0..20 {
        assert_eq!(v[i], v[i % 8], "o elemento {i} toma a banda {}", i % 8);
    }
}

/// Um WAV mono de 32 bits float — o mínimo para o `decode_any` reabrir.
fn write_wav(path: &std::path::Path, data: &SampleData) {
    let s = data.samples();
    let bytes = s.len() * 4;
    let mut w: Vec<u8> = Vec::with_capacity(44 + bytes);
    w.extend(b"RIFF");
    w.extend(((36 + bytes) as u32).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(3u16.to_le_bytes()); // IEEE float
    w.extend(1u16.to_le_bytes()); // mono
    w.extend(48_000u32.to_le_bytes());
    w.extend((48_000u32 * 4).to_le_bytes());
    w.extend(4u16.to_le_bytes());
    w.extend(32u16.to_le_bytes());
    w.extend(b"data");
    w.extend((bytes as u32).to_le_bytes());
    for v in s {
        w.extend(v.to_le_bytes());
    }
    std::fs::write(path, w).expect("escreve o wav da fixture");
}

/// **UM PARAM CONDUZIDO POR FIO CUNHA A MESMA CHAVE DOS DOIS LADOS** — o gêmeo do
/// `publish_then_cook_the_node_reads_its_own_bands`, e o defeito que ele não via.
///
/// ⚠️ **O `eval` do nó monta a chave com `ctx.param`, que resolve `conduzido → override →
/// default`; esta membrana lia só `override → default`.** Conduza qualquer um dos oito params
/// e as duas chaves DIVERGEM: o nó pede uma análise que ninguém publicou, `levels` vem vazio, e
/// ele emite **um campo de zeros** — todas as barras planas, sem erro nenhum.
///
/// ⚠️ **O CONTROLO é o autorado DISCORDAR do fio:** o `count` autorado fica no default (`16`)
/// e só o fio diz `8`. Com o autorado já em `8` a escada errada acertaria por acidente, e o
/// gate ficaria verde sobre o defeito — que é a forma de gate vazio que este módulo já pagou.
#[test]
fn a_driven_param_mints_the_same_key_on_both_sides() {
    let dir = std::env::temp_dir().join("ph2d_audio_bands_gate");
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("driven.wav");
    write_wav(&path, &tone(1000.0, 0.4));

    let reg = registry();
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("audio.bands");
    state
        .doc
        .graph
        .set_text_param(n, FILE_KEY, path.to_string_lossy().as_ref());
    // O `count` NÃO é autorado — ele vem de um fio, e o default do manifesto é outro número.
    let default_count = MANIFEST
        .params
        .iter()
        .find(|p| p.name == param::COUNT)
        .expect("o `count` e' declarado")
        .default;
    assert_ne!(
        default_count, 8.0,
        "o controle exige que o autorado DISCORDE do fio"
    );
    let num = state.doc.graph.add_node("value.number");
    state.doc.graph.set_param(num, "value", 8.0);
    state
        .doc
        .graph
        .drive_param(n, param::COUNT, (num, 0))
        .expect("o `count` aceita fio");

    publish(&mut state, 0.2);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &reg, n, 0.2)
        .expect("coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e' um stream")
    };
    assert_eq!(
        s.count(),
        8,
        "a cardinalidade e' o `count` do FIO, nao o do manifesto"
    );
    let Some(Column::Scalar(v)) = Stream::get(s, VALUE_COL) else {
        panic!("emite a coluna de valor")
    };
    assert!(
        v.iter().any(|x| *x > 0.5),
        "e o tom CHEGA ao no' — um campo de zeros e' o sintoma exacto do defeito: {v:?}"
    );
}
