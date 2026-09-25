//! **O preço do WET PAINT (`PaintMedia::WetPaint`) no PRODUTO** — e não no build de teste.
//!
//! Irmão do [`mede_a_aquarela`](mede_a_aquarela.rs), pelo mesmo motivo, e aqui o motivo pesa mais:
//! TODAS as sondas `measure_wetpaint_*` desta crate correm sob `cfg(test)`, onde o `capture_canvas`
//! do journal do undo está vivo — e o composite do Wet Paint passa `area: None`, logo ali cada
//! composite fotografa a TELA INTEIRA, coisa que o app em release nunca faz. *Uma sonda que mede o
//! build de teste mede outro programa.* Este exemplo compila a biblioteca SEM `cfg(test)` e entra
//! só pelas portas públicas que o app usa: `set_paint_media` + setters do pincel,
//! `on_canvas_pointer`, `Tool::on_tick` (o batimento por quadro) e `take_preview_arc` (a drenagem).
//!
//! O quadro do app é: os eventos de ponteiro que chegaram (a ~1 kHz, 16 por quadro) → `on_tick` →
//! `take_preview_arc`, e o resto do quadro é a ESPERA pelo vsync — que aqui é um `sleep` real,
//! porque a simulação corre numa thread própria (`wetpaint/offthread.rs`) a perseguir o relógio de
//! parede, e um laço que não espera nunca lhe daria tempo de correr.
//!
//! O split sai do [`ph2d_tool_painter::wet_diag`] — o MESMO instrumento que o log do produto
//! (`PH2D_FLUID_PROFILE`) imprime.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_o_wet_paint
//! bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_o_wet_paint -- [tela] [raio]
//! bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_o_wet_paint -- [tela] [raio] pousos [n]
//! ```
//!
//! Imprime `/proc/loadavg` ao lado: acima de `load ~5` nenhum relógio desta máquina vale nada em
//! absoluto — leia DIFERENÇAS dentro da mesma ronda.

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::{PaintMedia, PainterTool, wet_diag};
use std::time::{Duration, Instant};

/// Comprimento do traço (px de tela), passo por evento, eventos por quadro (1 kHz ÷ 60 Hz).
const COMPRIMENTO: f32 = 1500.0;
const PASSO: f32 = 4.0;
const EV_POR_QUADRO: u32 = 16;
const DT_MS: f32 = 1000.0 / 60.0;
/// Quadros depois do pen-up em que a água corre sozinha (2 s).
const QUADROS_DEPOIS: u32 = 120;
const CORRIDAS: usize = 3;

fn wet(size: u32, raio: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(
        vec![255u8; (size as usize) * (size as usize) * 4],
        size,
        size,
    );
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8([204, 51, 26]);
    assert_eq!(
        t.paint_media(),
        PaintMedia::WetPaint,
        "o meio não armou — a régua mediria outro pincel"
    );
    t
}

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn ms(t0: Instant) -> f64 {
    t0.elapsed().as_secs_f64() * 1e3
}

/// Dorme até ao fim do quadro que começou em `t0` (o vsync do app).
fn vsync(t0: Instant) {
    let quadro = Duration::from_secs_f32(DT_MS / 1000.0);
    if let Some(resta) = quadro.checked_sub(t0.elapsed()) {
        std::thread::sleep(resta);
    }
}

/// Um traço: `(pen_down, quadros do gesto (ev, tick, dren), pen_up, quadros depois (tick+dren),
/// passos da água depois, segundos depois)`.
struct Traco {
    /// O carimbo DURANTE o gesto: `(dabs, depósito ms, composite ms, entregas)` — `wet_diag`.
    carimbo: (u64, f64, f64, u64),
    pen_down: f64,
    gesto: Vec<(f64, f64, f64)>,
    pen_up: f64,
    depois: Vec<f64>,
    passos: u64,
    passo_ms: (f64, f64),
    comp_ms: (f64, f64, u64),
    espera_ms: (f64, f64),
    celulas: u64,
    segundos: f64,
}

