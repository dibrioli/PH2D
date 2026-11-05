//! A rodada de W0, uma só: custo intercalado no mesmo processo (mínimo + mediana), semente com
//! vizinho editado, hash do rack e volume contra `ffmpeg ebur128` e `sox stats`.
//! Uso: `sonda <pasta de trabalho>`.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_edit::Effect;
use sonda_w0::{Camada, Receita, fd, hash, kernels, receita_pior, receita_tipica};

const RODADAS: usize = 7;
const SR: u32 = 48_000;

fn formato() -> AudioFormat {
    AudioFormat {
        sample_rate: SR,
        channels: ChannelLayout::Stereo,
    }
}

fn estereo(mono: &[f32]) -> Vec<f32> {
    mono.iter().flat_map(|s| [*s, *s]).collect()
}

fn rack(pcm: &[f32]) -> Vec<f32> {
    let mut d = SampleData::from_interleaved(estereo(pcm), formato());
    for e in [
        Effect::Compress {
            threshold: 0.3,
            ratio: 4.0,
            attack_secs: 0.005,
            release_secs: 0.1,
        },
        Effect::LowPass {
            cutoff: 6_000.0,
            q: 0.707,
        },
        Effect::Limiter {
            ceiling_db: -1.0,
            release_secs: 0.05,
        },
    ] {
        d = e.apply(&d);
    }
    d.samples().to_vec()
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_owned()
}

