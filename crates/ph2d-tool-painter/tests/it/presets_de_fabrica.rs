//! **Os presets de FÁBRICA do Wet Painter** (`assets/brushes/WetPainter/`, W1): contados, ao byte, e a
//! miniatura de cada um é a que o motor de HOJE pinta — o corpus de regressão do motor (02 §5).
//!
//! Os valores vêm do doc 02 §6.9 (05 §1.2 diz como as colunas viram unidades do motor). Regerar depois
//! de uma mudança de lei do motor (a miniatura acusa):
//!
//! ```text
//! PH2D_REGERA_PRESETS=1 cargo test -p ph2d-tool-painter --profile smoke --test it \
//!     gera_os_presets_de_fabrica -- --ignored --nocapture
//! ```
//!
//! `PH2D_REGERA_SO="Ink Brush,Dip Pen"` regera só esses.

use super::receitas_de_fabrica::{DIREITA, PRESETS_DE_FABRICA, RECEITAS, Receita, grao};
use super::sonda_gestos::{fnv1a, largura_que_se_ve, traco_recto_alto};
use ph2d_brush_presets::{
    Biblioteca, Origem, Parametros, Preset, caminho_relativo, escrever, gravar, ler,
};
use ph2d_tool_painter::{
    MEIO_DO_PRESET, MOTOR_DOS_PRESETS, PaintMedia, PainterTool, WetKnobs, miniatura_do_preset,
};
use ph2d_wet_paint::tuning::Knob;
use std::path::PathBuf;

/// Os raios do MOTOR da lei raio → largura (W2, 05 §4b): geométricos de 1,5 a 1 100 — a largura 600
/// (o raio 300 do portão) pede ~860 no Wet Round. O slider pára em 512 (`BRUSH_SIZE_MAX_PX`): acima, a
/// sonda pede metade e o motor recebe o dobro (`set_wet_largura_crua(2)`).
const PONTOS_DA_LEI: usize = 64;
const RAIO_MIN: f32 = 1.5;
const RAIO_MAX: f32 = 1100.0;
/// Os quadros de 60 Hz depois do traço: a largura que se promete é a que FICA, com a água assentada
/// (a 0 quadros o Wet Round a raio 30 lê 8 px; aos 8 s, 32 — 05 §4b).
const QUADROS_DA_LEI: u32 = 480;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes")
}

/// O pincel de fábrica do meio inteiro, com os knobs da receita. Tamanho e opacidade NÃO são fixados
/// (a tabela não os tem): fica o do artista.
///
/// ⚠️ **Parte do ficheiro de fábrica que JÁ existe** (prompt 35): desde a W2 do plano 27 as cerdas vivem no
/// Shape (`WetKnobs::cerdas_no_shape`) e gravar não as põe no motor — capturar de fresco perdia a ponta de
/// cada preset. Só um preset sem ficheiro nasce da captura.
fn parametros(r: &Receita) -> Parametros {
    let rel = caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome);
    let mut p = match std::fs::read(raiz().join(&rel)) {
        Ok(bytes) => ler(&bytes).expect("o ficheiro de fábrica lê-se").parametros,
        Err(_) => {
            let mut t = PainterTool::default();
            t.set_paint_media(PaintMedia::WetPaint);
            t.wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, r.grupo, r.nome))
                .0
        }
    };
    p.volumes.size_px = r.size_px;
    p.volumes.opacity = Some(r.opacity);
    p.volumes.water = Some(r.water);
    p.tilt = Some(r.tilt);
    // As opções da W1g não são do pincel de fábrica: aplicá-lo não mexe nelas (a regra do preset antigo).
    (p.live_water, p.heavy_edge) = (None, None);
    assert!(
        p.brush
            .insert("hardness".to_owned(), serde_json::json!(r.hardness))
            .is_some(),
        "campo hardness existe"
    );
    if let Some(s) = r.spacing {
        assert!(
            p.brush
                .insert("spacing".to_owned(), serde_json::json!(s))
                .is_some(),
            "campo spacing existe"
        );
    }
    if let Some((flatten, angulo)) = r.achatado {
        for (campo, v) in [
            ("dab_flatten", serde_json::json!(flatten)),
            ("dab_angle_deg", serde_json::json!(angulo)),
        ] {
            assert!(
                p.brush.insert(campo.to_owned(), v).is_some(),
                "campo {campo} existe"
            );
        }
    }
    for (campo, json) in r.pincel {
        let v: serde_json::Value = serde_json::from_str(json).expect("o JSON do campo");
        assert!(
            p.brush.insert((*campo).to_owned(), v).is_some(),
            "campo {campo} existe"
        );
    }
    if let Some((kind, lado)) = r.grao {
        assert!(
            p.brush
                .insert("texture".to_owned(), grao(kind, lado))
                .is_some()
        );
    }
    let d = WetKnobs::DEFAULT;
    for (chave, v) in [
        ("pigmentPerDab", r.pigment),
        ("paperGate", r.gate),
        ("evaporation", r.evaporation),
        ("pickup", d.get(Knob::Pickup) * r.pickup),
        ("edgeDarkening", r.edge),
    ]
    .into_iter()
    .chain(r.extras.iter().copied())
    {
        assert!(
            p.engine.insert(chave.to_owned(), v).is_some(),
            "knob {chave} existe"
        );
    }
    // As cerdas: no motor só num ficheiro de antes da W2; depois dela estão no Shape, e o ficheiro é a fonte.
    for (chave, v) in [
        ("bristleSize", r.bristle_size),
        ("bristleCount", r.bristle_count),
    ] {
        if let Some(antigo) = p.engine.get_mut(chave) {
            *antigo = v;
        }
    }
    p
}

