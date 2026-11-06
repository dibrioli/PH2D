//! **A escuta da W3:** limitador, cancelamento e mudo (`docs/SfxLab/04_W3_plano.md` §5).

use std::cell::Cell;

use ph2d_sfx_dsp::{RenderCache, RenderError, TETO_DA_ESCUTA, limitar_escuta, pcm_hash, render};
use ph2d_sfx_recipe::{
    Body, ENGINE_V1, Envelope, ModeLaw, Recipe, Recording, SAMPLE_RATE, Source, Take, Takes,
    factory,
};

fn pico(s: &[f32]) -> f32 {
    s.iter().fold(0.0, |m, x| x.abs().max(m))
}

fn corpo(base_hz: f32, modes: u32, t60: f32) -> Source {
    Source::Body(Body {
        law: ModeLaw::Stretched { stretch: 1.0 },
        base_hz,
        modes,
        t60,
        damping: 0.0,
        rolloff: 0.0,
        contact_ms: 1.0,
        noise: 0.0,
    })
}

/// Uma ressonância longa com ganho que estoura, a fixture do plano.
fn ressonancia_que_explode(ganho: f32) -> Vec<f32> {
    let mut r = Recipe::new(3, 0.5);
    r.add(ganho, corpo(220.0, 16, 4.0), Envelope::default());
    render(&r, r.seed, ENGINE_V1, &Takes::default())
        .unwrap()
        .samples
}

#[test]
fn limitador_nunca_passa_do_teto() {
    let quadrada: Vec<f32> = (0..48_000)
        .map(|i| if i / 100 % 2 == 0 { 40.0 } else { -40.0 })
        .collect();
    let mut golpes = vec![0.1f32; 4_800];
    golpes[0] = 1.0e6;
    golpes[2_400] = -30.0;
    golpes[4_799] = 500.0;
    for (nome, x) in [
        ("ressonância ×50", ressonancia_que_explode(50.0)),
        ("ressonância ×1e6", ressonancia_que_explode(1.0e6)),
        ("quadrada", quadrada),
        ("golpes nas pontas", golpes),
    ] {
        assert!(pico(&x) > TETO_DA_ESCUTA, "{nome}: a fixture não estoura");
        let y = limitar_escuta(&x, SAMPLE_RATE);
        assert_eq!(y.len(), x.len(), "{nome}");
        assert!(pico(&y) <= TETO_DA_ESCUTA, "{nome}: pico {}", pico(&y));
        assert!(pico(&y) > TETO_DA_ESCUTA * 0.9, "{nome}: esmagou o som");
    }
}

#[test]
fn som_abaixo_do_teto_sai_igual_ao_bit() {
    for (nome, r) in factory::all() {
        let x = render(&r, r.seed, ENGINE_V1, &Takes::default())
            .unwrap()
            .samples;
        if pico(&x) <= TETO_DA_ESCUTA {
            assert_eq!(
                pcm_hash(&limitar_escuta(&x, SAMPLE_RATE)),
                pcm_hash(&x),
                "{nome}"
            );
        }
    }
}

/// Um estouro curto só abaixa em volta dele: depois da soltura o som volta ao que era.
#[test]
fn limitador_devolve_o_volume_depois_do_estouro() {
    let mut x = vec![0.5f32; 48_000];
    x[1_000] = 20.0;
    let y = limitar_escuta(&x, SAMPLE_RATE);
    assert!(y[1_000].abs() <= TETO_DA_ESCUTA);
    assert!(
        y[1_000 - 120] < 0.5,
        "antecipa: abaixa 2,5 ms antes do golpe"
    );
    assert_eq!(y[1_000 - 480], 0.5, "10 ms antes ainda não mexeu");
    assert_eq!(y[47_000], 0.5, "voltou");
}

#[test]
fn limitador_nao_deixa_passar_nao_numero() {
    let y = limitar_escuta(
        &[0.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.5],
        SAMPLE_RATE,
    );
    assert!(
        y.iter().all(|s| s.is_finite() && s.abs() <= TETO_DA_ESCUTA),
        "{y:?}"
    );
    assert!(limitar_escuta(&[], SAMPLE_RATE).is_empty());
}