fn medir() {
    type Corrida = (&'static str, usize, Box<dyn Fn() -> u64>);
    let v =
        |nome, frames: usize, r: fn(usize) -> Receita, f: fn(&Receita) -> Vec<f32>| -> Corrida {
            (nome, frames, Box::new(move || hash(&f(&r(frames)))))
        };
    let mut var: Vec<Corrida> = Vec::new();
    for frames in [24_000usize, 96_000] {
        var.push(v("tipica kernels", frames, receita_tipica, kernels::render));
        var.push(v(
            "tipica fundsp tick",
            frames,
            receita_tipica,
            fd::render_tick,
        ));
        var.push(v(
            "tipica fundsp bloco",
            frames,
            receita_tipica,
            fd::render_bloco,
        ));
        var.push(v("pior kernels", frames, receita_pior, kernels::render));
        var.push(v(
            "pior fundsp bloco",
            frames,
            receita_pior,
            fd::render_bloco,
        ));
        var.push((
            "tipica kernels + rack",
            frames,
            Box::new(move || hash(&rack(&kernels::render(&receita_tipica(frames))))),
        ));
    }
    var.push((
        "fundsp só montar (tipica)",
        0,
        Box::new(|| {
            let n = fd::monta(&receita_tipica(1));
            std::hint::black_box(&n);
            0
        }),
    ));
    let camadas: [(&str, Camada); 4] = [
        (
            "camada modal 32",
            Camada::Modal {
                modos: 32,
                base: 180.0,
                t60: 4.0,
            },
        ),
        (
            "camada ruido",
            Camada::Ruido {
                hz: 2_400.0,
                q: 1.2,
                t60: 0.5,
            },
        ),
        (
            "camada tom",
            Camada::Tom {
                hz: 1_760.0,
                queda: 3.0,
                t60: 0.4,
            },
        ),
        (
            "camada fm",
            Camada::Fm {
                hz: 440.0,
                razao: 1.41,
                indice: 3.0,
                t60: 0.8,
            },
        ),
    ];
    for (nome, c) in camadas {
        var.push((
            nome,
            96_000,
            Box::new(move || {
                hash(&kernels::render(&Receita {
                    semente: 1,
                    frames: 96_000,
                    camadas: vec![(1, c, 0.2)],
                }))
            }),
        ));
    }

    println!("loadavg antes: {}", loadavg());
    for c in &var {
        let _ = (c.2)(); // aquece fora da régua
    }
    let mut tempos = vec![Vec::with_capacity(RODADAS); var.len()];
    let mut digitos = vec![0u64; var.len()];
    for rodada in 0..RODADAS {
        for k in 0..var.len() {
            let i = (k + rodada) % var.len();
            let t = Instant::now();
            digitos[i] = (var[i].2)();
            tempos[i].push(t.elapsed().as_secs_f64() * 1_000.0);
        }
    }
    println!("loadavg depois: {}", loadavg());
    println!("| variante | duração (s) | mínimo (ms) | mediana (ms) | hash |");
    println!("|---|---|---|---|---|");
    for (i, c) in var.iter().enumerate() {
        tempos[i].sort_by(f64::total_cmp);
        println!(
            "| {} | {:.1} | {:.2} | {:.2} | {:016x} |",
            c.0,
            c.1 as f64 / f64::from(SR),
            tempos[i][0],
            tempos[i][RODADAS / 2],
            digitos[i]
        );
    }
}

fn semente() {
    println!("\ncaso: base · parâmetro do vizinho · nó a mais no vizinho · vizinho depois do alvo");
    let k = kernels::vizinho(48_000);
    println!("kernels: {}", veredito(&k));
    for (nome, h) in fd::vizinho(48_000) {
        println!("fundsp {nome}: {}", veredito(&h));
    }
}

fn veredito(h: &[u64; 4]) -> String {
    let marca = |x: u64| if x == h[0] { "igual" } else { "MUDOU" };
    format!("{} · {} · {}", marca(h[1]), marca(h[2]), marca(h[3]))
}

fn rack_hash() {
    let pcm = kernels::render(&receita_tipica(96_000));
    println!(
        "\nrack duas vezes no mesmo processo: {:016x} {:016x}",
        hash(&rack(&pcm)),
        hash(&rack(&pcm))
    );
}

fn escreve_wav(p: &Path, estereo: &[f32]) {
    let bytes = estereo.len() * 4;
    let mut b = Vec::with_capacity(bytes + 44);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&((36 + bytes) as u32).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&3u16.to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&SR.to_le_bytes());
    b.extend_from_slice(&(SR * 8).to_le_bytes());
    b.extend_from_slice(&8u16.to_le_bytes());
    b.extend_from_slice(&32u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&(bytes as u32).to_le_bytes());
    for s in estereo {
        b.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(p, b).expect("gravar o wav");
}

fn numero_depois(texto: &str, chave: &str) -> Option<f32> {
    let i = texto.rfind(chave)?;
    texto[i + chave.len()..]
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn volume(pasta: &Path) {
    let sr = SR as usize;
    let seno = |db: f32, n: usize| -> Vec<f32> {
        let a = 10f32.powf(db / 20.0);
        (0..n)
            .map(|i| a * (std::f32::consts::TAU * 1_000.0 * i as f32 / SR as f32).sin())
            .collect()
    };
    // Normalizada a -3 dBFS: a mistura crua passa de 0 dB, e o `sox stats` satura em 0.
    let mut receita = kernels::render(&receita_tipica(2 * sr));
    let pico_cru = receita.iter().fold(0f32, |m, s| m.max(s.abs()));
    for s in &mut receita {
        *s *= 0.707 / pico_cru;
    }
    let curto = kernels::render(&Receita {
        semente: 3,
        frames: sr * 15 / 100,
        camadas: vec![(
            1,
            Camada::Ruido {
                hz: 3_000.0,
                q: 0.8,
                t60: 0.08,
            },
            0.6,
        )],
    });
    let mut com_silencio = kernels::render(&Receita {
        semente: 3,
        frames: sr,
        camadas: vec![(
            1,
            Camada::Ruido {
                hz: 1_000.0,
                q: 0.8,
                t60: 0.9,
            },
            0.6,
        )],
    });
    com_silencio.resize(10 * sr, 0.0);
    let casos: [(&str, Vec<f32>); 4] = [
        ("seno 1 kHz -23 dBFS, 5 s", seno(-23.0, 5 * sr)),
        ("receita típica, 2 s", receita),
        ("golpe curto, 150 ms", curto),
        ("1 s de som + 9 s de silêncio", com_silencio),
    ];
    println!(
        "\n| fixture | nosso LUFS | ebur128 LUFS | dif. | nosso pico dB | sox pico dB | nosso true peak dB | ebur128 true peak dB |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for (i, (nome, mono)) in casos.iter().enumerate() {
        let st = estereo(mono);
        let p = pasta.join(format!("fixture_{i}.wav"));
        escreve_wav(&p, &st);
        let nosso = ph2d_audio::dsp::integrated_lufs(&st, 2, SR);
        let pico = st.iter().fold(0f32, |m, s| m.max(s.abs()));
        let tp = ph2d_audio_edit::true_peak(&SampleData::from_interleaved(st.clone(), formato()));
        let ff = Command::new("ffmpeg")
            .args(["-nostats", "-hide_banner", "-i"])
            .arg(&p)
            .args(["-af", "ebur128=peak=true", "-f", "null", "-"])
            .output()
            .expect("ffmpeg");
        let ff = String::from_utf8_lossy(&ff.stderr).into_owned();
        let sox = Command::new("sox")
            .arg(&p)
            .args(["-n", "stats"])
            .output()
            .expect("sox");
        let sox = String::from_utf8_lossy(&sox.stderr).into_owned();
        let db = |x: f32| 20.0 * x.max(1e-9).log10();
        let i_ff = numero_depois(&ff, "I:").unwrap_or(f32::NAN);
        println!(
            "| {nome} | {nosso:.2} | {i_ff:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
            nosso - i_ff,
            db(pico),
            numero_depois(&sox, "Pk lev dB").unwrap_or(f32::NAN),
            db(tp),
            numero_depois(&ff, "Peak:").unwrap_or(f32::NAN),
        );
    }
}

fn main() {
    let pasta = std::env::args().nth(1).expect("uso: sonda <pasta>");
    medir();
    semente();
    rack_hash();
    volume(Path::new(&pasta));
}