fn raios_da_lei() -> Vec<f32> {
    (0..PONTOS_DA_LEI)
        .map(|i| {
            let f = i as f32 / (PONTOS_DA_LEI - 1) as f32;
            (RAIO_MIN * (RAIO_MAX / RAIO_MIN).powf(f) * 10.0).round() / 10.0
        })
        .collect()
}

/// O preset com a tela DIREITA: a largura é a do traço; o escorrimento do Wet Drip é da tela (W1e).
fn direito(p: &Preset) -> Preset {
    let mut d = p.clone();
    d.parametros.tilt = Some(DIREITA);
    d
}

/// A largura que o pincel `p` deixa a `raio`: `crua` = o raio chega ao MOTOR sem a promessa.
fn largura_de(p: &Preset, raio: f32, crua: bool) -> f32 {
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    let (px, w, h) = traco_recto_alto(raio / escala, raio, QUADROS_DA_LEI, |t| {
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&direito(p)).is_empty());
        if crua {
            t.set_wet_largura_crua(escala);
        }
    });
    largura_que_se_ve(&px, w, h)
}

/// A lei CRUA medida: raio do motor → largura, em paralelo (uma thread por raio).
fn tabela_de_largura(p: &Preset) -> Vec<[f64; 2]> {
    let raios = raios_da_lei();
    let larguras: Vec<f32> = std::thread::scope(|s| {
        let h: Vec<_> = raios
            .iter()
            .map(|&r| s.spawn(move || largura_de(p, r, true)))
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    raios
        .iter()
        .zip(larguras)
        // O raio pelo menor texto que volta ao mesmo f32 (7.8, não 7.800000190734863).
        .map(|(&r, l)| [format!("{r}").parse().unwrap(), f64::from(l)])
        .collect()
}

#[test]
#[ignore = "gerador: escreve assets/brushes/WetPainter (PH2D_REGERA_PRESETS=1)"]
fn gera_os_presets_de_fabrica() {
    let escreve = std::env::var("PH2D_REGERA_PRESETS").as_deref() == Ok("1");
    // `PH2D_REGERA_SO="Nome,Nome"`: só esses (cada preset custa ~2 min de lei da largura sob carga).
    let so = std::env::var("PH2D_REGERA_SO").ok();
    for r in RECEITAS.iter().filter(|r| {
        so.as_deref()
            .is_none_or(|s| s.split(',').any(|n| n == r.nome))
    }) {
        let mut preset = Preset {
            parametros: parametros(r),
            origem: Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![255; 4],
            },
            texturas: Default::default(),
        };
        preset.parametros.width_table = tabela_de_largura(&preset);
        if r.nome == "Wet Round" {
            // A tabela que o motor embute (`wetpaint/largura.rs::LARGURA_DO_WET_ROUND`), já limpa.
            let mut t = PainterTool::default();
            assert!(t.wet_preset_aplicar(&preset).is_empty());
            println!(
                "[fabrica] LARGURA_DO_WET_ROUND = {:?}",
                t.wet_largura_tabela()
            );
        }
        preset.miniatura = miniatura_do_preset(&preset);
        let larguras: Vec<String> = preset
            .parametros
            .width_table
            .iter()
            .map(|[r, l]| format!("{r}→{l}"))
            .collect();
        println!(
            "[fabrica] {:<14} miniatura fnv {:#018x} · largura {}",
            r.nome,
            fnv1a(&preset.miniatura.rgba),
            larguras.join(" ")
        );
        if escreve {
            let rel = caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome);
            gravar(&raiz(), &rel, &preset).unwrap();
        }
    }
    if !escreve {
        println!("[fabrica] nada escrito (PH2D_REGERA_PRESETS=1 para gravar)");
    }
}

fn ficheiros_de_fabrica() -> Vec<(String, Vec<u8>)> {
    Biblioteca::carregar(&raiz(), None)
        .entradas
        .into_iter()
        .map(|e| (e.relativo, std::fs::read(&e.caminho).unwrap()))
        .collect()
}

