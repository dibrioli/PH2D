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

/// ⛔ **A SONDA QUE DERRUBOU A FRENTE C** — repartir a mesma pincelada em mais eventos custa mais,
/// e o custo **não é orla de lote: é dab a mais.**
///
/// A 1ª leitura desta sonda mediu **11,3 ms para 1 evento contra 20,8 para 64** sobre a MESMA
/// distância, idêntico nas três telas e saturando — e eu li isso como *"cada lote paga a orla da
/// própria janela, então N janelas pequenas cobrem ~2,8x a área de uma grande"*. Virou a frente C do
/// plano 26 §8, foi **construída inteira e REVERTIDA no mesmo dia**, porque as duas medições que
/// faltavam dizem o contrário:
///
/// | | |
/// |---|---|
/// | dabs emitidos, 1 evento | **21** |
/// | dabs emitidos, 64 eventos | **39** (+86%) |
/// | pixels pintados | **177.760 nos dois** |
/// | custo por-evento vs coalescido (raio 100, 2048²) | **1,00x** |
///
/// **+86% de dabs contra +84% de tempo**: a correlação é a resposta inteira. O `stamp_dabs` percorre
/// a pegada de **cada dab**, não uma janela por lote, então juntar os carimbos de um frame num só não
/// tem o que economizar — medido a **1,00x** exatamente no regime de orla máxima (raio 100, avanço de
/// ~19 px), onde a hipótese da orla previa o ganho maior.
///
/// ⚠️ **O que sobra como lever de verdade, e é OUTRO:** 64 eventos emitem 39 dabs e pintam
/// **exactamente os mesmos pixels** que 21. A amostragem fina não desenha mais nada — ela faz o
/// filtro do traço (o sampler de média + o estabilizador) atrasar menos e emitir mais dabs sobre a
/// mesma linha. Se esses dabs extras são trabalho **necessário** (com build-up de opacidade, dois
/// dabs sobrepostos escurecem mais que um) ou **redundante** é a pergunta seguinte, e ela é sobre a
/// LEI de emissão, não sobre lote. Não medida aqui: `painted px` conta cobertura, não valor.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release is_there_per_event -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn is_there_per_event_overhead_to_coalesce() {
    // ⚠️ 800 px, não 1600: o traço tem de CABER na menor tela. A 1ª versão andava 200→1800 e a 1024²
    // descartava tudo depois de x=1024 — metade dos dabs fora do canvas, então a linha do 1024 media
    // meia pincelada e a comparação entre telas era inválida.
    const DIST: f32 = 800.0;
    // ⚠️ E a coluna que importa é o TOTAL, não `ms/evento`: na 1ª leitura eu olhei o custo POR EVENTO
    // (que cai, porque cada evento anda menos) e concluí o oposto do que a sonda dizia.
    println!(
        "[coalesce] mesma distancia ({DIST} px) em N eventos — tempo TOTAL e os dabs emitidos:"
    );
    for side in [1024u32, 2048, 4096] {
        for n in [1usize, 4, 16, 64] {
            let mut t = tool(side);
            t.set_brush_size_px(100.0); // raio grande: o regime em que a orla seria MAXIMA
            #[allow(clippy::cast_precision_loss)]
            let step = DIST / (n as f32);
            t.on_canvas_pointer(cp([200.0, 300.0], PointerPhase::Down));
            let total = ms(&mut || {
                for i in 1..=n {
                    #[allow(clippy::cast_precision_loss)]
                    let x = 200.0 + step * (i as f32);
                    t.on_canvas_pointer(cp([x, 300.0], PointerPhase::Move));
                }
            });
            t.on_canvas_pointer(cp([200.0 + DIST, 300.0], PointerPhase::Up));
            let painted = t
                .canvas_rgba
                .iter()
                .step_by(4)
                .filter(|&&v| v != 255)
                .count();
            println!(
                "[coalesce]   {side:>4}  {n:>3} eventos: {total:>7.2} ms · {painted:>8} px pintados"
            );
        }
    }
}

