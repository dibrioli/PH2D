//! A faixa de tela de cada campo contra o motor: as duas pontas soam, e nada sai não-número.

use ph2d_panel_sfxlab::tabela::CAMPOS;
use ph2d_sfx_recipe::{
    Body, Envelope, ModeLaw, Noise, NoiseFilter, Recipe, Recording, Source, Take, Takes, Tone, Wave,
};
use std::time::Instant;

const CURTO: f32 = 0.25;

fn fontes(take: [u8; 32]) -> Vec<Source> {
    vec![
        Source::Noise(Noise {
            filter: NoiseFilter::Band,
            hz: 800.0,
            hz_end: 2000.0,
            q: 2.0,
        }),
        Source::Tone(Tone {
            wave: Wave::Fm {
                ratio: 2.0,
                index: 1.0,
                index_end: 3.0,
            },
            hz: 440.0,
            hz_end: 880.0,
            glide: 0.1,
        }),
        Source::Body(Body {
            law: ModeLaw::Stretched { stretch: 1.5 },
            base_hz: 420.0,
            modes: 12,
            t60: 0.2,
            damping: 0.1,
            rolloff: 0.8,
            contact_ms: 1.0,
            noise: 0.2,
        }),
        Source::Recording(Recording {
            take,
            semitones: 0.0,
            offset: 0.0,
        }),
    ]
}

/// `cargo test … -- --nocapture` imprime a tabela anotada em `04_W3_plano.md` §6.
#[test]
fn os_extremos_de_cada_campo_soam_e_sao_finitos() {
    let mut takes = Takes::default();
    let take = takes.add(Take::new(
        48_000,
        1,
        (0..48_000)
            .map(|i| ((i % 97) as f32 / 97.0) - 0.5)
            .collect(),
    ));
    let mut pior = (0.0_f64, "");
    let mut vistos = 0;
    for fonte in fontes(take) {
        let mut base = Recipe::new(9, CURTO);
        let id = base.add(0.5, fonte, Envelope::hit(0.002, 0.1, 0.01));
        for c in CAMPOS {
            let aplica = c.le(&base, base.layer(id)).is_some() || c.le(&base, None).is_some();
            if !aplica {
                continue;
            }
            for ponta in [c.faixa.0, c.faixa.1] {
                let mut r = base.clone();
                c.escreve(&mut r, Some(id), ponta);
                let mut na_lei = r.clone();
                na_lei.sanitize();
                assert_eq!(
                    na_lei, r,
                    "{} = {ponta}: a faixa de tela sai da lei do motor",
                    c.chave
                );
                let t = Instant::now();
                let pcm = ph2d_sfx_dsp::render(&r, r.seed, r.engine, &takes).expect("render");
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                assert!(
                    pcm.samples.iter().all(|s| s.is_finite()),
                    "{} = {ponta}: não-número",
                    c.chave
                );
                println!("{:32} {ponta:>14.3} {ms:>10.2} ms", c.chave);
                if ms > pior.0 {
                    pior = (ms, c.chave);
                }
                vistos += 1;
            }
        }
    }
    assert!(
        vistos >= CAMPOS.len(),
        "só {vistos} pontas: a varredura perdeu o alvo"
    );
    println!("pior: {:.2} ms em {}", pior.0, pior.1);
}
