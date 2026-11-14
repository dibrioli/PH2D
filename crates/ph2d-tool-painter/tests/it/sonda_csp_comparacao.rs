//! Sonda (08/11, pedido do dono): os MESMOS traços do Clip Studio (oráculo corrido por xdotool) nos nossos pincéis.
//! Os caminhos vêm de ficheiros de texto (uma linha = um traço, pontos `x,y` a cada ~3 px), gerados pelo mesmo
//! script que conduz o CSP. Pressão constante 1 (o rato XTest do CSP também não tem pressão). Corre à mão:
//! `PH2D_CSP_CENA=<pasta> cargo test --profile smoke -p ph2d-tool-painter --test it sonda_csp -- --ignored --nocapture`
//! Doc: `docs/Painter/COMPARACAO_PH2D_X_CLIP_STUDIO_2026-11-08.md`. Nada de produto muda.

use crate::exame_bancada::pinceis;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};
use std::path::{Path, PathBuf};

const W: usize = 420;
const H: usize = 240;
const BRUSH: u8 = 0;
const SMEAR: u8 = 1;
const BLUR: u8 = 2;

fn le(d: &Path, nome: &str) -> Vec<Vec<[f32; 2]>> {
    std::fs::read_to_string(d.join(format!("{nome}.txt")))
        .expect("cena")
        .lines()
        .map(|l| {
            l.split_whitespace()
                .map(|p| {
                    let (x, y) = p.split_once(',').expect("x,y");
                    [x.parse().expect("x"), y.parse().expect("y")]
                })
                .collect()
        })
        .collect()
}

fn traca(t: &mut PainterTool, tracos: &[Vec<[f32; 2]>]) {
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    for s in tracos {
        t.on_canvas_pointer(cp(s[0], PointerPhase::Down));
        for &p in &s[1..] {
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(*s.last().expect("ponto"), PointerPhase::Up));
        t.on_tick(8.0);
    }
}

fn novo(base: Vec<u8>, preset: Option<&str>, tam: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(base, W as u32, H as u32);
    if let Some(n) = preset {
        pinceis(n).remove(0).aplica(&mut t, true);
    }
    t.sonda_raio(tam);
    t
}

fn grava(d: &Path, nome: &str, p: &[u8]) {
    std::fs::write(d.join(format!("{nome}.rgba")), p).expect("grava");
}

fn pasta() -> (PathBuf, PathBuf) {
    let d = PathBuf::from(std::env::var("PH2D_CSP_CENA").expect("PH2D_CSP_CENA"));
    let o = d.join("out");
    std::fs::create_dir_all(&o).expect("out");
    (d, o)
}

/// Cena A: as linhas pretas, um pincel por imagem, no tamanho em px que o CSP recebeu.
#[test]
#[ignore = "sonda"]
fn sonda_csp_linhas() {
    let (d, o) = pasta();
    let a = le(&d, "a");
    for (nome, tam) in [
        ("G-Pen", 8.0),
        ("Turnip Pen", 8.0),
        ("Sketch Pencil", 8.0),
        ("Charcoal Stick", 16.0),
        ("Soft Airbrush", 30.0),
        ("Round Watercolor", 16.0),
        ("Watercolor Basic", 16.0),
    ] {
        let mut t = novo(vec![255u8; W * H * 4], Some(nome), tam);
        t.set_brush_color_srgb8([0, 0, 0]);
        traca(&mut t, &a);
        grava(
            &o,
            &format!("a_{}", nome.replace(' ', "_")),
            &t.run_full().0,
        );
    }
}