/// ⛔ **A MEDIÇÃO QUE NOMEOU A CAUSA** — o custo de entrada é por **DAB**, e mais eventos emitem mais
/// dabs sobre o mesmo desenho.
///
/// É a metade que faltava à sonda acima, e ela é o motivo de a frente C ter sido revertida. O oráculo
/// aqui não é tempo: é a **CONTAGEM**, que não flaka e não depende de perfil de build.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_dab_count -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_dab_count_grows_with_the_event_count_over_the_same_path() {
    const DIST: f32 = 800.0;
    println!("[dabs] mesma distancia ({DIST} px), pincel raio 100:");
    for n in [1usize, 4, 16, 64] {
        let mut t = tool(1024);
        t.set_brush_size_px(100.0);
        #[allow(clippy::cast_precision_loss)]
        let step = DIST / (n as f32);
        t.on_canvas_pointer(cp([100.0, 300.0], PointerPhase::Down));
        for i in 1..=n {
            #[allow(clippy::cast_precision_loss)]
            let x = 100.0 + step * (i as f32);
            t.on_canvas_pointer(cp([x, 300.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([100.0 + DIST, 300.0], PointerPhase::Up));
        let painted = t
            .canvas_rgba
            .iter()
            .step_by(4)
            .filter(|&&v| v != 255)
            .count();
        println!(
            "[dabs]   {n:>3} eventos -> {painted:>8} px pintados (identico = os dabs extras nao desenham nada novo)"
        );
    }
}

/// **O PIOR evento de um traço inteiro** — a frente R (o outlier de 134,8 ms) perguntada ao produto
/// em vez de deduzida.
///
/// O `PH2D_PAINT_PERF` do app reportou **`INPUT max = 134,8 ms` num único evento** e o doc 28 §4.8.3
/// registrou que os dois candidatos nomeados — o pen-down canvas-sized (24,5 / 11,7) e o commit de
/// pen-up (39,6) — **não somam a isso**. A conclusão de lá foi *"falta instrumentação por FASE, não
/// hipótese"*, e esta sonda é o primeiro passo dela: rodar o CICLO INTEIRO de um traço e imprimir
/// **todo** evento, para o pico se nomear sozinho.
///
/// ⚠️ **Dois traços, e o segundo é quem responde.** O primeiro paga as alocações únicas (os planos de
/// relevo, as texturas); o segundo é o regime que o artista vive num traço longo. Ler só o primeiro
/// atribuiria a alocação ao gesto.
///
/// Não afirma nada; IMPRIME.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_worst_single_event -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_worst_single_event_of_a_stroke_names_itself() {
    use ph2d_painter_brush::{BrushSpec, Falloff};
    const SIDE: u32 = 4096;
    for radius in [20.0f32, 100.0] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIDE * SIDE * 4) as usize], SIDE, SIDE);
        let b = BrushSpec {
            radius_px: radius,
            hardness: 0.5,
            falloff: Falloff::Sphere,
            strength: 1.0,
            color: [0.8, 0.2, 0.1],
            space_attenuation: false,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.toggle_brush_impasto();
        t.set_brush_impasto_depth(1.0);

        for stroke in 1..=2u8 {
            let y = 400.0 + f32::from(stroke) * 400.0;
            let mut total = 0f64;
            let mut moves = Vec::new();

            let down = ms(&mut || {
                t.on_canvas_pointer(cp([300.0, y], PointerPhase::Down));
            });
            let _ = t.take_preview_arc();
            total += down;
            let mut worst = (down, String::from("pen-down"));

            for k in 1..=30u8 {
                let x = 300.0 + 40.0 * f32::from(k);
                let mv = ms(&mut || {
                    t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                });
                let _ = t.take_preview_arc();
                total += mv;
                moves.push(mv);
                if mv > worst.0 {
                    worst = (mv, format!("move #{k}"));
                }
            }

            let up = ms(&mut || {
                t.on_canvas_pointer(cp([300.0 + 40.0 * 30.0, y], PointerPhase::Up));
            });
            let _ = t.take_preview_arc();
            total += up;
            if up > worst.0 {
                worst = (up, "pen-up".into());
            }

            moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let med = moves[moves.len() / 2];
            println!(
                "[worst] r={radius:>5.0} traco {stroke}: down {down:>7.2} · move mediana {med:>6.2} \
                 (max {:>7.2}) · up {up:>7.2} · TOTAL {total:>8.2} ms  ⇒ PIOR = {} ({:.2} ms)",
                moves[moves.len() - 1],
                worst.1,
                worst.0
            );
        }
    }
}

