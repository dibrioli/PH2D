//! **Sonda: o cartão Pen linha a linha** (auditoria 24/10). O Glaze Round no Digital, um traço com a pressão, a
//! inclinação e a velocidade a variar; cada linha fora do neutro (pela porta do painel) contra a fábrica, o neutro
//! explícito ao byte, o esmaecido (`dica_de`) antes e depois; e o toque parado com a «Speed softness».

use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_tool_painter::{LINHAS_DA_MAO, MAO_DE_FABRICA, PaintMedia, PainterTool, ids};
use std::path::PathBuf;

const W: usize = 600;
const H: usize = 200;
const Y: f32 = 100.0;

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
    t.set_brush_size_px(30.0);
    t
}

/// (pos, pressão, inclinação): lento (2 px/tique ≈ 120 px/s, a pressão 0,1 → 1) e depois rápido (40 px/tique ≈ 2 400
/// px/s, pressão 0,5); a caneta deita de [0,2;0] a [0,7;0,3].
fn gesto(caneta: bool) -> Vec<([f32; 2], f32, [f32; 2])> {
    let mut v = Vec::new();
    for k in 0..=100 {
        let u = k as f32 / 100.0;
        v.push(([40.0 + 2.0 * k as f32, Y], 0.1 + 0.9 * u, [0.2 + 0.3 * u, 0.15 * u]));
    }
    for k in 1..=8 {
        let u = k as f32 / 8.0;
        v.push(([240.0 + 40.0 * k as f32, Y], 0.5, [0.5 + 0.2 * u, 0.15 + 0.15 * u]));
    }
    if !caneta {
        for s in &mut v {
            s.1 = 1.0;
            s.2 = [0.0, 0.0];
        }
    }
    v
}