#[test]
fn the_factory_presets_are_counted() {
    let b = Biblioteca::carregar(&raiz(), None);
    assert!(
        b.erros.is_empty(),
        "ficheiros de fábrica ilegíveis: {:?}",
        b.erros
    );
    assert_eq!(
        b.entradas.len(),
        PRESETS_DE_FABRICA,
        "a fábrica mudou de tamanho: conte-a de novo aqui e no doc 05"
    );
    for e in &b.entradas {
        assert_eq!(e.partes()[0], MOTOR_DOS_PRESETS, "{}", e.relativo);
        assert_eq!(e.preset.parametros.media, MEIO_DO_PRESET, "{}", e.relativo);
        assert_eq!(e.preset.origem, Origem::Fabrica, "{}", e.relativo);
        assert_eq!(
            e.partes()[3],
            e.preset.parametros.name,
            "o nome do ficheiro é o do preset"
        );
    }
}

#[test]
fn cada_preset_de_fabrica_volta_ao_byte() {
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        assert!(escrever(&p).unwrap() == b, "{rel}: escrever(ler(b)) != b");
    }
}

#[test]
fn a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje() {
    let mut vistas = Vec::new();
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        let hoje = miniatura_do_preset(&p);
        let diferem = hoje
            .rgba
            .chunks(4)
            .zip(p.miniatura.rgba.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            hoje == p.miniatura,
            "{rel}: {diferem} px mudaram (fnv {:#018x} → {:#018x}) — o motor mudou de lei. Se foi de \
             propósito, regere a fábrica (cabeçalho deste ficheiro) e diga-o no handoff",
            fnv1a(&p.miniatura.rgba),
            fnv1a(&hoje.rgba)
        );
        vistas.push((rel, hoje.rgba));
    }
    // O leitor DECIDE: dois presets diferentes pintam miniaturas diferentes.
    for (i, (ra, a)) in vistas.iter().enumerate() {
        for (rb, b) in &vistas[i + 1..] {
            assert_ne!(
                a, b,
                "{ra} e {rb} pintam igual: um knob do preset não tem leitor"
            );
        }
    }
}

/// Os raios do portão da W2 (05 §3): fora da tabela, de 2 a 300.
const RAIOS_DA_PROMESSA: [f32; 8] = [2.0, 3.5, 7.0, 13.0, 26.0, 55.0, 120.0, 300.0];
/// A promessa do dono (05 §4.1): o número do painel é a largura que se vê ± 2 px — e, acima de 100 px,
/// ± 2 %: a lei raio → largura ONDULA de raio para raio (a franja das cerdas), e nenhuma tabela a segue
/// melhor (medido 11/10, 05 §4b W2: 64 e 128 pontos erram o mesmo, ≤ 1,8 %; cinco traços ao mesmo raio
/// variam ±1–3 px).
const TOLERANCIA_PX: f32 = 2.0;
const TOLERANCIA_REL: f32 = 0.02;
/// ⚠️ **Os pincéis com GRÃO (o slot Grain armado: grafite, carvão, os pastéis) — ± 10 %**: o grão e o
/// papel têm estrutura, e a largura a ¼ do máximo varia com ONDE o traço cai (medido 11/10,
/// `sonda_ruido_da_largura`, o traço a descer 30 px de cada vez: até 7,7 % no Graphite e 9,6 % no Charcoal
/// a 52 px; 3,8 % a 240). A cura é um grão sem estrutura grande, não a tabela (05 §4b).
const TOLERANCIA_REL_GRAO: f32 = 0.10;

