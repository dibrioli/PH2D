//! `aliasing` medido com tom agudo varrido. Oráculo: `sox`, só executado, por
//! `docs/SfxLab/ferramentas/sonda_w2/corre.sh`; o limite vem da medição em `docs/SfxLab/03_W2_medicao.md`.

use ph2d_sfx_dsp::render;
use ph2d_sfx_recipe::{ENGINE_V1, Envelope, Recipe, Source, Take, Takes, Tone, Wave};

use crate::regua::{abaixo_de_db, alias_db};

const SR: f32 = 48_000.0;

fn tom(wave: Wave, de: f32, ate: f32, segundos: f32) -> Vec<f32> {
    let mut r = Recipe::new(1, segundos);
    r.add(
        0.5,
        Source::Tone(Tone {
            wave,
            hz: de,
            hz_end: ate,
            glide: segundos,
        }),
        Envelope::default(),
    );
    render(&r, 1, ENGINE_V1, &Takes::default()).unwrap().samples
}

/// A varredura de 2 kHz a 20 kHz em 2 s.
fn fundamental(t: f32) -> f32 {
    2_000.0 * 10.0f32.powf(t / 2.0)
}

/// A serra ingênua, para a régua provar que enxerga o fenômeno.
fn serra_ingenua() -> Vec<f32> {
    let mut fase = 0.0f64;
    (0..96_000)
        .map(|i| {
            let f = f64::from(fundamental(i as f32 / SR));
            let s = 2.0 * fase - 1.0;
            fase = (fase + f / f64::from(SR)).fract();
            s as f32 * 0.5
        })
        .collect()
}

/// A mesma ingênua a uma altura só: a linha de controlo do oráculo.
fn serra_ingenua_fixa(hz: f32) -> Vec<f32> {
    let inc = f64::from(hz) / f64::from(SR);
    (0..96_000)
        .map(|i| {
            let fase = (f64::from(i) * inc).fract();
            (2.0 * fase - 1.0) as f32 * 0.5
        })
        .collect()
}

/// O pior `aliasing` admitido, em dB sobre o sinal. Medido na mesma régua (`03_W2_medicao.md`): serra ingênua −7,8,
/// polyBLEP −33,6, o produto abaixo de −87.
const LIMITE_DB: f64 = -80.0;

#[test]
fn a_regua_enxerga_o_aliasing_de_uma_serra_ingenua() {
    let db = alias_db(&serra_ingenua(), SR, &fundamental);
    assert!(db > -20.0, "a régua não viu a serra ingênua: {db:.1} dB");
}

#[test]
fn aliasing_de_tom_agudo_varrido() {
    for (nome, onda) in [
        ("seno", Wave::Sine),
        ("triângulo", Wave::Triangle),
        ("serra", Wave::Saw),
        ("quadrada", Wave::Square),
    ] {
        let pcm = tom(onda, 2_000.0, 20_000.0, 2.0);
        let db = alias_db(&pcm, SR, &fundamental);
        assert!(db < LIMITE_DB, "{nome}: {db:.1} dB, limite {LIMITE_DB}");
    }
}

/// A banda de Carson presa: um índice absurdo em tom agudo não dobra. Modulador igual à portadora, então o espectro
/// são múltiplos dela e nada cai abaixo da fundamental, salvo o que dobrou.
#[test]
fn fm_nao_dobra_em_nyquist_com_indice_alto() {
    let onda = Wave::Fm {
        ratio: 1.0,
        index: 50.0,
        index_end: 50.0,
    };
    let pcm = tom(onda, 7_000.0, 7_000.0, 1.0);
    let db = alias_db(&pcm, SR, &|_| 7_000.0);
    assert!(db < LIMITE_DB, "FM: {db:.1} dB, limite {LIMITE_DB}");
}

/// A leitura mais rápida de um material agudo: acima de Nyquist ela tem de cortar, não dobrar.
#[test]
fn gravacao_acelerada_nao_dobra() {
    // Em `f64`: a fase em `f32` tem ruído próprio a −44 dB, e a régua leria a fixture em vez do kernel.
    let seno: Vec<f32> = (0..96_000)
        .map(|i| (std::f64::consts::TAU * 15_000.0 * f64::from(i) / 48_000.0).sin() as f32 * 0.5)
        .collect();
    let take = Take::new(48_000, 1, seno);
    let mut takes = Takes::default();
    let id = takes.add(take);
    // Uma oitava acima: 30 kHz, que dobraria para 18 kHz sem o filtro.
    let mut r = Recipe::new(1, 0.5);
    r.add(
        1.0,
        Source::Recording(ph2d_sfx_recipe::Recording {
            take: id,
            semitones: 12.0,
            offset: 0.0,
        }),
        Envelope::default(),
    );
    let pcm = render(&r, 1, ENGINE_V1, &takes).unwrap().samples;
    let pico = pcm[2048..22_000].iter().fold(0.0f32, |m, s| m.max(s.abs()));
    assert!(pico < 0.000_5, "o tom dobrado passou: pico {pico}");
}

/// **O oráculo:** o `sox` filtra o mesmo render abaixo da fundamental e mede o que sobra, sem passar por nenhuma
/// conta nossa. Linhas em `fixtures/aliasing_sox.tsv`, com cabeçalho, geradas pelo `corre.sh`.
#[test]
fn a_regua_bate_com_o_sox() {
    let tsv = include_str!("fixtures/aliasing_sox.tsv");
    let mut vistas = 0;
    for linha in tsv.lines().filter(|l| !l.starts_with('#')) {
        let c: Vec<&str> = linha.split('\t').collect();
        let (onda, hz): (&str, f32) = (c[0], c[1].parse().unwrap());
        let sox_db: f64 = c[2].parse().unwrap();
        let pcm = match onda {
            "serra" => tom(Wave::Saw, hz, hz, 2.0),
            "quadrada" => tom(Wave::Square, hz, hz, 2.0),
            "triangulo" => tom(Wave::Triangle, hz, hz, 2.0),
            "ingenua" => serra_ingenua_fixa(hz),
            _ => panic!("{onda}"),
        };
        let nosso = abaixo_de_db(&pcm, SR, 0.8 * hz);
        // Abaixo de −110 dB os dois leem o ruído numérico da própria régua, não o sinal.
        let diferenca = (nosso.max(-110.0) - sox_db.max(-110.0)).abs();
        assert!(
            diferenca < 3.0,
            "{onda} {hz} Hz: nós {nosso:.1} dB, sox {sox_db:.1} dB"
        );
        vistas += 1;
    }
    assert!(vistas >= 10, "a fixture só tem {vistas} linhas");
}
