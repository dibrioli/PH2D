//! **Sonda (ignorada): o MAPA DAS MÍDIAS medido antes de construir** (prompt 17, Fase A) — para cada
//! mídia do dono, composições dos knobs de HOJE do Wet Paint pintadas na cena da bancada
//! (`sonda_gestos::pinta_cena`) ao raio cuja largura medida é a da foto do Rebelle da mesma mídia
//! (05 §4.1: compara-se à largura MEDIDA igual). Nenhum produto muda: o candidato é um `Preset` em
//! memória. A régua é a do outro lado (`ferramentas/mede_todos.sh`).
//!
//! ```text
//! PH2D_SONDA_OUT=/tmp/x cargo test -p ph2d-tool-painter --profile smoke --test it \
//!     sonda_midias -- --ignored --nocapture --test-threads 1
//! ```
//! Escreve `<candidato>.ppm` (a cena) e imprime a lei raio → largura de cada candidato; e
//! `a_lei_densa_dos_presets_de_fabrica` imprime a mesma lei, densa, para os quatro de fábrica (W2).

use super::sonda_gestos::{fnv1a, largura_que_se_ve, pinta_cena_ate, tela, traco_recto_com};
use ph2d_brush_presets::{Biblioteca, Parametros, Preset, Tilt};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_tool_painter::{MEIO_DO_PRESET, PaintMedia, PainterTool, ids};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Uma composição: os knobs do motor por chave, os campos do pincel por nome, e os volumes.
struct Candidato {
    nome: &'static str,
    /// A foto do Rebelle da mídia (`fixtures/rebelle/<foto>.png`) e a largura dela, recto.
    foto: &'static str,
    alvo: f32,
    water: f64,
    knobs: &'static [(&'static str, f64)],
    brush: fn() -> Vec<(&'static str, Value)>,
}

fn nada() -> Vec<(&'static str, Value)> {
    Vec::new()
}
fn duro() -> Vec<(&'static str, Value)> {
    vec![("hardness", json!(0.9))]
}
fn meio_duro() -> Vec<(&'static str, Value)> {
    vec![("hardness", json!(0.6))]
}
fn chato() -> Vec<(&'static str, Value)> {
    vec![("dab_flatten", json!(0.6)), ("dab_angle_deg", json!(90))]
}
fn chato_duro() -> Vec<(&'static str, Value)> {
    vec![
        ("dab_flatten", json!(0.6)),
        ("dab_angle_deg", json!(90)),
        ("hardness", json!(0.8)),
    ]
}

const ROUND: &[(&str, f64)] = &[];
const WASH: &[(&str, f64)] = &[
    ("pigmentPerDab", 21.0),
    ("paperGate", 0.3),
    ("evaporation", 0.7),
    ("pickup", 0.003),
    ("edgeDarkening", 20.0),
    ("bristleSize", 3.0),
    ("bristleCount", 3000.0),
];

