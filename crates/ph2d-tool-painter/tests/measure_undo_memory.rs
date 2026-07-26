//! **U0 do plano 26** — quanto o histórico de undo do Painter de fato retém.
//!
//! O molde é o do **ADR-0117** (`ph2d-audio-edit/tests/measure_memory.rs`), que emendou o HR-13 com
//! *quem declara budget possui um gate que MEDE*; o áudio chegou a **4351 MB** com a regra em vigor e
//! nenhum byte observado. Esta é a mesma pergunta feita ao Painter, e a aritmética já assusta:
//!
//! - o cap é por **CONTAGEM** (`DEFAULT_MAX_DEPTH = 300`), e contagem é **multiplicador, não teto**;
//! - cada entrada carrega os **DOIS** endpoints (`before` + `after`), ambos `ModelSnapshot`;
//! - com impasto, um traço toca **quatro** planos canvas-shaped da camada ativa — a 2048² isso é
//!   `rgba 16 MB + heights 16 MB + covers 4 MB + mats 28 MB = 64 MB`.
//!
//! ⚠️ **Ela IMPRIME e afirma uma propriedade ESTRUTURAL, não um número absoluto** — a mesma escolha
//! que o ADR-0117 §4.1 fez ao descobrir que a barra fixa dele era aritmeticamente impossível. O que
//! um histórico saudável promete é *o documento + um punhado de planos em construção + deltas*;
//! **não N documentos**. É essa frase que 300 entradas de dois endpoints podem violar.
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
/// Traços de um artista numa sessão curta. O cap é 300; se o retido crescer linearmente aqui, ele
/// cresce linearmente até lá.
const STROKES: usize = 24;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// **NASCEU VERMELHO, e o defeito é REAL — este é um repro, não um gate verde.**
///
/// ## Medido (2026-07-25, 2048², 24 traços com impasto)
///
/// | | |
/// |---|---|
/// | um documento (4 planos) | 64,0 MB |
/// | pico | **1.669,2 MB** (26,1 documentos) |
/// | retido no fim | **1.627,2 MB** (25,4 documentos) |
/// | barra estrutural | 268,0 MB |
///
/// **Um documento por traço, linear.** E os números que isso implica, que é onde ele deixa de ser
/// uma curiosidade:
///
/// - o teto do app INTEIRO é **3.500 MB** (HR-13) — 24 traços já comem **46%** dele;
/// - a 4096² tudo isto **quadruplica**: 24 traços = ~6,5 GB, quase o dobro do orçamento;
/// - e o cap é por **CONTAGEM** (`DEFAULT_MAX_DEPTH = 300`), então ele **multiplica** isto por 300 em
///   vez de limitá-lo. É a frase exata do ADR-0117 (*"`MAX_HISTORY=64` era um multiplicador, não um
///   teto"*), no Painter, com um zero a mais.
///
/// ⚠️ **`#[ignore]` porque o defeito está ABERTO, não porque a medição é opcional.** Ele fica com o
/// `assert` de pé para que promovê-lo seja apagar uma linha no dia em que a cura entrar; um repro
/// vermelho comentado é um repro que ninguém volta a rodar.
///
/// ⚠️ **A CURA NÃO É MECÂNICA E É DECISÃO DO ENIO.** Há duas, e as duas custam:
///
/// 1. **cap em BYTES** (o que o ADR-0117 fez) — resolve o teto imediatamente e **encurta o undo de
///    forma visível**: com um orçamento de 512 MB o artista tem 8 passos a 2048² e 2 a 4096². É uma
///    regressão de PRODUTO, não um detalhe de implementação;
/// 2. **histórico por DELTA** (o U1 do plano 26) — o passo guarda só a região que mudou, e aí o cap
///    em bytes deixa de morder. É re-arquitetura do undo do Painter, que é a coisa em que o artista
///    mais confia, e o plano já a marca 🔴.
#[test]
#[ignore = "RED: repro de um defeito ABERTO (1.627 MB em 24 tracos) — a cura e decisao de produto"]
fn twenty_four_impasto_strokes_do_not_retain_twenty_four_documents() {
    // Os quatro planos canvas-shaped de UMA camada tocada, a esta resolução.
    let px = f64::from(N) * f64::from(N);
    let one_doc = (px * 4.0 + px * 4.0 + px + px * 7.0) / MB; // rgba + heights(f32) + covers + mats
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
    t.set_brush_size_px(24.0);
    for k in 0..STROKES {
        #[allow(clippy::cast_precision_loss)]
        let y = 200.0 + (k as f32) * 40.0;
        t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
        t.on_canvas_pointer(cp([1800.0, y], PointerPhase::Move));
        t.on_canvas_pointer(cp([1800.0, y], PointerPhase::Up));
    }
    let after = dhat::HeapStats::get();
    drop(profiler);
    #[allow(clippy::cast_precision_loss)]
    let peak = after.max_bytes as f64 / MB;
    #[allow(clippy::cast_precision_loss)]
    let held = (after.curr_bytes as f64 - before.curr_bytes as f64) / MB;

    println!(
        "[undo-mem] {N}x{N} · {STROKES} tracos com impasto\n\
         [undo-mem]   um documento (4 planos) = {one_doc:.1} MB\n\
         [undo-mem]   pico             {peak:.1} MB  ({:.1} documentos)\n\
         [undo-mem]   retido no fim    {held:.1} MB  ({:.1} documentos)",
        peak / one_doc,
        held / one_doc,
    );

    // A propriedade ESTRUTURAL: o histórico é *o documento + um punhado de planos em construção +
    // deltas*, NÃO um documento por traço. A barra é generosa de propósito — ela existe para pegar
    // o crescimento LINEAR em traços, que é o modo de falha, e não para afinar um número.
    #[allow(clippy::cast_precision_loss)]
    let bar = 4.0 * one_doc + (STROKES as f64) * 0.5;
    assert!(
        held < bar,
        "o historico retem {held:.1} MB depois de {STROKES} tracos ({:.1} documentos) — a barra \
         estrutural e {bar:.1} MB. Isto e crescimento por TRACO, e o cap por CONTAGEM \
         (DEFAULT_MAX_DEPTH = 300) e um multiplicador dele, nao um teto.",
        held / one_doc,
    );
}
