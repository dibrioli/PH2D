//! Os medidores leem o arquivo, sem limitador; a escuta continua limitada.

use crate::Mesa;
use ph2d_audio::dsp::integrated_lufs;
use ph2d_panel_sfxlab::medidor::{BANDAS, pico_db};
use ph2d_sfx_dsp::TETO_DA_ESCUTA;
use ph2d_sfx_recipe::{Envelope, Recipe, SAMPLE_RATE, Source, Tone, Wave};

const HZ: f32 = 1000.0;

fn seno(ganho: f32) -> Recipe {
    let mut r = Recipe::new(7, 0.25);
    r.add(
        ganho,
        Source::Tone(Tone {
            wave: Wave::Sine,
            hz: HZ,
            hz_end: HZ,
            glide: 0.0,
        }),
        Envelope::default(),
    );
    r
}

#[test]
fn o_medidor_le_o_buffer_sem_limitador_em_som_que_estoura() {
    let mut m = Mesa::nova();
    let id = m.abre("Alto", seno(4.0));
    let pcm = m.docs().pcm(id).expect("pcm").to_vec();
    let pico = pcm.iter().fold(0.0_f32, |a, s| a.max(s.abs()));
    assert!(pico > 1.0, "a fixture não estoura: {pico}");

    let medida = m.docs().medida(id).expect("medida").clone();
    assert!(medida.pico_db > 0.0, "Peak {}", medida.pico_db);
    assert!(medida.clip(), "CLIP apagado com o arquivo a {pico}");
    assert_eq!(medida.pico_db, pico_db(&pcm));
    assert_eq!(medida.lufs, integrated_lufs(&pcm, 1, SAMPLE_RATE));

    let ouvido = m.docs().escuta(id).expect("escuta").to_vec();
    let teto = ouvido.iter().fold(0.0_f32, |a, s| a.max(s.abs()));
    assert!(teto <= TETO_DA_ESCUTA, "a escuta passou do teto: {teto}");
}

#[test]
fn som_baixo_nao_acende_o_clip() {
    let mut m = Mesa::nova();
    let id = m.abre("Baixo", seno(0.25));
    let medida = m.docs().medida(id).expect("medida").clone();
    assert!(
        medida.pico_db < 0.0 && medida.pico_db.is_finite(),
        "{}",
        medida.pico_db
    );
    assert!(!medida.clip());
}

#[test]
fn o_espectro_acha_o_seno_na_banda_certa() {
    let mut m = Mesa::nova();
    let id = m.abre("Seno", seno(0.5));
    let e = m.docs().medida(id).expect("medida").espectro;
    let maior = (0..BANDAS)
        .max_by(|a, b| e[*a].total_cmp(&e[*b]))
        .expect("bandas");
    let t = (HZ / 20.0).ln() / (20_000.0_f32 / 20.0).ln();
    let certa = (t * BANDAS as f32) as usize;
    assert_eq!(maior, certa, "{e:?}");
    // Meia escala lê perto de −6 dB; a referência do espectro está calibrada.
    assert!((e[maior] + 6.0).abs() < 1.5, "{}", e[maior]);
}