#[rustfmt::skip]
const CANDIDATOS: &[Candidato] = &[
    // Aquarela — o CONTROLO (Wet Round de fábrica) e as variantes do 02 §6.1.
    Candidato { nome: "aq_round", foto: "wcround", alvo: 32.0, water: 1.0, knobs: ROUND, brush: nada },
    Candidato { nome: "aq_wash", foto: "wcwash", alvo: 40.0, water: 0.4, knobs: WASH, brush: nada },
    Candidato { nome: "aq_wash_b", foto: "wcwash", alvo: 40.0, water: 0.6, knobs: &[("pigmentPerDab", 8.0), ("edgeDarkening", 40.0), ("paperGate", 0.2)], brush: meio_duro },
    Candidato { nome: "aq_flat", foto: "wcflat", alvo: 34.0, water: 1.0, knobs: &[("pigmentPerDab", 40.0)], brush: chato },
    Candidato { nome: "aq_flat_b", foto: "wcflat", alvo: 34.0, water: 0.6, knobs: &[("pigmentPerDab", 30.0), ("edgeDarkening", 20.0)], brush: chato_duro },
    Candidato { nome: "aq_round_b", foto: "wcround", alvo: 32.0, water: 0.6, knobs: &[("pigmentPerDab", 300.0), ("felt", 0.1)], brush: nada },
    Candidato { nome: "aq_wash_c", foto: "wcwash", alvo: 40.0, water: 0.6, knobs: &[("pigmentPerDab", 10.0), ("edgeDarkening", 40.0), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.5)], brush: meio_duro },
    Candidato { nome: "aq_flat_c", foto: "wcflat", alvo: 34.0, water: 0.6, knobs: &[("pigmentPerDab", 60.0), ("edgeDarkening", 20.0), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.5)], brush: chato_duro },
    Candidato { nome: "gu_d", foto: "gouache", alvo: 38.0, water: 0.4, knobs: &[("pigmentPerDab", 25.0), ("edgeDarkening", 0.0), ("paperGate", 0.1), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.5)], brush: duro },
    Candidato { nome: "aq_liner", foto: "wcliner", alvo: 38.0, water: 0.4, knobs: &[("pigmentPerDab", 150.0)], brush: duro },
    // Guache — chapado, sem orla escura, papel que não fura.
    Candidato { nome: "gu_a", foto: "gouache", alvo: 38.0, water: 0.4, knobs: &[("pigmentPerDab", 40.0), ("edgeDarkening", 0.0), ("paperGate", 0.1)], brush: meio_duro },
    Candidato { nome: "gu_b", foto: "gouache", alvo: 38.0, water: 0.3, knobs: &[("pigmentPerDab", 60.0), ("edgeDarkening", 0.0), ("paperGate", 0.0), ("felt", 0.1)], brush: duro },
    Candidato { nome: "gu_c", foto: "gouacheflat", alvo: 39.0, water: 0.4, knobs: &[("pigmentPerDab", 150.0), ("edgeDarkening", 0.0), ("paperGate", 0.1), ("felt", 0.06), ("bristleSize", 1.0), ("bristleCount", 3000.0)], brush: chato_duro },
    // Nanquim (Ink Brush) — escuro, liso.
    Candidato { nome: "na_a", foto: "inkbrush", alvo: 40.0, water: 0.3, knobs: &[("pigmentPerDab", 400.0), ("edgeDarkening", 0.0), ("paperGate", 0.0), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.5)], brush: meio_duro },
    Candidato { nome: "na_b", foto: "inkbrush", alvo: 40.0, water: 0.5, knobs: &[("pigmentPerDab", 250.0), ("edgeDarkening", 10.0), ("paperGate", 0.1), ("felt", 0.1)], brush: nada },
    Candidato { nome: "na_c", foto: "sumi", alvo: 33.0, water: 0.6, knobs: &[("pigmentPerDab", 150.0), ("edgeDarkening", 10.0), ("paperGate", 0.2), ("felt", 0.05)], brush: nada },
    // Caneta (Bullet, Dip Pen) — escura, lisa, borda limpa.
    Candidato { nome: "ca_a", foto: "dippen", alvo: 38.0, water: 0.3, knobs: &[("pigmentPerDab", 1500.0), ("edgeDarkening", 0.0), ("paperGate", 0.0), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.3)], brush: duro },
    Candidato { nome: "ca_b", foto: "ink", alvo: 42.0, water: 0.15, knobs: &[("pigmentPerDab", 2000.0), ("edgeDarkening", 0.0), ("paperGate", 0.0), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.3)], brush: duro },
    // Óleo / acrílica — corpo cheio com fios (sem altura: a W3 não existe).
    Candidato { nome: "ol_a", foto: "oil", alvo: 41.0, water: 0.05, knobs: &[("pigmentPerDab", 2000.0), ("paperGate", 0.0), ("edgeDarkening", 0.0), ("bristleStrength", 2.0), ("bristleSize", 2.0), ("bristleCount", 1500.0)], brush: duro },
    Candidato { nome: "ol_b", foto: "oilround", alvo: 39.0, water: 0.1, knobs: &[("pigmentPerDab", 1500.0), ("paperGate", 0.0), ("edgeDarkening", 0.0), ("bristleSize", 3.0), ("bristleCount", 800.0), ("felt", 0.0)], brush: chato },
    // Lápis / carvão — sem água, pouco pigmento, o papel a furar.
    Candidato { nome: "la_a", foto: "pencil", alvo: 30.0, water: 0.0, knobs: &[("pigmentPerDab", 150.0), ("paperGate", 1.5), ("felt", 0.0), ("bristleCount", 800.0)], brush: duro },
    Candidato { nome: "la_b", foto: "pencil", alvo: 30.0, water: 0.0, knobs: &[("pigmentPerDab", 400.0), ("paperGate", 1.2), ("gateSaturation", 300.0), ("felt", 0.0), ("bristleSize", 0.5)], brush: duro },
    Candidato { nome: "la_c", foto: "charcoal", alvo: 21.0, water: 0.0, knobs: &[("pigmentPerDab", 80.0), ("paperGate", 1.5), ("felt", 0.0), ("bristleCount", 600.0), ("bristleSize", 1.0)], brush: nada },
    // Pastel seco / pastel de óleo / giz de cera.
    Candidato { nome: "pa_a", foto: "pastel", alvo: 17.0, water: 0.0, knobs: &[("pigmentPerDab", 600.0), ("paperGate", 1.0), ("felt", 0.0), ("bristleSize", 1.0)], brush: duro },
    Candidato { nome: "pa_b", foto: "oilpastel2", alvo: 30.0, water: 0.0, knobs: &[("pigmentPerDab", 300.0), ("paperGate", 1.5), ("felt", 0.0), ("bristleSize", 2.0), ("bristleCount", 1000.0)], brush: nada },
    // Marcador — cobertura lisa e cheia, sem orla.
    Candidato { nome: "ma_a", foto: "marker", alvo: 18.0, water: 0.1, knobs: &[("pigmentPerDab", 800.0), ("paperGate", 0.0), ("edgeDarkening", 0.0), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.3)], brush: duro },
    Candidato { nome: "ma_b", foto: "markerbullet", alvo: 40.0, water: 0.1, knobs: &[("pigmentPerDab", 800.0), ("paperGate", 0.0), ("edgeDarkening", 0.0), ("felt", 0.2), ("bristleCount", 4000.0), ("bristleSize", 0.3)], brush: duro },
    // Aerógrafo — névoa clara, orla larga.
    Candidato { nome: "ae_a", foto: "airbrush", alvo: 26.0, water: 0.0, knobs: &[("pigmentPerDab", 20.0), ("paperGate", 0.0), ("edgeDarkening", 0.0), ("felt", 0.2), ("bristleCount", 4000.0)], brush: nada },
];

