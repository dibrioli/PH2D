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

use super::sonda_gestos::{fnv1a, largura_mediana, traco_recto_com};
use ph2d_brush_presets::{
    Biblioteca, Origem, Parametros, Preset, Tilt, caminho_relativo, escrever, gravar, ler,
};
use ph2d_tool_painter::{
    MEIO_DO_PRESET, MOTOR_DOS_PRESETS, PaintMedia, PainterTool, WetKnobs, miniatura_do_preset,
};
use ph2d_wet_paint::tuning::Knob;
use std::path::PathBuf;

/// Quantos presets a fábrica tem — CONTADO (`the_factory_presets_are_counted`), nunca escolhido.
const PRESETS_DE_FABRICA: usize = 4;
const MEIO_DA_PASTA: &str = "Watercolor";
const GRUPO: &str = "Fluid";
/// Os raios da lei raio → largura (05 §1.1), os mesmos da sonda `sonda_largura`.
const RAIOS: [f32; 14] = [
    2.0, 4.0, 7.8, 12.2, 15.3, 19.1, 23.8, 29.8, 46.6, 72.8, 114.0, 178.0, 278.0, 434.0,
];

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes")
}

/// Uma linha do 02 §6.9. `pigment`/`gate`/`evaporation`/cerdas/`water` estão nas unidades do motor;
/// `pickup`/`edge` são RAZÕES ao Wet Round, que é o padrão do dono (05 §1.2). A gravidade é a MESMA em
/// todos (W1e, F5: o Rebelle não a tem por pincel — a velocidade é inclinação × água × cor); o `tilt` é o
/// do dial (W1e, F4: Round, Wash e Dry com a tela direita, como o Rebelle; o Drip, o único que promete
/// escorrer, inclinado no máximo, para baixo).
struct Receita {
    nome: &'static str,
    water: f64,
    pigment: f64,
    pickup: f64,
    gate: f64,
    evaporation: f64,
    edge: f64,
    tilt: Tilt,
    bristle_size: f64,
    bristle_count: f64,
}

/// A tela direita, com o dial guardado no meio e para baixo (para quando o artista a ligar).
const DIREITA: Tilt = Tilt {
    on: false,
    ring: 4,
    spoke: 3,
};

const RECEITAS: [Receita; PRESETS_DE_FABRICA] = [
    Receita {
        nome: "Wet Round",
        water: 1.0,
        pigment: 800.0,
        pickup: 1.0,
        gate: 0.4,
        evaporation: 1.0,
        edge: 1.0,
        tilt: DIREITA,
        bristle_size: 2.0,
        bristle_count: 2000.0,
    },
    Receita {
        nome: "Wet Wash",
        water: 1.0,
        pigment: 400.0,
        pickup: 0.6,
        gate: 0.3,
        evaporation: 0.7,
        edge: 0.75,
        tilt: DIREITA,
        bristle_size: 3.0,
        bristle_count: 3000.0,
    },
    Receita {
        nome: "Wet Dry Brush",
        water: 0.2,
        pigment: 1200.0,
        pickup: 1.6,
        gate: 0.6,
        evaporation: 1.5,
        edge: 1.125,
        tilt: DIREITA,
        bristle_size: 1.5,
        bristle_count: 1500.0,
    },
    Receita {
        nome: "Wet Drip",
        water: 1.0,
        pigment: 600.0,
        pickup: 0.8,
        gate: 0.2,
        evaporation: 0.6,
        edge: 1.0,
        tilt: Tilt {
            on: true,
            ring: 8,
            spoke: 3,
        },
        bristle_size: 2.0,
        bristle_count: 2000.0,
    },
];

/// O pincel de fábrica do meio inteiro, com os knobs da receita. Tamanho e opacidade NÃO são fixados
/// (a tabela não os tem): fica o do artista.
fn parametros(r: &Receita) -> Parametros {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let (mut p, _) = t.wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, GRUPO, r.nome));
    p.volumes.size_px = None;
    p.volumes.opacity = None;
    p.volumes.water = Some(r.water);
    p.tilt = Some(r.tilt);
    // As opções da W1g não são do pincel de fábrica: aplicá-lo não mexe nelas (a regra do preset antigo).
    (p.live_water, p.heavy_edge) = (None, None);
    let d = WetKnobs::DEFAULT;
    for (chave, v) in [
        ("pigmentPerDab", r.pigment),
        ("paperGate", r.gate),
        ("evaporation", r.evaporation),
        ("pickup", d.get(Knob::Pickup) * r.pickup),
        ("edgeDarkening", d.get(Knob::EdgeDarkening) * r.edge),
        ("bristleSize", r.bristle_size),
        ("bristleCount", r.bristle_count),
    ] {
        assert!(
            p.engine.insert(chave.to_owned(), v).is_some(),
            "knob {chave} existe"
        );
    }
    p
}

fn tabela_de_largura(p: &Preset) -> Vec<[f64; 2]> {
    RAIOS
        .iter()
        .map(|&r| {
            let (px, w, h) = traco_recto_com(r, 0, |t| {
                t.set_wet_relogio_fixo(true);
                let falhas = t.wet_preset_aplicar(p);
                assert!(falhas.is_empty());
            });
            // O raio pelo menor texto que volta ao mesmo f32 (7.8, não 7.800000190734863).
            let raio: f64 = format!("{r}").parse().unwrap();
            [raio, f64::from(largura_mediana(&px, w, h))]
        })
        .collect()
}

#[test]
#[ignore = "gerador: escreve assets/brushes/WetPainter (PH2D_REGERA_PRESETS=1)"]
fn gera_os_presets_de_fabrica() {
    let escreve = std::env::var("PH2D_REGERA_PRESETS").as_deref() == Ok("1");
    for r in &RECEITAS {
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
            let rel = caminho_relativo(MOTOR_DOS_PRESETS, MEIO_DA_PASTA, GRUPO, r.nome);
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
