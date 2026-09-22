//! DIAGNÓSTICO — **o que a ordem por TRAÇO custa**, medido na porta do produto.
//!
//! A recomposição regional replaya os lotes cuja caixa toca a do lote novo. Esse número **não
//! cresce com o traço** — ele vale `~2/spacing`, a sobreposição —, mas é o preço inteiro da wave e
//! tem de estar escrito com o relógio ao lado.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_preco_da_pilha -- --ignored --nocapture --test-threads=1
//! ```
//!
//! ⚠️ **Em `--release`**: em debug esta crate lê `~20×` mais lento e o tecto sairia cinco vezes
//! menor. ⚠️ E `/proc/loadavg` vai impresso ao lado — *nenhuma leitura de relógio desta máquina
//! vale nada acima de `load ~5`*.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const SIZE: u32 = 1024;
const Y: f32 = 512.0;
const X0: f32 = 152.0;
const X1: f32 = 872.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tela(raio: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = raio;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..composite::N_CAMADAS {
        t.paint.composite[pos].strength = 0.0;
    }
    t
}

/// O traço inteiro em passos de 2 px — o regime do rato real.
fn traco(t: &mut PainterTool) -> std::time::Duration {
    let ini = std::time::Instant::now();
    t.on_canvas_pointer(cp([X0, Y], PointerPhase::Down));
    let mut x = X0;
    while x < X1 {
        x += 2.0;
        t.on_canvas_pointer(cp([x.min(X1), Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([X1, Y], PointerPhase::Up));
    ini.elapsed()
}

#[test]
#[ignore = "sonda de relógio: corre à mão, em --release e com a máquina calma"]
fn diag_preco_da_pilha() {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("\n  O PREÇO DA ORDEM POR TRAÇO   (load {})", carga.trim());
    println!("  canvas {SIZE}² · traço de {} px · passo 2 px", X1 - X0);
    // ⭐ **A coluna que DECIDE é a de EVENTO, e não a do traço.** Um traço de 720 px em passos de
    // 2 px são `362` eventos de ponteiro, e o que tem de caber num quadro de `16,7 ms` é o custo de
    // UM — *somar o traço inteiro faz um custo perfeitamente interactivo parecer um congelamento*.
    // (Auditoria de 2026-09-22: a tabela de 21/09 foi lida só na coluna do traço, e a leitura dela
    // não sobreviveu à máquina calma.)
    let eventos = ((X1 - X0) / 2.0).ceil() + 2.0;
    println!("\n  pilha                          |  raio |    ms | ms/evento | % de um quadro");
    println!("  -------------------------------+-------+-------+-----------+----------------");
    let casos: [(&str, &[(CompositeOp, f32)]); 5] = [
        ("1 Brush (sem recomposição)", &[(CompositeOp::Brush, 1.0)]),
        (
            "2 Brush",
            &[(CompositeOp::Brush, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "3 Brush",
            &[
                (CompositeOp::Brush, 1.0),
                (CompositeOp::Brush, 1.0),
                (CompositeOp::Brush, 1.0),
            ],
        ),
        (
            "Blur sobre Brush",
            &[(CompositeOp::Blur, 1.0), (CompositeOp::Brush, 1.0)],
        ),
        (
            "Smear sobre Brush",
            &[(CompositeOp::Smear, 1.0), (CompositeOp::Brush, 1.0)],
        ),
    ];
    for raio in [24.0f32, 96.0] {
        for (nome, ops) in casos {
            let mut melhor = f64::MAX;
            for _ in 0..3 {
                let mut t = tela(raio);
                for (i, &(op, s)) in ops.iter().enumerate() {
                    t.paint.composite[i] = CompositeLayer {
                        op,
                        strength: s,
                        ..CompositeLayer::default()
                    };
                }
                melhor = melhor.min(traco(&mut t).as_secs_f64() * 1e3);
            }
            let por_ev = melhor / f64::from(eventos);
            println!(
                "  {nome:30} | {raio:5.0} | {melhor:6.2} | {por_ev:9.3} | {:14.1}",
                por_ev / 16.7 * 100.0
            );
        }
    }
}
