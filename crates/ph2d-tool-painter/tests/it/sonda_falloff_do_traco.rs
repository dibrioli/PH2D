//! **Sonda: o falloff do traço contra o de UM dab** (dono 24/10: *«o falloff não é perfeitamente obedecido desde
//! que Glaze e Flow entraram»*; *«Glaze Round parece que acumula mesmo com o Accumulate desmarcado»*). No Digital,
//! sobre a tela transparente (o alfa É a cobertura): o corte de um recto horizontal no meio dele contra o corte de
//! um toque (um dab só — a curva do Falloff), por Flow; e o Glaze Round com o Accumulate ligado e desligado, numa
//! passada e na volta por cima dentro do MESMO traço.

use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use std::path::PathBuf;

const W: usize = 400;
const H: usize = 120;
const Y: f32 = 60.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tool(nome: &str) -> PainterTool {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let p = Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .unwrap()
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.preset_aplicar(&p).expect("aplica");
    t.set_brush_size_px(40.0);
    t
}

/// O Accumulate pela porta do painel (um clique alterna).
fn acumula(t: &mut PainterTool, on: bool) {
    if t.brush_settings().accumulate != on {
        t.handle_panel_event(PanelEvent::Click(ph2d_tool_painter::ids::PAINTER_BRUSH_ACCUMULATE));
    }
    assert_eq!(t.brush_settings().accumulate, on);
}

fn alfa(t: &mut PainterTool, x: usize, y: usize) -> f32 {
    f32::from(t.run_full().0[(y * W + x) * 4 + 3]) / 255.0
}

fn pontos(t: &mut PainterTool, pts: &[[f32; 2]]) {
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
}

fn corte(t: &mut PainterTool, x: usize) -> Vec<f32> {
    let px = t.run_full().0;
    (0..=26)
        .map(|d| f32::from(px[((Y as usize + d) * W + x) * 4 + 3]) / 255.0)
        .collect()
}

