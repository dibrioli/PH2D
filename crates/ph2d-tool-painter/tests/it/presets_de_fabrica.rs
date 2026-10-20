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

use super::receitas_de_fabrica::{
    DIREITA, NASCE_DE, PRESETS_DE_FABRICA, RECEITAS, Receita, SPACING_DA_FABRICA, grao,
};
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
        // As formas (prompt 44) nascem do Wet Round — a ponta, as cerdas e a lei dele; a receita muda o resto.
        Err(_) if matches!(r.nome, "Wet Flat" | "Gouache Flat") => {
            let rel = caminho_relativo(MOTOR_DOS_PRESETS, "Watercolor", "Fluid", "Wet Round");
            let mut p = ler(&std::fs::read(raiz().join(rel)).unwrap())
                .expect("o Wet Round de fábrica")
                .parametros;
            (p.name, p.group) = (r.nome.to_owned(), r.grupo.to_owned());
            p.brush
                .insert("dab_quina".to_owned(), serde_json::json!(0.0));
            p.engine.insert("pressureSize".to_owned(), 0.9);
            p.engine.insert("pressureOpacity".to_owned(), 0.0);
            p
        }
        // Os do prompt 53 nascem do ficheiro do vizinho, como a sonda do doc 52 os mediu.
        Err(_) if NASCE_DE.iter().any(|(n, _)| *n == r.nome) => {
            let base = NASCE_DE.iter().find(|(n, _)| *n == r.nome).unwrap().1;
            let mut p = super::midias_de_fabrica::de_fabrica(base).parametros;
            (p.name, p.group) = (r.nome.to_owned(), r.grupo.to_owned());
            p
        }
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
    assert!(
        p.brush
            .insert("spacing".to_owned(), serde_json::json!(SPACING_DA_FABRICA))
            .is_some(),
        "campo spacing existe"
    );
    // Um ficheiro de antes do Continuous não o grava; o campo é do `CAMPOS_DO_PINCEL`.
    p.brush
        .insert("continuous_spacing".to_owned(), serde_json::json!(true));
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
    // Um campo que o ficheiro ainda não grava (a ponta reta do Square Hard) entra também — desde que seja do pincel.
    for (campo, json) in r.pincel {
        let v: serde_json::Value = serde_json::from_str(json).expect("o JSON do campo");
        assert!(
            ph2d_tool_painter::CAMPOS_DO_PINCEL.contains(campo),
            "campo {campo} existe"
        );
        p.brush.insert((*campo).to_owned(), v);
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
        // Um knob que o ficheiro ainda não grava (a pressão do pincel, prompt 44) entra também — desde que
        // seja do REGISTO.
        assert!(
            ph2d_wet_paint::tuning::KNOB_DEFS
                .iter()
                .any(|d| d.key == chave),
            "knob {chave} existe"
        );
        p.engine.insert(chave.to_owned(), v);
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
pub(super) fn largura_de(p: &Preset, raio: f32, crua: bool) -> f32 {
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

/// A lei CRUA medida: raio do motor → largura, numa fila do MAIOR para o menor com `PH2D_LEI_FIOS` trabalhadores
/// (por omissão um por raio). ⚠️ A memória é a do raio: o Dabber (Jitter 0,5) a 64 de uma vez passava dos 48 GB; depois da
/// margem da janela pelo CAMINHO (`Trail::on_segment_no_caminho`) 21,9 GiB do tecto de 24 (prompt 53); com UMA lane por
/// ponta (L13) o raio 1 100 pesa 1,7 GiB (era 4,3) — mas a fatia é da LINHA e o `/tmp` (tmpfs) dos agentes conta nela
/// (14,6 G de `shmem` a 20/10): o Dabber regerou-se a `PH2D_LEI_FIOS=4`, pico 5,3 GiB, 4,8 min (03_MOTORES §4 L13).
fn tabela_de_largura(p: &Preset) -> Vec<[f64; 2]> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let raios = raios_da_lei();
    let fios = std::env::var("PH2D_LEI_FIOS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(raios.len());
    let proximo = AtomicUsize::new(0);
    let mut larguras = vec![0.0_f32; raios.len()];
    std::thread::scope(|s| {
        let h: Vec<_> = (0..fios)
            .map(|_| {
                s.spawn(|| {
                    let mut feitas = Vec::new();
                    loop {
                        let k = proximo.fetch_add(1, Ordering::Relaxed);
                        let Some(i) = raios.len().checked_sub(k + 1) else {
                            break feitas;
                        };
                        feitas.push((i, largura_de(p, raios[i], true)));
                    }
                })
            })
            .collect();
        for (i, l) in h.into_iter().flat_map(|h| h.join().unwrap()) {
            larguras[i] = l;
        }
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
        // `PH2D_REGERA_SO_MINIATURA=1`: a tabela do ficheiro fica (só a miniatura, quando a lei da largura não mudou).
        if std::env::var("PH2D_REGERA_SO_MINIATURA").as_deref() != Ok("1") {
            preset.parametros.width_table = tabela_de_largura(&preset);
        }
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
            "[fabrica] {:<14} miniatura fnv {:#018x} · pico {:.0} MiB · largura {}",
            r.nome,
            fnv1a(&preset.miniatura.rgba),
            pico_mib(),
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

/// ⭐ **A REGRA do Spacing** (dono 18/10, prompt 51): cada pincel de fábrica — e o que nasce SEM preset — leva o
/// Spacing [`SPACING_DA_FABRICA`] com o Continuous ligado.
#[test]
fn a_regra_do_spacing_da_fabrica() {
    let cumpre = |b: &std::collections::BTreeMap<String, serde_json::Value>| {
        b.get("spacing")
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|s| (s - SPACING_DA_FABRICA).abs() < 1e-6)
            && b.get("continuous_spacing") == Some(&serde_json::json!(true))
    };
    let fora: Vec<String> = Biblioteca::carregar(&raiz(), None)
        .entradas
        .into_iter()
        .filter(|e| !cumpre(&e.preset.parametros.brush))
        .map(|e| e.relativo)
        .collect();
    assert!(fora.is_empty(), "fora da regra do Spacing: {fora:?}");
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let sem_preset = t
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "g", "sem preset"))
        .0;
    assert!(
        cumpre(&sem_preset.brush),
        "o pincel sem preset: spacing {:?} · continuous {:?}",
        sem_preset.brush.get("spacing"),
        sem_preset.brush.get("continuous_spacing")
    );
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

/// O pico de memória do processo (`VmHWM`), em MiB.
fn pico_mib() -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .and_then(|r| r.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
        / 1024.0
}

/// **Sonda da MEMÓRIA da lei da largura** (J2 Dabber, L13): UM raio por processo (`VmHWM` é do processo), o traço
/// de [`largura_de`] com `PH2D_SONDA_PRESET` (Dabber), `PH2D_SONDA_RAIO`, `PH2D_SONDA_CRUA` (1), `PH2D_SONDA_JITTER`
/// (o do preset) e `PH2D_SONDA_QUADROS` (os da lei; `0` = só o gesto). Imprime o pico depois do gesto e depois da
/// água, as lanes e as janelas. O portão [`a_memoria_do_dabber_cabe`] corre-a num processo filho.
#[test]
#[ignore = "sonda: a memória da lei da largura, um raio por processo"]
fn sonda_memoria_da_lei() {
    use ph2d_editor_core::{Tool, tool::RasterEditTool};
    let var = |k: &str| std::env::var(k).ok();
    let nome = var("PH2D_SONDA_PRESET").unwrap_or_else(|| "Dabber".into());
    let raio: f32 = var("PH2D_SONDA_RAIO").map_or(300.0, |v| v.parse().unwrap());
    let crua = var("PH2D_SONDA_CRUA").as_deref() != Some("0");
    let quadros: u32 = var("PH2D_SONDA_QUADROS").map_or(QUADROS_DA_LEI, |v| v.parse().unwrap());
    let mut p = ficheiros_de_fabrica()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .find(|p| p.parametros.name == nome)
        .expect("o preset de fábrica");
    // `PH2D_SONDA_RECEITA=1`: o preset da RECEITA de hoje (antes de o regerar), com a tabela do ficheiro.
    if var("PH2D_SONDA_RECEITA").as_deref() == Some("1") {
        let r = RECEITAS.iter().find(|r| r.nome == nome).expect("a receita");
        let tabela = std::mem::take(&mut p.parametros.width_table);
        p.parametros = parametros(r);
        p.parametros.width_table = tabela;
    }
    if let Some(j) = var("PH2D_SONDA_JITTER") {
        p.parametros
            .brush
            .insert("jitter".into(), serde_json::json!(j.parse::<f64>().unwrap()));
    }
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    let h = ((raio * 6.0) as u32 + 80).max(120);
    let w = super::sonda_gestos::W;
    let cy = h as f32 * 0.5;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&direito(&p)).is_empty());
    if crua {
        t.set_wet_largura_crua(escala);
    }
    t.set_brush_size_px(raio / escala);
    t.set_brush_color_srgb8(super::sonda_gestos::AZUL);
    let t0 = std::time::Instant::now();
    super::sonda_gestos::gesto(&mut t, &[[100.0, cy], [900.0, cy]], 3.0, |_| 1.0);
    let gesto = pico_mib();
    let mut janelas = Vec::new();
    t.wet_sonda_motor(|e| {
        janelas = e
            .lane_trails
            .iter()
            .map(ph2d_wet_paint::trail::Trail::window_half_for_measure)
            .collect();
    });
    for _ in 0..quadros {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, pw, ph) = t.run_full();
    println!(
        "[memoria] {nome} raio {raio} crua {crua} · tela {w}×{h} · gesto {gesto:.0} MiB · água {:.0} MiB · lanes {} \
         · meias-larguras {janelas:?} · largura {:.1} · {:.1} s",
        pico_mib(),
        janelas.len(),
        largura_que_se_ve(&px, pw, ph),
        t0.elapsed().as_secs_f64()
    );
}

/// O pico do GESTO (MiB) e as lanes da [`sonda_memoria_da_lei`] num processo filho (o `VmHWM` é do processo).
fn memoria_num_filho(raio: f32) -> (f64, usize) {
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "presets_de_fabrica::sonda_memoria_da_lei",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env("PH2D_SONDA_RAIO", raio.to_string())
        .env("PH2D_SONDA_QUADROS", "0")
        .env_remove("PH2D_SONDA_JITTER")
        .env_remove("PH2D_SONDA_PRESET")
        .output()
        .expect("o processo filho da sonda");
    let txt = String::from_utf8_lossy(&out.stdout);
    let linha = txt
        .split("[memoria]")
        .nth(1)
        .unwrap_or_else(|| panic!("a sonda não respondeu a raio {raio}:\n{txt}"));
    let num = |k: &str| -> f64 {
        linha.split(k).nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap()
    };
    println!("[memoria]{}", linha.lines().next().unwrap_or(""));
    (num(" gesto "), num(" lanes ") as usize)
}

/// Os tectos do PICO do gesto do Dabber na lei crua da largura (a do gerador), em MiB, medidos 20/10 (L13, smoke,
/// um processo por raio; 03_MOTORES §4 L13):
///
/// | raio | limiar `r` (o mutante: 4 lanes) | uma ponta = uma lane | tecto |
/// |---|---|---|---|
/// | 300  | 298 | 165 | 230 |
/// | 1000 | 2 966 | 1 221 | 1 800 |
///
/// (O Dabber de antes — lâmina, Jitter 0,5 — com o limiar `r`: 467 a raio 300 e 3 955 a 1 100, 6–7 lanes.)
const TECTO_DA_MEMORIA_MIB: [(f32, f64); 2] = [(300.0, 230.0), (1000.0, 1800.0)];

/// ⭐ **A MEMÓRIA do Dabber cabe** (L13, EXAME §3.6): o Jitter espalha UMA ponta — o traço inteiro numa lane — e o
/// pico do gesto a raio 300 e 1000 fica abaixo do tecto medido. Mutação: o limiar da lane sem o Jitter
/// (`BrushSpec::limiar_da_lane` = `r`) parte o traço em lanes e passa os dois tectos.
#[test]
fn a_memoria_do_dabber_cabe() {
    for (raio, tecto) in TECTO_DA_MEMORIA_MIB {
        let (pico, lanes) = memoria_num_filho(raio);
        assert_eq!(lanes, 1, "raio {raio}: o Jitter partiu o traço em {lanes} lanes");
        assert!(pico <= tecto, "raio {raio}: pico do gesto {pico:.0} MiB > tecto {tecto:.0}");
    }
}
