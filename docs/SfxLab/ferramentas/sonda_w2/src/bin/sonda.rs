//! A rodada de W2: variantes no mesmo processo, intercaladas, resumo = o mínimo (`docs/DevOps/MEDIR_VELOCIDADE.md`).

#[path = "../../../../../../crates/ph2d-sfx-dsp/tests/it/regua.rs"]
mod regua;

use std::time::Instant;

use ph2d_sfx_dsp::{RenderCache, render};
use ph2d_sfx_recipe::{
    Body, ENGINE_V1, Envelope, Layer, LayerId, ModeLaw, Noise, NoiseFilter, Recipe, Recording,
    Source, Take, Takes, Tone, Wave,
};
use regua::alias_db;

const SR: f32 = 48_000.0;

fn f0(t: f32) -> f32 {
    2_000.0 * 10.0f32.powf(t / 2.0)
}

// ─── aliasing: a serra em cada variante ────────────────────────────────────────────────────────

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-9 {
        1.0
    } else {
        let a = std::f64::consts::PI * x;
        a.sin() / a
    }
}

fn janela(x: f64) -> f64 {
    let t = std::f64::consts::PI * (x + 1.0);
    0.35875 - 0.48829 * t.cos() + 0.14128 * (2.0 * t).cos() - 0.01168 * (3.0 * t).cos()
}

fn residuo(raio: usize, corte: f64) -> Vec<f32> {
    let por = 256usize;
    let n = 2 * raio * por;
    let dx = 1.0 / por as f64;
    let mut d = Vec::with_capacity(n + 1);
    let mut soma = 0.0;
    for i in 0..=n {
        d.push(soma);
        let x = (i as f64 + 0.5) * dx - raio as f64;
        soma += corte * sinc(corte * x) * janela(x / raio as f64) * dx;
    }
    d.iter().map(|s| (s / soma) as f32).collect()
}

/// `None` = ingênua; `Some((0, _))` = polyBLEP; `Some((raio, corte))` = tabela.
fn serra(modo: Option<(usize, f64)>) -> Vec<f32> {
    let tab = modo.filter(|m| m.0 > 0).map(|(r, c)| (r, residuo(r, c)));
    let poly = modo.is_some_and(|m| m.0 == 0);
    let n = 96_000;
    let mut buf = vec![0.0f32; n];
    let mut fase = 0.5f64;
    for i in 0..n {
        let inc = f64::from(f0(i as f32 / SR)) / f64::from(SR);
        let mut v = 2.0 * fase - 1.0;
        if poly {
            if fase < inc {
                let x = fase / inc;
                v -= x + x - x * x - 1.0;
            } else if fase > 1.0 - inc {
                let x = (fase - 1.0) / inc;
                v -= x * x + x + x + 1.0;
            }
        }
        buf[i] += v as f32;
        let s = fase + inc;
        if s >= 1.0 {
            if let Some((raio, t)) = &tab {
                let quando = i as f64 + (1.0 - fase) / inc;
                let r = *raio as f64;
                let de = (quando - r).ceil().max(0.0) as usize;
                let ate = ((quando + r).floor() as usize).min(n);
                for (j, o) in buf.iter_mut().enumerate().take(ate).skip(de) {
                    let x = j as f64 - quando;
                    let p = (x + r) * 256.0;
                    let k = p as usize;
                    if k + 1 < t.len() {
                        let f = (p - k as f64) as f32;
                        let ideal = if x >= 0.0 { 1.0 } else { 0.0 };
                        *o += -2.0 * (t[k] + (t[k + 1] - t[k]) * f - ideal);
                    }
                }
            }
            fase = s - 1.0;
        } else {
            fase = s;
        }
    }
    buf
}

fn tom(wave: Wave, de: f32, ate: f32, seg: f32) -> Vec<f32> {
    let mut r = Recipe::new(1, seg);
    r.add(
        0.5,
        Source::Tone(Tone {
            wave,
            hz: de,
            hz_end: ate,
            glide: seg,
        }),
        Envelope::default(),
    );
    render(&r, 1, ENGINE_V1, &Takes::default())
        .expect("motor")
        .samples
}

fn aliasing() {
    println!("=== aliasing na varredura 2→20 kHz (dB sobre o sinal, pior janela; serra)");
    println!("ingênua\t{:.1}", alias_db(&serra(None), SR, &f0));
    println!("polyBLEP\t{:.1}", alias_db(&serra(Some((0, 0.0))), SR, &f0));
    for raio in [8, 16, 32] {
        for corte in [0.8, 0.9, 0.95] {
            let db = alias_db(&serra(Some((raio, corte))), SR, &f0);
            println!("tabela raio {raio} corte {corte}\t{db:.1}");
        }
    }
    println!("--- o produto, varredura");
    for (nome, w) in [
        ("seno", Wave::Sine),
        ("triângulo", Wave::Triangle),
        ("serra", Wave::Saw),
        ("quadrada", Wave::Square),
    ] {
        let db = alias_db(&tom(w, 2_000.0, 20_000.0, 2.0), SR, &f0);
        println!("{nome}\t{db:.1}");
    }
    println!("--- o produto, FM de índice 50 em 7 kHz");
    let fm = Wave::Fm {
        ratio: 1.0,
        index: 50.0,
        index_end: 50.0,
    };
    let db = alias_db(&tom(fm, 7_000.0, 7_000.0, 1.0), SR, &|_| 7_000.0);
    println!("fm\t{db:.1}");
}

