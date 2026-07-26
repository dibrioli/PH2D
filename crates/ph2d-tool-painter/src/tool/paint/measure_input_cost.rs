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
/// ⚠️ **Isto NASCEU como "o mesmo defeito que `measure_undo_memory` mede pelo outro lado", e a U1
/// provou que não é** — a nota que estava aqui dizia *"a cura é a mesma, e é a frente U1"*, e ela está
/// ERRADA. A U1 landou (2026-07-26): o histórico caiu de 1.627 para 242 MB e **este número não se
/// moveu** (16,49 → 16,07 ms a 4096²).
///
/// O raciocínio que falhou é sutil e vale escrito: guardar só a região no HISTÓRICO é uma decisão sobre
/// o que sobra **depois** do traço. O que força a cópia é precisar do estado anterior **durante** ele —
/// e enquanto QUALQUER segunda referência ao canvas existir (o snapshot do pen-down, o cursor do
/// histórico, o `base` de uma sessão de proteção), o primeiro dab paga um `Arc::make_mut`. Tirar uma das
/// referências não ajuda: basta uma.
///
/// ## Medido (2026-07-25, re-medido em 2026-07-26 com a U1 no lugar)
///
/// | tela | copiar o canvas | **pen-down medido (sem impasto)** |
/// |---|---|---|
/// | 1024² | 0,55 ms | **1,20** |
/// | 2048² | 2,64 ms | ~3,2 |
/// | 4096² | **12,35 ms** | **16,07** |
///
/// **O pen-down É a cópia do canvas**, dentro do ruído. E o move é PLANO na tela (0,97 ms a 1024² e a
/// 4096²) — trabalho honesto por dab, não um defeito.
///
/// ⚠️ **E a DECOMPOSIÇÃO refuta metade da receita escrita.** A §13.12.5 do doc 25 prescrevia *"semeadura
/// lazy por TILE **+ reuso da alocação** (mata o page-fault)"*. Medido: com o buffer já mapeado a cópia
/// custa 11,68 dos 12,35 ms ⇒ **a alocação vale 5%**. O page-fault não é o alvo; o memcpy é. Sobra a
/// outra metade da receita, e ela é a única: **capturar o "antes" por REGIÃO, sob demanda**, na primeira
/// escrita de cada tile — o *tile-based undo* do GIMP/Krita. Isso quer uma porta única de escrita de
/// canvas, e hoje há ~25 sítios chamando `Arc::make_mut` direto: é wave própria, com gates próprios.
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
        // …e a DECOMPOSIÇÃO: quanto do fork é a ALOCAÇÃO (page-faults do kernel mapeando 64 MB novos) e
        // quanto é o memcpy. É o que decide se a cura é um pool de buffers ou uma captura por tile —
        // a receita da §13.12.5 nomeia as duas e nunca as separou num número.
        let mut warm: Vec<u8> = vec![0u8; n];
        let copy_only = ms(&mut || warm.copy_from_slice(&src));
        println!(
            "[fork]   … so o memcpy (buffer ja mapeado): {copy_only:.2} ms  \
             ⇒ a ALOCACAO custa {:.2} ms ({:.0}%)",
            cost - copy_only,
            100.0 * (cost - copy_only) / cost
        );
        std::hint::black_box(warm.len());
    }
}

/// **O que o histórico por delta CUSTA em tempo** — a outra metade do trade que a wave U1 fez.
///
/// Guardar a janela em vez do documento troca memória por trabalho na hora de DESFAZER: o endpoint é
/// reconstruído clonando o plano do cursor e escrevendo a janela por cima, onde antes era um
/// `Arc::clone` grátis. Um undo é user-paced (não é um frame de 60 fps), mas *"user-paced"* não é
/// desculpa para não medir — o ADR-0117 trocou 4351 por 156 MB **e** publicou o custo.
///
/// Não afirma nada; IMPRIME. O gate de razão que protege o produto é o irmão `the_input_cost_*`.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_delta_history_costs_this_much_to_undo() {
    for side in [1024u32, 2048, 4096] {
        let mut t = tool(side);
        t.toggle_brush_impasto();
        t.set_brush_size_px(24.0);
        #[allow(clippy::cast_precision_loss)]
        let x1 = (side as f32) - 100.0;
        for k in 0..4 {
            #[allow(clippy::cast_precision_loss)]
            let y = 100.0 + (k as f32) * 40.0;
            t.on_canvas_pointer(cp([100.0, y], PointerPhase::Down));
            t.on_canvas_pointer(cp([x1, y], PointerPhase::Move));
            t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
        }
        let undo = ms(&mut || {
            t.undo_last();
        });
        let redo = ms(&mut || {
            t.redo_last();
        });
        #[allow(clippy::cast_precision_loss)]
        let held = t.undo_retained_bytes() as f64 / 1_048_576.0;
        println!(
            "[undo-cost] {side}x{side}: undo {undo:.2} ms · redo {redo:.2} ms  (retido {held:.1} MB)"
        );
    }
}

/// **O defeito do pen-down SEGUE ABERTO, e este é o número dele.**
///
/// Um gate, não uma medição impressa — no molde do
/// `the_documented_hardening_is_still_there_and_this_is_its_number` (§13.10): quando a wave da captura
/// por região chegar, é este teste que vira vermelho e diz que ela funcionou.
///
/// ⚠️ **O oráculo é a RAZÃO contra a CÓPIA DO CANVAS medida no mesmo instante**, e a primeira versão
/// errou isso: ela comparava o pen-down a 1024² com o de 4096², que são dois instantes diferentes — sob
/// a carga da suíte completa os dois flutuam de forma independente e o gate **flakou na 1ª rodada**.
/// Um gate que flaka é pior que ausente. Medidos juntos, os dois números sobem e descem juntos, e a
/// razão entre eles é exatamente a afirmação que interessa: **o pen-down É a cópia do canvas**.
#[test]
fn the_pen_down_is_still_a_canvas_copy_and_this_is_its_number() {
    const SIDE: u32 = 2048;
    let n = (SIDE as usize) * (SIDE as usize) * 4;
    let mut t = tool(SIDE);
    let down = ms(&mut || {
        t.on_canvas_pointer(cp([100.0, 300.0], PointerPhase::Down));
    });
    let src: Vec<u8> = vec![7u8; n];
    let mut dst = Vec::new();
    let copy = ms(&mut || dst = src.clone());
    std::hint::black_box(dst.len());
    let ratio = down / copy.max(f64::MIN_POSITIVE);
    println!(
        "[pen-down] {SIDE}: pen-down {down:.2} ms · copiar o canvas {copy:.2} ms · {ratio:.2}x"
    );
    assert!(
        ratio > 0.5,
        "o pen-down ({down:.2} ms) deixou de ter a ordem de grandeza de uma copia do canvas \
         ({copy:.2} ms) — se isto foi a wave da captura por REGIAO, apague este gate e escreva o irmao \
         que afirma o contrario"
    );
}
