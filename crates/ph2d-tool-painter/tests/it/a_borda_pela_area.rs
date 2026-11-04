//! ⭐ **A borda pela ÁREA do píxel** (smoke 04/11: o aro do Bokeh «pixelado», os pontos do Halftone «não suaves»): o
//! que é mais fino que um píxel não se lê num ponto. Os dois braços correm no MESMO processo (`mede_sem_a_area`).
//! `PH2D_RENDER_OUT=<pasta>` grava os PNG de antes e depois.

use ph2d_brush_presets::{Biblioteca, Preset, escrever_textura_rgba};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::texture::mede_sem_a_area;
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::path::PathBuf;
use std::sync::Mutex;

const W: usize = 360;
const H: usize = 200;
/// O interruptor é do processo: as corridas deste ficheiro não se cruzam.
static VEZ: Mutex<()> = Mutex::new(());
/// O aro de fábrica até 04/11: a crista de 0,05 do raio.
const ARO_FINO: &str = r#"[{"handle": "Vector", "id": 0, "x": 0.0, "y": 0.28}, {"handle": "Vector", "id": 1, "x": 0.92, "y": 0.31}, {"handle": "Vector", "id": 2, "x": 0.965, "y": 1.0}, {"handle": "Vector", "id": 3, "x": 0.99, "y": 0.6}, {"handle": "Vector", "id": 4, "x": 1.0, "y": 0.0}]"#;

fn preset(nome: &str) -> Preset {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap_or_else(|| panic!("{nome} de fábrica"))
        .preset
}

/// Uma onda de um lado ao outro da tela, a pressão 1; a tela no fim.
fn pinta(p: &Preset, raio: f32, sem_area: bool) -> Vec<u8> {
    let at = |k: f32| {
        let f = k / 120.0;
        let onda = 1.0 - 4.0 * (f * 3.0 - (f * 3.0 + 0.5).floor()).abs();
        [40.0 + 280.0 * f, 100.0 + 35.0 * onda]
    };
    let pts: Vec<[f32; 2]> = (0..=120).map(|k| at(k as f32)).collect();
    pinta_em(p, raio, sem_area, &pts, 1.0)
}

