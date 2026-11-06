//! Sonda de velocidade (`docs/DevOps/MEDIR_VELOCIDADE.md`): variantes no mesmo processo, intercaladas, o mínimo. O resultado
//! vai para `docs/SfxLab/04_W3_plano.md` §6.
//! `cargo test --profile smoke -p ph2d-panel-sfxlab --test it diag_ -- --ignored --nocapture --test-threads=1`

use crate::Mesa;
use ph2d_audio::dsp::integrated_lufs;
use ph2d_panel_sfxlab::medidor::{espectro, pico_db};
use ph2d_sfx_recipe::{Envelope, Noise, NoiseFilter, Recipe, SAMPLE_RATE, Source};
use std::hint::black_box;
use std::time::Instant;

const BLOCOS: usize = 7;
const QUADROS_POR_BLOCO: usize = 30;

fn com_camadas(n: usize) -> Recipe {
    let mut r = Recipe::new(3, 1.0);
    for _ in 0..n {
        r.add(
            0.1,
            Source::Noise(Noise {
                filter: NoiseFilter::Band,
                hz: 800.0,
                hz_end: 800.0,
                q: 1.0,
            }),
            Envelope::hit(0.001, 0.4, 0.01),
        );
    }
    r
}

#[test]
#[ignore = "sonda: mede, não julga"]
fn diag_tempo_por_quadro_com_8_e_64_camadas() {
    let mut mesas: Vec<(usize, Mesa)> = [8, 64]
        .map(|n| {
            let mut m = Mesa::nova();
            let id = m.abre("Sonda", com_camadas(n));
            m.espera(id);
            (n, m)
        })
        .into_iter()
        .collect();
    let mut minimo = [f64::INFINITY; 2];
    for bloco in 0..BLOCOS {
        for k in 0..mesas.len() {
            let i = (k + bloco) % mesas.len();
            let t = Instant::now();
            for _ in 0..QUADROS_POR_BLOCO {
                mesas[i].1.quadro();
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / QUADROS_POR_BLOCO as f64;
            minimo[i] = minimo[i].min(ms);
        }
    }
    for (i, (n, _)) in mesas.iter().enumerate() {
        println!(
            "quadro, {n:>2} camadas: {:.3} ms (mínimo de {BLOCOS} blocos)",
            minimo[i]
        );
    }
}

#[test]
#[ignore = "sonda: mede, não julga"]
fn diag_custo_dos_medidores_por_render() {
    let de = |s: f32| -> Vec<f32> {
        let mut x = 0x9E37_79B9_u32;
        (0..(s * SAMPLE_RATE as f32) as usize)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x as f32 / u32::MAX as f32) - 0.5
            })
            .collect()
    };
    for (nome, buf) in [("2 s", de(2.0)), ("120 s", de(120.0))] {
        let mut minimo = [f64::INFINITY; 3];
        for bloco in 0..BLOCOS {
            for k in 0..3 {
                let i = (k + bloco) % 3;
                let t = Instant::now();
                match i {
                    0 => {
                        black_box(pico_db(&buf));
                    }
                    1 => {
                        black_box(integrated_lufs(&buf, 1, SAMPLE_RATE));
                    }
                    _ => {
                        black_box(espectro(&buf, SAMPLE_RATE));
                    }
                }
                minimo[i] = minimo[i].min(t.elapsed().as_secs_f64() * 1000.0);
            }
        }
        println!(
            "{nome:>6}: pico {:.3} ms · LUFS {:.3} ms · espectro {:.3} ms",
            minimo[0], minimo[1], minimo[2]
        );
    }
}