#[derive(Clone, Copy)]
enum Mix {
    Preset(&'static str),
    Pigment(&'static str),
    Composite,
    WatercolorPigment,
}

/// Cena B: blocos azul/amarelo/vermelho SECOS (G-Pen opaco, achatados em píxeis), depois os arrastos brancos.
#[test]
#[ignore = "sonda"]
fn sonda_csp_mistura() {
    let (d, o) = pasta();
    let mut t = novo(vec![255u8; W * H * 4], Some("G-Pen"), 16.0);
    t.set_brush_mistura_modo(ModoMisturaOpaca::Off);
    for (i, c) in [[0u8, 70, 190], [240, 200, 0], [200, 20, 20]]
        .into_iter()
        .enumerate()
    {
        t.set_brush_color_srgb8(c);
        traca(&mut t, &le(&d, &format!("blocos{i}")));
    }
    let base = t.run_full().0;
    grava(&o, "b_blocos", &base);
    let b = le(&d, "b");
    for (rot, v) in [
        ("pigment_hard_round", Mix::Pigment("Hard Round")),
        ("composite_dono", Mix::Composite),
        ("watercolor_basic_pigment", Mix::WatercolorPigment),
        ("dabber", Mix::Preset("Dabber")),
        ("smear_basic", Mix::Preset("Smear Basic")),
        ("blender_soft", Mix::Preset("Blender Soft")),
    ] {
        let preset = match v {
            Mix::Preset(p) | Mix::Pigment(p) => Some(p),
            Mix::WatercolorPigment => Some("Watercolor Basic"),
            Mix::Composite => None,
        };
        let mut t = novo(base.clone(), preset, 30.0);
        match v {
            Mix::Pigment(_) => t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend),
            Mix::WatercolorPigment => t.set_brush_pigment_mixing(1.0),
            Mix::Composite => {
                // A receita do dono (07/11): 1 Blur (1, Size 1, Hardness 0) · 2 Smear (0,25, Size 1, Hardness 0) ·
                // 3 Brush (0,10, Hardness 0,43), Flow 0,03.
                if !t.composite_enabled() {
                    t.toggle_composite();
                }
                for (i, (op, s, h)) in [(BLUR, 1.0, 0.0), (SMEAR, 0.25, 0.0), (BRUSH, 0.10, 0.43)]
                    .into_iter()
                    .enumerate()
                {
                    t.acrescenta_camada(op);
                    t.set_composite_layer_strength(i, s);
                    t.set_composite_layer_hardness(i, h);
                    if op != BRUSH {
                        t.set_composite_layer_size(i, 1.0);
                    }
                }
                assert_eq!(t.composite_len(), 3, "a pilha montada");
                t.set_brush_flow(0.03);
            }
            Mix::Preset(_) => {}
        }
        t.set_brush_color_srgb8([255, 255, 255]);
        traca(&mut t, &b);
        grava(&o, &format!("b_{rot}"), &t.run_full().0);
    }
}

// ---- v2 (08/11): tamanho em DIÂMETRO (o `set_brush_size_px` é RAIO; o Size do painel = 2 × raio, como o
// Brush Size do CSP) e mistura com cor de arrasto colorida sobre blocos idênticos (`cena_v2.py`).

const W2: usize = 420;
const H2: usize = 280;

/// O raio por pincel: `PH2D_CSP_R=<nome>=<raio>;…` (calibração pela largura medida); senão diâmetro / 2.
fn raio(nome: &str, diametro: f32) -> f32 {
    std::env::var("PH2D_CSP_R")
        .ok()
        .and_then(|v| {
            v.split(';').find_map(|kv| {
                kv.split_once('=')
                    .filter(|(k, _)| *k == nome)
                    .and_then(|(_, r)| r.parse().ok())
            })
        })
        .unwrap_or(diametro / 2.0)
}

fn novo2(base: Vec<u8>, w: usize, h: usize, preset: Option<&str>, r: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(base, w as u32, h as u32);
    if let Some(n) = preset {
        pinceis(n).remove(0).aplica(&mut t, true);
    }
    t.sonda_raio(r);
    t
}

/// Cena A (linhas) com o diâmetro do CSP: raio = d/2 × {0,75 · 1 · 1,25} para a calibração por largura medida.
#[test]
#[ignore = "sonda"]
fn sonda_csp_v2_linhas() {
    let (d, o) = pasta();
    let a = le(&d, "a");
    for (nome, diam) in [
        ("G-Pen", 8.0),
        ("Turnip Pen", 8.0),
        ("Sketch Pencil", 8.0),
        ("Charcoal Stick", 16.0),
        ("Soft Airbrush", 30.0),
        ("Round Watercolor", 16.0),
        ("Watercolor Basic", 16.0),
    ] {
        for f in [0.75f32, 1.0, 1.25] {
            let r = raio(nome, diam) * f;
            let mut t = novo2(vec![255u8; W * H * 4], W, H, Some(nome), r);
            t.set_brush_color_srgb8([0, 0, 0]);
            traca(&mut t, &a);
            grava(
                &o,
                &format!("v2a_{}_r{:.2}", nome.replace(' ', "_"), r),
                &t.run_full().0,
            );
        }
    }
}

/// Cena B v2: amarelo sobre o azul, azul sobre o amarelo e o vermelho, diâmetro 30 (raio 15 ou calibrado).
#[test]
#[ignore = "sonda"]
fn sonda_csp_v2_mistura() {
    let (d, o) = pasta();
    let base = std::fs::read(d.join("base_celula.rgba")).expect("base");
    assert_eq!(base.len(), W2 * H2 * 4);
    let (y, b) = (le(&d, "y"), le(&d, "b"));
    for (rot, v) in [
        ("pigment_hard_round", Mix::Pigment("Hard Round")),
        ("composite_dono", Mix::Composite),
        ("watercolor_basic_pigment", Mix::WatercolorPigment),
        ("dabber", Mix::Preset("Dabber")),
        ("smear_basic", Mix::Preset("Smear Basic")),
        ("blender_soft", Mix::Preset("Blender Soft")),
    ] {
        let preset = match v {
            Mix::Preset(p) | Mix::Pigment(p) => Some(p),
            Mix::WatercolorPigment => Some("Watercolor Basic"),
            Mix::Composite => None,
        };
        let mut t = novo2(base.clone(), W2, H2, preset, raio(rot, 30.0));
        match v {
            Mix::Pigment(_) => t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend),
            Mix::WatercolorPigment => t.set_brush_pigment_mixing(1.0),
            Mix::Composite => {
                if !t.composite_enabled() {
                    t.toggle_composite();
                }
                for (i, (op, s, h)) in [(BLUR, 1.0, 0.0), (SMEAR, 0.25, 0.0), (BRUSH, 0.10, 0.43)]
                    .into_iter()
                    .enumerate()
                {
                    t.acrescenta_camada(op);
                    t.set_composite_layer_strength(i, s);
                    t.set_composite_layer_hardness(i, h);
                    if op != BRUSH {
                        t.set_composite_layer_size(i, 1.0);
                    }
                }
                assert_eq!(t.composite_len(), 3, "a pilha montada");
                t.set_brush_flow(0.03);
            }
            Mix::Preset(_) => {}
        }
        t.set_brush_color_srgb8([240, 200, 0]);
        traca(&mut t, &y);
        t.set_brush_color_srgb8([0, 70, 190]);
        traca(&mut t, &b);
        grava(&o, &format!("v2b_{rot}"), &t.run_full().0);
    }
}