// ─── o custo ───────────────────────────────────────────────────────────────────────────────────

fn camada(id: u64, gain: f32, source: Source) -> Layer {
    Layer {
        id: LayerId(id),
        gain,
        start: 0.0,
        source,
        envelope: Envelope::default(),
    }
}

fn corpo(base_hz: f32, modes: u32, t60: f32) -> Source {
    Source::Body(Body {
        law: ModeLaw::Stretched { stretch: 1.3 },
        base_hz,
        modes,
        t60,
        damping: 0.15,
        rolloff: 1.0,
        contact_ms: 2.0,
        noise: 1.0,
    })
}

/// A mesma estrutura da sonda de W0: 2 corpos de 32 modos, 2 ruídos, 2 tons, 2 FM.
fn tipica(seg: f32) -> Recipe {
    let ruido = |hz, q| {
        Source::Noise(Noise {
            filter: NoiseFilter::Band,
            hz,
            hz_end: hz,
            q,
        })
    };
    let tom = |wave, hz: f32, fim: f32| {
        Source::Tone(Tone {
            wave,
            hz,
            hz_end: fim,
            glide: seg,
        })
    };
    let fm = |r, i| Wave::Fm {
        ratio: r,
        index: i,
        index_end: i,
    };
    Recipe {
        layers: vec![
            camada(1, 0.2, corpo(180.0, 32, 4.0)),
            camada(2, 0.15, corpo(730.0, 32, 1.0)),
            camada(3, 0.15, ruido(2_400.0, 1.2)),
            camada(4, 0.1, ruido(9_000.0, 0.7)),
            camada(5, 0.1, tom(Wave::Saw, 1_760.0, 400.0)),
            camada(6, 0.1, tom(Wave::Square, 110.0, 80.0)),
            camada(7, 0.1, tom(fm(1.41, 3.0), 440.0, 440.0)),
            camada(8, 0.1, tom(fm(2.0, 8.0), 66.0, 66.0)),
        ],
        next_layer: 9,
        ..Recipe::new(0x5F3, seg)
    }
}

/// O pior caso de W0: 8 corpos de 128 modos.
fn pior(seg: f32) -> Recipe {
    Recipe {
        layers: (0..8u32)
            .map(|i| camada(u64::from(i) + 1, 0.05, corpo(90.0 + 70.0 * i as f32, 128, 4.0)))
            .collect(),
        next_layer: 9,
        ..Recipe::new(0x5F3, seg)
    }
}

fn mede(f: &mut dyn FnMut()) -> f64 {
    let t = Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1000.0
}

/// O mínimo de `rodadas` passagens rotativas por todas as variantes.
fn rodada(mut variantes: Vec<(&str, Box<dyn FnMut()>)>, rodadas: usize) {
    let n = variantes.len();
    let mut melhor = vec![f64::INFINITY; n];
    for r in 0..rodadas {
        for k in 0..n {
            let i = (k + r) % n;
            let ms = mede(&mut *variantes[i].1);
            melhor[i] = melhor[i].min(ms);
        }
    }
    for (i, (nome, _)) in variantes.iter().enumerate() {
        println!("{nome}\t{:.2} ms", melhor[i]);
    }
}

