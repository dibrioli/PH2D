//! **Medição W0(c) do Óleo Real** (doc `49_realshader/plano.md`): a escala de altura do Wet Paint nas pinceladas
//! equivalentes às sondas do Rebelle (`49_realshader/ferramentas/tracos.py`). Imprime, nunca barra:
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_altura -- --ignored --nocapture`.
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

const W: u32 = 1200;
const H: u32 = 800;
const AZ: [u8; 3] = [20, 150, 230];
const BR: [u8; 3] = [240, 240, 236];
const ES: [u8; 3] = [30, 25, 60];
#[path = "medida_oleo_real_folha.rs"]
mod medida_oleo_real_folha;
use medida_oleo_real_folha::*;
#[path = "medida_oleo_real_faca.rs"]
mod medida_oleo_real_faca;
use medida_oleo_real_faca::*;
#[path = "medida_oleo_real_acumulo.rs"]
mod medida_oleo_real_acumulo;
#[path = "medida_oleo_real_fenda.rs"]
mod medida_oleo_real_fenda;
use medida_oleo_real_fenda::*;

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn biblioteca() -> Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
}

/// O preset pelo `relativo` da fábrica — ou, se saiu dela (o Oil Bristle, 27/10), o seu candidato.
fn preset_rel(b: &Biblioteca, rel: &str) -> ph2d_brush_presets::Preset {
    b.entradas.iter().find(|e| e.relativo == rel).map_or_else(
        || {
            let nome = rel.rsplit('/').next().expect("nome");
            super::de_fabrica_ou_candidato(nome.trim_end_matches(".ph2d-brush.png"))
        },
        |e| e.preset.clone(),
    )
}

/// Os knobs do relevo de cada pincel de fábrica do Wet Paint, depois de aplicado.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_altura_knobs_de_fabrica() {
    let b = biblioteca();
    println!(
        "{:<44} {:>7} {:>7} {:>7} {:>7} {:>7} {:>8}",
        "preset", "Emboss", "EmbExp", "EmbRef", "BrHgt", "BrRem", "Pigment"
    );
    for e in b.entradas.iter().filter(|e| e.partes()[0] == "WetPainter") {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_media(PaintMedia::WetPaint);
        let _ = t.wet_preset_aplicar(&e.preset);
        let k = &t.paint.wetpaint.knobs;
        println!(
            "{:<44} {:>7.3} {:>7.3} {:>7.1} {:>7.3} {:>7.3} {:>8.1}",
            e.relativo,
            k.get(Knob::Emboss),
            k.get(Knob::EmbossSizeExp),
            k.get(Knob::EmbossSizeRef),
            k.get(Knob::BristleHeight),
            k.get(Knob::BristleRemove),
            k.get(Knob::PigmentPerDab),
        );
    }
}

struct Traco {
    rotulo: &'static str,
    preset: String,
    cor: [u8; 3],
    a: [f32; 2],
    b: [f32; 2],
    diametro: f32,
}

fn pct(v: &mut [f64], p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    let i = ((v.len() - 1) as f64 * p).round() as usize;
    v[i]
}

/// As pinceladas do `tracos.py` com os NOSSOS pincéis de fábrica, numa tela 1200×800, como no Rebelle.
/// Presets por env (`PH2D_OLEO_FACA`, `PH2D_OLEO_FACA_TEX`, `PH2D_OLEO_OILY`, `PH2D_OLEO_THICK`: o `relativo`).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_altura_tracos() {
    let mut t = pinta_folha();
    imprime_alturas(&mut t);
}

/// A folha do `tracos.py` sai em `$PH2D_OLEO_PNG/hoje.ppm` (o render do produto com o emboss do preset) e
/// `oleo.ppm` (o Óleo Real), sobre papel branco — para pôr ao lado de `capturas/A_base.jpg`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_folha_png() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    let mut t = pinta_folha();
    grava_folha(&mut t, &dir);
}

/// **As entradas do oráculo Cycles** (W1c, `49_realshader/oraculo/`): a cena da captura do dono (vermelho
/// saturado empilhado com Dabber e Oil Bristle, e uma pilha clara) em `$PH2D_OLEO_PNG/` — a folha de
/// sempre mais a cor SEM relevo (`albedo.ppm`), a cobertura (`cobertura.pgm`) e as três alturas em f32 LE,
/// células (`h_forma.f32`, `h_fina.f32`, `h_sombra.f32`), descritas em `oraculo.txt`.
/// `PH2D_OLEO_VARIANTES="nome:campo=v,…;nome2:…"` pinta a cena UMA vez e grava cada variante em
/// `$PH2D_OLEO_PNG/<nome>/` (o `PH2D_OLEO_AF` de cada uma vem depois do `:`).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_oraculo() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    // `PH2D_OLEO_CENA=lisos`: os traços LISOS e baixos da bateria (régua do halo, K4).
    // `=smooth2`: o recto do Knife Smooth 2 (candidato W2) com `knifeLevel` 0 · 0,3 · 1 (as paredes, K6).
    // `=travessia`: a mesma cena e, por cima, um traço vertical do Smooth 2 SEM nível (a marca que se apaga, K8).
    let mut t = match std::env::var("PH2D_OLEO_CENA").as_deref() {
        Ok("lisos") => pinta_cena_lisa(),
        Ok("smooth2") => pinta_cena_smooth2(),
        Ok("travessia") => pinta_cena_travessia(),
        Ok("dabber3") => pinta_cena_dabber3(),
        Ok("retangulo") => pinta_cena_retangulo(),
        _ => pinta_cena_do_dono(),
    };
    match std::env::var("PH2D_OLEO_VARIANTES") {
        Ok(vs) => {
            for v in vs.split(';').filter(|v| !v.is_empty()) {
                let (nome, af) = v.split_once(':').unwrap_or((v, ""));
                let d = dir.join(nome);
                std::fs::create_dir_all(&d).expect("pasta");
                grava_folha_com(&mut t, &d, oleo_de(af));
            }
        }
        Err(_) => grava_folha(&mut t, &dir),
    }
}

/// A cena da captura do dono (W1c): o vermelho saturado empilhado com Dabber e Oil Bristle e uma pilha clara.
pub(super) const VERMELHO_DO_DONO: [u8; 3] = [215, 25, 30];

/// Pinta a cena da captura do dono numa tela `OW×OH` (a de [`medida_oleo_real_oraculo`] e do gate da fenda).
type Pilha = (&'static str, [u8; 3], f32, [[f32; 2]; 2]);

pub(super) fn pinta_cena_do_dono() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    const VERMELHO: [u8; 3] = VERMELHO_DO_DONO;
    const CLARO: [u8; 3] = [238, 226, 196];
    let pilhas: [Pilha; 6] = [
        (
            "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
            VERMELHO,
            60.0,
            [[60.0, 120.0], [460.0, 150.0]],
        ),
        (
            "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
            VERMELHO,
            60.0,
            [[120.0, 60.0], [300.0, 260.0]],
        ),
        (
            "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
            VERMELHO,
            55.0,
            [[40.0, 330.0], [640.0, 290.0]],
        ),
        (
            "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
            VERMELHO,
            55.0,
            [[60.0, 400.0], [620.0, 380.0]],
        ),
        (
            "WetPainter/Oil/Dabber/Dabber.ph2d-brush.png",
            CLARO,
            55.0,
            [[500.0, 60.0], [640.0, 220.0]],
        ),
        (
            "WetPainter/Oil/Bristle/Oil Bristle.ph2d-brush.png",
            CLARO,
            50.0,
            [[520.0, 240.0], [660.0, 120.0]],
        ),
    ];
    for (preset, cor, raio, [a, z]) in pilhas {
        let p = &preset_rel(&b, preset);
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(raio);
        t.set_brush_color_srgb8(cor);
        // Quatro passagens, a 2.ª e a 4.ª de volta e deslocadas: as cristas da captura do dono.
        for k in 0..4 {
            let d = 10.0 * (k % 2) as f32;
            let (a, z) = ([a[0], a[1] + d], [z[0], z[1] + d]);
            let ticks = if k % 2 == 0 {
                arrasta(&mut t, a, z)
            } else {
                arrasta(&mut t, z, a)
            };
            println!("[oraculo] {preset} passagem {} (água {ticks})", k + 1);
        }
    }
    t
}

/// Os traços LISOS da bateria (K4, o halo): uma passagem de cada faca e do Filbert a 40 px, azul sobre o
/// papel, horizontais e um vertical — o que a foto do Rebelle mostra sem sombra no papel.
pub(super) fn pinta_cena_lisa() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    let tracos: [(&str, [[f32; 2]; 2]); 5] = [
        ("Knife/Knife Smooth 3", [[60.0, 70.0], [520.0, 70.0]]),
        ("Knife/Knife Oily", [[60.0, 160.0], [520.0, 160.0]]),
        ("Knife/Knife Dry", [[60.0, 250.0], [520.0, 250.0]]),
        ("Flat/Filbert", [[60.0, 340.0], [520.0, 340.0]]),
        ("Knife/Knife Smooth 3", [[620.0, 60.0], [620.0, 400.0]]),
    ];
    for (preset, [a, z]) in tracos {
        let rel = format!("WetPainter/Oil/{preset}.ph2d-brush.png");
        let p = &preset_rel(&b, &rel);
        let erros = t.wet_preset_aplicar(p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(20.0);
        t.set_brush_color_srgb8(AZ);
        let ticks = arrasta(&mut t, a, z);
        println!("[oraculo] {preset} (água {ticks})");
    }
    t
}

/// O `recto_lento` da bateria (40 px, passo 3, o azul da bateria) com o Knife Smooth 2 candidato
/// (`Pinceis/W2_Knife_candidatos/`), a `knifeLevel` 0 · 0,3 · 1 em `y` 90 · 200 · 310 — o perfil transversal das
/// paredes × o Rebelle (K6).
pub(super) fn pinta_cena_smooth2() -> PainterTool {
    let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Painter/Pinceis/W2_Knife_candidatos/Knife Smooth 2.ph2d-brush.png");
    let mut p = ph2d_brush_presets::ler(&std::fs::read(&f).expect("preset")).expect("preset");
    // K7 (a largura × o Rebelle): `PH2D_OLEO_S2_BRUSH="dab_flatten=0,…"` e `PH2D_OLEO_S2_ENGINE="knifeRidge=0,…"`
    // mudam o preset antes de pintar.
    for (var, motor) in [("PH2D_OLEO_S2_BRUSH", false), ("PH2D_OLEO_S2_ENGINE", true)] {
        for kv in std::env::var(var)
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
        {
            let (k, v) = kv.split_once('=').expect("chave=valor");
            let v: f64 = v.parse().expect("número");
            if motor {
                p.parametros.engine.insert(k.to_owned(), v);
            } else {
                p.parametros
                    .brush
                    .insert(k.to_owned(), serde_json::json!(v));
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    for (nivel, y) in [(0.0, 90.0), (0.3, 200.0), (1.0, 310.0)] {
        let erros = t.wet_preset_aplicar(&p);
        assert!(erros.is_empty(), "{erros:?}");
        t.paint.wetpaint.knobs.set(Knob::KnifeLevel, nivel);
        t.set_brush_size_px(
            std::env::var("PH2D_OLEO_S2_RAIO").map_or(20.0, |v| v.parse().expect("raio")),
        );
        t.set_brush_color_srgb8([23, 101, 179]);
        let ticks = arrasta(&mut t, [60.0, y], [560.0, y]);
        println!("[oraculo] Knife Smooth 2 knifeLevel {nivel} (água {ticks})");
    }
    t
}

/// K8: a [`pinta_cena_smooth2`] e um vertical do MESMO candidato a `knifeLevel` 0 (x 300, y 250..370) que atravessa
/// os planaltos de 0,3 e 1 — a tinta por nivelar que chega à borda deles não herda a parede viva.
fn pinta_cena_travessia() -> PainterTool {
    let mut t = pinta_cena_smooth2();
    t.paint.wetpaint.knobs.set(Knob::KnifeLevel, 0.0);
    t.set_brush_color_srgb8([200, 120, 40]);
    let ticks = arrasta(&mut t, [300.0, 150.0], [300.0, 370.0]);
    println!("[oraculo] travessia knifeLevel 0 (água {ticks})");
    t
}

/// K10: o candidato Dabber 3 (`Pinceis/W2_Dabber_candidatos/`) no recto da prancha a 40 e a 200 px (× a escala
/// 0,673 da bancada), o azul da bateria — os vãos de papel entre as lascas grossas.
fn pinta_cena_dabber3() -> PainterTool {
    let f = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/Painter/Pinceis/W2_Dabber_candidatos/Dabber 3.ph2d-brush.png");
    let p = ph2d_brush_presets::ler(&std::fs::read(&f).expect("preset")).expect("preset");
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    for (diametro, y) in [(40.0f32, 90.0), (200.0, 290.0)] {
        let erros = t.wet_preset_aplicar(&p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(diametro * 0.673 / 2.0);
        t.set_brush_color_srgb8([23, 101, 179]);
        let ticks = arrasta(&mut t, [60.0, y], [640.0, y]);
        println!("[oraculo] Dabber 3 a {diametro} px (água {ticks})");
    }
    t
}

/// §BUG-ressalto (smoke do dono 27/10): dois rectângulos FECHADOS de tinta grossa azul, o da esquerda com o Oil
/// Bristle de fábrica, o da direita com o Knife Oily — duas passagens por lado, a janela de papel no meio.
pub(super) fn pinta_cena_retangulo() -> PainterTool {
    let b = biblioteca();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (OW * OH * 4) as usize], OW, OH);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    for (preset, x0, x1) in [
        ("Bristle/Oil Bristle", 50.0f32, 320.0f32),
        ("Knife/Knife Oily", 390.0, 660.0),
    ] {
        let rel = format!("WetPainter/Oil/{preset}.ph2d-brush.png");
        let p = preset_rel(&b, &rel);
        let erros = t.wet_preset_aplicar(&p);
        assert!(erros.is_empty(), "{erros:?}");
        t.set_brush_size_px(30.0);
        t.set_brush_color_srgb8([30, 70, 210]);
        let (y0, y1) = (80.0f32, 380.0f32);
        let lados = [
            ([x0, y0], [x1, y0]),
            ([x1, y0], [x1, y1]),
            ([x1, y1], [x0, y1]),
            ([x0, y1], [x0, y0]),
        ];
        for k in 0..2 {
            for (a, z) in lados {
                let ticks = arrasta(&mut t, a, z);
                println!("[oraculo] {preset} passagem {} (água {ticks})", k + 1);
            }
        }
    }
    t
}

#[path = "medida_bordas_w3b.rs"]
mod bordas_w3b;
#[path = "medida_paper_press.rs"]
mod paper_press;
#[path = "medida_transparencia.rs"]
mod transparencia;
