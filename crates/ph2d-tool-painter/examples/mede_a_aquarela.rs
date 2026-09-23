//! **O preço da AQUARELA (`PaintMedia::Watercolor`) no PRODUTO** — e não no build de teste.
//!
//! Irmão do [`mede_a_pilha`](mede_a_pilha.rs), pelo mesmo motivo: as sondas `measure_watercolor_*` /
//! `diag_*` desta crate correm sob `cfg(test)` e ali o traço paga o journal do undo do canvas
//! (`capture_canvas` é `cfg(any(test, debug_assertions))`) — *uma sonda que mede o build de teste mede
//! outro programa*. Este exemplo compila a biblioteca SEM `cfg(test)` e entra só pelas portas públicas
//! que o app usa: `set_paint_media` + os setters do painel, `on_canvas_pointer` (o ponteiro),
//! `Tool::on_tick` (o batimento por quadro da `fase_fixed_step_clocks`) e `take_preview_arc` (a
//! drenagem da ponte do app).
//!
//! ⚠️ **Watercolor não é Wet Paint** (doc 31 §1): aqui não há simulação de fluido.
//!
//! O quadro do app é: os eventos de ponteiro que chegaram (a ~1 kHz, 16 por quadro) → `on_tick` →
//! `take_preview_arc`. A lavagem compõe UMA vez por quadro, dentro do `on_tick` (doc 32 §2.1).
//!
//! Os knobs são os do dono (doc 32 §1): Charge 0,755 · Dilution 0,168 · Pull 0,477 · Rewet 0,400 ·
//! Smudge 0,197 · Pigment 0,195 · Drying 10 s · Preview 0,300, pincel (raio) 250 — e uma variante com
//! Rewet 0.
//!
//! O split por fase sai do [`ph2d_tool_painter::wash_diag`] — o MESMO instrumento que o log do produto
//! (`PH2D_PAINT_PERF`) imprime; ele mede sempre (o env só decide quem imprime), e este exemplo é o seu
//! único leitor (o `take()` zera).
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_a_aquarela
//! bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_a_aquarela -- ablacao [2048|4096]
//! bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_a_aquarela -- perfil [2048|4096] [rewet0]
//! ```
//!
//! Imprime `/proc/loadavg` ao lado: acima de `load ~5` nenhum relógio desta máquina vale nada em
//! absoluto — leia DIFERENÇAS dentro da mesma ronda.

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::{PaintMedia, PainterTool, wash_diag};
use std::time::Instant;

/// Comprimento do traço (px de tela), eventos por quadro (1 kHz ÷ 60 Hz), quadros de secagem depois.
const COMPRIMENTO: f32 = 720.0;
const EV_POR_QUADRO: u32 = 16;
const DT_MS: f32 = 1000.0 / 60.0;
const QUADROS_DEPOIS: u32 = 30;
const CORRIDAS: usize = 5;