fn pinta_em(p: &Preset, raio: f32, sem_area: bool, pts: &[[f32; 2]], pressao: f32) -> Vec<u8> {
    mede_sem_a_area(sem_area);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([200, 30, 30]);
    t.preset_aplicar(p).expect("aplica");
    t.set_brush_size_px(raio);
    let ponto = |pos, phase| CanvasPointer {
        pos,
        pressure: pressao,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(ponto(pts[0], PointerPhase::Down));
    for &q in &pts[1..] {
        t.on_canvas_pointer(ponto(q, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(*pts.last().unwrap(), PointerPhase::Up));
    for _ in 0..5 {
        t.on_tick(1000.0 / 60.0);
    }
    let px = t.run_full().0;
    mede_sem_a_area(false);
    px
}

/// Os degraus entre vizinhos no canal que a tinta mais muda (o verde), em tinta inteira `0..=1`: só os da borda de
/// uma marca (`> 0,02`), ordenados.
fn degraus(px: &[u8]) -> Vec<f32> {
    let g = |x: usize, y: usize| f32::from(px[(y * W + x) * 4 + 1]) / (255.0 - 30.0);
    let mut d = Vec::new();
    for y in 0..H - 1 {
        for x in 0..W - 1 {
            for v in [(g(x, y) - g(x + 1, y)).abs(), (g(x, y) - g(x, y + 1)).abs()] {
                if v > 0.02 {
                    d.push(v);
                }
            }
        }
    }
    d.sort_by(f32::total_cmp);
    d
}

fn q(d: &[f32], f: f32) -> f32 {
    d.get(((d.len() as f32 - 1.0) * f) as usize)
        .copied()
        .unwrap_or(0.0)
}

fn grava(nome: &str, px: &[u8]) {
    if let Ok(dir) = std::env::var("PH2D_RENDER_OUT") {
        let b = escrever_textura_rgba(px, W as u32, H as u32);
        std::fs::write(PathBuf::from(dir).join(format!("{nome}.png")), b).unwrap();
    }
}

fn aro_fino() -> Preset {
    let mut p = preset("Bokeh");
    p.parametros.brush.insert(
        "custom_falloff".into(),
        serde_json::from_str(ARO_FINO).unwrap(),
    );
    p
}

/// ⭐ Os pontos do Halftone: a borda era um degrau num ponto (mediana 0,57 da tinta inteira entre vizinhos, os «pontos em
/// bloco» da foto), e pela área é uma rampa (0,29). As fronteiras: o meio entre os dois braços, medido 04/11.
#[test]
fn os_pontos_do_halftone_tem_a_borda_gradual() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    let p = preset("Halftone");
    let (depois, antes) = (
        degraus(&pinta(&p, 40.0, false)),
        degraus(&pinta(&p, 40.0, true)),
    );
    let (q_d, q_a) = (q(&depois, 0.5), q(&antes, 0.5));
    assert!(q_d < 0.42, "a borda do ponto salta {q_d:.3} entre vizinhos");
    assert!(
        q_a > 0.5,
        "o controlo sem a cura {q_a:.3}: a régua não separa"
    );
}

/// ⭐ O aro mais fino que um píxel (a crista de 0,05 do raio, a raio 12: 0,6 px) acendia e apagava: a cauda dos degraus
/// (q90) 0,59 num ponto, 0,37 pela área. Vale para qualquer curva que o artista desenhe, não só a de fábrica.
#[test]
fn o_aro_fino_do_bokeh_tem_a_borda_gradual() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    let p = aro_fino();
    let (depois, antes) = (
        degraus(&pinta(&p, 12.0, false)),
        degraus(&pinta(&p, 12.0, true)),
    );
    let (q_d, q_a) = (q(&depois, 0.9), q(&antes, 0.9));
    assert!(q_d < 0.48, "o aro salta {q_d:.3} entre vizinhos");
    assert!(
        q_a > 0.5,
        "o controlo sem a cura {q_a:.3}: a régua não separa"
    );
}

/// O Bokeh de fábrica, com o aro largo que o dono desenhou, já não tem degrau de aro.
#[test]
fn o_bokeh_de_fabrica_tem_o_aro_suave() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    let d = degraus(&pinta(&preset("Bokeh"), 25.0, false));
    let q90 = q(&d, 0.9);
    assert!(q90 < 0.3, "o aro de fábrica salta {q90:.3} entre vizinhos");
}

/// Um pincel sem curva Custom e sem Dots não vê a cura, e a curva suave do Burn a tamanho normal também não.
#[test]
fn quem_nao_tem_crista_nao_muda() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    for nome in ["Soft Round", "Stars", "Burn"] {
        let p = preset(nome);
        assert!(
            pinta(&p, 40.0, false) == pinta(&p, 40.0, true),
            "{nome} mudou"
        );
    }
}

#[test]
#[ignore = "sonda: imprime e grava os dois braços"]
fn sonda_a_borda_pela_area() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    for (nome, p, r) in [
        ("halftone_r40", preset("Halftone"), 40.0),
        ("halftone_r12", preset("Halftone"), 12.0),
        ("bokeh_aro_fino_r12", aro_fino(), 12.0),
        ("bokeh_aro_fino_r25", aro_fino(), 25.0),
        ("bokeh_aro_fino_r42", aro_fino(), 42.0),
        ("bokeh_fabrica_r12", preset("Bokeh"), 12.0),
        ("bokeh_fabrica_r25", preset("Bokeh"), 25.0),
        ("bokeh_fabrica_r42", preset("Bokeh"), 42.0),
        ("burn_r5", preset("Burn"), 5.0),
    ] {
        for (braco, sem) in [("antes", true), ("depois", false)] {
            let px = pinta(&p, r, sem);
            let d = degraus(&px);
            println!(
                "{nome} {braco}: n {} · q50 {:.3} q90 {:.3} q99 {:.3} máx {:.3}",
                d.len(),
                q(&d, 0.5),
                q(&d, 0.9),
                q(&d, 0.99),
                q(&d, 1.0),
            );
            grava(&format!("{nome}_{braco}"), &px);
        }
    }
}

