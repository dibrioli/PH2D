//! **O CARIMBO da aquarela é função da PEGADA, ou do estado do CANVAS?**
//!
//! Irmão do [`super::measure_watercolor_cost`], e o corte é de responsabilidade: lá se mede *o que uma
//! FEATURE cobra* por move (ablação por knob); aqui *de que o carimbo é FUNÇÃO* — a pergunta que decide
//! se há defeito, e que nenhuma ablação por knob pode responder (ela varia a receita, não o estado).
//!
//! ## Por que a pergunta existe
//!
//! No log do artista (`PH2D_PAINT_PERF`, sessão de 2026-08-04) a linha `AQUARELA` mostra o `carimbo`
//! subindo **1,45 → 2,17 → 2,47 → 2,88 → 3,46 → 3,68 ms** ao longo de sete janelas, e ele é o **maior
//! item por quadro** do meio (~3,3 entregas/quadro × 3,46 = **11,3 ms**, contra 4,4 do composite e 6,5
//! do véu). Duas leituras cabem no log e pedem curas OPOSTAS:
//!
//! * **o artista** engrossou o pincel / andou mais rápido ⇒ mais dabs por lote, e não há nada a
//!   consertar — o carimbo cobra pelo que carimba;
//! * **o produto** tem, dentro de uma operação limitada pela pegada, um passe que segue o estado do
//!   canvas ⇒ é a família de defeito que este repo já curou cinco vezes (o fold da luz §4.8.2, o
//!   `make_mut` do Wet Paint §5.12, o clone do Smudge §5.73).
//!
//! ⚠️ **A leitura do código não decide.** `accumulate_wet_coverage` e `accumulate_wet_color` são laços
//! por-dab em `x0..x1 / y0..y1` — limitados pela pegada *por construção* —, e `wet_splat_gates` é
//! `Arc::clone`. Mas "eu li e não vi" é uma afirmação sobre onde eu OLHEI, e a §0 do CLAUDE.md pede o
//! número. Esta sonda o dá.
//!
//! ## O desenho, e a armadilha que ele evita
//!
//! Dez traços de geometria **IDÊNTICA** (mesmo raio, mesmo caminho, mesmo passo ⇒ mesma lista de dabs,
//! conferível na coluna `n`), cada um numa FAIXA própria do canvas — nunca sobre o anterior. Sobrepor
//! mediria o *wet-on-wet*, que é trabalho legítimo e novo; em faixas separadas a pegada de cada traço é
//! papel virgem, então **tudo o que muda entre o 1º e o 10º é o resto do canvas**.
//!
//! ⚠️ **E o CONTROLE é metade da sonda.** Um carimbo plano só é achado se o canvas de fato tiver
//! molhado — senão a fixture não contém o fenômeno e o verde é vácuo (a armadilha que este repo pagou
//! na §5.41 e na §5.47). Por isso a tabela imprime, ao lado, o `composite` e a `secagem`, que **têm** de
//! crescer: são eles que provam que a poça acumulou.
//!
//! ## Como ler
//!
//! | carimbo | controle (composite/secagem) | veredito |
//! |---|---|---|
//! | plano | subindo | o carimbo é da PEGADA — o crescimento do log é o artista |
//! | subindo | subindo | há um passe seguindo o canvas dentro do carimbo — **frente** |
//! | plano | plano | a fixture não molhou; a sonda não diz nada |
//!
//! ## Medido na RTX, 2026-08-04 (re-meça antes de citar)
//!
//! | traço | carimbo | n | composite | secagem | janela Mtx |
//! |---|---|---|---|---|---|
//! | 1 | **0,488** | 43 | 2,402 | 0,045 | 0,08 |
//! | 5 | **0,502** | 43 | 3,386 | 0,144 | 0,16 |
//! | 10 | **0,531** | 43 | 4,877 | 0,314 | 0,25 |
//!
//! **O carimbo é PLANO — 1,01× e 1,09× em duas corridas** — enquanto o controle prova que a poça de
//! fato cresceu: a janela do composite **triplica** (0,08 → 0,25 M), o composite **dobra** (2,03×) e a
//! secagem faz **7,0×**. ⇒ **O crescimento de 1,45 → 3,68 ms no log do artista é o pincel dele**, não
//! um defeito: o carimbo cobra pela pegada, e uma pegada maior custa mais. O que segue o canvas na
//! aquarela é o **composite** e a **secagem**, os dois já mapeados no [doc 32](../../../../../docs/Painter/32_aquarela_o_que_custa_hoje.md).
//!
//! ⚠️ **Um só leitor do `wash_diag`.** O `take()` ZERA, então esta sonda é `#[ignore]` e roda com
//! `--test-threads=1`: um segundo leitor drenaria a janela deste e os dois publicariam pedaços do mesmo
//! traço como se fossem traços.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_whether_the_carimbo -- --ignored --nocapture --test-threads=1`

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um tool de aquarela pela porta do produto, com os defaults do meio.
fn wash(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = ph2d_painter_brush::BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    t.paint.brush_by_mode.fill(b);
    t.set_paint_media(PaintMedia::Watercolor);
    t
}