/// As larguras de `p` nos [`RAIOS_DA_PROMESSA`], com a promessa (`crua = false`) ou sem ela.
fn larguras_da_promessa(p: &Preset, crua: bool) -> Vec<f32> {
    std::thread::scope(|s| {
        let h: Vec<_> = RAIOS_DA_PROMESSA
            .iter()
            .map(|&r| s.spawn(move || largura_de(p, r, crua)))
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn fora_da_promessa(larguras: &[f32], rel: f32) -> Vec<String> {
    RAIOS_DA_PROMESSA
        .iter()
        .zip(larguras)
        .filter(|(r, l)| (*l - 2.0 * *r).abs() > TOLERANCIA_PX.max(rel * 2.0 * *r))
        .map(|(r, l)| format!("raio {r}: {l} px (pedido {})", 2.0 * r))
        .collect()
}

/// **A LARGURA PROMETIDA (W2)**: em cada preset de fábrica, o traço a raio `r` mede `2·r ± 2 px` (± 2 %
/// acima de 100 px), de 2 a 300 — raios fora da tabela do preset. CONTROLO dentro: a lei CRUA (o raio direto ao motor, a de antes
/// da W2) reprova no mesmo pincel.
#[test]
fn a_largura_e_a_prometida() {
    let mut falhas = Vec::new();
    let ficheiros = ficheiros_de_fabrica();
    // Os presets em paralelo (cada um já pinta os seus raios em paralelo): medido 11/10, 55 → 51 s.
    let medidos: Vec<(Preset, Vec<f32>, Option<Vec<f32>>)> = std::thread::scope(|s| {
        let h: Vec<_> = ficheiros
            .iter()
            .map(|(rel, b)| {
                s.spawn(move || {
                    let p = ler(b).unwrap();
                    let (larguras, crua) = std::thread::scope(|s| {
                        let crua = rel
                            .ends_with("Wet Round.ph2d-brush.png")
                            .then(|| s.spawn(|| larguras_da_promessa(&p, true)));
                        let larguras = larguras_da_promessa(&p, false);
                        (larguras, crua.map(|h| h.join().unwrap()))
                    });
                    (p, larguras, crua)
                })
            })
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for ((rel, _), (p, larguras, crua)) in ficheiros.iter().zip(medidos) {
        println!("[largura] {rel}: {larguras:?} (pedidas {RAIOS_DA_PROMESSA:?} × 2)");
        let com_grao = p
            .parametros
            .brush
            .get("texture")
            .and_then(|t| t.get("kind"))
            .is_some_and(|k| k != "None");
        let tol = if com_grao {
            TOLERANCIA_REL_GRAO
        } else {
            TOLERANCIA_REL
        };
        for f in fora_da_promessa(&larguras, tol) {
            falhas.push(format!("{rel}: {f}"));
        }
        if let Some(crua) = crua {
            assert!(
                !fora_da_promessa(&crua, TOLERANCIA_REL).is_empty(),
                "controlo: a lei crua do Wet Round já cumpria a promessa — o portão não mede nada"
            );
        }
    }
    assert!(
        falhas.is_empty(),
        "a largura não é a do painel:\n{}",
        falhas.join("\n")
    );
}

/// O pincel que nasce SEM preset tem a lei da largura do Wet Round de fábrica (o `WetKnobs::DEFAULT` é
/// o Wet Round): a tabela embutida no motor é a do ficheiro.
#[test]
fn a_lei_de_omissao_e_a_do_wet_round() {
    let b = Biblioteca::carregar(&raiz(), None);
    let e = b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o Wet Round de fábrica");
    let mut t = PainterTool::default();
    assert!(t.wet_preset_aplicar(&e.preset).is_empty());
    let mut nasce = PainterTool::default();
    nasce.set_paint_media(PaintMedia::WetPaint);
    assert_eq!(
        nasce.wet_largura_tabela(),
        t.wet_largura_tabela(),
        "a LARGURA_DO_WET_ROUND do motor não é a do ficheiro: regere-a (o gerador imprime-a)"
    );
}

/// **Sonda de custo da miniatura** — o número que decide onde o preview vivo corre (no quadro ou numa
/// thread). Mínimo de 5 corridas por preset, a tela inteira e só o traço (sem a água depois).
#[test]
#[ignore = "sonda: mede o custo de pintar a miniatura"]
fn sonda_custo_da_miniatura() {
    println!(
        "[custo] load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        let min = (0..5)
            .map(|_| {
                let t0 = std::time::Instant::now();
                let _ = miniatura_do_preset(&p);
                t0.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min);
        println!("[custo] {rel:<60} {min:7.1} ms");
    }
}

/// **Sonda do RUÍDO da largura** (W2): o mesmo pincel ao mesmo raio prometido, cinco traços que só
/// diferem na posição vertical (a tela cresce 60 px de cada vez; o traço desce 30) — o quanto a largura de um traço varia
/// por si. É o chão de qualquer promessa: nenhuma tabela a acerta melhor do que isto.
#[test]
#[ignore = "sonda: o ruído da largura por traço (W2)"]
fn sonda_ruido_da_largura() {
    for (rel, b) in ficheiros_de_fabrica() {
        let p = direito(&ler(&b).unwrap());
        for &r in &RAIOS_DA_PROMESSA {
            let ls: Vec<f32> = std::thread::scope(|s| {
                let h: Vec<_> = (0..5)
                    .map(|k| {
                        let p = &p;
                        s.spawn(move || {
                            let (px, w, h) =
                                traco_recto_alto(r, r + 10.0 * k as f32, QUADROS_DA_LEI, |t| {
                                    t.set_wet_relogio_fixo(true);
                                    assert!(t.wet_preset_aplicar(p).is_empty());
                                });
                            largura_que_se_ve(&px, w, h)
                        })
                    })
                    .collect();
                h.into_iter().map(|h| h.join().unwrap()).collect()
            });
            println!(
                "[ruido] {rel:<55} raio {r:>5} pedido {:>5} → {ls:?}",
                2.0 * r
            );
        }
    }
}