/// Desiste na n-ésima pergunta, depois termina no mesmo cache: os bytes são os de sempre, em todo ponto de desistência.
#[test]
fn cancelar_nao_muda_um_bit() {
    let t = Takes::default();
    for (nome, r) in factory::all() {
        let verdade = pcm_hash(&render(&r, r.seed, ENGINE_V1, &t).unwrap().samples);
        let mut desistiu = 0;
        for depois_de in 0..40 {
            let perguntas = Cell::new(0);
            let mut cache = RenderCache::default();
            let cancelado = || {
                perguntas.set(perguntas.get() + 1);
                perguntas.get() > depois_de
            };
            match cache.render_cancelavel(&r, r.seed, ENGINE_V1, &t, &cancelado) {
                Err(RenderError::Cancelled) => desistiu += 1,
                Ok(p) => assert_eq!(pcm_hash(&p.samples), verdade, "{nome}: acabou sem desistir"),
                Err(e) => panic!("{nome}: {e}"),
            }
            let p = cache.render(&r, r.seed, ENGINE_V1, &t).unwrap();
            assert_eq!(
                pcm_hash(&p.samples),
                verdade,
                "{nome}, depois de {depois_de}"
            );
        }
        assert!(desistiu > 0, "{nome}: nunca desistiu");
    }
}

/// O pior corpo do motor (9,4 s em 120 s) responde entre modos: a pergunta tem de chegar perto do número de modos.
#[test]
fn o_corpo_longo_pergunta_entre_modos() {
    let mut r = Recipe::new(3, 2.0);
    r.add(1.0, corpo(20.0, u32::MAX, 4.0), Envelope::default());
    let perguntas = Cell::new(0u32);
    let cancelado = || {
        perguntas.set(perguntas.get() + 1);
        false
    };
    RenderCache::default()
        .render_cancelavel(&r, r.seed, ENGINE_V1, &Takes::default(), &cancelado)
        .unwrap();
    assert!(perguntas.get() >= 1_000, "só {} perguntas", perguntas.get());
}

/// Gravação rápida com material longo pergunta durante a leitura, não só no fim.
#[test]
fn a_gravacao_pergunta_durante_a_leitura() {
    let mut t = Takes::default();
    let id = t.add(Take::new(44_100, 1, vec![0.25; 441_000]));
    let mut r = Recipe::new(3, 2.0);
    r.add(
        1.0,
        Source::Recording(Recording {
            take: id,
            semitones: 24.0,
            offset: 0.0,
        }),
        Envelope::default(),
    );
    let perguntas = Cell::new(0u32);
    let cancelado = || {
        perguntas.set(perguntas.get() + 1);
        false
    };
    RenderCache::default()
        .render_cancelavel(&r, r.seed, ENGINE_V1, &t, &cancelado)
        .unwrap();
    assert!(perguntas.get() >= 20, "só {} perguntas", perguntas.get());
}

#[test]
fn mudo_nao_muda_a_semente_das_outras() {
    let (_, r) = factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Impact")
        .unwrap();
    assert!(r.layers.len() >= 2);
    let t = Takes::default();
    let mut so_a_primeira = r.clone();
    so_a_primeira.layers.truncate(1);
    let sozinha = render(&so_a_primeira, r.seed, ENGINE_V1, &t)
        .unwrap()
        .samples;
    let mut mudas = r.clone();
    for l in &mut mudas.layers[1..] {
        l.muted = true;
    }
    let com_mudas = render(&mudas, r.seed, ENGINE_V1, &t).unwrap().samples;
    assert_eq!(pcm_hash(&sozinha), pcm_hash(&com_mudas));
    assert_ne!(
        pcm_hash(&com_mudas),
        pcm_hash(&render(&r, r.seed, ENGINE_V1, &t).unwrap().samples),
        "o mudo não fez nada"
    );
}

/// Mudar uma camada não calcula nada, e tirar o mudo também não: o cache sobrevive.
#[test]
fn mudo_nao_refaz_camada_nenhuma() {
    let (_, mut r) = factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Impact")
        .unwrap();
    let t = Takes::default();
    let mut c = RenderCache::default();
    c.render(&r, r.seed, ENGINE_V1, &t).unwrap();
    r.layers[0].muted = true;
    c.render(&r, r.seed, ENGINE_V1, &t).unwrap();
    assert_eq!(c.last_rendered, 0);
    r.layers[0].muted = false;
    c.render(&r, r.seed, ENGINE_V1, &t).unwrap();
    assert_eq!(c.last_rendered, 0);
}

/// O solo escuta só uma camada, e a receita fica como estava.
#[test]
fn solo_nao_entra_no_documento() {
    let (_, r) = factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Impact")
        .unwrap();
    let antes = r.clone();
    let solo = r.solo(r.layers[0].id);
    assert_eq!(r, antes);
    assert!(!solo.layers[0].muted);
    assert!(solo.layers[1..].iter().all(|l| l.muted));
}
