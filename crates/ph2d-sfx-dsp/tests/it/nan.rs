//! **NaN não entra em estado.** A fixture põe o valor ruim em cada campo de cada fonte, um de cada vez: um filtro ou
//! ressonador envenenado ficaria NaN até o fim do buffer, e um único campo sem lei apareceria aqui.

use ph2d_sfx_dsp::render;
use ph2d_sfx_recipe::{
    Body, ENGINE_V1, EnvPoint, Envelope, ModeLaw, Noise, NoiseFilter, Recipe, Recording, Source,
    Take, Takes, Tone, Wave,
};

const RUINS: [f32; 6] = [
    f32::NAN,
    f32::INFINITY,
    f32::NEG_INFINITY,
    f32::MAX,
    -f32::MAX,
    0.0,
];

fn ruido() -> Noise {
    Noise {
        filter: NoiseFilter::Band,
        hz: 1_000.0,
        hz_end: 2_000.0,
        q: 1.0,
    }
}

fn tom(wave: Wave) -> Tone {
    Tone {
        wave,
        hz: 500.0,
        hz_end: 900.0,
        glide: 0.05,
    }
}

fn corpo() -> Body {
    Body {
        law: ModeLaw::Stretched { stretch: 1.3 },
        base_hz: 300.0,
        modes: 8,
        t60: 0.2,
        damping: 0.1,
        rolloff: 1.0,
        contact_ms: 1.0,
        noise: 0.5,
    }
}

/// Toda fonte com um campo trocado pelo valor `x`, pelo nome do campo.
fn variantes(x: f32, take: [u8; 32]) -> Vec<(&'static str, Source)> {
    let fm = |ratio, index, index_end| {
        tom(Wave::Fm {
            ratio,
            index,
            index_end,
        })
    };
    vec![
        ("noise.hz", Source::Noise(Noise { hz: x, ..ruido() })),
        (
            "noise.hz_end",
            Source::Noise(Noise {
                hz_end: x,
                ..ruido()
            }),
        ),
        ("noise.q", Source::Noise(Noise { q: x, ..ruido() })),
        (
            "tone.hz",
            Source::Tone(Tone {
                hz: x,
                ..tom(Wave::Saw)
            }),
        ),
        (
            "tone.hz_end",
            Source::Tone(Tone {
                hz_end: x,
                ..tom(Wave::Square)
            }),
        ),
        (
            "tone.glide",
            Source::Tone(Tone {
                glide: x,
                ..tom(Wave::Triangle)
            }),
        ),
        ("fm.ratio", Source::Tone(fm(x, 1.0, 1.0))),
        ("fm.index", Source::Tone(fm(1.0, x, 1.0))),
        ("fm.index_end", Source::Tone(fm(1.0, 1.0, x))),
        (
            "body.stretch",
            Source::Body(Body {
                law: ModeLaw::Stretched { stretch: x },
                ..corpo()
            }),
        ),
        (
            "body.base_hz",
            Source::Body(Body {
                base_hz: x,
                ..corpo()
            }),
        ),
        ("body.t60", Source::Body(Body { t60: x, ..corpo() })),
        (
            "body.damping",
            Source::Body(Body {
                damping: x,
                ..corpo()
            }),
        ),
        (
            "body.rolloff",
            Source::Body(Body {
                rolloff: x,
                ..corpo()
            }),
        ),
        (
            "body.contact_ms",
            Source::Body(Body {
                contact_ms: x,
                ..corpo()
            }),
        ),
        (
            "body.noise",
            Source::Body(Body {
                noise: x,
                ..corpo()
            }),
        ),
        (
            "rec.semitones",
            Source::Recording(Recording {
                take,
                semitones: x,
                offset: 0.0,
            }),
        ),
        (
            "rec.offset",
            Source::Recording(Recording {
                take,
                semitones: 1.0,
                offset: x,
            }),
        ),
    ]
}

fn finito(pcm: &[f32]) -> bool {
    pcm.iter().all(|s| s.is_finite())
}