// ---- Wet blend (08/11): a foto do dono — um bloco azul seco e traços em ZIGUE-ZAGUE amarelo/verde que cruzam a
// fronteira dele, na camada transparente. Corre sem o CSP: `PH2D_CSP_CENA=<pasta>` só dá a pasta de saída.

/// Um zigue-zague vertical de `x0` a `x1`, de `y0` a `y1`, `n` dentes, pontos a cada ~3 px.
fn ziguezague(x0: f32, x1: f32, y0: f32, y1: f32, n: usize) -> Vec<[f32; 2]> {
    let mut v = Vec::new();
    for k in 0..n {
        let (ya, yb) = (
            y0 + (y1 - y0) * k as f32 / n as f32,
            y0 + (y1 - y0) * (k as f32 + 0.5) / n as f32,
        );
        let (a, b) = if k % 2 == 0 {
            ([x0, ya], [x1, yb])
        } else {
            ([x1, ya], [x0, yb])
        };
        let passos = (((b[0] - a[0]).hypot(b[1] - a[1])) / 3.0).ceil() as usize;
        for i in 0..passos {
            let t = i as f32 / passos as f32;
            v.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
        }
    }
    v
}

/// O bloco azul (Hard Round, Off, raio 40) à esquerda e os zigue-zagues que o cruzam; uma imagem por variante.
#[test]
#[ignore = "sonda"]
fn sonda_wet_blend_ziguezague() {
    let o = PathBuf::from(std::env::var("PH2D_CSP_CENA").expect("PH2D_CSP_CENA"));
    std::fs::create_dir_all(&o).expect("out");
    let mut base = novo2(vec![0u8; W2 * H2 * 4], W2, H2, Some("Hard Round"), 40.0);
    base.set_brush_mistura_modo(ModoMisturaOpaca::Off);
    base.set_brush_color_srgb8([30, 90, 200]);
    let bloco: Vec<Vec<[f32; 2]>> = (0..5)
        .map(|i| {
            vec![
                [110.0, 40.0 + 50.0 * i as f32],
                [150.0, 45.0 + 50.0 * i as f32],
            ]
        })
        .collect();
    traca(&mut base, &bloco);
    let base = base.run_full().0;
    grava(&o, "zz_base", &base);
    type Wet = Option<(f32, f32, f32, f32)>;
    let variantes: [(&str, Wet); 5] = [
        ("off", None),
        ("wet_a50_s50", Some((0.5, 1.0, 0.5, 0.0))),
        ("wet_a30_s80", Some((0.3, 1.0, 0.8, 0.0))),
        ("wet_a50_s50_pig", Some((0.5, 1.0, 0.5, 1.0))),
        ("wet_a70_d50", Some((0.7, 0.5, 0.4, 0.0))),
    ];
    for (rot, v) in variantes {
        let mut t = novo2(base.clone(), W2, H2, Some("Hard Round"), 22.0);
        match v {
            None => t.set_brush_mistura_modo(ModoMisturaOpaca::Off),
            Some((a, d, s, p)) => {
                t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
                for (i, x) in [a, d, s, 0.0, 0.0, p].into_iter().enumerate() {
                    t.set_brush_mistura_numero(i, x);
                }
            }
        }
        for (cor, x0, x1, y0, y1) in [
            ([240u8, 220, 0], 150.0, 260.0, 20.0, 120.0),
            ([120, 180, 20], 140.0, 250.0, 100.0, 200.0),
            ([200, 210, 30], 160.0, 280.0, 180.0, 270.0),
        ] {
            t.set_brush_color_srgb8(cor);
            traca(&mut t, &[ziguezague(x0, x1, y0, y1, 6)]);
        }
        grava(&o, &format!("zz_{rot}"), &t.run_full().0);
    }
}