/// **O carimbo cresce conforme a poça cresce?**
///
/// Dez traços idênticos em faixas separadas de um canvas 4096². A coluna `n` prova que os lotes são os
/// mesmos; o `composite`/`secagem` provam que a poça acumulou; o `carimbo` responde.
#[test]
#[ignore = "measurement, not a gate — run explicitly with --test-threads=1"]
fn measure_whether_the_carimbo_follows_the_canvas_or_the_footprint() {
    const SIZE: u32 = 4096;
    const RADIUS: f32 = 100.0;
    const STROKES: u32 = 10;
    const STEP_PX: f32 = 40.0;
    const MOVES: u32 = 20;
    const DT: f32 = 1.0 / 60.0;

    // Faixas disjuntas: o passo entre elas é > 2·raio, então a pegada de um traço nunca encosta na do
    // anterior — o que muda entre o 1º e o 10º é o RESTO do canvas, nunca o trabalho do próprio dab.
    let band = f64::from(SIZE - 600) as f32 / f64::from(STROKES - 1) as f32;
    assert!(
        band > 2.0 * RADIUS + 60.0,
        "as faixas TÊM de ser disjuntas, senão a sonda mede wet-on-wet e não o que ela diz medir"
    );

    let mut t = wash(SIZE, RADIUS);
    // Drena o que o setup tenha deixado nos contadores: a 1ª linha tem de descrever o 1º traço.
    let _ = crate::wash_diag::take();

    println!(
        "\naquarela: {STROKES} tracos IDENTICOS em faixas disjuntas, canvas {SIZE}², raio {RADIUS:.0}\n"
    );
    println!(
        "{:<7} {:>10} {:>6} {:>12} {:>11} {:>13} {:>12}",
        "traco", "carimbo", "n", "composite", "secagem", "janela Mtx", "ms/traco"
    );
    let mut first_stamp = 0.0f64;
    let mut last_stamp = 0.0f64;
    for k in 0..STROKES {
        let y = 300.0 + band * f64::from(k) as f32;
        let x0 = RADIUS + 20.0;
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
        let _ = t.take_preview_arc();
        for i in 1..=MOVES {
            let x = x0 + STEP_PX * f64::from(i) as f32;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            t.paint_tick(DT);
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp(
            [x0 + STEP_PX * f64::from(MOVES) as f32, y],
            PointerPhase::Up,
        ));
        let _ = t.take_preview_arc();
        let wall = t0.elapsed().as_secs_f64() * 1e3;

        let rd = crate::wash_diag::take();
        if k == 0 {
            first_stamp = rd.stamp.avg_ms;
        }
        last_stamp = rd.stamp.avg_ms;
        println!(
            "{:<7} {:>10.3} {:>6} {:>12.3} {:>11.3} {:>13.2} {:>12.1}",
            k + 1,
            rd.stamp.avg_ms,
            rd.stamp.n,
            rd.composite.avg_ms,
            rd.dry.avg_ms,
            rd.window_px_per_composite / 1.0e6,
            wall,
        );
    }
    let ratio = if first_stamp > 0.0 {
        last_stamp / first_stamp
    } else {
        0.0
    };
    println!("\ncarimbo do 10o / do 1o = {ratio:.2}x  (1.00 = funcao da PEGADA)\n");
}