fn soa(r: &Recipe, takes: &Takes) -> Vec<f32> {
    render(r, r.seed, ENGINE_V1, takes).unwrap().samples
}

fn material() -> Takes {
    let mut t = Takes::default();
    let s: Vec<f32> = (0..2_000).map(|i| (i as f32 * 0.03).sin()).collect();
    t.add(Take::new(44_100, 1, s));
    t
}

#[test]
fn nan_nao_entra_em_estado_por_nenhum_campo_da_fonte() {
    let takes = material();
    let id = takes.iter().next().unwrap().id;
    let mut erros = Vec::new();
    for x in RUINS {
        for (campo, fonte) in variantes(x, id) {
            let mut r = Recipe::new(3, 0.05);
            r.add(1.0, fonte, Envelope::default());
            if !finito(&soa(&r, &takes)) {
                erros.push(format!("{campo} = {x}"));
            }
        }
    }
    assert!(erros.is_empty(), "não-número no PCM: {erros:?}");
}

#[test]
fn nan_nao_entra_pelos_campos_da_camada_nem_do_envelope() {
    let takes = Takes::default();
    for x in RUINS {
        let mut ganho = Recipe::new(3, 0.05);
        let id = ganho.add(x, Source::Noise(ruido()), Envelope::default());
        ganho.layer_mut(id).unwrap().start = x;
        assert!(finito(&soa(&ganho, &takes)), "gain/start = {x}");

        for (t, v, exp) in [(x, 0.5, false), (0.02, x, true), (0.0, x, false)] {
            let mut r = Recipe::new(3, 0.05);
            let env = Envelope {
                points: vec![
                    EnvPoint {
                        t: 0.0,
                        v: 1.0,
                        exp: false,
                    },
                    EnvPoint { t, v, exp },
                    EnvPoint {
                        t: 0.04,
                        v: 0.5,
                        exp: true,
                    },
                ],
            };
            r.add(1.0, Source::Body(corpo()), env);
            assert!(finito(&soa(&r, &takes)), "envelope t={t} v={v}");
        }
    }
}

/// Uma duração ruim não pede memória infinita.
#[test]
fn duracao_ruim_nao_explode() {
    for x in [f32::NAN, f32::NEG_INFINITY, -1.0, 0.0] {
        let mut r = Recipe::new(3, x);
        r.add(1.0, Source::Noise(ruido()), Envelope::default());
        assert!(finito(&soa(&r, &Takes::default())), "seconds = {x}");
    }
}

/// O material com um NaN no meio: a amostra vira silêncio na entrada, e o que vem depois continua a soar.
#[test]
fn nan_no_material_nao_envenena_o_resto() {
    let mut s = vec![0.25f32; 4_000];
    s[1_000] = f32::NAN;
    s[1_001] = f32::INFINITY;
    let mut takes = Takes::default();
    let take = takes.add(Take::new(44_100, 1, s));
    let mut r = Recipe::new(3, 0.05);
    r.add(
        1.0,
        Source::Recording(Recording {
            take,
            semitones: 0.0,
            offset: 0.0,
        }),
        Envelope::default(),
    );
    let pcm = soa(&r, &takes);
    assert!(finito(&pcm));
    assert!(pcm.last().unwrap().abs() > 0.1, "o resto calou");
}

/// O mesmo vale depois de gravar e abrir: o blob pode ter vindo com qualquer coisa.
#[test]
fn receita_ruim_gravada_abre_dentro_da_lei() {
    let mut lib = ph2d_sfx_recipe::SfxLibrary::default();
    let mut r = Recipe::new(3, f32::NAN);
    r.add(
        f32::NAN,
        Source::Noise(Noise {
            q: f32::NAN,
            ..ruido()
        }),
        Envelope::default(),
    );
    lib.recipes.insert("x".into(), r);
    let aberta = ph2d_sfx_recipe::SfxLibrary::from_bytes(&lib.to_bytes()).unwrap();
    let r = &aberta.recipes["x"];
    assert_eq!(r.seconds, 1.0);
    assert_eq!(r.layers[0].gain, 1.0);
    assert!(finito(&soa(r, &Takes::default())));
}