/// A aquarela do dono, montada pelas portas do painel.
fn aquarela(size: u32, rewet: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(
        vec![255u8; (size as usize) * (size as usize) * 4],
        size,
        size,
    );
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_size_px(250.0);
    t.set_brush_color_srgb8([204, 51, 26]);
    t.set_brush_wet_charge(0.755);
    t.set_brush_wet_dilution(0.168);
    t.set_brush_wet_pull(0.477);
    t.set_brush_wet_rewet(rewet);
    t.set_brush_wet_smudge(0.197);
    t.set_brush_pigment_mixing(0.195);
    t.set_dry_time_s(10.0);
    t.set_wet_preview_intensity(0.300);
    assert_eq!(
        t.paint_media(),
        PaintMedia::Watercolor,
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

/// Uma corrida: o traço inteiro, com cada fase cronometrada e o `wash_diag` lido por fase.
struct Corrida {
    pen_down: f64,
    /// ms de cada quadro do gesto: (eventos, tick, drenagem).
    quadros: Vec<(f64, f64, f64)>,
    /// ms do `on_canvas_pointer(Up)` sozinho (o bake + pour + undo).
    pen_up: f64,
    /// ms do quadro que segue o pen-up (tick + drenagem).
    quadro_do_up: f64,
    /// ms dos quadros de secagem depois do traço (tick + drenagem).
    secagem: Vec<f64>,
    d_down: wash_diag::WashRead,
    d_gesto: wash_diag::WashRead,
    d_up: wash_diag::WashRead,
    d_depois: wash_diag::WashRead,
}

fn traco(t: &mut PainterTool, size: u32, passo: f32) -> Corrida {
    let _ = wash_diag::take(); // zera o que o setup possa ter deixado
    let y = (size / 2) as f32;
    let x0 = (size / 2) as f32 - COMPRIMENTO / 2.0;
    let x1 = x0 + COMPRIMENTO;

    let t0 = Instant::now();
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let pen_down = ms(t0);
    let d_down = wash_diag::take();

    let mut quadros = Vec::new();
    let mut x = x0;
    while x < x1 {
        let te = Instant::now();
        for _ in 0..EV_POR_QUADRO {
            if x >= x1 {
                break;
            }
            x = (x + passo).min(x1);
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        let ev = ms(te);
        let tt = Instant::now();
        t.on_tick(DT_MS);
        let tk = ms(tt);
        let td = Instant::now();
        let _ = t.take_preview_arc();
        quadros.push((ev, tk, ms(td)));
    }
    let d_gesto = wash_diag::take();

    let tu = Instant::now();
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
    let pen_up = ms(tu);
    let tq = Instant::now();
    t.on_tick(DT_MS);
    let _ = t.take_preview_arc();
    let quadro_do_up = ms(tq);
    let d_up = wash_diag::take();

    let mut secagem = Vec::new();
    for _ in 0..QUADROS_DEPOIS {
        let ts = Instant::now();
        t.on_tick(DT_MS);
        let _ = t.take_preview_arc();
        secagem.push(ms(ts));
    }
    let d_depois = wash_diag::take();
    Corrida {
        pen_down,
        quadros,
        pen_up,
        quadro_do_up,
        secagem,
        d_down,
        d_gesto,
        d_up,
        d_depois,
    }
}

/// FNV-1a de 64 bits sobre os bytes de cada quadro drenado, encadeado quadro a quadro.
fn fnv(mut h: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// A impressão digital de uma sessão de DOIS traços que se cruzam — o segundo com outra cor, outro
/// Rewet e outro tamanho, para a tabela de donos ter dois estilos —, cada quadro drenado a entrar no
/// hash, incluindo os de secagem depois do segundo pen-up.
fn impressao(size: u32, rewet: f32, passo: f32) -> u64 {
    let mut t = aquarela(size, rewet);
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let drena = |t: &mut PainterTool, h: &mut u64| {
        t.on_tick(DT_MS);
        if let Some((px, w, hh)) = t.take_preview_arc() {
            *h = fnv(*h, &px);
            *h = fnv(*h, &[w as u8, hh as u8]);
        }
    };
    let c = size as f32 / 2.0;
    let pernas: [([f32; 2], [f32; 2]); 2] = [
        ([c - 300.0, c], [c + 300.0, c]),
        ([c, c - 280.0], [c + 60.0, c + 280.0]),
    ];
    for (k, (a, b)) in pernas.into_iter().enumerate() {
        if k == 1 {
            t.set_brush_color_srgb8([40, 90, 200]);
            t.set_brush_wet_rewet((rewet * 0.5 + 0.2).min(1.0));
            t.set_brush_size_px(140.0);
        }
        t.on_canvas_pointer(cp(a, PointerPhase::Down));
        let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let n = (len / passo) as usize;
        for i in 1..=n {
            let f = i as f32 / n as f32;
            t.on_canvas_pointer(cp(
                [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
                PointerPhase::Move,
            ));
            if i % EV_POR_QUADRO as usize == 0 {
                drena(&mut t, &mut h);
            }
        }
        t.on_canvas_pointer(cp(b, PointerPhase::Up));
        for _ in 0..6 {
            drena(&mut t, &mut h);
        }
    }
    h
}

fn p50(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn maximo(v: &[f64]) -> f64 {
    v.iter().copied().fold(0.0, f64::max)
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn diag(nome: &str, d: &wash_diag::WashRead) {
    println!(
        "      {nome:<8} composite {:7.2}/{:7.2} x{:<3} | carimbo {:6.3}/{:6.2} x{:<4} | pour {:6.2}/{:6.2} x{:<3} \
         | secagem {:6.2}/{:6.2} x{:<3} | pen-down {:6.2} x{} | janela {:5.2} Mtx | {:5.1} ns/tx",
        d.composite.avg_ms,
        d.composite.max_ms,
        d.composite.n,
        d.stamp.avg_ms,
        d.stamp.max_ms,
        d.stamp.n,
        d.pour.avg_ms,
        d.pour.max_ms,
        d.pour.n,
        d.dry.avg_ms,
        d.dry.max_ms,
        d.dry.n,
        d.pendown.avg_ms,
        d.pendown.n,
        d.window_px_per_composite / 1e6,
        d.ns_per_texel,
    );
}

fn celula(size: u32, rewet: f32, passo: f32) {
    let mut corridas: Vec<Corrida> = (0..CORRIDAS)
        .map(|_| {
            let mut t = aquarela(size, rewet);
            traco(&mut t, size, passo)
        })
        .collect();
    // Estatística por corrida, depois o MÍNIMO sobre as corridas (a leitura menos contaminada pela
    // carga das outras linhas); o `pior` é o máximo absoluto de todas.
    let mut q50 = Vec::new();
    let mut q90 = Vec::new();
    let mut qmx = Vec::new();
    let mut acima = Vec::new();
    let mut ev50 = Vec::new();
    let mut tk50 = Vec::new();
    let mut dr50 = Vec::new();
    let mut s50 = Vec::new();
    for c in &mut corridas {
        let mut tot: Vec<f64> = c.quadros.iter().map(|q| q.0 + q.1 + q.2).collect();
        qmx.push(maximo(&tot));
        acima.push(tot.iter().filter(|&&v| v > f64::from(DT_MS)).count() as f64);
        q50.push(p50(&mut tot));
        q90.push(tot[(tot.len() * 9) / 10]);
        ev50.push(p50(&mut c.quadros.iter().map(|q| q.0).collect::<Vec<_>>()));
        tk50.push(p50(&mut c.quadros.iter().map(|q| q.1).collect::<Vec<_>>()));
        dr50.push(p50(&mut c.quadros.iter().map(|q| q.2).collect::<Vec<_>>()));
        s50.push(p50(&mut c.secagem.clone()));
    }
    let mn = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let pd: Vec<f64> = corridas.iter().map(|c| c.pen_down).collect();
    let pu: Vec<f64> = corridas.iter().map(|c| c.pen_up).collect();
    let qu: Vec<f64> = corridas.iter().map(|c| c.quadro_do_up).collect();
    println!(
        "  {size:>5}² rewet {rewet:.3} passo {passo:3.1} ({:>2} q) | quadro p50 {:6.2} p90 {:6.2} max {:6.2} (pior {:6.2}) >16,7ms {:>2} \
         = ev {:6.2} + tick {:6.2} + dren {:5.2} | pen-down {:7.2} | pen-up {:7.2} | quadro do up {:6.2} \
         | secagem p50 {:5.2}",
        corridas[0].quadros.len(),
        mn(&q50),
        mn(&q90),
        mn(&qmx),
        maximo(&qmx),
        mn(&acima),
        mn(&ev50),
        mn(&tk50),
        mn(&dr50),
        mn(&pd),
        mn(&pu),
        mn(&qu),
        mn(&s50),
    );
    // O split do `wash_diag` da corrida com o menor p50 de quadro.
    let k = q50
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(b.1))
        .map_or(0, |(i, _)| i);
    let c = &corridas[k];
    diag("down", &c.d_down);
    diag("gesto", &c.d_gesto);
    diag("up", &c.d_up);
    diag("depois", &c.d_depois);
}

/// Uma linha da ablação: o nome e o toque nos knobs (pelas MESMAS portas do painel), aplicado
/// depois da aquarela do dono.
type Toque = (&'static str, fn(&mut PainterTool));

/// **A ablação pela ENTRADA** (o método do doc 31 §2.2): um knob de cada vez volta ao neutro, e o
/// que o quadro perde é o que aquele knob custava. As variantes correm **ALTERNADAS** dentro de
/// cada ronda (a carga desta máquina deriva entre rondas), e cada coluna é o mínimo das rondas.
fn ablacao(size: u32, passo: f32) {
    const TOQUES: [Toque; 6] = [
        ("dono", |_| {}),
        ("Rewet 0", |t| t.set_brush_wet_rewet(0.0)),
        ("Charge 1 (mixer off)", |t| t.set_brush_wet_charge(1.0)),
        ("Smudge 0", |t| t.set_brush_wet_smudge(0.0)),
        ("Pigment 0", |t| t.set_brush_pigment_mixing(0.0)),
        ("Charge 1 + Rewet 0", |t| {
            t.set_brush_wet_charge(1.0);
            t.set_brush_wet_rewet(0.0);
        }),
    ];
    // [variante] -> (q50, q90, comp_gesto, ns/tx, janela, pen-down, comp_up, pen-up)
    let mut melhor = [[f64::INFINITY; 8]; TOQUES.len()];
    for _ in 0..CORRIDAS {
        for (i, (_, toque)) in TOQUES.iter().enumerate() {
            let mut t = aquarela(size, 0.400);
            toque(&mut t);
            let c = traco(&mut t, size, passo);
            let mut tot: Vec<f64> = c.quadros.iter().map(|q| q.0 + q.1 + q.2).collect();
            let q50 = p50(&mut tot);
            let q90 = tot[(tot.len() * 9) / 10];
            let linha = [
                q50,
                q90,
                c.d_gesto.composite.avg_ms,
                c.d_gesto.ns_per_texel,
                c.d_gesto.window_px_per_composite / 1e6,
                c.pen_down,
                c.d_up.composite.avg_ms,
                c.pen_up,
            ];
            for (m, v) in melhor[i].iter_mut().zip(linha) {
                *m = m.min(v);
            }
        }
    }
    println!(
        "\n  ABLAÇÃO {size}² passo {passo} (load {}) — min de {CORRIDAS} rondas alternadas (ms)",
        carga()
    );
    println!(
        "  {:<22} | quadro p50 |   p90 | composite/quadro | ns/tx | Mtx/comp | pen-down | commit comp | pen-up",
        "variante"
    );
    for ((nome, _), m) in TOQUES.iter().zip(melhor) {
        println!(
            "  {nome:<22} | {:10.2} | {:5.2} | {:16.2} | {:5.1} | {:8.2} | {:8.2} | {:11.2} | {:6.2}",
            m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7]
        );
    }
    println!("  load {}", carga());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // `-- ablacao [2048|4096]`: um knob de cada vez ao neutro, variantes alternadas por ronda.
    if args.iter().any(|a| a == "ablacao") {
        let size = if args.iter().any(|a| a == "2048") {
            2048
        } else {
            4096
        };
        ablacao(size, 1.0);
        ablacao(size, 3.0);
        return;
    }
    // `-- dono [2048|4096]`: SÓ a célula do dono (Rewet 0,4, passo 1 px) — a régua curta do A/B
    // alternado entre dois binários, onde o que vale é a diferença dentro da mesma ronda.
    if args.iter().any(|a| a == "dono") {
        let size = if args.iter().any(|a| a == "2048") {
            2048
        } else {
            4096
        };
        println!("  load {}", carga());
        celula(size, 0.400, 1.0);
        println!("  load {}", carga());
        return;
    }
    // `-- impressao`: a IMPRESSÃO DIGITAL de cada quadro que a ponte drena, numa sessão de dois
    // traços que se cruzam com estilos diferentes (a tabela de donos com dois estilos, o campo de
    // estilo, o campo molhado, a reserva, o aro, a água e a secagem). Não mede tempo: é o gate
    // PONTA A PONTA de que uma optimização não mudou um byte do PRODUTO — dois binários (antes e
    // depois, ou dois alvos de CPU) têm de imprimir as MESMAS linhas.
    if args.iter().any(|a| a == "impressao") {
        for (rewet, passo) in [(0.400f32, 1.0f32), (0.0, 3.0), (1.0, 2.0)] {
            println!(
                "  rewet {rewet} passo {passo}: {:016x}",
                impressao(1024, rewet, passo)
            );
        }
        return;
    }
    // `-- perfil [2048|4096] [rewet0]`: a mesma célula, muitas vezes — carga para o amostrador.
    if args.iter().any(|a| a == "perfil") {
        let size = if args.iter().any(|a| a == "2048") {
            2048
        } else {
            4096
        };
        let rewet = if args.iter().any(|a| a == "rewet0") {
            0.0
        } else {
            0.400
        };
        for _ in 0..12 {
            let mut t = aquarela(size, rewet);
            let _ = traco(&mut t, size, 1.0);
        }
        return;
    }
    println!(
        "\n  A AQUARELA DO DONO NO PRODUTO — traço de {COMPRIMENTO} px, {EV_POR_QUADRO} eventos/quadro (1 kHz), \
         raio 250, min de {CORRIDAS} corridas (ms)"
    );
    for size in [2048u32, 4096] {
        for rewet in [0.400f32, 0.0] {
            for passo in [1.0f32, 3.0] {
                println!("  load {}", carga());
                celula(size, rewet, passo);
            }
        }
    }
    println!("  load {}", carga());
}