fn traco(t: &mut PainterTool, y: f32, x0: f32, so_gesto: bool) -> Traco {
    let _ = wet_diag::take_window();
    let _ = wet_diag::take_cells();
    let _ = wet_diag::take_stamp();
    let x1 = x0 + COMPRIMENTO;
    let t0 = Instant::now();
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let pen_down = ms(t0);
    vsync(t0);

    let mut gesto = Vec::new();
    let mut x = x0;
    while x < x1 {
        let tq = Instant::now();
        for _ in 0..EV_POR_QUADRO {
            if x >= x1 {
                break;
            }
            x = (x + PASSO).min(x1);
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        let ev = ms(tq);
        let tt = Instant::now();
        t.on_tick(DT_MS);
        let tk = ms(tt);
        let td = Instant::now();
        let _ = t.take_preview_arc();
        gesto.push((ev, tk, ms(td)));
        vsync(tq);
    }
    let (_, _, _) = wet_diag::take_window();
    let carimbo = wet_diag::take_stamp();
    let tu = Instant::now();
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
    let pen_up = ms(tu);
    let _ = wet_diag::take_window();
    let _ = wet_diag::take_cells();

    let mut depois = Vec::new();
    let ts = Instant::now();
    let n_depois = if so_gesto { 1 } else { QUADROS_DEPOIS };
    for _ in 0..n_depois {
        let tq = Instant::now();
        t.on_tick(DT_MS);
        let _ = t.take_preview_arc();
        depois.push(ms(tq));
        vsync(tq);
    }
    let segundos = ts.elapsed().as_secs_f64();
    let (passo, comp, espera) = wet_diag::take_window();
    let celulas = wet_diag::take_cells();
    let media = |h: (f64, f64, u64)| if h.2 == 0 { 0.0 } else { h.0 / h.2 as f64 };
    Traco {
        carimbo,
        pen_down,
        gesto,
        pen_up,
        depois,
        passos: passo.2,
        passo_ms: (media(passo), passo.1),
        comp_ms: (media(comp), comp.1, comp.2),
        espera_ms: (media(espera), espera.1),
        celulas,
        segundos,
    }
}

fn p(v: &[f64], q: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    s[((s.len() - 1) as f64 * q) as usize]
}

fn linha(nome: &str, tr: &Traco) {
    let tot: Vec<f64> = tr.gesto.iter().map(|q| q.0 + q.1 + q.2).collect();
    let ev: Vec<f64> = tr.gesto.iter().map(|q| q.0).collect();
    let tk: Vec<f64> = tr.gesto.iter().map(|q| q.1).collect();
    let dr: Vec<f64> = tr.gesto.iter().map(|q| q.2).collect();
    let lentos = |v: &[f64]| v.iter().filter(|&&x| x > f64::from(DT_MS)).count();
    println!(
        "  {nome:<8} pen-down {:7.2} | GESTO ({:>3} q) p50 {:6.2} p90 {:6.2} max {:6.2} >16,7 {:>3} = ev {:6.2} + tick {:5.2} + dren {:5.2} | {} dabs em {} entregas: depósito {:6.2} ms/dab, composite {:5.2} ms/entrega | pen-up {:6.2}",
        tr.pen_down,
        tot.len(),
        p(&tot, 0.5),
        p(&tot, 0.9),
        p(&tot, 1.0),
        lentos(&tot),
        p(&ev, 0.5),
        p(&tk, 0.5),
        p(&dr, 0.5),
        tr.carimbo.0,
        tr.carimbo.3,
        tr.carimbo.1 / tr.carimbo.0.max(1) as f64,
        tr.carimbo.2 / tr.carimbo.3.max(1) as f64,
        tr.pen_up,
    );
    println!(
        "           DEPOIS ({:>3} q) p50 {:6.2} p90 {:6.2} max {:6.2} >16,7 {:>3} | água {:5.1} passos/s (passo {:6.2}/{:6.2} ms, {:.2} M células) | composite {:5.2}/{:5.2} x{} | espera {:5.2}/{:5.2}",
        tr.depois.len(),
        p(&tr.depois, 0.5),
        p(&tr.depois, 0.9),
        p(&tr.depois, 1.0),
        lentos(&tr.depois),
        tr.passos as f64 / tr.segundos,
        tr.passo_ms.0,
        tr.passo_ms.1,
        tr.celulas as f64 / 1e6,
        tr.comp_ms.0,
        tr.comp_ms.1,
        tr.comp_ms.2,
        tr.espera_ms.0,
        tr.espera_ms.1,
    );
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// **O POUSAR** — traços curtos em sequência, com a água a correr entre eles (meio segundo de
/// quadros), como quem pinta pinceladas soltas. Imprime o pen-down de cada um; é também o modo que
/// o amostrador de pilhas usa para ver o pousar sem o gesto por cima.
fn pousos(size: u32, raio: f32, n: usize) {
    let mut t = wet(size, raio);
    let c = size as f32 / 2.0;
    let mut pd = Vec::new();
    for k in 0..n {
        let x = c - 600.0 + (k % 12) as f32 * 100.0;
        let y = c - 400.0 + (k / 12) as f32 * 160.0;
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
        pd.push(ms(t0));
        vsync(t0);
        for j in 1..=3 {
            let tq = Instant::now();
            t.on_canvas_pointer(cp([x + 20.0 * j as f32, y], PointerPhase::Move));
            t.on_tick(DT_MS);
            let _ = t.take_preview_arc();
            vsync(tq);
        }
        t.on_canvas_pointer(cp([x + 60.0, y], PointerPhase::Up));
        for _ in 0..30 {
            let tq = Instant::now();
            t.on_tick(DT_MS);
            let _ = t.take_preview_arc();
            vsync(tq);
        }
    }
    let primeiro = pd[0];
    let mut resto = pd[1..].to_vec();
    resto.sort_by(f64::total_cmp);
    println!(
        "POUSOS {n} (raio {raio}) — 1.º {primeiro:.2} ms | seguintes p50 {:.2} p90 {:.2} max {:.2} ms",
        resto[resto.len() / 2],
        resto[resto.len() * 9 / 10],
        resto[resto.len() - 1]
    );
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let size: u32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(4096);
    let raio: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100.0);
    if args.get(2).is_some_and(|s| s == "pousos") {
        let n = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(24);
        println!("load {}", carga());
        pousos(size, raio, n);
        return;
    }
    // `perfil`: só os gestos, e mais traços — para o amostrador de pilhas ver o que o pincel custa.
    let perfil = args.get(2).is_some_and(|s| s == "perfil");
    let corridas = if perfil { 6 } else { CORRIDAS };
    println!(
        "WET PAINT — tela {size}², raio {raio} px, traço {COMPRIMENTO} px a {PASSO} px/evento, {EV_POR_QUADRO} ev/quadro"
    );
    for k in 0..corridas {
        println!("corrida {k} — load {}", carga());
        let mut t = wet(size, raio);
        let c = size as f32 / 2.0;
        let x0 = c - COMPRIMENTO / 2.0;
        let a = traco(&mut t, c - 150.0, x0, perfil);
        linha("1.º", &a);
        // O 2.º traço corre ao lado da água que o 1.º deixou a correr — o caso de quem pinta a seguir.
        let b = traco(&mut t, c + 150.0, x0, perfil);
        linha("2.º", &b);
    }
}
