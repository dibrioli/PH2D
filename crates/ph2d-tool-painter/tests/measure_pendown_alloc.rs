//! **QUANTA MEMÓRIA UM PEN-DOWN PEDE** — a sonda que atribui o custo do pen-down do filme.
//!
//! O relógio já disse a FORMA (`measure_pendown_cost::is_the_film_pen_down_the_planes_or_the_dab`):
//! o custo que o filme acrescenta ao pen-down é **plano no raio** (0,62× e 1,90× entre r10 e r100,
//! onde uma pegada preveria 100×) e **plano na tela** — logo não é o primeiro dab nem a cópia do
//! canvas, é **setup por gesto**. O que ele não diz é QUAL setup.
//!
//! ⚠️ **Esta pergunta não se responde com um relógio, e a tentativa de o fazer produziu o número mais
//! instável desta jornada:** as mesmas cinco alocações medidas em sequência deram **0,008 · 0,028 ·
//! 7,586 ms** — três ordens de grandeza, porque o custo de `alloc_zeroed` depende de o alocador ter
//! páginas zeradas do SO na mão ou de ter de as fabricar. Um número desses não sustenta uma
//! atribuição.
//!
//! **Uma CONTAGEM sustenta.** O `dhat` conta bytes pedidos, e bytes pedidos não flutuam com a carga da
//! máquina — é o mesmo raciocínio do `measure_undo_memory` (ADR-0117: *quem declara um budget possui um
//! gate que MEDE*) e da própria auditoria deste módulo, cujo número central (as 4,00 amostragens por
//! texel) é uma contagem.
//!
//! Um `#[test]` por binário, de propósito: os contadores do `dhat` são globais do processo.
//!
//! ```text
//! cargo test -p ph2d-tool-painter --release --test measure_pendown_alloc -- --nocapture
//! ```

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::TextureKind;
use ph2d_tool_painter::PainterTool;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

const MB: f64 = 1_048_576.0;
const N: u32 = 2048;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// **Um traço do filme NÃO pede mais os cinco planos da TELA** — o gate da cura.
///
/// Nasceu VERMELHO com **83,0 MB por traço** a 2048², dos quais **56,0 eram os cinco planos**
/// (`4+4+1+1+4 = 14 B/px`): o `reset_stroke_height` fazia `clear()` — que preserva a capacidade — e o
/// primeiro dab do traço seguinte jogava-a fora numa linha (`h.len() != n ⇒ h = vec![0.0; n]`). Duas
/// linhas discordando sobre o mesmo buffer. Com os planos a CIRCULAR: **36,4 MB**.
///
/// ⚠️ **A asserção é sobre a INCLINAÇÃO — quanto custa *mais um traço*** —, não sobre o total: o
/// primeiro traço de um documento aloca o que for lazy e isso acontece uma vez, não por traço. É a
/// mesma régua do `measure_undo_memory`, e é ela que isola o que é constante da sessão.
///
/// ⚠️ **E o que sobra (36,4 MB) NÃO é resíduo desta cura:** é o fork do canvas, a janela do journal e os
/// buffers por-batch. Nomeá-lo aqui é o que impede a próxima leitura de o atribuir aos planos.
#[test]
fn a_film_stroke_no_longer_asks_for_five_canvas_planes() {
    let px = f64::from(N) * f64::from(N);
    let planes = px * 14.0 / MB; // f32 + f32 + u8 + u8 + f32

    let profiler = dhat::Profiler::builder().testing().build();

    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_brush_shape_kind(TextureKind::Stripes as u8);
    t.set_shape_relief(1.0);
    t.set_brush_size_px(40.0);

    // ⚠️ **POR FASE, e não por traço:** um número por traço não distingue quem PEDE. As três fases de
    // um gesto fazem coisas diferentes (o Down arma, o Move carimba, o Up COMITA), e o total esconde
    // isso — foi por medir o total que a atribuição do pen-down passou meses a apontar para a cópia do
    // canvas.
    let mut phase = [0u64; 3];
    let stroke = |t: &mut PainterTool, k: u8, phase: &mut [u64; 3]| {
        let y = 300.0 + f32::from(k) * 8.0;
        let mut at = |t: &mut PainterTool, x: f32, p: PointerPhase, slot: usize| {
            let a = dhat::HeapStats::get().total_bytes;
            t.on_canvas_pointer(cp([x, y], p));
            phase[slot] += dhat::HeapStats::get().total_bytes - a;
        };
        at(t, 400.0, PointerPhase::Down, 0);
        at(t, 440.0, PointerPhase::Move, 1);
        at(t, 480.0, PointerPhase::Up, 2);
    };
    // Aquece: o 1º traço aloca o que for lazy uma vez por documento e não é o regime.
    stroke(&mut t, 0, &mut [0; 3]);
    let before = dhat::HeapStats::get();
    const K: u8 = 6;
    for k in 1..=K {
        stroke(&mut t, k, &mut phase);
    }
    let after = dhat::HeapStats::get();

    let per_stroke = (after.total_bytes - before.total_bytes) as f64 / f64::from(K) / MB;
    let mb = |b: u64| b as f64 / f64::from(K) / MB;
    println!(
        "[pendown-alloc] {N}^2 filme: {per_stroke:.1} MB pedidos por traco (os cinco planos medem {planes:.1} MB; eram 83,0 antes do pool)"
    );
    println!(
        "[pendown-alloc] por fase: down {:.1} | move {:.1} | UP {:.1} MB",
        mb(phase[0]),
        mb(phase[1]),
        mb(phase[2])
    );
    drop(profiler);

    // CONTROLE: a sonda mediu ALGUMA coisa. Um zero aqui seria a fixture a não pintar, e a asserção
    // abaixo passaria por vácuo.
    assert!(
        per_stroke > 1.0,
        "a sonda nao mediu um traco ({per_stroke:.1} MB) — a fixture nao pintou"
    );
    assert!(
        per_stroke < planes * 0.9,
        "os cinco planos voltaram a ser pedidos por traco: {per_stroke:.1} MB contra {planes:.1} deles"
    );
}