const DIREITA: Tilt = Tilt {
    on: false,
    ring: 4,
    spoke: 3,
};

/// Os raios onde se lê a lei de cada candidato (a largura alvo interpola-se entre eles).
const RAIOS_CANDIDATO: [f32; 11] = [
    5.0, 8.0, 11.0, 14.0, 18.0, 22.0, 27.0, 33.0, 40.0, 50.0, 62.0,
];

/// Uma receita da sonda, dona dos dados: de [`CANDIDATOS`] ou de um ficheiro JSON
/// (`PH2D_SONDA_RECEITAS=<f>`: `[{"nome","foto","alvo","water","knobs":{…},"brush":{…}}]`) — a varredura
/// sem recompilar. Com `"base": "<preset de fábrica>"` e `"raio"`, parte DESSE preset (os knobs e os campos
/// por cima dos dele, `water`/`opacity` só se vierem) e pinta a cena ao raio dado, como
/// `cada_preset_parece_a_sua_midia` (prompt 23: a varredura da família C).
struct Receita {
    nome: String,
    foto: String,
    alvo: f32,
    water: f64,
    knobs: Vec<(String, f64)>,
    brush: Vec<(String, Value)>,
    heavy_edge: Option<f64>,
    base: Option<String>,
    raio: Option<f32>,
    opacity: Option<f64>,
}

fn receitas() -> Vec<Receita> {
    if let Ok(f) = std::env::var("PH2D_SONDA_RECEITAS") {
        let v: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        return v
            .iter()
            .map(|r| Receita {
                nome: r["nome"].as_str().unwrap().to_owned(),
                foto: r["foto"].as_str().unwrap_or("").to_owned(),
                alvo: r["alvo"].as_f64().unwrap_or(0.0) as f32,
                water: r["water"].as_f64().unwrap_or(f64::NAN),
                knobs: r["knobs"].as_object().map_or(Vec::new(), |o| {
                    o.iter()
                        .map(|(k, v)| (k.clone(), v.as_f64().unwrap()))
                        .collect()
                }),
                brush: r["brush"].as_object().map_or(Vec::new(), |o| {
                    o.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                }),
                heavy_edge: r["heavy_edge"].as_f64(),
                base: r["base"].as_str().map(str::to_owned),
                raio: r["raio"].as_f64().map(|v| v as f32),
                opacity: r["opacity"].as_f64(),
            })
            .collect();
    }
    CANDIDATOS
        .iter()
        .map(|c| Receita {
            nome: c.nome.to_owned(),
            foto: c.foto.to_owned(),
            alvo: c.alvo,
            water: c.water,
            knobs: c.knobs.iter().map(|&(k, v)| (k.to_owned(), v)).collect(),
            brush: (c.brush)()
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v))
                .collect(),
            heavy_edge: None,
            base: None,
            raio: None,
            opacity: None,
        })
        .collect()
}

