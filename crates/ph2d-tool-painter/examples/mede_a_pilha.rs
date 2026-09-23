//! **O preço da pilha do Composite Brush no PRODUTO** — e não no build de teste.
//!
//! ⛔⛔ As sondas `diag_*` desta crate correm sob `cfg(test)`, e ali o traço paga duas coisas que o
//! app nunca paga: o **journal do undo** do canvas (`capture_canvas` é `cfg(any(test,
//! debug_assertions))`) e a **espia do esfregão**, que copia o campo de deslocamento inteiro
//! (`8 MB` a `1024²`) a cada lote. *Uma sonda que mede o build de teste mede outro programa.* Este
//! exemplo compila a biblioteca SEM `cfg(test)`, pelas portas públicas que a cena
//! `PH2D_COMPOSITE_SMOKE` usa.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_a_pilha
//! ```
//!
//! Imprime `/proc/loadavg` ao lado: acima de `load ~5` nenhum relógio desta máquina vale nada.

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;

const SIZE: u32 = 1024;
const Y: f32 = 512.0;
const X0: f32 = 152.0;
const X1: f32 = 872.0;

/// A pilha da foto do dono (a mesma de `ph2d_app_painter::composite_smoke`), posição 0 = topo.
fn pilha_do_dono() -> PainterTool {
    const PILHA: [(u8, f32, f32, Option<[f32; 3]>); 6] = [
        (2, 1.0, 2.048, None),
        (0, 0.133, 0.574, Some([1.0, 1.0, 1.0])),
        (0, 0.204, 1.002, Some([1.0, 0.0, 0.0])),
        (0, 0.176, 1.221, Some([0.0, 0.0, 0.0])),
        (1, 0.596, 1.0, None),
        (3, 0.104, 1.0, None),
    ];
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    for (pos, &(op, forca, tamanho, cor)) in PILHA.iter().enumerate() {
        t.acrescenta_camada(op);
        t.set_composite_layer_strength(pos, forca);
        t.set_composite_layer_size(pos, tamanho);
        match cor {
            Some(rgb) => t.set_composite_layer_color(pos, rgb),
            None => t.clear_composite_layer_color(pos),
        }
    }
    if !t.composite_enabled() {
        t.toggle_composite();
    }
    t.set_brush_size_norm(0.4);
    t.set_compor_por_quadro(true);
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

/// O traço de `X0` a `X1`, drenando a pré-visualização a cada `drena` eventos (a ponte do app
/// drena uma vez por quadro). Devolve (ms, drenagens).
fn traco(t: &mut PainterTool, passo: f32, drena: u32) -> (f64, u32) {
    let ini = std::time::Instant::now();
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    let (mut x, mut n, mut q) = (X0, 0u32, 0u32);
    while x < X1 {
        x += passo;
        t.on_canvas_pointer(cp([x.min(X1), Y], PointerPhase::Move));
        n += 1;
        if n.is_multiple_of(drena) {
            let _ = t.take_preview_arc();
            q += 1;
        }
    }
    t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
    (ini.elapsed().as_secs_f64() * 1e3, q.max(1))
}

fn main() {
    // `-- perfil`: só a pilha cheia, muitas vezes — a carga que se dá a um perfilador de amostras.
    if std::env::args().any(|a| a == "perfil") {
        for _ in 0..40 {
            let mut t = pilha_do_dono();
            let _ = traco(&mut t, 8.0, 16);
        }
        return;
    }
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n  A PILHA DO DONO NO PRODUTO   (load {} · canvas {SIZE}² · traço de {} px · drena a cada 16)",
        carga.trim(),
        X1 - X0
    );
    println!("  passo px |  ms do traço (min de 5) | ms por quadro");
    for passo in [2.0f32, 8.0] {
        let mut melhor = (f64::MAX, 1);
        for _ in 0..5 {
            let mut t = pilha_do_dono();
            let r = traco(&mut t, passo, 16);
            if r.0 < melhor.0 {
                melhor = r;
            }
        }
        println!(
            "  {passo:8.1} | {:23.1} | {:8.2}",
            melhor.0,
            melhor.0 / f64::from(melhor.1)
        );
    }
    // ⭐ A partição ACUMULAR / COMPOR sem uma linha de instrumento no produto: drenar a cada 16
    // eventos compõe uma vez por quadro; NUNCA drenar compõe uma vez só, no pen-up. A diferença é
    // o que as composições dos quadros custam; o que sobra é acumular (mais UMA composição).
    // E cada camada SOZINHA (as outras caladas) diz o preço dela sem o avental das vizinhas.
    const NOMES: [&str; 6] = ["Blur", "Brush 1", "Brush 2", "Brush 3", "Smear", "Erase"];
    println!("\n  passo 8           | por quadro ms | só no pen-up ms | compor ≈ | acumular ≈");
    let mede = |so: Option<usize>, drena: u32| {
        let mut melhor = f64::MAX;
        for _ in 0..5 {
            let mut t = pilha_do_dono();
            if let Some(k) = so {
                for pos in 0..NOMES.len() {
                    if pos != k {
                        t.set_composite_layer_strength(pos, 0.0);
                    }
                }
            }
            melhor = melhor.min(traco(&mut t, 8.0, drena).0);
        }
        melhor
    };
    let linha = |nome: &str, so: Option<usize>| {
        let q = mede(so, 16);
        let u = mede(so, u32::MAX);
        println!(
            "  {nome:17} | {q:13.1} | {u:15.1} | {:8.1} | {:10.1}",
            q - u,
            u
        );
    };
    linha("pilha cheia", None);
    // ⚠️ Uma camada SOZINHA não passa pela pilha (com menos de duas activas a rota é o depósito
    // directo), logo a linha dela não é comparável. A régua é a ABLAÇÃO: a pilha menos ela.
    let sem = |k: usize, drena: u32| {
        let mut melhor = f64::MAX;
        for _ in 0..5 {
            let mut t = pilha_do_dono();
            t.set_composite_layer_strength(k, 0.0);
            melhor = melhor.min(traco(&mut t, 8.0, drena).0);
        }
        melhor
    };
    let (qc, uc) = (mede(None, 16), mede(None, u32::MAX));
    println!("\n  ablação           | Δ por quadro | Δ só no pen-up | ⇒ Δ compor | Δ acumular");
    for (k, nome) in NOMES.iter().enumerate() {
        let (q, u) = (sem(k, 16), sem(k, u32::MAX));
        println!(
            "  sem {nome:13} | {:12.1} | {:14.1} | {:10.1} | {:10.1}",
            qc - q,
            uc - u,
            (qc - q) - (uc - u),
            uc - u
        );
    }
}