fn custo() {
    println!("\n=== custo de render, ms (mínimo de 7 rodadas intercaladas)");
    let t = Takes::default();
    let mut v: Vec<(&str, Box<dyn FnMut()>)> = Vec::new();
    for (nome, r) in [
        ("típica 2 s, sem cache", tipica(2.0)),
        ("pior 2 s, sem cache", pior(2.0)),
    ] {
        let t = t.clone();
        v.push((
            nome,
            Box::new(move || {
                let _ = render(&r, r.seed, ENGINE_V1, &t);
            }),
        ));
    }
    for (nome, base, muda_fonte) in [
        ("típica 2 s, edita o ruído (camada 3)", tipica(2.0), 2usize),
        ("típica 2 s, edita o corpo (camada 1)", tipica(2.0), 0),
        ("pior 2 s, edita 1 corpo", pior(2.0), 0),
    ] {
        let t = t.clone();
        let mut cache = RenderCache::default();
        let _ = cache.render(&base, base.seed, ENGINE_V1, &t);
        let mut r = base;
        let mut lado = false;
        v.push((
            nome,
            Box::new(move || {
                lado = !lado;
                match &mut r.layers[muda_fonte].source {
                    Source::Noise(n) => n.q = if lado { 1.3 } else { 1.2 },
                    Source::Body(b) => b.damping = if lado { 0.16 } else { 0.15 },
                    _ => {}
                }
                let _ = cache.render(&r, r.seed, ENGINE_V1, &t);
                assert_eq!(cache.last_rendered, 1);
            }),
        ));
    }
    for (nome, base) in [
        ("típica 2 s, só o ganho", tipica(2.0)),
        ("pior 2 s, só o ganho", pior(2.0)),
    ] {
        let t = t.clone();
        let mut cache = RenderCache::default();
        let _ = cache.render(&base, base.seed, ENGINE_V1, &t);
        let mut r = base;
        let mut lado = false;
        v.push((
            nome,
            Box::new(move || {
                lado = !lado;
                r.layers[0].gain = if lado { 0.21 } else { 0.2 };
                let _ = cache.render(&r, r.seed, ENGINE_V1, &t);
                assert_eq!(cache.last_rendered, 0);
            }),
        ));
    }
    rodada(v, 7);

    println!("\n=== uma camada de 2 s, por fonte (mínimo)");
    let mut v: Vec<(&str, Box<dyn FnMut()>)> = Vec::new();
    for (nome, s) in [
        ("corpo 32 modos t60 4 s", corpo(180.0, 32, 4.0)),
        ("corpo 128 modos t60 4 s", corpo(90.0, 128, 4.0)),
        ("corpo 16 modos t60 0,2 s (cala cedo)", corpo(300.0, 16, 0.2)),
        (
            "ruído com corte fixo",
            Source::Noise(Noise {
                filter: NoiseFilter::Band,
                hz: 2_400.0,
                hz_end: 2_400.0,
                q: 1.2,
            }),
        ),
        (
            "ruído com corte varrendo",
            Source::Noise(Noise {
                filter: NoiseFilter::Band,
                hz: 300.0,
                hz_end: 8_000.0,
                q: 1.2,
            }),
        ),
        (
            "serra 15 kHz",
            Source::Tone(Tone {
                wave: Wave::Saw,
                hz: 15_000.0,
                hz_end: 15_000.0,
                glide: 0.0,
            }),
        ),
    ] {
        let t = Takes::default();
        let mut r = Recipe::new(1, 2.0);
        r.add(0.5, s, Envelope::default());
        v.push((
            nome,
            Box::new(move || {
                let _ = render(&r, 1, ENGINE_V1, &t);
            }),
        ));
    }
    rodada(v, 7);

    println!("\n=== gravação: 2 s de saída, material de 10 s a 44,1 kHz (mínimo); o teto de semitons");
    let mut takes = Takes::default();
    let seno: Vec<f32> = (0..441_000)
        .map(|i| (f64::from(i) * 0.02).sin() as f32)
        .collect();
    let id = takes.add(Take::new(44_100, 1, seno));
    let mut v: Vec<(&str, Box<dyn FnMut()>)> = Vec::new();
    for (nome, semitons) in [
        ("-48", -48.0f32),
        ("-24", -24.0),
        ("0 (44,1→48)", 0.0),
        ("+12", 12.0),
        ("+24", 24.0),
        ("+48", 48.0),
    ] {
        let t = takes.clone();
        let mut r = Recipe::new(1, 2.0);
        r.add(
            1.0,
            Source::Recording(Recording {
                take: id,
                semitones: semitons,
                offset: 0.0,
            }),
            Envelope::default(),
        );
        v.push((
            nome,
            Box::new(move || {
                let _ = render(&r, 1, ENGINE_V1, &t);
            }),
        ));
    }
    rodada(v, 5);

    println!("\n=== a duração: a pior receita de W0, sem cache; a RAM do cache é a conta 4 B × 48 000 × s × camadas");
    for seg in [2.0f32, 30.0, 120.0] {
        let r = pior(seg);
        let t = Takes::default();
        let ms = mede(&mut || {
            let _ = render(&r, r.seed, ENGINE_V1, &t);
        });
        let mb = 4.0 * 48_000.0 * f64::from(seg) * 8.0 / 1.0e6;
        println!("{seg} s\t{ms:.0} ms\t{mb:.0} MB no cache");
    }
}

fn grava_wav(caminho: &str, pcm: &[f32]) {
    let dados = pcm.len() as u32 * 4;
    let mut b = Vec::with_capacity(dados as usize + 44);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + dados).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&48_000u32.to_le_bytes());
    b.extend_from_slice(&192_000u32.to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&dados.to_le_bytes());
    for s in pcm {
        b.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(caminho, b).expect("grava o wav");
}

/// Os arquivos que o `corre.sh` entrega ao `sox`.
fn wavs(dir: &str) {
    for (nome, w) in [
        ("serra", Wave::Saw),
        ("quadrada", Wave::Square),
        ("triangulo", Wave::Triangle),
    ] {
        for hz in [5_000.0f32, 9_000.0, 15_000.0] {
            grava_wav(&format!("{dir}/{nome}_{hz}.wav"), &tom(w, hz, hz, 2.0));
        }
    }
}

fn main() {
    let dir = std::env::args().nth(1).expect("a pasta de saída");
    aliasing();
    custo();
    wavs(&dir);
}