fn preset_de(c: &Receita) -> Preset {
    if let Some(base) = &c.base {
        let mut p = super::midias_de_fabrica::de_fabrica(base);
        let q = &mut p.parametros;
        if !c.water.is_nan() {
            q.volumes.water = Some(c.water);
        }
        if let Some(o) = c.opacity {
            q.volumes.opacity = Some(o);
        }
        // Qualquer chave do REGISTO (o preset só grava as que usa: a varredura dos pincéis novos parte do vizinho).
        for (k, v) in &c.knobs {
            assert!(
                ph2d_wet_paint::tuning::KNOB_DEFS.iter().any(|d| d.key == k),
                "knob {k}"
            );
            q.engine.insert(k.clone(), *v);
        }
        // Um campo que o ficheiro de fábrica ainda não grava (a ponta reta, prompt 44) entra também — desde
        // que seja do pincel.
        for (k, v) in &c.brush {
            assert!(
                ph2d_tool_painter::CAMPOS_DO_PINCEL.contains(&k.as_str()),
                "campo {k}"
            );
            q.brush.insert(k.clone(), v.clone());
        }
        return p;
    }
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let (mut p, _): (Parametros, _) =
        t.wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "Sonda", &c.nome));
    p.volumes.size_px = None;
    p.volumes.opacity = Some(1.0);
    p.volumes.water = Some(c.water);
    p.live_water = None;
    p.heavy_edge = c.heavy_edge;
    // A tela direita: a largura é a do TRAÇO; o escorrimento é da tela (W1e).
    p.tilt = Some(DIREITA);
    // A captura guarda só os knobs que o pincel mostra (as cerdas e o feltro saíram para o Shape a 13/10):
    // o candidato pode pôr qualquer chave do REGISTO.
    for (k, v) in &c.knobs {
        assert!(
            ph2d_wet_paint::tuning::KNOB_DEFS.iter().any(|d| d.key == k),
            "knob {k}"
        );
        p.engine.insert(k.clone(), *v);
    }
    for (k, v) in &c.brush {
        assert!(p.brush.insert(k.clone(), v.clone()).is_some(), "campo {k}");
    }
    Preset {
        parametros: p,
        origem: ph2d_brush_presets::Origem::Utilizador,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas: Default::default(),
    }
}

fn largura_ao_raio(p: &Preset, r: f32) -> f32 {
    let (px, w, h) = traco_recto_com(r, ticks(), |t| {
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(p).is_empty());
    });
    largura_que_se_ve(&px, w, h)
}

/// O raio cuja largura é `alvo`, por interpolação linear no primeiro intervalo que o contém.
fn raio_para(lei: &[(f32, f32)], alvo: f32) -> Option<f32> {
    lei.windows(2).find_map(|w| {
        let ((r0, l0), (r1, l1)) = (w[0], w[1]);
        (l0 <= alvo && alvo <= l1 && l1 > l0).then(|| r0 + (r1 - r0) * (alvo - l0) / (l1 - l0))
    })
}