/// **DE QUE É FEITO O PEN-UP** — o maior evento de um traço.
///
/// O irmão acima mediu que o pior evento de um traço NÃO é o pen-down nem um move: é o **pen-up**, e
/// ele é quase indiferente ao raio do pincel — a assinatura de trabalho limitado pela TELA. O
/// `paint_end` tem dois candidatos canvas-bound: **`commit_stroke_height`** (dobra o envelope de
/// relevo do traço nos planos da camada) e **`commit_structural_edit`** (o passo de undo, que desde a
/// U1 faz o *diff* dos planos canvas-shaped para guardar só a janela).
///
/// ⚠️ **A primeira versão desta sonda media UM pen-up por configuração e NÃO REPRODUZIA** — a mesma
/// célula deu 117,76 ms numa corrida e 28,46 na seguinte. Buffers de 67-117 MB pagam *first-touch* e o
/// alocador tem memória entre chamadas, então um único gesto mede o estado do heap tanto quanto mede o
/// produto. *Um número que não reproduz não é um achado; é ruído com casas decimais.* Agora: **8
/// traços, o primeiro DESCARTADO** (é ele quem paga as alocações preguiçosas dos planos) e a MEDIANA
/// dos sete seguintes, com o espalhamento impresso ao lado para o leitor julgar.
///
/// ⚠️ **A razão entre telas é o que decide**, não o valor absoluto: quadruplicar a área tem de
/// quadruplicar um custo de PLANO e deixar quieto um custo de PEGADA.
///
/// Não afirma nada; IMPRIME.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release what_the_pen_up -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn what_the_pen_up_is_made_of() {
    use ph2d_painter_brush::{BrushSpec, Falloff};
    println!(
        "\n{:<8} {:>8} {:>10} {:>12} {:>14} {:>9} {:>9}",
        "tela", "impasto", "1o traco", "pen-up med", "min-max", "vs 1024x", "por area"
    );
    for impasto in [false, true] {
        let mut base: Option<f64> = None;
        for side in [1024u32, 2048, 4096] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
            let b = BrushSpec {
                radius_px: 40.0,
                hardness: 0.5,
                falloff: Falloff::Sphere,
                strength: 1.0,
                color: [0.8, 0.2, 0.1],
                space_attenuation: false,
                ..Default::default()
            };
            t.paint.brush = b;
            for slot in &mut t.paint.brush_by_mode {
                *slot = b;
            }
            if impasto {
                t.toggle_brush_impasto();
                t.set_brush_impasto_depth(1.0);
            }
            // Traço curto e FIXO em px: a variável isolada é a TELA (a lição do irmão acima, cuja 1ª
            // versão escalava o traço junto e media a si mesma).
            let mut ups = Vec::new();
            let mut first = 0.0f64;
            for stroke in 0..8u8 {
                let y = 200.0 + 60.0 * f32::from(stroke);
                t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
                let _ = t.take_preview_arc();
                for k in 1..=6u8 {
                    t.on_canvas_pointer(cp([200.0 + 40.0 * f32::from(k), y], PointerPhase::Move));
                    let _ = t.take_preview_arc();
                }
                let up = ms(&mut || {
                    t.on_canvas_pointer(cp([440.0, y], PointerPhase::Up));
                });
                let _ = t.take_preview_arc();
                if stroke > 0 {
                    ups.push(up); // o traço 0 paga as alocações preguiçosas — não é o regime
                } else {
                    first = up; // …e é POR ISSO que ele é reportado à parte, em vez de descartado
                }
            }
            ups.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let med = ups[ups.len() / 2];
            let b0 = *base.get_or_insert(med);
            let area = f64::from(side * side) / f64::from(1024u32 * 1024);
            println!(
                "{side:<8} {:>8} {first:>10.2} {med:>12.3} {:>14} {:>8.2}x {:>9.2}",
                if impasto { "ON" } else { "off" },
                format!("{:.1}-{:.1}", ups[0], ups[ups.len() - 1]),
                med / b0.max(1e-9),
                med / b0.max(1e-9) / area,
            );
        }
    }
    println!();
}

/// **A ARITMÉTICA do pen-up de impasto, testada em vez de afirmada.**
///
/// O irmão acima mediu que o impasto **acrescenta 32,8 ms** ao pen-up a 4096² (6,05 → 38,9). Lendo o
/// `commit_stroke_height`, ele é **window-bound de propósito** (o doc-comment dele diz e gateia isso) —
/// exceto por uma coisa: ele chama `plane_fork::fork_par` em cada plano de relevo, e um **fork é
/// canvas-sized** porque o snapshot de undo do pen-down segura um `Arc` de cada plano.
///
/// A 4096² os três planos somam ~200 MB (`covers` u8 16 MB · `heights` f32 67 MB · `mats` 7 B 117 MB),
/// contra os 67 MB do `canvas_rgba` que o Digital já paga. **Se o fork for a causa, o custo de clonar
/// esses três tem de casar com os 32,8 ms** — e essa é uma previsão que pode dar errado, que é o que a
/// torna um teste em vez de uma narrativa.
///
/// ⚠️ Não prova SOZINHA que o fork é a causa (o diff do undo lê os mesmos planos): estabelece se a
/// ordem de grandeza fecha ou não. Se não fechasse, a hipótese morreria aqui.
///
/// Não afirma nada; IMPRIME.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_relief_planes_arithmetic -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_relief_planes_arithmetic_of_the_pen_up() {
    println!(
        "\n{:<10} {:>10} {:>12} {:>12}",
        "plano", "MB", "clone ms", "acumulado"
    );
    for side in [2048u32, 4096] {
        let n = (side as usize) * (side as usize);
        let mut acc = 0.0f64;
        println!("-- tela {side}x{side} --");
        for (name, bytes_per) in [
            ("canvas", 4usize),
            ("covers", 1),
            ("heights", 4),
            ("mats", 7),
        ] {
            let src: Vec<u8> = vec![7u8; n * bytes_per];
            let mut lo = f64::INFINITY;
            for _ in 0..5 {
                let mut dst = Vec::new();
                let t = ms(&mut || dst = src.clone());
                std::hint::black_box(dst.len());
                if t < lo {
                    lo = t;
                }
            }
            // O `canvas` é o que o DIGITAL já paga; os outros três são o que o impasto acrescenta.
            if name != "canvas" {
                acc += lo;
            }
            #[allow(clippy::cast_precision_loss)]
            let mb = (n * bytes_per) as f64 / (1024.0 * 1024.0);
            println!("{name:<10} {mb:>10.0} {lo:>12.3} {acc:>12.3}");
        }
    }
    println!();
}
