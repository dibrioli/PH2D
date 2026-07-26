//! **U0/U1 do plano 26** — quanto o histórico de undo do Painter de fato retém.
//!
//! O molde é o do **ADR-0117** (`ph2d-audio-edit/tests/measure_memory.rs`), que emendou o HR-13 com
//! *quem declara budget possui um gate que MEDE*; o áudio chegou a **4351 MB** com a regra em vigor e
//! nenhum byte observado. Esta é a mesma pergunta feita ao Painter.
//!
//! ## Como ele nasceu (2026-07-25) e o que a cura moveu (2026-07-26)
//!
//! | | antes (documento por passo) | agora (delta por janela) |
//! |---|---|---|
//! | pico, 24 traços a 2048² | 1.669,2 MB | **345,8 MB** |
//! | retido no fim | **1.627,2 MB** (25,4 documentos) | **242,2 MB** (3,8) |
//! | **por passo** | **~67,8 MB** = mais que um documento | **~4,3 MB** = a janela do traço |
//!
//! Um traço de 24 px de largura por 1.600 de comprimento cobre um bbox de ~1600×80 depois da orla do
//! falloff; os quatro planos daquela janela somam ~2 MB e a entrada guarda **os dois lados** dela. O
//! custo deixou de ser função do DOCUMENTO e passou a ser função da REGIÃO TOCADA.
//!
//! ⚠️ **A barra mudou junto com a cura, e tinha de mudar.** A antiga (`4 documentos + 0,5 MB/traço`)
//! descrevia o DEFEITO, não a propriedade: com a cura ela passaria a 24 traços por acaso e voltaria a
//! falhar a 60. E ela media o **total**, que mistura o histórico com o working set do tool — o documento
//! vivo, o composite, os envelopes de relevo, ~139 MB que barra nenhuma deveria ter de adivinhar.
//!
//! ⚠️ **O oráculo é a INCLINAÇÃO: quanto custa MAIS UM PASSO.** Ela isola tudo o que é constante (a 2ª
//! metade da sessão já encontra os planos alocados) e é literalmente a frase que o defeito violava —
//! *"um documento por traço, LINEAR"*.
//!
//! Um `#[test]` por binário, de propósito: os contadores do dhat são globais do processo e o
//! `cargo test` roda os testes de um binário em threads — dois profilers num processo se atropelam.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --release --test measure_undo_memory -- --nocapture
//! ```

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;
/// O regime que o Enio reporta. (A 4096² tudo abaixo quadruplica — a propriedade é a mesma.)
const N: u32 = 2048;
/// Traços de um artista numa sessão curta. Medidos em DUAS metades, para a inclinação.
const STROKES: usize = 24;
/// O pincel, e portanto a altura da janela de cada traço.
const BRUSH_PX: f32 = 24.0;
/// O comprimento do traço — de `X0` a `X1`, na horizontal.
const X0: f32 = 200.0;
const X1: f32 = 1800.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn stroke(t: &mut PainterTool, k: usize) {
    #[allow(clippy::cast_precision_loss)]
    let y = 200.0 + (k as f32) * 40.0;
    t.on_canvas_pointer(cp([X0, y], PointerPhase::Down));
    t.on_canvas_pointer(cp([X1, y], PointerPhase::Move));
    t.on_canvas_pointer(cp([X1, y], PointerPhase::Up));
}

/// **O gate de memória do histórico.** Nasceu VERMELHO (1.627 MB) e é o que a wave U1 fechou.
///
/// Duas asserções, e a segunda existe porque a primeira sozinha não protege o cap: um controller que
/// **conta** errado é um cap que não morde na hora certa, e nada nos pixels denunciaria isso.
#[test]
fn a_stroke_costs_the_window_it_touched_not_a_document() {
    let px = f64::from(N) * f64::from(N);
    // Os quatro planos canvas-shaped de UMA camada tocada: rgba + heights(f32) + covers + mats([u8;7]).
    let bytes_per_px = 4.0 + 4.0 + 1.0 + 7.0;
    let one_doc = px * bytes_per_px / MB;

    // ⚠️ O profiler tem de estar VIVO para o `HeapStats::get` — sem ele o dhat entra em pânico
    // (*"getting heap stats when no profiler is running"*), que é como esta sonda nasceu.
    let profiler = dhat::Profiler::builder().testing().build();
    let before = dhat::HeapStats::get();

    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    // Impasto pela PORTA do produto (o `BrushSpec` não é público) — é ele que faz um traço tocar
    // QUATRO planos em vez de um, e é o caso do produto desde 2026-07-13. Sem impasto a medição
    // descreveria um Painter que não existe.
    t.toggle_brush_impasto();
    t.set_brush_size_px(BRUSH_PX);
    for k in 0..STROKES / 2 {
        stroke(&mut t, k);
    }
    let half = dhat::HeapStats::get();
    let counted_half = t.undo_retained_bytes();
    for k in STROKES / 2..STROKES {
        stroke(&mut t, k);
    }
    let counted = t.undo_retained_bytes();
    let after = dhat::HeapStats::get();
    drop(profiler);

    #[allow(clippy::cast_precision_loss)]
    let mb = |b: usize| b as f64 / MB;
    let peak = mb(after.max_bytes);
    let held = mb(after.curr_bytes) - mb(before.curr_bytes);
    #[allow(clippy::cast_precision_loss)]
    let counted_mb = counted as f64 / MB;
    #[allow(clippy::cast_precision_loss)]
    let n_half = (STROKES / 2) as f64;
    let slope = (mb(after.curr_bytes) - mb(half.curr_bytes)) / n_half;
    #[allow(clippy::cast_precision_loss)]
    let slope_counted = (counted - counted_half) as f64 / MB / n_half;

    println!(
        "[undo-mem] {N}x{N} · {STROKES} tracos com impasto\n\
         [undo-mem]   um documento (4 planos) = {one_doc:.1} MB\n\
         [undo-mem]   pico             {peak:.1} MB  ({:.1} documentos)\n\
         [undo-mem]   retido no fim    {held:.1} MB  ({:.1} documentos)\n\
         [undo-mem]   o historico CONTA {counted_mb:.1} MB\n\
         [undo-mem]   POR PASSO        {slope:.2} MB medido · {slope_counted:.2} MB contado  \
         ({:.1}% de um documento)",
        peak / one_doc,
        held / one_doc,
        100.0 * slope / one_doc,
    );

    // (1) A LEI: mais um passo custa a JANELA que ele tocou, não um documento. A barra é uma fração
    // pequena do documento e não um número afinado — o defeito media **67,8 MB/traço**, isto é MAIS que
    // um documento inteiro (os dois endpoints), contra os ~4,3 MB que o delta cobra.
    let bar = one_doc / 5.0;
    assert!(
        slope < bar,
        "cada traco a mais retem {slope:.1} MB ({:.0}% de um documento de {one_doc:.0} MB) — a barra e \
         {bar:.1} MB. Um passo voltou a custar um DOCUMENTO em vez da janela que ele tocou.",
        100.0 * slope / one_doc,
    );
    // (2) …e a CONTABILIDADE do controller tem de acompanhar a inclinação REAL, não apenas ser pequena:
    // é `retained_bytes` que o cap em BYTES consulta.
    assert!(
        slope_counted > 0.5 * slope && slope_counted < 2.0 * slope,
        "o historico CONTA {slope_counted:.2} MB por passo e o processo RETEM {slope:.2} — a \
         contabilidade do cap perdeu contato com a realidade"
    );
}
