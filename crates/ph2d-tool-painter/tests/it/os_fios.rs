//! **Os FIOS e a textura do traço** (W2, prompt 48; 05 §3): a textura, a uniformidade (CV) e o período de cada
//! pincel de fábrica com cerdas ou grão, na cena seca da bancada (`midias_de_fabrica::cena_seca`), a 40 px e à
//! largura da foto do Rebelle da mesma mídia, contra os números da foto (`ferramentas/mede_traco.py`, recto).

use super::midias_de_fabrica::{cena_seca, de_fabrica};
use super::regua_midia::Medida;

/// A foto do Rebelle de uma mídia e os números dela (`mede_traco.py`, recto, 18/10): o raio que dá a largura
/// dela, a textura (%), o CV e a fracção da potência no período.
#[derive(Clone, Copy, Debug)]
pub(super) struct Foto {
    pub nome: &'static str,
    pub raio: f32,
    pub textura: f64,
    pub cv: f64,
    pub periodo_fracao: f64,
}

/// Os pincéis de fábrica com cerdas (o Shape de fábrica) ou grão — todos menos o Airbrush (sem forma) — e a
/// foto da mídia de cada um; o Wet Dry Brush e o Wet Drip não têm foto na bancada dos gestos.
#[rustfmt::skip]
pub(super) const FIOS: [(&str, Option<Foto>); 19] = [
    ("Wet Round", Some(Foto { nome: "wcround", raio: 16.0, textura: 2.35, cv: 0.038, periodo_fracao: 0.05 })),
    ("Wet Wash", Some(Foto { nome: "wcwash", raio: 20.0, textura: 4.96, cv: 0.018, periodo_fracao: 0.12 })),
    ("Wet Dry Brush", None),
    ("Wet Drip", None),
    ("Wet Liner", Some(Foto { nome: "wcliner", raio: 19.0, textura: 1.71, cv: 0.008, periodo_fracao: 0.10 })),
    ("Wet Flat", Some(Foto { nome: "wcflat", raio: 17.0, textura: 5.08, cv: 0.020, periodo_fracao: 0.05 })),
    ("Gouache Flat", Some(Foto { nome: "gouacheflat", raio: 19.5, textura: 2.50, cv: 0.023, periodo_fracao: 0.08 })),
    ("Oil Bristle", Some(Foto { nome: "oil", raio: 20.5, textura: 5.86, cv: 0.011, periodo_fracao: 0.09 })),
    ("Acrylic Round", Some(Foto { nome: "oilround", raio: 19.5, textura: 3.13, cv: 0.000, periodo_fracao: 0.03 })),
    ("Express Oil", Some(Foto { nome: "express", raio: 21.0, textura: 9.00, cv: 0.008, periodo_fracao: 0.47 })),
    ("Graphite 8B", Some(Foto { nome: "pencil", raio: 15.0, textura: 9.09, cv: 0.037, periodo_fracao: 0.07 })),
    ("Charcoal", Some(Foto { nome: "charcoal", raio: 10.5, textura: 8.30, cv: 0.053, periodo_fracao: 0.07 })),
    ("Soft Pastel", Some(Foto { nome: "pastel", raio: 8.5, textura: 7.21, cv: 0.042, periodo_fracao: 0.03 })),
    ("Oil Pastel", Some(Foto { nome: "oilpastel2", raio: 15.0, textura: 17.35, cv: 0.053, periodo_fracao: 0.06 })),
    ("Ink Bullet", Some(Foto { nome: "ink", raio: 21.0, textura: 0.39, cv: 0.002, periodo_fracao: 0.07 })),
    ("Dip Pen", Some(Foto { nome: "dippen", raio: 19.0, textura: 1.43, cv: 0.008, periodo_fracao: 0.04 })),
    ("Ink Brush", Some(Foto { nome: "inkbrush", raio: 20.0, textura: 1.27, cv: 0.008, periodo_fracao: 0.04 })),
    ("Marker Bullet", Some(Foto { nome: "markerbullet", raio: 20.0, textura: 0.66, cv: 0.001, periodo_fracao: 0.30 })),
    ("Marker Liner", Some(Foto { nome: "marker", raio: 9.0, textura: 2.12, cv: 0.001, periodo_fracao: 0.24 })),
];

/// A textura no LEVE (a rampa a pressão ~0,33–0,58) e no FORTE (~0,94–1): os recortes da rampa da bancada.
pub(super) const LEVE: [u32; 4] = [90, 250, 80, 100];
pub(super) const FORTE: [u32; 4] = [270, 250, 80, 100];

