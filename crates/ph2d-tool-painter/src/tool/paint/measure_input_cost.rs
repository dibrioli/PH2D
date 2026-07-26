//! **O que um EVENTO de ponteiro custa** — a frente L do plano 26, depois que o relógio
//! `EVENTO->FRAME` mostrou onde o tempo estava.
//!
//! O `PaintFrameTimer` cronometra o `run_render_frame` e o `on_canvas_pointer` **não roda lá dentro**
//! (ele roda no handler de input do winit), então o custo de carimbar dabs nunca apareceu em `frame`,
//! nem em `dispatch`, nem em nenhum dos 17 sub-slots. Medido no produto a 4096² (Enio, 2026-07-25):
//!
//! | | |
//! |---|---|
//! | `período real` | **25,0 ms/frame** (40 fps) |
//! | `frame` (o que o timer via) | 12,8 ms |
//! | **`INPUT` (fora do frame)** | **12,6 ms** |
//! | `INPUT max` num ÚNICO evento | **67 a 139 ms** |
//!
//! `período = frame + INPUT`, e a conta fecha. Esta sonda parte o `INPUT` em **pen-down** e **move**,
//! porque os dois relatos do Enio são grandezas diferentes: *"o primeiro traço tem um delay"* é o
//! pen-down, *"pintar rápido cai fps"* é o move.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um tool com impasto ligado pela PORTA do produto — é ele que faz um traço tocar os cinco planos
/// por-traço, e é o caso do produto desde 2026-07-13.
fn tool(side: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.toggle_brush_impasto();
    t.set_brush_size_px(24.0);
    t
}

fn ms(f: &mut dyn FnMut()) -> f64 {
    let t0 = std::time::Instant::now();
    f();
    t0.elapsed().as_secs_f64() * 1e3
}

/// **O pen-down e o move, separados** — as duas grandezas dos dois relatos.
///
/// Não afirma nada; IMPRIME. O gate que sai desta medição é o irmão abaixo.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_input_cost -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_input_cost_is_measured_not_assumed() {
    println!("[input] tela    traco  pen-down    move   (ms)");
    for side in [1024u32, 4096] {
        let mut t = tool(side);
        #[allow(clippy::cast_precision_loss)]
        let m = f32::from(u16::try_from(side).unwrap_or(u16::MAX)) / 1024.0;
        // DOIS traços: o 1º paga a alocação dos planos, o 2º deveria REUSÁ-LA. Se os dois custarem o
        // mesmo, a capacidade está sendo jogada fora entre traços — e é isso que a sonda procura.
        for stroke in 1..=2u8 {
            let y = (100.0 + f32::from(stroke) * 60.0) * m;
            let down = ms(&mut || {
                t.on_canvas_pointer(cp([100.0 * m, y], PointerPhase::Down));
            });
            let mv = ms(&mut || {
                t.on_canvas_pointer(cp([300.0 * m, y], PointerPhase::Move));
            });
            t.on_canvas_pointer(cp([300.0 * m, y], PointerPhase::Up));
            println!("[input] {side:>5}  {stroke:>5}  {down:>8.2}  {mv:>6.2}");
        }
    }
}

/// **O CONTROLE: o mesmo gesto SEM impasto.**
///
/// Se o pen-down encolher, o custo é dos cinco planos por-traço; se não encolher, ele é do carimbo, e
/// a frente muda de alvo. Uma medição sem controle nomeia um suspeito, não uma causa.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_input_cost -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_input_cost_without_impasto_is_the_control() {
    println!("[input-ctl] tela   impasto  pen-down    move   (ms)");
    for side in [1024u32, 4096] {
        for impasto in [false, true] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
            if impasto {
                t.toggle_brush_impasto();
            }
            t.set_brush_size_px(24.0);
            // ⚠️ O traço tem comprimento FIXO em px, NÃO escalado com a tela. A 1ª versão desta sonda
            // escalava (`100*m → 300*m`) e mediu razões de ~4× para 16× de área — que eu quase li como
            // *"o move é canvas-shaped"*. Não é: 4× é exatamente o fator de COMPRIMENTO que a própria
            // fixture introduziu. A variável tem de ser isolada, senão a sonda mede a si mesma.
            let down = ms(&mut || {
                t.on_canvas_pointer(cp([100.0, 300.0], PointerPhase::Down));
            });
            let mv = ms(&mut || {
                t.on_canvas_pointer(cp([300.0, 300.0], PointerPhase::Move));
            });
            println!(
                "[input-ctl] {side:>5}  {:>7}  {down:>8.2}  {mv:>6.2}",
                if impasto { "ON" } else { "off" }
            );
        }
    }
}

