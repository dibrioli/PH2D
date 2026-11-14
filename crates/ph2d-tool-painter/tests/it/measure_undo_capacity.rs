//! **Quantos passos o cap em BYTES compra** — o número que `DEFAULT_MAX_BYTES` promete, medido em vez
//! de afirmado (§0 dos inegociáveis: *antes de escrever qualquer limite, MEÇA; depois escreva o número
//! que a medição deu, com a tabela ao lado dele*).
//!
//! Binário próprio, sem `dhat`: ele lê o `retained_bytes()` do controller, que é **determinístico** e
//! não depende de profiler nem de alocador. O irmão `measure_undo_memory.rs` faz a pergunta inversa —
//! *o que o processo de fato retém* — e reconcilia os dois números.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --release --test measure_undo_capacity -- --nocapture
//! ```

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{
    PainterTool, history_budget_bytes, history_budget_here, history_budget_on,
};

const MB: f64 = 1_048_576.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um traço horizontal atravessando a tela, com impasto (quatro planos tocados).
fn stroke(t: &mut PainterTool, side: u32, k: usize) {
    #[allow(clippy::cast_precision_loss)]
    let y = 100.0 + (k as f32) * 24.0;
    #[allow(clippy::cast_precision_loss)]
    let x1 = (side as f32) - 100.0;
    t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down));
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Move));
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
}

/// **A TABELA que o `DEFAULT_MAX_BYTES` carrega no doc-comment.** Não afirma o número do cap; afirma que
/// ele foi MEDIDO e que a promessa qualitativa se sustenta: *o cap não morde no uso normal — ele existe
/// para o caso irredutível.*
#[test]
fn the_byte_cap_buys_this_many_steps() {
    println!("[undo-cap] orcamento = 2x documento + 256 MB (o molde do ADR-0117):");
    let mut worst_gain = f64::MAX;
    for side in [1024u32, 2048, 4096] {
        #[allow(clippy::cast_precision_loss)]
        let budget = history_budget_bytes(side, side) as f64;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
        t.toggle_brush_impasto();
        t.sonda_raio(24.0);
        for k in 0..8 {
            stroke(&mut t, side, k);
        }
        #[allow(clippy::cast_precision_loss)]
        let per_stroke = t.undo_retained_bytes() as f64 / 8.0;
        // O que o MESMO orçamento comprava com um documento por endpoint — o modelo que a U1 substituiu.
        let px = f64::from(side) * f64::from(side);
        let old_step = 2.0 * px * (4.0 + 4.0 + 1.0 + 7.0);
        let gain = old_step / per_stroke;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let strokes = (budget / per_stroke) as usize;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let old_strokes = (budget / old_step) as usize;
        println!(
            "[undo-cap]   {side:>4}: orcamento {:>5.0} MB · passo {:>6.2} MB => {strokes:>4} tracos  \
             (o modelo antigo comprava {old_strokes:>2}: {gain:>4.1}x)",
            budget / MB,
            per_stroke / MB,
        );
        worst_gain = worst_gain.min(gain);
    }
    // ⚠️ **A afirmação é uma RAZÃO, não um número de passos.** Quantos passos cabem depende da tela e do
    // comprimento do traço, e cravar um número seria afiná-lo até passar; o que a U1 promete é que o
    // passo deixou de ser um DOCUMENTO — e isso se mede contra o que o mesmo orçamento comprava antes.
    assert!(
        worst_gain > 5.0,
        "o delta compra so {worst_gain:.1}x mais passos que um documento por endpoint na pior tela — \
         um passo voltou a ter a ordem de grandeza do documento"
    );
}

/// **B42.3 — o tecto do undo é o da RAM, não o da fórmula do documento.** Recurso: a RAM física, à fração
/// de cada cache (`1/32`); a fórmula `2·doc + 256 MB` fica como piso (máquina sem leitura).
#[test]
fn teto_do_undo_deriva_da_ram() {
    const GIB: u64 = 1 << 30;
    let piso = history_budget_bytes(3840, 2160);
    assert_eq!(
        history_budget_on(3840, 2160, None),
        piso,
        "sem leitura, o piso"
    );
    assert_eq!(
        history_budget_on(3840, 2160, Some(8 * GIB)),
        piso,
        "8 GiB: RAM/32 < piso"
    );
    assert_eq!(
        history_budget_on(3840, 2160, Some(128 * GIB)),
        (4 * GIB) as usize
    );
    // O documento aberto leva o da máquina, nas duas rotas (set_source e a troca de documento).
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 3840 * 2160 * 4], 3840, 2160);
    assert_eq!(t.undo_max_bytes(), history_budget_here(3840, 2160));
}

/// O passo de undo a 3840×2160 (a tela 4K do produto, não 4096²): a tabela de `history_budget_on`.
#[test]
#[ignore = "sonda: imprime o passo e quantos cabem na RAM desta máquina"]
fn mede_o_passo_de_undo_a_4k() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 3840 * 2160 * 4], 3840, 2160);
    t.toggle_brush_impasto();
    t.sonda_raio(24.0);
    for k in 0..8 {
        #[allow(clippy::cast_precision_loss)]
        let y = 100.0 + (k as f32) * 24.0;
        t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down));
        t.on_canvas_pointer(cp([3740.0, y], PointerPhase::Move));
        t.on_canvas_pointer(cp([3740.0, y], PointerPhase::Up));
    }
    #[allow(clippy::cast_precision_loss)]
    let passo = t.undo_retained_bytes() as f64 / 8.0;
    for (nome, ram) in [
        ("sem leitura", None),
        ("8 GiB", Some(8u64 << 30)),
        ("32 GiB", Some(32u64 << 30)),
        ("esta", ph2d_core::razao::ram_fisica()),
    ] {
        #[allow(clippy::cast_precision_loss)]
        let b = history_budget_on(3840, 2160, ram) as f64;
        println!(
            "[undo-4k] {nome:>11}: orcamento {:>7.0} MB · passo {:.2} MB => {:.0} tracos",
            b / MB,
            passo / MB,
            b / passo
        );
    }
}
