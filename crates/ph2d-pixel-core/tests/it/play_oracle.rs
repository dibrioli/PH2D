//! **P5 contra o Trial**: tags aninhadas e sobrepostas, *Play Subtags & Repetitions* ligado e
//! desligado, *Play Once*, *Play All* e as velocidades, tocados sobre as entradas NOSSAS do
//! `trial_p5_play.py` (8 quadros de 100 ms; `s0..s4`) e comparados com a sequência do vídeo
//! (`oracle_trial_play_p5.json`, `docs/PixelArt/ferramentas/oraculo_timeline.md` «Ronda 3»).

use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::play::{Play, PlayOptions, PlaySpeed};
use ph2d_pixel_core::{Background, ColorMode, Direction, NewSprite, Sprite, apply};
use serde_json::Value;

fn fx() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_trial_play_p5.json")).expect("json")
}

/// A entrada `nome` (o `gera` do condutor): `(nome, de, até, direcção, repeat)` por tag.
fn entrada(nome: &str) -> Sprite {
    let tags: &[(&str, u32, u32, Direction, u16)] = match nome {
        "s1" => &[
            ("out", 1, 6, Direction::Forward, 0),
            ("in", 2, 4, Direction::PingPong, 2),
        ],
        "s2" => &[
            ("out", 1, 6, Direction::Forward, 2),
            ("in", 2, 4, Direction::Reverse, 0),
        ],
        "s3" => &[
            ("a", 1, 4, Direction::Forward, 0),
            ("b", 3, 6, Direction::Forward, 0),
        ],
        "s4" => &[
            ("in", 2, 4, Direction::PingPong, 2),
            ("out", 1, 6, Direction::Forward, 0),
        ],
        _ => &[],
    };
    let mut s = Sprite::new(NewSprite {
        width: 4,
        height: 4,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite");
    for _ in 1..8 {
        let after = s.frames().last().expect("último").id;
        let (op, _) = frames::new_frame(&mut s, after, NewFrame::Empty).expect("frame");
        apply(&mut s, op).expect("frame");
    }
    let ids: Vec<_> = s.frames().iter().map(|f| f.id).collect();
    if let Some(op) = frames::set_duration(&s, &ids, 100) {
        apply(&mut s, op).expect("ms");
    }
    for (i, &(n, a, b, d, r)) in tags.iter().enumerate() {
        let op = frames::new_tag(&s, n, a, b).expect("tag");
        apply(&mut s, op).expect("tag");
        if let Some(op) = frames::edit_tag(&s, i, |t| {
            t.direction = d;
            t.repeat = r;
        }) {
            apply(&mut s, op).expect("props");
        }
    }
    s
}

fn velocidade(v: &Value) -> PlaySpeed {
    match v.as_str().unwrap_or("1x").split(' ').next() {
        Some("0.25x") => PlaySpeed::Quarter,
        Some("0.5x") => PlaySpeed::Half,
        Some("1.5x") => PlaySpeed::OneAndHalf,
        Some("2x") => PlaySpeed::Two,
        Some("3x") => PlaySpeed::Three,
        _ => PlaySpeed::One,
    }
}

/// A sequência que o vídeo veria: o quadro de partida (mostrado antes do Enter), depois cada passo,
/// com os repetidos seguidos fundidos (o vídeo só vê mudanças de cor).
fn tocada(s: &Sprite, start: usize, opt: PlayOptions, n: usize) -> Vec<usize> {
    let mut p = Play::start(s, start, opt);
    let mut seq = vec![start];
    let mut guarda = 0;
    let empurra = |seq: &mut Vec<usize>, i: usize| {
        if seq.last() != Some(&i) {
            seq.push(i);
        }
    };
    empurra(&mut seq, p.index);
    while p.playing && seq.len() < n && guarda < 1000 {
        p.step(s);
        empurra(&mut seq, p.index);
        guarda += 1;
    }
    seq
}

#[test]
fn nested_tags_subtags_once_and_all_play_the_sequence_the_trial_played() {
    let f = fx();
    let mut vistos = 0;
    for corrida in ["corrida1", "corrida2"] {
        for (nome, c) in f[corrida].as_object().expect("casos") {
            // Sem a opção que o caso diz (o clique tocou outro item), só depois de parar, ou só a
            // velocidade da 1.ª corrida (refeita na 2.ª): fora.
            let clique = c["clique_real"].as_str().unwrap_or("");
            let seq: Vec<usize> = c["seq"]
                .as_array()
                .expect("seq")
                .iter()
                .map(|v| v.as_u64().expect("i") as usize)
                .collect();
            if !clique.is_empty() || seq.len() < 3 || (corrida == "corrida1" && nome.contains("_v"))
            {
                continue;
            }
            let e = &c["estado"];
            let liga = |k: &str| e[k].as_u64() == Some(1);
            let opt = PlayOptions {
                once: liga("once"),
                all: liga("all"),
                subtags: liga("sub"),
            };
            let entrada_nome = nome
                .trim_start_matches("fix_")
                .split('_')
                .next()
                .expect("s");
            let start = nome
                .split('_')
                .find_map(|p| {
                    p.strip_prefix('f')
                        .and_then(|k| k.trim_end_matches('b').parse().ok())
                })
                .unwrap_or(0);
            let s = entrada(entrada_nome);
            let got = tocada(&s, start, opt, seq.len());
            assert_eq!(got, seq, "{corrida}/{nome} ({opt:?}, desde {start})");
            vistos += 1;
        }
    }
    assert!(vistos >= 40, "{vistos}");
}

/// Cada quadro dura `100 ms ÷ velocidade` (a 2.ª corrida; ±20 ms do vídeo a 50 q/s).
#[test]
fn playback_speed_divides_every_frame_duration() {
    let f = fx();
    let mut vistos = 0;
    for (nome, c) in f["corrida2"].as_object().expect("casos") {
        if !nome.starts_with("s1_v") || nome.ends_with("_set") || nome.ends_with("_reaberto") {
            continue;
        }
        let v = velocidade(&c["estado"]["speed"]);
        let ms = 100.0 * 4.0 / f64::from(v.quarters());
        let runs = c["runs_idx_ms"].as_array().expect("runs");
        for r in &runs[1..runs.len() - 1] {
            let m = r[1].as_f64().expect("ms");
            assert!((m - ms).abs() <= 20.0, "{nome}: {m} ms, devia {ms}");
        }
        let s = entrada("s1");
        let mut p = Play::start(&s, 0, PlayOptions::default());
        p.speed = v;
        let m = 400_u32.div_ceil(v.quarters());
        assert!(!p.advance(&s, m - 1), "{nome}: ainda no quadro 0");
        assert!(p.advance(&s, 1), "{nome}: muda ao fim de {m} ms");
        vistos += 1;
    }
    assert_eq!(vistos, 6);
}

/// As filas dos rótulos (lei do Trial): a de dentro por baixo, as sobrepostas desfasadas, as
/// separadas e a sozinha na fila 0.
#[test]
fn tag_rows_put_the_inner_tag_below_and_overlaps_apart() {
    use ph2d_pixel_core::play::tag_rows;
    assert_eq!(tag_rows(&[(1, 6)]), [0], "sozinha");
    assert_eq!(tag_rows(&[(1, 6), (2, 4)]), [0, 1], "aninhadas (cena 17)");
    assert_eq!(
        tag_rows(&[(2, 4), (1, 6)]),
        [1, 0],
        "a ordem da lista não manda"
    );
    assert_eq!(
        tag_rows(&[(0, 7), (1, 6), (2, 4)]),
        [0, 1, 2],
        "três níveis"
    );
    assert_eq!(
        tag_rows(&[(0, 3), (2, 5)]),
        [0, 1],
        "sobrepostas sem se conter"
    );
    assert_eq!(tag_rows(&[(0, 1), (3, 5)]), [0, 0], "separadas");
    assert_eq!(
        tag_rows(&[(0, 7), (1, 2), (4, 5)]),
        [0, 1, 1],
        "irmãs no mesmo nível"
    );
    assert_eq!(
        tag_rows(&[(2, 4), (2, 4)]),
        [0, 1],
        "o mesmo trecho: a 1.ª é a de fora"
    );
}