/// **DE ONDE vem o pen-down: o snapshot de undo compartilha o `Arc` do canvas, e o 1º dab o BIFURCA.**
///
/// O move é plano na tela (0,75 ms a 1024² e a 4096²) — trabalho honesto por dab. O pen-down é
/// **linear na ÁREA** e **mesmo sem impasto**: 0,73 → **11,47 ms**. Um `memcpy` de 67 MB a 4096² custa
/// exatamente isso.
///
/// O mecanismo: `paint_begin` tira um `ModelSnapshot` para o undo, e ele guarda `canvas_rgba` como
/// **`Arc` clonado**. O 1º dab do traço escreve no canvas ⇒ `Arc::make_mut` vê `strong_count == 2` e
/// **copia o buffer inteiro**. Copy-on-write, uma vez por traço, do tamanho da tela.
///
/// ⚠️ **Isto é o MESMO defeito que a sonda `measure_undo_memory` mede pelo outro lado** — o snapshot
/// guarda um documento inteiro por passo. Lá ele custa **memória** (1.627 MB em 24 traços); aqui custa
/// **latência** (11,5 ms no pen-down a 4096²). A cura é a mesma, e é a frente **U1** do plano 26:
/// histórico por DELTA. Ela deixou de ser só uma questão de orçamento de RAM.
///
/// ## Medido (2026-07-25)
///
/// | tela | copiar o canvas | **pen-down medido (sem impasto)** |
/// |---|---|---|
/// | 1024² | 0,70 ms | **0,73** |
/// | 2048² | 2,54 ms | ~3,2 |
/// | 4096² | **9,40 ms** | **11,47** |
///
/// **O pen-down É a cópia do canvas**, dentro do ruído. E o move é PLANO na tela (0,75 ms a 1024² e a
/// 4096²) — trabalho honesto por dab, não um defeito.
///
/// ⚠️ **A cura é a frente U1 (histórico por DELTA), e não há atalho:** duas versões do canvas têm de
/// coexistir enquanto o traço corre, então UMA cópia é irredutível — a menos que o passo de undo
/// guarde só a REGIÃO que o traço tocou, que é exatamente o que a U1 propõe. Ela deixou de ser uma
/// questão de orçamento de RAM: ela é também os 9,4 ms do primeiro traço.
///
/// Não afirma nada; IMPRIME.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_pen_down_forks_the_canvas_because_undo_holds_it() {
    use std::sync::Arc;
    let mut t = tool(2048);
    println!(
        "[fork] antes do pen-down: strong_count = {}",
        Arc::strong_count(&t.canvas_rgba)
    );
    t.on_canvas_pointer(cp([100.0, 300.0], PointerPhase::Down));
    println!(
        "[fork] depois do pen-down: strong_count = {} (⚠️ NAO decide nada — ver abaixo)",
        Arc::strong_count(&t.canvas_rgba)
    );
    // ⚠️ O `strong_count` DEPOIS do pen-down e um oraculo RUIM: `1` e o que se ve tanto se o buffer
    // nunca foi compartilhado quanto se ele JA foi bifurcado (o tool fica com a copia nova, unica, e o
    // snapshot com a velha). Ele nao distingue as duas, entao nao decide nada.
    //
    // O que se pode afirmar honestamente e a MAGNITUDE: se copiar o canvas custa o que o pen-down
    // custa, a atribuicao e credivel; se nao custa, ela esta errada.
    for side in [1024u32, 2048, 4096] {
        let n = (side as usize) * (side as usize) * 4;
        let src: Vec<u8> = vec![7u8; n];
        let mut dst = Vec::new();
        let cost = ms(&mut || dst = src.clone());
        println!(
            "[fork] copiar o canvas {side}x{side} ({} MB): {cost:.2} ms",
            n / 1_048_576
        );
        std::hint::black_box(dst.len());
    }
}