fn pinta(t: &mut PainterTool, g: &[([f32; 2], f32, [f32; 2])]) -> Vec<u8> {
    let cp = |s: &([f32; 2], f32, [f32; 2]), phase| CanvasPointer {
        pos: s.0,
        pressure: s.1,
        tilt: s.2,
        phase,
    };
    t.on_canvas_pointer(cp(&g[0], PointerPhase::Down));
    for s in &g[1..] {
        t.on_canvas_pointer(cp(s, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(g.last().unwrap(), PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
    t.run_full().0
}

fn a(px: &[u8], x: usize, y: usize) -> f32 {
    f32::from(px[(y * W + x) * 4 + 3]) / 255.0
}

/// (pico, largura a meio pico, orla 10–90 % de cima) na coluna `x`.
fn coluna(px: &[u8], x: usize) -> (f32, usize, usize) {
    let c: Vec<f32> = (0..H).map(|y| a(px, x, y)).collect();
    let pico = c.iter().copied().fold(0.0, f32::max);
    if pico <= 0.0 {
        return (0.0, 0, 0);
    }
    let larg = c.iter().filter(|v| **v >= 0.5 * pico).count();
    let y10 = c.iter().position(|v| *v >= 0.1 * pico).unwrap_or(0);
    let y90 = c.iter().position(|v| *v >= 0.9 * pico).unwrap_or(0);
    (pico, larg, y90.saturating_sub(y10))
}

fn resumo(px: &[u8]) -> String {
    let soma: f64 = (0..W * H).map(|i| f64::from(px[i * 4 + 3])).sum::<f64>() / 255.0;
    let area = (0..W * H).filter(|i| px[i * 4 + 3] > 2).count();
    let cols: Vec<String> = [60usize, 120, 200, 260, 420]
        .iter()
        .map(|x| {
            let (p, l, o) = coluna(px, *x);
            format!("x{x}:{p:.2}/{l}/{o}")
        })
        .collect();
    format!("soma {soma:8.0} área {area:6} · {}", cols.join(" "))
}

fn diff(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

fn set(t: &mut PainterTool, i: usize, v: f64) {
    if ids::PAINTER_PEN_FIELDS[i] == ids::PAINTER_PEN_ROTATION {
        t.handle_panel_event(PanelEvent::SelectOption(
            ids::PAINTER_PEN_ROTATION,
            format!("{}", v as u8),
        ));
    } else {
        t.handle_panel_event(PanelEvent::SetValue(ids::PAINTER_PEN_FIELDS[i], v));
    }
}

fn dica(t: &PainterTool, i: usize) -> String {
    t.brush_settings()
        .inercias
        .dica_de(ids::PAINTER_PEN_FIELDS[i])
        .map_or("vivo".into(), |d| format!("ESMAECIDO({d})"))
}

#[test]
#[ignore = "sonda: o cartão Pen linha a linha"]
fn sonda_cartao_pen() {
    let base = tool("Glaze Round");
    let b = base.brush_settings();
    println!(
        "[fabrica] hardness {:.3} method {} mao {:?}\n          MAO_DE_FABRICA {:?}",
        b.hardness, b.stroke_method, b.mao, MAO_DE_FABRICA
    );
    // (linha, valor fora do neutro, linhas de apoio que a ligam)
    let casos: &[(usize, f64, &[(usize, f64)])] = &[
        (0, 0.0, &[]),
        (0, 0.5, &[]),
        (1, 2.5, &[]),
        (2, 0.8, &[]),
        (3, 2.5, &[]),
        (3, 2.5, &[(2, 0.8)]),
        (4, 0.4, &[]),
        (5, 0.5, &[]),
        (6, 0.8, &[]),
        (7, 60.0, &[]),
        (8, 1.0, &[]),
        (8, 2.0, &[]),
        (9, 10.0, &[]),
        (9, 10.0, &[(11, 3.0)]),
        (10, 40.0, &[(11, 3.0)]),
        (11, 3.0, &[]),
        (12, 3.0, &[]),
        (13, 1.0, &[]),
        (14, 100.0, &[]),
        (14, 200.0, &[(11, 3.0)]),
        (15, 0.8, &[]),
        (16, 0.5, &[]),
        (17, 0.5, &[]),
        (18, 1.0, &[]),
        (19, 0.5, &[]),
        (20, 0.2, &[]),
        (20, 0.2, &[(16, 0.5)]),
        (21, 0.04, &[]),
        (21, 0.3, &[]),
        (22, 0.5, &[]),
    ];
    for caneta in [true, false] {
        let g = gesto(caneta);
        let mut t0 = tool("Glaze Round");
        let ref_px = pinta(&mut t0, &g);
        println!(
            "\n=== {} === fábrica: {}",
            if caneta { "CANETA" } else { "RATO" },
            resumo(&ref_px)
        );
        for &(i, v, apoio) in casos {
            // o apoio sozinho (a régua da linha ligada)
            let mut ta = tool("Glaze Round");
            for &(j, w) in apoio {
                set(&mut ta, j, w);
            }
            let antes = dica(&ta, i);
            let apoio_px = pinta(&mut ta, &g);
            // o apoio + a linha no neutro explícito
            let mut tn = tool("Glaze Round");
            for &(j, w) in apoio {
                set(&mut tn, j, w);
            }
            set(&mut tn, i, LINHAS_DA_MAO[i].omissao());
            let neutro_px = pinta(&mut tn, &g);
            // o apoio + a linha fora do neutro
            let mut tv = tool("Glaze Round");
            for &(j, w) in apoio {
                set(&mut tv, j, w);
            }
            set(&mut tv, i, v);
            let lido = tv.brush_settings().mao[i];
            let px = pinta(&mut tv, &g);
            println!(
                "[{i:2} {:<15}] = {v:<6} (lido {lido}) apoio {apoio:?} · {antes} · neutroΔ {} · Δpx {:6} · {}",
                format!("{:?}", LINHAS_DA_MAO[i]),
                diff(&neutro_px, &apoio_px),
                diff(&px, &apoio_px),
                resumo(&px)
            );
        }
    }
    // O Reset: todas as linhas fora do neutro, depois o clique.
    let mut t = tool("Glaze Round");
    let pre = t.brush_settings().mao;
    for &(i, v, _) in casos {
        set(&mut t, i, v);
    }
    t.handle_panel_event(PanelEvent::Click(ids::PAINTER_PEN_RESET));
    let pos = t.brush_settings().mao;
    let fora: Vec<usize> = (0..pos.len()).filter(|k| pos[*k] != MAO_DE_FABRICA[*k]).collect();
    let fora_preset: Vec<usize> = (0..pos.len()).filter(|k| pos[*k] != pre[*k]).collect();
    println!("[reset] fora da MAO_DE_FABRICA {fora:?} · fora da do preset {fora_preset:?}");
}

/// O TOQUE (um clique) e o arranque do traço rápido com a «Speed softness» e a «Pen hardness».
#[test]
#[ignore = "sonda: a dureza pela velocidade num clique"]
fn sonda_speed_softness_clique() {
    for nome in ["Glaze Round", "Soft Round", "Hard Round"] {
        for (linha, ks) in [(21usize, [0.0, 0.02, 0.04, 0.1, 0.3, 1.0]), (22, [0.0, 0.1, 0.3, 0.5, 1.0, 1.0])] {
            for k in ks {
                // o clique
                let mut t = tool(nome);
                set(&mut t, linha, k);
                let s = ([300.0, Y], 1.0, [0.0, 0.0]);
                let px = pinta(&mut t, &[s, s]);
                let (p, l, o) = coluna(&px, 300);
                // o traço rápido (40 px/tique), o arranque × o meio
                let mut tr = tool(nome);
                set(&mut tr, linha, k);
                let g: Vec<_> = (0..=12).map(|i| ([60.0 + 40.0 * i as f32, Y], 1.0, [0.0, 0.0])).collect();
                let pr = pinta(&mut tr, &g);
                let (p0, l0, o0) = coluna(&pr, 62);
                let (pm, lm, om) = coluna(&pr, 300);
                // o traço lento (2 px/tique)
                let mut tl = tool(nome);
                set(&mut tl, linha, k);
                let g: Vec<_> = (0..=120).map(|i| ([60.0 + 2.0 * i as f32, Y], 1.0, [0.0, 0.0])).collect();
                let pl = pinta(&mut tl, &g);
                let (ps, ls, os) = coluna(&pl, 180);
                println!(
                    "[{nome:<11} linha {linha} k {k:<4}] clique pico {p:.2} larg {l} orla {o} · rápido início {p0:.2}/{l0}/{o0} meio {pm:.2}/{lm}/{om} · lento {ps:.2}/{ls}/{os}"
                );
            }
        }
    }
}

/// O esmaecido «sem pressão» (Min/Max pressure, Smoothing) quando a pressão ainda chega à dureza ou ao espalhar.
#[test]
#[ignore = "sonda: a mão sem pressão no tamanho nem na opacidade"]
fn sonda_mao_sem_pressao() {
    let g = gesto(true);
    for leitor in [(22usize, 0.8), (19, 0.5)] {
        for (i, v) in [(4usize, 0.5), (5, 0.4), (6, 0.85), (18, 1.0)] {
            let mk = || {
                let mut t = tool("Soft Round");
                set(&mut t, 0, 0.0);
                set(&mut t, 2, 0.0);
                set(&mut t, leitor.0, leitor.1);
                t
            };
            let mut ta = mk();
            let d = dica(&ta, i);
            let pa = pinta(&mut ta, &g);
            let mut tv = mk();
            set(&mut tv, i, v);
            let pv = pinta(&mut tv, &g);
            println!("[sem pressão] leitor {leitor:?} · linha {i} = {v} · {d} · Δpx {}", diff(&pa, &pv));
        }
    }
}