fn linha(nome: &str, raio: f32, m: &Medida, lf: (f64, f64), f: Option<&Foto>) -> String {
    let r = f.map_or_else(String::new, |f| {
        format!(
            " | R {}: tex {:5.2} cv {:.3} per {:2.0}%",
            f.nome,
            f.textura,
            f.cv,
            100.0 * f.periodo_fracao
        )
    });
    format!(
        "[fios] {nome:<24} r {raio:4.1} larg {:5.1} tex {:5.2} cv {:.3} per {:5.1}px ({:2.0}%) carril {:4.2} cob {:.3} leve {:5.2} forte {:5.2}{r}",
        m.largura,
        m.textura,
        m.cv,
        m.periodo,
        100.0 * m.periodo_fracao,
        m.carril,
        m.cobertura,
        lf.0,
        lf.1
    )
}

/// Uma variante da sonda: o nome e os knobs trocados.
type Variante = (String, Vec<(String, f64)>);

/// **SONDA (W2, passo 1): a barra da W2 HOJE** — cada pincel de [`FIOS`] a 40 px (raio 20) e à largura da foto
/// dele, seco; `PH2D_SONDA_OUT=<dir>` grava os PPM (`midia_fios_<preset>[@variante]_<raio>.ppm`) para conferir a
/// régua com o Python. `PH2D_SONDA_PRESETS="a,b"` restringe; `PH2D_SONDA_KNOBS="nome:k=v,b.k=v|…"` são as
/// variantes (como `sonda_as_metricas_das_variantes`; a vazia = o preset de fábrica); `PH2D_SONDA_RAIOS="20,30"`
/// troca os raios.
#[test]
#[ignore = "sonda: a barra da W2 nos pincéis de fábrica; corre-se à mão"]
fn sonda_a_barra_da_w2() {
    let so: Option<Vec<String>> = std::env::var("PH2D_SONDA_PRESETS")
        .ok()
        .map(|v| v.split(',').map(str::to_owned).collect());
    let variantes: Vec<Variante> = std::env::var("PH2D_SONDA_KNOBS").map_or_else(
        |_| vec![(String::new(), Vec::new())],
        |v| {
            v.split('|')
                .map(|s| {
                    let (n, kv) = s.split_once(':').unwrap_or((s, ""));
                    let kv = kv
                        .split(',')
                        .filter(|x| !x.is_empty())
                        .map(|x| {
                            let (k, v) = x.split_once('=').unwrap();
                            (k.to_owned(), v.parse().unwrap())
                        })
                        .collect();
                    (n.to_owned(), kv)
                })
                .collect()
        },
    );
    let raios: Option<Vec<f32>> = std::env::var("PH2D_SONDA_RAIOS")
        .ok()
        .map(|v| v.split(',').map(|x| x.parse().unwrap()).collect());
    let casos: Vec<(&str, f32, Option<&Foto>, &Variante)> = FIOS
        .iter()
        .filter(|(n, _)| so.as_ref().is_none_or(|s| s.iter().any(|x| x == n)))
        .flat_map(|(n, f)| {
            let rs = raios.clone().unwrap_or_else(|| {
                let mut v = vec![20.0];
                if let Some(f) = f
                    && f.raio != 20.0
                {
                    v.push(f.raio);
                }
                v
            });
            rs.into_iter().map(move |r| (*n, r, f.as_ref()))
        })
        .flat_map(|(n, r, f)| variantes.iter().map(move |v| (n, r, f, v)))
        .collect();
    let linhas: Vec<String> = std::thread::scope(|s| {
        let hs: Vec<_> = casos
            .iter()
            .map(|&(n, raio, f, (vn, kv))| {
                s.spawn(move || {
                    let mut p = de_fabrica(n);
                    for (k, x) in kv {
                        if let Some(b) = k.strip_prefix("b.") {
                            p.parametros
                                .brush
                                .insert(b.to_owned(), serde_json::json!(x));
                        } else {
                            p.parametros.engine.insert(k.clone(), *x);
                        }
                    }
                    let nome = if vn.is_empty() {
                        n.to_owned()
                    } else {
                        format!("{n}@{vn}")
                    };
                    let (m, lf) = cena_seca_com_rampa(&p, raio, &format!("fios_{nome}_{raio}"));
                    linha(&nome, raio, &m, lf, f)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for l in linhas {
        println!("{l}");
    }
}

/// A cena seca medida no recto e a textura no leve e no forte da rampa.
pub(super) fn cena_seca_com_rampa(
    p: &ph2d_brush_presets::Preset,
    raio: f32,
    ficheiro: &str,
) -> (Medida, (f64, f64)) {
    let (m, c) = cena_seca(p, raio, &[ficheiro.to_owned()]);
    (m, (c.leve, c.forte))
}

/// Os knobs dos fios (W2): o CONTROLO de cada portão é o preset sem eles — o tapete de antes.
pub(super) const KNOBS_DOS_FIOS: [&str; 12] = [
    "bristlePressure",
    "bristleNoiseLength",
    "bristleLaneWidth",
    "bristleWave",
    "bristleWaveLength",
    "bristleOscillation",
    "bristleSpan",
    "bristleSpanLength",
    "bristleNoiseHue",
    "bristleNoiseSat",
    "bristleNoiseLight",
    "bristleLift",
];

fn sem_fios(nome: &str) -> ph2d_brush_presets::Preset {
    let mut p = de_fabrica(nome);
    // … e sem o RELEVO deles (W3): a altura de cada fio também é dos fios.
    for k in KNOBS_DOS_FIOS
        .into_iter()
        .chain(["bristleHeight", "bristleRemove"])
    {
        p.parametros.engine.remove(k);
    }
    p
}

/// `n` do lado do Rebelle `r` do ponto médio entre ele e o controlo `c`, e o controlo longe dele (3× o ruído).
fn no_vale(n: f64, r: f64, c: f64, ruido: f64) -> Result<(), String> {
    if (c - r).abs() <= 3.0 * ruido {
        return Err(format!("o controlo {c:.3} não se distingue do Rebelle {r}"));
    }
    if (n - r).abs() < 0.5 * (c - r).abs() {
        Ok(())
    } else {
        Err(format!(
            "{n:.3} fora do vale do Rebelle {r} (controlo {c:.3})"
        ))
    }
}

/// ⭐ **A barra da W2** (05 §3; doc 48_medicao §4) nos dois pincéis cujas cerdas o Rebelle tem como FIOS: o Oil
/// Bristle (Bristle Flat, `PARTICLE`) e o Express Oil (Hairy), secos, à largura da foto e a 40 px, contra o
/// preset SEM os fios (o tapete de antes):
/// - os dois: **CV ≤ 0,02** e **período < 15 %** da potência (a barra de 08/10);
/// - Oil Bristle: os **carris** no vale do Rebelle (3,32; ruído 1,0 — o recto × o rápido da foto) e a textura a
///   subir pelo menos 1 ponto (o resto até 5,86 são as sombras do relevo de cada fio: W3);
/// - Express Oil: **5 ≤ textura ≤ 10 %** e no vale (9,0; ruído 0,37), e o LEVE e o FORTE da rampa no vale
///   (10,16 · 7,31).
#[test]
fn os_oleos_tem_fios_e_nao_carris() {
    let casos = [
        ("Oil Bristle", 20.5f32),
        ("Oil Bristle", 20.0),
        ("Express Oil", 21.0),
        ("Express Oil", 20.0),
    ];
    let medidas: Vec<_> = std::thread::scope(|s| {
        let hs: Vec<_> = casos
            .iter()
            .map(|&(n, r)| {
                s.spawn(move || {
                    let com = cena_seca_com_rampa(&de_fabrica(n), r, &format!("fios_{n}_{r}"));
                    let sem = cena_seca_com_rampa(&sem_fios(n), r, &format!("fios_{n}@sem_{r}"));
                    (com, sem)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (&(n, r), ((m, lf), (c, clf))) in casos.iter().zip(&medidas) {
        println!("[fios] {n} r {r}: {m:?} leve/forte {lf:?} · SEM os fios {c:?} {clf:?}");
        let mut f = |ok: Result<(), String>, o: &str| {
            if let Err(e) = ok {
                falhas.push(format!("{n} r {r}: {o} {e}"));
            }
        };
        f(
            (m.cv <= 0.02).then_some(()).ok_or(format!("{:.3}", m.cv)),
            "CV",
        );
        f(
            (m.periodo_fracao < 0.15)
                .then_some(())
                .ok_or(format!("{:.3} a {:.1} px", m.periodo_fracao, m.periodo)),
            "período",
        );
        if n == "Oil Bristle" {
            f(no_vale(m.carril, 3.32, c.carril, 1.0), "carris");
            f(
                (m.textura >= c.textura + 1.0)
                    .then_some(())
                    .ok_or(format!("{:.2} × {:.2}", m.textura, c.textura)),
                "textura",
            );
        } else {
            f(
                ((5.0..=10.0).contains(&m.textura))
                    .then_some(())
                    .ok_or(format!("{:.2}", m.textura)),
                "textura 5–10",
            );
            f(no_vale(m.textura, 9.0, c.textura, 0.37), "textura");
            f(no_vale(lf.0, 10.16, clf.0, 0.37), "leve");
            f(no_vale(lf.1, 7.31, clf.1, 0.37), "forte");
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// Um traço recto de 400 px com a pressão a subir de 0,3 a 1 (a mão leve pesa no `bristleLift`), o preset
/// `p` ao raio 20, 30 quadros depois: o FNV dos píxeis.
pub(super) fn traco_com_rampa(p: &ph2d_brush_presets::Preset) -> u64 {
    use super::sonda_gestos::{AZUL, fnv1a, gesto};
    use ph2d_editor_core::tool::RasterEditTool;
    let (w, h) = (520u32, 120u32);
    let mut t = ph2d_tool_painter::PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8(AZUL);
    gesto(&mut t, &[[60.0, 60.0], [460.0, 60.0]], 3.0, |f| {
        0.3 + 0.7 * f
    });
    for _ in 0..30 {
        ph2d_editor_core::Tool::on_tick(&mut t, 1000.0 / 60.0);
    }
    fnv1a(&t.run_full().0)
}

/// ⭐ **Cada knob dos fios MORDE e os comprimentos sozinhos não** (W2): sobre o Express Oil sem os fios, cada
/// amplitude sozinha muda o traço; cada comprimento muda-o só com a amplitude que ele escala ligada; os quatro
/// comprimentos sem amplitude nenhuma dão o traço de antes AO BYTE (o neutro não chama a lei). Os 12 nomes
/// são os de [`KNOBS_DOS_FIOS`].
#[test]
fn cada_knob_dos_fios_morde() {
    let base = sem_fios("Express Oil");
    let com = |kv: &[(&str, f64)]| {
        let mut p = base.clone();
        for &(k, v) in kv {
            p.parametros.engine.insert(k.to_owned(), v);
        }
        p
    };
    // (o knob a provar, os que ele precisa ligados)
    let casos: [(&[(&str, f64)], &str); 13] = [
        (
            &[
                ("bristleNoiseLength", 77.0),
                ("bristleLaneWidth", 9.0),
                ("bristleWaveLength", 90.0),
                ("bristleSpanLength", 3.0),
            ],
            "neutro",
        ),
        (&[("bristlePressure", 1.0)], "bristlePressure"),
        (&[("bristleLift", 0.8)], "bristleLift"),
        (
            &[("bristleWave", 0.6), ("bristlePressure", 1.0)],
            "bristleWave",
        ),
        (&[("bristleOscillation", 4.0)], "bristleOscillation"),
        (&[("bristleSpan", 3.0)], "bristleSpan"),
        (&[("bristleNoiseHue", 0.05)], "bristleNoiseHue"),
        (&[("bristleNoiseSat", 0.2)], "bristleNoiseSat"),
        (&[("bristleNoiseLight", 0.2)], "bristleNoiseLight"),
        (
            &[("bristlePressure", 1.0), ("bristleNoiseLength", 40.0)],
            "bristleNoiseLength",
        ),
        (
            &[("bristlePressure", 1.0), ("bristleLaneWidth", 6.0)],
            "bristleLaneWidth",
        ),
        (
            &[
                ("bristleWave", 0.6),
                ("bristlePressure", 1.0),
                ("bristleWaveLength", 80.0),
            ],
            "bristleWaveLength",
        ),
        (
            &[("bristleSpan", 3.0), ("bristleSpanLength", 40.0)],
            "bristleSpanLength",
        ),
    ];
    let (antes, hs): (u64, Vec<(&str, u64, u64)>) = std::thread::scope(|s| {
        let a = s.spawn(|| traco_com_rampa(&base));
        let hs: Vec<_> = casos
            .iter()
            .map(|&(kv, nome)| {
                // O comprimento compara-se com a amplitude dele sozinha (sem o comprimento).
                let sem: Vec<(&str, f64)> =
                    kv.iter().copied().filter(|&(k, _)| k != nome).collect();
                let (p, q) = (com(kv), com(&sem));
                (
                    nome,
                    s.spawn(move || traco_com_rampa(&p)),
                    s.spawn(move || traco_com_rampa(&q)),
                )
            })
            .collect();
        let hs = hs
            .into_iter()
            .map(|(n, p, q)| (n, p.join().unwrap(), q.join().unwrap()))
            .collect();
        (a.join().unwrap(), hs)
    });
    for (nome, h, sem) in hs {
        if nome == "neutro" {
            assert_eq!(h, antes, "os comprimentos sozinhos mudaram o traço");
        } else {
            assert_ne!(h, sem, "{nome} não morde");
        }
    }
    let nomes: Vec<&str> = casos.iter().skip(1).map(|c| c.1).collect();
    let mut todos = KNOBS_DOS_FIOS.to_vec();
    todos.sort_unstable();
    let mut n = nomes.clone();
    n.sort_unstable();
    assert_eq!(n, todos, "cada knob dos fios tem o seu caso");
}