/// Wet blend × CSP (08/11): a cena `cena_zz.py` (blocos secos + 5 zigue-zagues: amarelo, azul, vermelho, verde e
/// o vermelho que cruza tudo), Hard Round diâmetro 30, com os números de fábrica dos 4 pincéis de mistura do CSP
/// (Amount/Density/Stretch) e cada um com Pigment 0 e 1. `PH2D_CSP_CENA=<pasta da cena>`.
#[test]
#[ignore = "sonda"]
fn sonda_wet_blend_vs_csp() {
    let (d, o) = pasta();
    let base = std::fs::read(d.join("base_celula.rgba")).expect("base");
    assert_eq!(base.len(), W2 * H2 * 4);
    let tracos: Vec<(&str, [u8; 3])> = vec![
        ("zz_y", [240, 200, 0]),
        ("zz_b", [0, 70, 190]),
        ("zz_r", [200, 20, 20]),
        ("zz_g", [78, 160, 60]),
        ("zz_x", [200, 20, 20]),
    ];
    for (rot, (a, dd, s)) in [
        ("round_mixing", (0.80, 0.50, 1.00)),
        ("gouache", (0.70, 0.45, 0.15)),
        ("oil_paint", (0.61, 0.81, 0.87)),
        ("paint_and_apply", (0.80, 1.00, 0.50)),
    ] {
        for pig in [0.0f32, 1.0] {
            let mut t = novo2(base.clone(), W2, H2, Some("Hard Round"), 15.0);
            t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
            for (i, x) in [a, dd, s, 0.0, 0.0, pig].into_iter().enumerate() {
                t.set_brush_mistura_numero(i, x);
            }
            for (n, c) in &tracos {
                t.set_brush_color_srgb8(*c);
                traca(&mut t, &le(&d, n));
            }
            grava(&o, &format!("wb_{rot}_p{}", pig as u8), &t.run_full().0);
        }
    }
}

/// Custo do Wet blend por tamanho (à mão, `--profile smoke`): ms de um arrasto de 300 px sobre um bloco azul,
/// Hard Round e Sketch Pencil (com grão) Off × Wet blend, raios 8/30/100, intercalados no MESMO processo, o mínimo de
/// 5 blocos (`docs/DevOps/MEDIR_VELOCIDADE.md`).
#[test]
#[ignore = "medida: corre à mão"]
fn custo_do_wet_blend_por_tamanho() {
    let (w, h) = (640usize, 400usize);
    let mut base = vec![255u8; w * h * 4];
    for y in 0..h {
        for x in 0..w / 2 {
            let i = (y * w + x) * 4;
            base[i..i + 3].copy_from_slice(&[30, 90, 200]);
        }
    }
    let casos: Vec<(&str, f32, bool)> = ["Hard Round", "Sketch Pencil"]
        .into_iter()
        .flat_map(|p| {
            [8.0f32, 30.0, 100.0]
                .into_iter()
                .flat_map(move |r| [(p, r, false), (p, r, true)])
        })
        .collect();
    let mut min = vec![f64::MAX; casos.len()];
    for bloco in 0..5 {
        for k in 0..casos.len() {
            let v = (k + bloco) % casos.len();
            let (p, r, wet) = casos[v];
            let mut t = novo2(base.clone(), w, h, Some(p), r);
            t.set_brush_mistura_modo(if wet {
                ModoMisturaOpaca::WetBlend
            } else {
                ModoMisturaOpaca::Off
            });
            t.set_brush_color_srgb8([240, 200, 0]);
            let traco: Vec<[f32; 2]> = (0..=100).map(|i| [170.0 + 3.0 * i as f32, 200.0]).collect();
            let t0 = std::time::Instant::now();
            traca(&mut t, &[traco]);
            let _ = std::hint::black_box(t.run_full());
            min[v] = min[v].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    for ((p, r, wet), m) in casos.iter().zip(min) {
        eprintln!(
            "[custo] {p} r{r} {}: {m:.1} ms",
            if *wet { "Wet blend" } else { "Off" }
        );
    }
}