/// O custo A×B (MEDIR_VELOCIDADE): os dois braços intercalados em blocos rodados no MESMO processo, o mínimo de cada.
#[test]
#[ignore = "sonda: o preço da cura"]
fn sonda_o_preco_da_area() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    for (nome, p, r) in [
        ("bokeh r25", preset("Bokeh"), 25.0),
        ("bokeh r150", preset("Bokeh"), 150.0),
        ("halftone r40", preset("Halftone"), 40.0),
        ("halftone r150", preset("Halftone"), 150.0),
    ] {
        let mut min = [f64::MAX; 2];
        for bloco in 0..16 {
            for k in 0..2 {
                let braco = (bloco + k) % 2;
                let t0 = std::time::Instant::now();
                std::hint::black_box(pinta(&p, r, braco == 1));
                min[braco] = min[braco].min(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        println!(
            "{nome}: pela área {:.2} ms · num ponto {:.2} ms · {:.3}×",
            min[0],
            min[1],
            min[0] / min[1]
        );
    }
    // A máscara assada (Custom + grão View): a reconstrução, 1024².
    let mut b = ph2d_painter_brush::BrushSpec {
        falloff: ph2d_painter_brush::Falloff::Custom,
        ..Default::default()
    };
    b.custom_falloff.add_point();
    let mut min = [f64::MAX; 2];
    for bloco in 0..16 {
        for k in 0..2 {
            let braco = (bloco + k) % 2;
            mede_sem_a_area(braco == 1);
            let t0 = std::time::Instant::now();
            std::hint::black_box(ph2d_painter_brush::render_stamp_mask(
                &b, None, None, None, 1024,
            ));
            min[braco] = min[braco].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    mede_sem_a_area(false);
    println!(
        "máscara 1024²: pela área {:.2} ms · num ponto {:.2} ms",
        min[0], min[1]
    );
}

/// A lista do que a cura muda em TODA a fábrica: a miniatura de cada preset nos dois braços, no mesmo processo.
#[test]
#[ignore = "sonda: lista os pincéis que a cura muda"]
fn sonda_quem_a_cura_muda() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let entradas = Biblioteca::carregar(&raiz, None).entradas;
    let mut mudaram = 0;
    for e in &entradas {
        mede_sem_a_area(true);
        let antes = ph2d_tool_painter::miniatura_do_preset(&e.preset);
        mede_sem_a_area(false);
        let depois = ph2d_tool_painter::miniatura_do_preset(&e.preset);
        let (mut n, mut maior) = (0, 0);
        for (a, b) in antes.rgba.chunks(4).zip(depois.rgba.chunks(4)) {
            let d = a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap();
            n += usize::from(d > 0);
            maior = maior.max(d);
        }
        if n > 0 {
            mudaram += 1;
            println!("{}: {n} px, até {maior} níveis", e.preset.parametros.name);
        }
    }
    println!("{mudaram} de {} presets mudam", entradas.len());
}

/// O Marker Liner em Mix: um braço recto que volta a cruzar-se. Imprime o verde ao longo do braço (emendas entre dabs) e
/// no cruzamento contra a passagem única, a duas pressões, e grava-o em Mix e no Multiply de antes.
#[test]
#[ignore = "sonda: o Liner em Mix sobre si mesmo"]
fn sonda_o_liner_em_mix() {
    let _vez = VEZ.lock().unwrap_or_else(|e| e.into_inner());
    let mut pts: Vec<[f32; 2]> = (0..=70).map(|k| [40.0 + 4.0 * k as f32, 50.0]).collect();
    pts.extend((1..=60).map(|k| {
        let f = k as f32 / 60.0;
        [320.0 - 200.0 * f, 50.0 + 120.0 * (4.0 * f * (1.0 - f))]
    }));
    pts.extend((1..=60).map(|k| [120.0, 50.0 - 0.5 * k as f32]));
    let mix = preset("Marker Liner");
    let mut mul = mix.clone();
    mul.parametros
        .brush
        .insert("blend".into(), "Multiply".into());
    for (nome, p) in [("mix", &mix), ("multiply", &mul)] {
        for pressao in [1.0, 0.5] {
            let px = pinta_em(p, 14.0, false, &pts, pressao);
            let g = |x: usize, y: usize| px[(y * W + x) * 4 + 1];
            let braco: Vec<u8> = (60..110).map(|x| g(x, 50)).collect();
            let (lo, hi) = (braco.iter().min().unwrap(), braco.iter().max().unwrap());
            println!(
                "liner {nome} p{pressao}: braço verde {lo}..{hi} · um só {} · cruzamento {}",
                g(80, 50),
                g(120, 50),
            );
            grava(&format!("liner_{nome}_p{pressao}"), &px);
        }
    }
}