/// Os quadros de 60 Hz depois do traço (`PH2D_SONDA_TICKS`, 480 = os 8 s da cena): a largura que se
/// promete é a que FICA depois de a água correr, não a do instante do traço.
fn ticks() -> u32 {
    std::env::var("PH2D_SONDA_TICKS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(480)
}

fn escreve_ppm(path: &Path, rgba: &[u8], w: u32, h: u32) {
    let mut f = Vec::with_capacity((w * h * 3) as usize + 32);
    f.extend_from_slice(format!("P6\n{w} {h}\n255\n").as_bytes());
    for px in rgba.as_chunks::<4>().0 {
        f.extend_from_slice(&px[..3]);
    }
    std::fs::write(path, f).unwrap();
}

fn saida() -> PathBuf {
    let out = std::env::var("PH2D_SONDA_OUT").map_or_else(
        |_| std::env::temp_dir().join("ph2d_sonda_midias"),
        PathBuf::from,
    );
    std::fs::create_dir_all(&out).unwrap();
    out
}

/// Uma LEI candidata do motor (prompt 42): `PH2D_SONDA_LEI="nome:k=v,k=v|nome2:…"` — cada receita pinta-se
/// uma vez por variante (`<receita>@<variante>.ppm`). Chaves: `obra_km`/`obra_glaze` (as caixas da obra,
/// pelo painel) e, no motor, `km` · `glaze` (a velatura das junções) · `filme` (a do filme sobre o
/// assentado) · `esconde` · `camadas`.
type Variante = (String, Vec<(String, f64)>);

fn variantes() -> Vec<Variante> {
    let Ok(v) = std::env::var("PH2D_SONDA_LEI") else {
        return vec![(String::new(), Vec::new())];
    };
    v.split('|')
        .map(|s| {
            let (nome, kv) = s.split_once(':').unwrap_or((s, ""));
            let kv = kv
                .split(',')
                .filter(|x| !x.is_empty())
                .map(|x| {
                    let (k, v) = x.split_once('=').unwrap();
                    (k.to_owned(), v.parse().unwrap())
                })
                .collect();
            (nome.to_owned(), kv)
        })
        .collect()
}

fn aplica_variante(t: &mut PainterTool, v: &[(String, f64)]) {
    for (k, x) in v {
        match k.as_str() {
            "obra_km" if *x > 0.0 => {
                t.handle_panel_event(PanelEvent::Click(ids::WET_TUNING_KM_MIXING))
            }
            "obra_glaze" if *x > 0.0 => {
                t.handle_panel_event(PanelEvent::Click(ids::WET_TUNING_KM_GLAZE))
            }
            _ => {}
        }
    }
    t.wet_sonda_motor(|e| {
        for (k, x) in v {
            match k.as_str() {
                "km" => e.sim.km_mixing = *x > 0.0,
                "glaze" => e.sim.glaze = *x,
                "filme" => e.sim.glaze_filme = *x,
                "esconde" => e.sim.esconde = *x,
                "camadas" => e.sim.camadas_transparentes = *x > 0.0,
                "obra_km" | "obra_glaze" => {}
                k if k.starts_with("k_") => {}
                o => panic!("chave da lei: {o}"),
            }
        }
    });
}

#[test]
#[ignore = "sonda: o mapa das mídias (prompt 17, Fase A); corre-se à mão"]
fn as_midias_com_os_knobs_de_hoje() {
    let out = saida();
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let todas = receitas();
    let leis = variantes();
    std::thread::scope(|s| {
        for c in todas.iter().filter(|c| {
            so.as_deref()
                .is_none_or(|q| q.split(',').any(|n| n == c.nome))
        }) {
            let (out, leis) = (&out, &leis);
            s.spawn(move || {
                let p = preset_de(c);
                let lei: Vec<(f32, f32)> = if c.raio.is_some() {
                    Vec::new()
                } else {
                    RAIOS_CANDIDATO
                        .iter()
                        .map(|&r| (r, largura_ao_raio(&p, r)))
                        .collect()
                };
                let r = c.raio.or_else(|| raio_para(&lei, c.alvo));
                let lei: Vec<String> = lei.iter().map(|(r, l)| format!("{r}→{l}")).collect();
                std::thread::scope(|s2| {
                    for (vnome, v) in leis {
                        let lei = &lei;
                        // `k_<chave>=v`: um knob do PINCEL (o preset), pela mesma porta do produto.
                        let mut p = p.clone();
                        for (k, x) in v {
                            if let Some(k) = k.strip_prefix("k_") {
                                p.parametros.engine.insert(k.to_owned(), *x);
                            }
                        }
                        s2.spawn(move || {
                            let mut t = tela();
                            t.set_wet_relogio_fixo(true);
                            assert!(t.wet_preset_aplicar(&p).is_empty());
                            t.set_brush_size_px(r.unwrap_or(c.alvo * 0.5));
                            // `PH2D_SONDA_KM=1`: as duas caixas K–M da tela (Pigment mixing + Glaze layering)
                            // ligadas — as opções da OBRA que nenhum preset guarda.
                            if std::env::var("PH2D_SONDA_KM").as_deref() == Ok("1") {
                                for id in [ids::WET_TUNING_KM_MIXING, ids::WET_TUNING_KM_GLAZE] {
                                    t.handle_panel_event(PanelEvent::Click(id));
                                }
                            }
                            aplica_variante(&mut t, v);
                            // `PH2D_SONDA_SECA=<quadros>`: mais tempo depois dos 8 s da cena (o Rebelle é
                            // fotografado SECO).
                            let extra = std::env::var("PH2D_SONDA_SECA")
                                .ok()
                                .and_then(|v| v.parse::<u32>().ok());
                            let (px, w, h) = pinta_cena_ate(&mut t, 480 + extra.unwrap_or(0));
                            let nome = if vnome.is_empty() {
                                c.nome.clone()
                            } else {
                                format!("{}@{vnome}", c.nome)
                            };
                            escreve_ppm(&out.join(format!("{nome}.ppm")), &px, w, h);
                            println!(
                                "[midia] {:<9} foto {:<12} alvo {:>4} raio {} fnv {:#018x} · lei {}",
                                nome,
                                c.foto,
                                c.alvo,
                                r.map_or("FORA DA LEI".to_owned(), |r| format!("{r:.2}")),
                                fnv1a(&px),
                                lei.join(" ")
                            );
                        });
                    }
                });
            });
        }
    });
}

/// A lei raio → largura DENSA dos presets de fábrica, logo depois do traço (relógio fixo) — a base
/// da W2 (05 §1.1). `PH2D_SONDA_OUT` recebe os traços de raio ≤ 40 para o perfil transversal.
#[test]
#[ignore = "sonda: a lei densa raio → largura dos presets de fábrica (W2); corre-se à mão"]
fn a_lei_densa_dos_presets_de_fabrica() {
    let out = saida();
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let bib = Biblioteca::carregar(&raiz, None);
    let rmax: f32 = std::env::var("PH2D_SONDA_RMAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300.0);
    let raios: Vec<f32> = (0..=44)
        .map(|i| (rmax.powf(i as f32 / 44.0) * 10.0).round() / 10.0)
        .collect();
    std::thread::scope(|s| {
        // Só os do Wet Paint (os outros meios têm presets irmãos desde a J3).
        for e in bib.entradas.iter().filter(|e| e.preset.parametros.media == ph2d_tool_painter::MEIO_DO_PRESET) {
            let (out, raios) = (&out, &raios);
            s.spawn(move || {
                let t0 = std::time::Instant::now();
                let nome = &e.preset.parametros.name;
                // A tela direita (o Wet Drip vem inclinado): a largura é a do TRAÇO.
                let mut direito = e.preset.clone();
                direito.parametros.tilt = Some(DIREITA);
                let direito = &direito;
                let lei: Vec<String> = raios
                    .iter()
                    .map(|&r| {
                        let (px, w, h) = traco_recto_com(r, ticks(), |t| {
                            t.set_wet_relogio_fixo(true);
                            assert!(t.wet_preset_aplicar(direito).is_empty());
                        });
                        if r <= 40.0 {
                            escreve_ppm(&out.join(format!("lei_{nome}_{r}.ppm")), &px, w, h);
                        }
                        format!("{r}→{}", largura_que_se_ve(&px, w, h))
                    })
                    .collect();
                println!(
                    "[lei] {nome:<14} ({:.0} s) {}",
                    t0.elapsed().as_secs_f32(),
                    lei.join(" ")
                );
            });
        }
    });
}

/// **SONDA (prompt 42): a CÉLULA do cruzamento** — o azul recto da linha 3, depois o vermelho que o cruza,
/// depois a cena seca; o estado do motor (assentado, suspenso, água) no cruzamento e no azul ao lado, para
/// cada receita × [`variantes`]. Mostra ONDE a tinta de cima tapa a de baixo.
#[test]
#[ignore = "sonda: a célula do cruzamento (prompt 42); corre-se à mão"]
fn sonda_a_celula_do_cruzamento() {
    use super::sonda_gestos::{AZUL, VERMELHO, gesto};
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let leis = variantes();
    for c in receitas().iter().filter(|c| {
        so.as_deref()
            .is_none_or(|q| q.split(',').any(|n| n == c.nome))
    }) {
        let p0 = preset_de(c);
        for (vnome, v) in &leis {
            let mut p = p0.clone();
            for (k, x) in v {
                if let Some(k) = k.strip_prefix("k_") {
                    p.parametros.engine.insert(k.to_owned(), *x);
                }
            }
            let mut t = tela();
            t.set_wet_relogio_fixo(true);
            assert!(t.wet_preset_aplicar(&p).is_empty());
            t.set_brush_size_px(c.raio.unwrap_or(c.alvo * 0.5));
            aplica_variante(&mut t, v);
            let le = |t: &mut PainterTool, quando: &str| {
                t.wet_sonda_motor(|e| {
                    let g = e.active_grid();
                    let razao = 1000.0 / g.w as f64;
                    let cel = |x: f64, y: f64| {
                        let (cx, cy) = ((x / razao) as usize + 1, (y / razao) as usize + 1);
                        cx + cy * g.s
                    };
                    for (nome, i) in [("cruz", cel(300.0, 480.0)), ("azul", cel(200.0, 480.0))] {
                        println!(
                            "[{}@{vnome}] {quando:<8} {nome}: sett {:7.1} {:?} · susp {:7.1} {:?} · film {:.4}",
                            c.nome,
                            g.sett[i],
                            g.sett_rgb[i].map(|v| v.round()),
                            g.susp[i],
                            g.susp_rgb[i].map(|v| v.round()),
                            g.film[i]
                        );
                        let e = e.sim.gather_params(&e.tuning).ecra.com_vela(g.vela[i]);
                        let a = |m: f32| ph2d_wet_paint::opacity::alpha_of_mass(f64::from(m));
                        let ecra = ph2d_wet_paint::assenta::sobre_branco(
                            e.seco(),
                            g.sett_rgb[i].map(f64::from),
                            g.susp_rgb[i].map(f64::from),
                            a(g.sett[i]),
                            a(g.susp[i]),
                        );
                        println!("    vela {:.3} ecrã {:?}", g.vela[i], ecra.map(f64::round));
                    }
                });
            };
            t.set_brush_color_srgb8(AZUL);
            gesto(&mut t, &[[60.0, 480.0], [560.0, 480.0]], 3.0, |_| 1.0);
            le(&mut t, "azul");
            t.set_brush_color_srgb8(VERMELHO);
            gesto(&mut t, &[[300.0, 400.0], [300.0, 560.0]], 3.0, |_| 1.0);
            le(&mut t, "vermelho");
            for _ in 0..1800 {
                t.on_tick(1000.0 / 60.0);
            }
            le(&mut t, "seco");
        }
    }
}

/// **SONDA (prompt 42b): o PERFIL do cruzamento no tempo** — as células da linha azul (`y` 480) de um lado ao outro
/// do vermelho (`PH2D_SONDA_XS="250,260,…"`, na escala da cena), depois do azul, depois do vermelho e em
/// `PH2D_SONDA_TEMPOS` quadros: assentado, suspenso, filme, vela e o ecrã seco. Mostra onde nasce o halo.
#[test]
#[ignore = "sonda: o perfil do cruzamento no tempo (prompt 42b); corre-se à mão"]
fn sonda_o_perfil_do_cruzamento() {
    use super::sonda_gestos::{AZUL, VERMELHO, gesto};
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let xs: Vec<f64> = std::env::var("PH2D_SONDA_XS").map_or_else(
        |_| (0..=20).map(|k| 250.0 + 5.0 * f64::from(k)).collect(),
        |v| v.split(',').map(|x| x.parse().unwrap()).collect(),
    );
    let tempos: Vec<u32> = std::env::var("PH2D_SONDA_TEMPOS").map_or_else(
        |_| vec![0, 30, 120, 480, 1800],
        |v| v.split(',').map(|x| x.parse().unwrap()).collect(),
    );
    let ys: Vec<f64> = std::env::var("PH2D_SONDA_YS").map_or_else(
        |_| vec![480.0],
        |v| v.split(',').map(|x| x.parse().unwrap()).collect(),
    );
    let leis = variantes();
    for c in receitas().iter().filter(|c| {
        so.as_deref()
            .is_none_or(|q| q.split(',').any(|n| n == c.nome))
    }) {
        let p0 = preset_de(c);
        for (vnome, v) in &leis {
            let mut p = p0.clone();
            for (k, x) in v {
                if let Some(k) = k.strip_prefix("k_") {
                    p.parametros.engine.insert(k.to_owned(), *x);
                }
            }
            let mut t = tela();
            t.set_wet_relogio_fixo(true);
            assert!(t.wet_preset_aplicar(&p).is_empty());
            t.set_brush_size_px(c.raio.unwrap_or(c.alvo * 0.5));
            aplica_variante(&mut t, v);
            let (xs, ys) = (&xs, &ys);
            let le = |t: &mut PainterTool, quando: &str| {
                t.wet_sonda_motor(|e| {
                    let g = e.active_grid();
                    let razao = 1000.0 / g.w as f64;
                    let ecra0 = e.sim.gather_params(&e.tuning).ecra;
                    for &y in ys {
                        println!("[perfil {}@{vnome}] {quando} y {y} (razão {razao:.2})", c.nome);
                        for &x in xs {
                            let (cx, cy) = ((x / razao) as usize + 1, (y / razao) as usize + 1);
                            let i = cx + cy * g.s;
                            let ec = ecra0.com_vela(g.vela[i]);
                            let a = |m: f32| ph2d_wet_paint::opacity::alpha_of_mass(f64::from(m));
                            let ecra = ph2d_wet_paint::assenta::sobre_branco(
                                ec.seco(),
                                g.sett_rgb[i].map(f64::from),
                                g.susp_rgb[i].map(f64::from),
                                a(g.sett[i]),
                                a(g.susp[i]),
                            );
                            println!(
                                "  x {x:5.0} sett {:7.1} {:?} susp {:7.1} {:?} film {:.4} wet {:3} vela {:.3} → {:?}",
                                g.sett[i],
                                g.sett_rgb[i].map(|v| v.round() as i32),
                                g.susp[i],
                                g.susp_rgb[i].map(|v| v.round() as i32),
                                g.film[i],
                                g.wet[i],
                                g.vela[i],
                                ecra.map(|v| v.round() as i32)
                            );
                        }
                    }
                });
            };
            // `PH2D_SONDA_CENA=azul|vermelho`: só um dos traços (a massa de cada um, para o balanço do cruzamento).
            let cena = std::env::var("PH2D_SONDA_CENA").unwrap_or_default();
            if cena != "vermelho" {
                t.set_brush_color_srgb8(AZUL);
                gesto(&mut t, &[[60.0, 480.0], [560.0, 480.0]], 3.0, |_| 1.0);
                le(&mut t, "azul");
            }
            if cena != "azul" {
                t.set_brush_color_srgb8(VERMELHO);
                gesto(&mut t, &[[300.0, 400.0], [300.0, 560.0]], 3.0, |_| 1.0);
            }
            let mut feito = 0;
            for &q in &tempos {
                for _ in feito..q {
                    t.on_tick(1000.0 / 60.0);
                }
                feito = q;
                le(&mut t, &format!("vermelho+{q}"));
            }
            if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
                let (px, w, h) = ph2d_editor_core::tool::RasterEditTool::run_full(&mut t);
                std::fs::create_dir_all(&dir).unwrap();
                escreve_ppm(
                    &Path::new(&dir).join(format!("perfil_{}@{vnome}.ppm", c.nome)),
                    &px,
                    w,
                    h,
                );
            }
        }
    }
}

/// **SONDA (prompt 42): o CUSTO do Glaze** — o cruzamento da bancada (o azul recto e o vermelho que o cruza) e
/// 120 quadros de água, com o Glaze do pincel 0 e 1 INTERCALADOS no mesmo processo; o mínimo de 9 por variante
/// (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
#[ignore = "sonda: o custo do Glaze (prompt 42); corre-se à mão"]
fn sonda_o_custo_do_glaze() {
    use super::sonda_gestos::{AZUL, VERMELHO, gesto};
    let base = super::midias_de_fabrica::de_fabrica("Wet Liner");
    let mut minimos = [(f64::MAX, f64::MAX); 2];
    for volta in 0..18 {
        let k = volta % 2;
        let mut p = base.clone();
        p.parametros.engine.insert("glaze".to_owned(), k as f64);
        let mut t = tela();
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        t.set_brush_size_px(18.0);
        t.set_brush_color_srgb8(AZUL);
        gesto(&mut t, &[[60.0, 480.0], [560.0, 480.0]], 3.0, |_| 1.0);
        // `PH2D_SONDA_MESMA=1`: o traço que cruza é da MESMA tinta (o primeiro toque assenta a de baixo, doc 42 §7).
        let mesma = std::env::var("PH2D_SONDA_MESMA").is_ok_and(|v| v == "1");
        t.set_brush_color_srgb8(if mesma { AZUL } else { VERMELHO });
        let t0 = std::time::Instant::now();
        gesto(&mut t, &[[300.0, 400.0], [300.0, 560.0]], 3.0, |_| 1.0);
        let traco = t0.elapsed().as_secs_f64() * 1e3;
        let t1 = std::time::Instant::now();
        for _ in 0..120 {
            t.on_tick(1000.0 / 60.0);
        }
        let agua = t1.elapsed().as_secs_f64() * 1e3;
        minimos[k] = (minimos[k].0.min(traco), minimos[k].1.min(agua));
    }
    for (k, (traco, agua)) in minimos.iter().enumerate() {
        println!("[custo] Glaze {k}: traço vermelho {traco:.1} ms · 120 quadros {agua:.1} ms");
    }
}
