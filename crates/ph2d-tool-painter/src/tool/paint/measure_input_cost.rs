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

/// **O TETO: quanto custa um texel de dab — e o impasto custa 19x o digital.**
///
/// A pergunta que decide se há melhoria a colher, e ela não tinha número. Medido a 2048², traço de
/// 600 px:
///
/// | raio | impasto | ns/texel | ms/traço |
/// |---|---|---|---|
/// | 100 | **off** | **9,6** | 9,0 |
/// | 100 | **ON** | **182,8** | **172,2** |
/// | 20 | off | 55,5 | 10,5 |
/// | 20 | ON | 238,9 | 45,0 |
///
/// **Duas leituras, e as duas são sobre o MECANISMO** (a razão entre ligado e desligado usa a mesma
/// contagem de dabs, então ela não depende da derivação analítica de `dabs`):
///
/// 1. **O impasto custa 19x o digital por texel** a raio 100 (4,3x a raio 20). Ele escreve 12 bytes a
///    mais por texel (`heights` f32 + `covers` u8 + `mats` 7xu8) contra os 4 do RGBA, ou seja **4x o
///    tráfego de memória** — e a distância entre 4x e 19x é a **folga candidata**. Um traço de raio
///    100 com impasto custa **172 ms**; o mesmo traço digital custa **9**.
/// 2. **O pincel digital a raio 100 está em 9,6 ns/texel** — eficiente. A raio 20 sobe para 55,5,
///    porque o custo FIXO por dab é amortizado sobre 25x menos texels: pincel pequeno é dominado por
///    overhead de dab, pincel grande por trabalho de texel. **São dois regimes, e uma frente que ataca
///    um não ajuda o outro.**
///
/// ⚠️ **O que este número NÃO diz, e a coluna `x-piso` é a armadilha:** o piso medido é um
/// `d = s + 1` byte a byte, que o compilador vetoriza — é um piso de **largura de banda**, não de
/// **trabalho**. Um dab legitimamente faz falloff, silhueta, grain, jitter, blend, gate de proteção,
/// envelope de altura, cápsula, material e simetria por texel. Então `3154x o piso` **não é uma
/// afirmação de desperdício**; a única comparação honesta aqui é impasto contra digital, porque as duas
/// rotas fazem o mesmo trabalho de pigmento e diferem exactamente no relevo.
///
/// ⚠️ **E a frente que sai disto ainda NÃO está justificada** — é a lição da frente C revertida (§8 do
/// plano 26), tomada a sério: eu tenho a razão 19x, e **não** tenho a decomposição de para onde ela vai
/// dentro do dab de impasto. A próxima medição é essa, e ela é barata: cronometrar as passadas do
/// depósito de altura separadamente (o envelope, a cápsula, o `settle`, o banco do Push) sobre a MESMA
/// pegada. Sem ela, "otimizar o impasto" é a mesma frase que "coalescer os eventos" era ontem.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_per_texel -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_per_texel_cost_of_a_dab_against_the_hardware_floor() {
    const DIST: f32 = 600.0;
    println!("[texel] raio  impasto   ms/traco  dabs   ns/texel   x-piso");
    for radius in [20.0f32, 100.0] {
        for impasto in [false, true] {
            let mut t = tool(2048);
            if !impasto {
                t.toggle_brush_impasto(); // `tool()` o liga; aqui desligamos p/ o digital puro
            }
            t.set_brush_size_px(radius * 2.0);
            // Aquece (o 1o traco paga a alocacao dos planos por-traco).
            for _ in 0..2 {
                t.on_canvas_pointer(cp([200.0, 300.0], PointerPhase::Down));
                for i in 1..=24u8 {
                    let x = 200.0 + DIST / 24.0 * f32::from(i);
                    t.on_canvas_pointer(cp([x, 300.0], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([200.0 + DIST, 300.0], PointerPhase::Up));
            }
            let ms_stroke = ms(&mut || {
                t.on_canvas_pointer(cp([200.0, 700.0], PointerPhase::Down));
                for i in 1..=24u8 {
                    let x = 200.0 + DIST / 24.0 * f32::from(i);
                    t.on_canvas_pointer(cp([x, 700.0], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([200.0 + DIST, 700.0], PointerPhase::Up));
            });
            // Dabs: o espacamento default e fracao do diametro, entao derivamos do arco/spacing real.
            let spacing_px = f64::from(radius * 2.0) * f64::from(t.brush_settings().spacing);
            let dabs = (f64::from(DIST) / spacing_px.max(1.0)).max(1.0);
            let texels = dabs * std::f64::consts::PI * f64::from(radius) * f64::from(radius);
            let ns_per_texel = ms_stroke * 1e6 / texels;
            // PISO: uma passada de leitura+escrita sobre o mesmo numero de texels (4 B/px cada).
            let n = texels as usize;
            let src = vec![7u8; n * 4];
            let mut dst = vec![0u8; n * 4];
            let floor_ms = ms(&mut || {
                for (d, s) in dst.iter_mut().zip(&src) {
                    *d = s.wrapping_add(1);
                }
            });
            let floor_ns = floor_ms * 1e6 / texels;
            println!(
                "[texel] {radius:>4}  {:>7}  {ms_stroke:>8.2}  {dabs:>4.0}  {ns_per_texel:>8.1}  {:>5.1}x",
                if impasto { "ON" } else { "off" },
                ns_per_texel / floor_ns.max(1e-9)
            );
        }
    }
}

/// **A DECOMPOSIÇÃO do 19x: o alvo está LOCALIZADO, o mecanismo NÃO.**
///
/// Medido a 2048², raio 100, traço de 600 px (ms por traço):
///
/// | config | 1º (camada virgem) | 2º (sobre tinta) |
/// |---|---|---|
/// | impasto **OFF** (controle) | 11,6 | 11,6 |
/// | impasto ON, smoothing 1,0 (default) | **130,3** | **162,6** |
/// | impasto ON, smoothing 0,0 | 125,1 | 156,8 |
///
/// **Três leituras, e uma não-leitura:**
///
/// 1. **O `settle` vale 3%** (5 ms de 162). Não é o alvo, e a nota que o suspeitava está corrigida.
/// 2. **A família do bow wave vale ~20%** (162 − 130 = 32 ms) — e ela entra pelo `ground`, que é
///    `Some` sempre que a **CAMADA** tem relevo, não quando o knob Push está alto. O comentário do
///    `impasto.rs` afirma *"o custo cai exactamente onde a feature está, em tinta sobre tinta"*, e a
///    medição CONFIRMA: 130 na camada virgem, 162 a partir do 2º traço.
///    ⚠️ Isto invalida qualquer sonda que aqueça no MESMO lugar onde mede — a minha primeira fazia
///    isso e media o caso tinta-sobre-tinta enquanto o cabeçalho dizia "traço".
/// 3. **O resto — ~118 ms, 73% — é o depósito de altura base**, e é ele o alvo.
/// 4. ⚠️ **O que eu NÃO consegui estabelecer:** *por que* esses 118 ms. Ver o irmão
///    `the_impasto_draw_to_split`, cujo termo "superaditivo" desmanchou quando a sonda ganhou medianas.
///    Nomear um mecanismo aqui seria a terceira hipótese não-medida do dia (§8 do plano 26).
///
/// Levers, todos portas do PRODUTO (nenhuma instrumentação):
/// * `impasto` off -> o controle (pigmento puro, mesma lista de dabs);
/// * **1o traço numa camada virgem** -> `ground = None`, logo o banco/bite/onda **não rodam**;
/// * **2o traço na mesma camada** -> `ground = Some`, e a familia do bow wave entra;
/// * `impasto_smoothing = 0` -> mata o `settle` (o blur que assenta a tinta).
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_impasto_dab_decomposition -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_impasto_dab_decomposition() {
    const DIST: f32 = 600.0;
    fn stroke(t: &mut PainterTool, y: f32) -> f64 {
        ms(&mut || {
            t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
            for i in 1..=24u8 {
                let x = 200.0 + DIST / 24.0 * f32::from(i);
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([200.0 + DIST, y], PointerPhase::Up));
        })
    }
    println!("[decomp] raio 100, 2048², traco de 600 px — ms por traco");
    println!("[decomp] config                              1o(virgem)  2o(sobre tinta)  3o");
    for (name, impasto, smoothing) in [
        ("impasto OFF (controle)", false, 1.0f32),
        ("impasto ON, smoothing 1.0 (default)", true, 1.0),
        ("impasto ON, smoothing 0.0", true, 0.0),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (2048 * 2048 * 4) as usize], 2048, 2048);
        if impasto {
            t.toggle_brush_impasto();
        }
        t.set_brush_size_px(200.0);
        t.paint.brush.impasto_smoothing = smoothing;
        for slot in &mut t.paint.brush_by_mode {
            slot.impasto_smoothing = smoothing;
        }
        // Os TRES na MESMA faixa: o 2o e o 3o encontram o relevo do anterior.
        let a = stroke(&mut t, 400.0);
        let b = stroke(&mut t, 400.0);
        let c = stroke(&mut t, 400.0);
        println!("[decomp] {name:<38} {a:>8.2}  {b:>13.2}  {c:>8.2}");
    }
}

/// **O 2º NÍVEL: dos 118 ms do depósito base, quanto é PIGMENTO e quanto é ALTURA?**
///
/// O `DrawTo` é a porta do produto que separa as duas metades sobre a MESMA pegada e a MESMA lista de
/// dabs: `Color` = pigmento sem corpo, `Depth` = corpo sem pigmento, `ColorAndDepth` = o default.
///
/// Medido a 2048², 1º traço em faixa virgem, **mediana de 5**:
///
/// | raio | Color | Depth | soma | ColorAndDepth | sobra |
/// |---|---|---|---|---|---|
/// | 10 | 6,8 | 24,5 | 31,3 | 32,0 | +2% |
/// | 25 | 10,5 | 33,6 | 44,1 | 47,4 | +8% |
/// | 50 | 20,1 | 45,9 | 66,0 | 72,1 | +9% |
/// | 100 | 9,0 | 111,6 | 120,7 | 160,8 | +33% |
///
/// **O que É robusto:** o depósito de ALTURA custa **2,3× a 12× o de pigmento** sobre a mesma lista de
/// dabs. Ele escreve 12 B/texel (`heights` f32 + `covers` u8 + `mats` 7×u8) contra os 4 do RGBA — 3× os
/// bytes — e faz a **própria varredura** da pegada, separada da varredura da cor.
///
/// ⚠️ **O que NÃO é robusto, e eu quase o publiquei como achado:** a primeira rodada desta sonda dava
/// UMA amostra por config e mostrava um termo **superaditivo de 40%** (as duas metades juntas custando
/// mais que a soma), que eu ia atribuir a **localidade de cache**. Com mediana de 5 e faixas
/// independentes ele **desmancha para 2-9%** nos raios 10-50; só o raio 100 mantém 33%, e ali o `Color`
/// sai **não-monotônico** (9,0 ms a raio 100 contra 20,1 a raio 50, com 2× os texels), o que denuncia a
/// própria medição. **Não há termo superaditivo estabelecido.**
///
/// ## A candidata, e por que ela NÃO é palpite de mecanismo
///
/// Fundir as duas varreduras numa só (silhueta avaliada UMA vez por texel, os 5 planos escritos juntos)
/// não é uma teoria sobre cache — é **remover uma varredura duplicada que se sabe existir**. E há
/// PRECEDENTE medido no próprio `impasto.rs`: o comentário do `PushBite` registra que fazer a mordida
/// *"num kernel de si mesma significava avaliar a silhueta DUAS vezes por texel, e isso sozinho punha o
/// custo do impasto em 5,0 ms/move, além do orçamento"*. O mesmo argumento, um nível acima.
///
/// ⚠️ É refatoração do kernel mais quente do módulo, com **byte-identidade** como gate. Wave própria,
/// e só com ordem do Enio.
///
/// Rodar: `cargo test -p ph2d-tool-painter --release the_impasto_draw_to_split -- --ignored --nocapture`
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_impasto_draw_to_split() {
    use ph2d_painter_brush::height::DrawTo;
    const DIST: f32 = 600.0;
    println!("[split] 2048², 1o traco numa camada VIRGEM (sem bow wave) — ms");
    for radius in [10.0f32, 25.0, 50.0, 100.0] {
        println!("[split] --- raio {radius} ---");
        for (name, draw_to) in [
            ("Color (pigmento sem corpo)", DrawTo::Color),
            ("Depth (corpo sem pigmento)", DrawTo::Depth),
            ("ColorAndDepth (o default)", DrawTo::ColorAndDepth),
        ] {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (2048 * 2048 * 4) as usize], 2048, 2048);
            t.toggle_brush_impasto();
            t.set_brush_size_px(radius * 2.0);
            t.paint.brush.impasto_draw_to = draw_to;
            for slot in &mut t.paint.brush_by_mode {
                slot.impasto_draw_to = draw_to;
            }
            // ⚠️ MEDIANA de 5, e cada traço numa faixa PRÓPRIA: a 1ª versão desta sonda dava UMA amostra
            // por config e o Color a raio 100 saiu ABAIXO do de raio 50 (não-monotônico), o que tornava o
            // termo "superaditivo" indistinguível de ruído. E as faixas têm de ser distintas, senão o 2º
            // traço encontra o relevo do 1º e o bow wave entra na conta (o `ground` é da CAMADA).
            let mut samples = Vec::new();
            for k in 0..5u8 {
                let y = 200.0 + f32::from(k) * 300.0;
                samples.push(ms(&mut || {
                    t.on_canvas_pointer(cp([200.0, y], PointerPhase::Down));
                    for i in 1..=24u8 {
                        let x = 200.0 + DIST / 24.0 * f32::from(i);
                        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                    }
                    t.on_canvas_pointer(cp([200.0 + DIST, y], PointerPhase::Up));
                }));
            }
            samples.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let ms_stroke = samples[samples.len() / 2];
            println!(
                "[split] {name:<30} {ms_stroke:>8.2}   (min {:.2} max {:.2})",
                samples[0],
                samples[samples.len() - 1]
            );
        }
    }
}