fn fmt(v: &[f32]) -> String {
    v.iter()
        .step_by(2)
        .map(|a| format!("{a:.2}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
#[ignore = "sonda: o falloff do traço × o de um dab, e o Accumulate do Glaze Round"]
fn sonda_falloff_do_traco() {
    let recto: Vec<[f32; 2]> = (0..=100).map(|k| [40.0 + 3.2 * k as f32, Y]).collect();
    for nome in ["Soft Round", "Glaze Round"] {
        for flow in [1.0, 0.3, 0.05] {
            let mut um = tool(nome);
            um.set_brush_strength(1.0);
            um.set_brush_flow(flow);
            pontos(&mut um, &[[200.0, Y], [200.0, Y]]);
            let dab = corte(&mut um, 200);
            let mut tr = tool(nome);
            tr.set_brush_strength(1.0);
            tr.set_brush_flow(flow);
            pontos(&mut tr, &recto);
            let tra = corte(&mut tr, 200);
            // A forma: cada corte normalizado ao seu centro.
            let n = |v: &[f32]| -> Vec<f32> { v.iter().map(|a| a / v[0].max(1e-6)).collect() };
            let (nd, nt) = (n(&dab), n(&tra));
            let pior = nd
                .iter()
                .zip(&nt)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f32, f32::max);
            println!(
                "[falloff] {nome:<11} flow {flow:<4} · centro dab {:.2} traço {:.2} · forma pior Δ {pior:.2}\n          dab   {}\n          traço {}",
                dab[0],
                tra[0],
                fmt(&nd),
                fmt(&nt)
            );
        }
    }
    // O Glaze Round de fábrica: Accumulate desligado (o de fábrica) e ligado — uma passada e a volta por cima.
    for acc in [false, true] {
        let mut t = tool("Glaze Round");
        acumula(&mut t, acc);
        let mut ida_e_volta = recto.clone();
        ida_e_volta.extend(recto.iter().rev().copied());
        pontos(&mut t, &recto);
        let uma = alfa(&mut t, 200, Y as usize);
        let mut t2 = tool("Glaze Round");
        acumula(&mut t2, acc);
        pontos(&mut t2, &ida_e_volta);
        let duas = alfa(&mut t2, 200, Y as usize);
        let mut t3 = tool("Glaze Round");
        acumula(&mut t3, acc);
        let mut quatro = ida_e_volta.clone();
        quatro.extend(ida_e_volta.iter().copied());
        pontos(&mut t3, &quatro);
        let q = alfa(&mut t3, 200, Y as usize);
        println!(
            "[accumulate] Glaze Round acc {acc:<5} · 1 passada {uma:.3} · ida e volta {duas:.3} · 4 passadas {q:.3} (Strength {:.2})",
            t.brush_settings().strength
        );
    }
}

/// O TOQUE PARADO (o print do dono 24/10: um clique do Soft Round com Smooth sai um disco chapado): o corte de um toque
/// segurado `n` tiques, com o airbrush de fábrica (um dab a cada 0,1 s parado) e sem ele, por Flow.
#[test]
#[ignore = "sonda: o toque parado — airbrush × falloff"]
fn sonda_toque_parado() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    for airbrush in [true, false] {
        for flow in [1.0, 0.5] {
            for tiques in [0usize, 6, 30, 60] {
                let mut p = Biblioteca::carregar(&raiz, None)
                    .entradas
                    .into_iter()
                    .find(|e| e.preset.parametros.name == "Soft Round")
                    .unwrap()
                    .preset;
                let b = &mut p.parametros.brush;
                b.insert("falloff".into(), serde_json::json!("Smooth"));
                if !airbrush {
                    b.insert("airbrush_rate_s".into(), serde_json::json!(0.0));
                }
                let mut t = PainterTool::default();
                t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
                t.set_paint_media(PaintMedia::Digital);
                t.set_brush_color_srgb8([0, 0, 0]);
                t.preset_aplicar(&p).expect("aplica");
                t.set_brush_size_px(40.0);
                t.set_brush_strength(1.0);
                t.set_brush_flow(flow);
                t.on_canvas_pointer(cp([200.0, Y], PointerPhase::Down));
                for _ in 0..tiques {
                    t.on_tick(1000.0 / 60.0);
                }
                t.on_canvas_pointer(cp([200.0, Y], PointerPhase::Up));
                t.on_tick(1000.0 / 60.0);
                let c = corte(&mut t, 200);
                println!(
                    "[parado] airbrush {airbrush:<5} flow {flow:<3} {tiques:>2} tiques · {}",
                    c.iter().step_by(2).map(|a| format!("{a:.2}")).collect::<Vec<_>>().join(" ")
                );
            }
        }
    }
}

/// O toque depois de TROCAS no mesmo tool (como no app): pincel anterior, Flow, falloff — contra o tool novo.
#[test]
#[ignore = "sonda: o toque depois de trocas"]
fn sonda_toque_depois_de_trocas() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let preset = |nome: &str, falloff: Option<&str>| {
        let mut p = Biblioteca::carregar(&raiz, None)
            .entradas
            .into_iter()
            .find(|e| e.preset.parametros.name == nome)
            .unwrap()
            .preset;
        if let Some(f) = falloff {
            p.parametros.brush.insert("falloff".into(), serde_json::json!(f));
        }
        p
    };
    let toque = |t: &mut PainterTool, x: f32| {
        t.on_canvas_pointer(cp([x, Y], PointerPhase::Down));
        for _ in 0..20 {
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(cp([x, Y], PointerPhase::Up));
        t.on_tick(1000.0 / 60.0);
    };
    let novo = || {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
        t.set_paint_media(PaintMedia::Digital);
        t.set_brush_color_srgb8([0, 0, 0]);
        t
    };
    let mostra = |rot: &str, c: &[f32]| {
        println!(
            "[trocas] {rot:<40} {}",
            c.iter().step_by(2).map(|a| format!("{a:.2}")).collect::<Vec<_>>().join(" ")
        );
    };
    let mut t = novo();
    t.preset_aplicar(&preset("Soft Round", Some("Smooth"))).unwrap();
    t.set_brush_size_px(40.0);
    toque(&mut t, 60.0);
    mostra("novo · Soft Smooth", &corte(&mut t, 60));
    let mut t = novo();
    t.preset_aplicar(&preset("Hard Round", None)).unwrap();
    t.set_brush_size_px(40.0);
    toque(&mut t, 50.0);
    t.preset_aplicar(&preset("Soft Round", Some("Smooth"))).unwrap();
    t.set_brush_size_px(40.0);
    toque(&mut t, 140.0);
    t.set_brush_flow(0.5);
    toque(&mut t, 240.0);
    t.set_brush_flow(1.0);
    toque(&mut t, 340.0);
    let px = t.run_full().0;
    for (rot, x) in [("Hard → Soft Smooth", 140usize), ("… Flow 0,5", 240), ("… Flow 1 de volta", 340)] {
        let c: Vec<f32> = (0..=26).map(|d| f32::from(px[((Y as usize + d) * W + x) * 4 + 3]) / 255.0).collect();
        mostra(rot, &c);
    }
}

/// O toque GRANDE (o print do dono: ~150 px): o corte radial normalizado pelo raio, por tamanho e Flow.
#[test]
#[ignore = "sonda: o toque por tamanho"]
fn sonda_toque_por_tamanho() {
    const L: usize = 800;
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    for nome in ["Soft Round", "Hard Round"] {
        for (tam, flow) in [(40.0f32, 1.0f32), (150.0, 1.0), (150.0, 0.5), (300.0, 1.0)] {
            let mut p = Biblioteca::carregar(&raiz, None)
                .entradas
                .into_iter()
                .find(|e| e.preset.parametros.name == nome)
                .unwrap()
                .preset;
            p.parametros.brush.insert("falloff".into(), serde_json::json!("Smooth"));
            p.parametros.brush.insert("hardness".into(), serde_json::json!(0.0));
            let mut t = PainterTool::default();
            t.set_source(vec![0u8; L * L * 4], L as u32, L as u32);
            t.set_paint_media(PaintMedia::Digital);
            t.set_brush_color_srgb8([0, 0, 0]);
            t.preset_aplicar(&p).expect("aplica");
            t.set_brush_size_px(tam);
            t.set_brush_strength(1.0);
            t.set_brush_flow(flow);
            let c = [400.0, 400.0];
            t.on_canvas_pointer(cp(c, PointerPhase::Down));
            for _ in 0..20 {
                t.on_tick(1000.0 / 60.0);
            }
            t.on_canvas_pointer(cp(c, PointerPhase::Up));
            t.on_tick(1000.0 / 60.0);
            let px = t.run_full().0;
            let a = |d: usize| f32::from(px[(400 * L + 400 + d) * 4 + 3]) / 255.0;
            let fim = (1..399).find(|&d| a(d) == 0.0).unwrap_or(399);
            let perfil: Vec<String> = (0..10).map(|k| format!("{:.2}", a(k * fim / 10))).collect();
            println!(
                "[tamanho] {nome:<10} {tam:>4} px flow {flow:<3} · raio pintado {fim:>3} · 0..1 do raio: {}",
                perfil.join(" ")
            );
        }
    }
}

/// O toque com a DUREZA DA MÃO (cartão Pen: Speed hardness, Pressure hardness): o perfil a 150 px.
#[test]
#[ignore = "sonda: a dureza da mão no toque"]
fn sonda_toque_com_dureza_da_mao() {
    const L: usize = 800;
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    for (rot, chave, v) in [
        ("sem", "", 0.0),
        ("Pressure hardness 0,9", "pressureHardness", 0.9),
        ("Speed hardness 0,3", "speedHardness", 0.3),
        ("Speed hardness 1", "speedHardness", 1.0),
    ] {
        let mut p = Biblioteca::carregar(&raiz, None)
            .entradas
            .into_iter()
            .find(|e| e.preset.parametros.name == "Soft Round")
            .unwrap()
            .preset;
        p.parametros.brush.insert("falloff".into(), serde_json::json!("Smooth"));
        if !chave.is_empty() {
            p.parametros.engine.insert(chave.into(), v);
        }
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; L * L * 4], L as u32, L as u32);
        t.set_paint_media(PaintMedia::Digital);
        t.set_brush_color_srgb8([0, 0, 0]);
        t.preset_aplicar(&p).expect("aplica");
        t.set_brush_size_px(150.0);
        t.set_brush_strength(1.0);
        let c = [400.0, 400.0];
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
        for _ in 0..20 {
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(cp(c, PointerPhase::Up));
        t.on_tick(1000.0 / 60.0);
        let px = t.run_full().0;
        let a = |d: usize| f32::from(px[(400 * L + 400 + d) * 4 + 3]) / 255.0;
        let perfil: Vec<String> = (0..10).map(|k| format!("{:.2}", a(k * 15))).collect();
        println!("[mao] {rot:<22} · 0..1 do raio: {}", perfil.join(" "));
    }
}
