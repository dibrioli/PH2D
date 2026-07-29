//! **De que é feito o TICK do Wet Paint** — irmão de [`super::measure_wetpaint_cost`] (o custo do
//! MOVE), separado por RESPONSABILIDADE quando o pai cruzou o teto de LOC.
//!
//! O corte é o assunto, não o tamanho: lá se pergunta *o que um movimento do mouse custa e quem
//! segura o canvas quando ele acontece*; aqui se pergunta *o que a SIMULAÇÃO custa por frame, e quem
//! governa esse custo*. As duas metades compartilham os helpers do pai (`cp`, `wetted`) via
//! `use super::*` — este módulo é FILHO dele.
//!
//! ## O que esta família mediu, em ordem
//!
//! 1. **O laço de realimentação** (`measure_whether_the_tick_feeds_back_on_a_slow_frame`): o `dt` é o
//!    relógio do frame ANTERIOR, então um frame lento compra mais passos e fica mais lento ainda.
//! 2. **A forma do trabalho** (`..._pays_for_the_water_or_for_its_bounding_box`): a sim varria a
//!    BBOX — o casco da água —, e num traço diagonal o casco é 28% da tela com 2,4% de células vivas.
//! 3. **O eixo que não era** (`..._scales_with_the_brush_radius`): o raio NÃO multiplica o custo — o
//!    `TRAIL_HALF` do engine clipa a janela do traço.
//! 4. **De que o passo é feito** — e a resposta que fechou a frente: `ns/célula` é **PLANO** de 512² a
//!    4096² (`ph2d-wet-paint/tests/measure_density.rs`), logo o custo não é layout nem cache; é
//!    trabalho por célula, e o número de células é o que a mão do artista escolhe. Não há
//!    paralelismo byte-idêntico a colher (o solver é Gauss-Seidel em toda parte — ADR-0134) e não há
//!    cache a consertar ⇒ **o custo por frame tem de ser ORÇADO, não otimizado.**

use super::*;

/// **O QUE CUSTA UM FRAME COM A ÁGUA VIVA** — o repro do smoke do Enio (2026-07-28): *"IMG 4096, uma
/// pincelada grande e molhada, FPS cai para 4"*.
///
/// ⚠️ **As sondas irmãs medem o MOVE, e o move não é o que ele viu.** Um move só acontece enquanto a
/// mão anda; o que derruba o FPS *depois* da pincelada é o **tick** — a sim continua correndo, e ela
/// roda uma vez por frame quer o artista mexa o mouse ou não. Medir o move e concluir sobre o FPS é
/// medir a coisa errada com precisão.
///
/// 4 FPS são **250 ms/frame**. Esta tabela diz quanto disso é o tick, por tela e por raio.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_what_a_frame_of_live_water_costs() {
    const FRAME_MS: f32 = 16.6;
    println!(
        "\n{:<8} {:>7} {:>12} {:>12} {:>12}",
        "tela", "raio", "traco ms", "tick p50 ms", "tick max ms"
    );
    for side in [1024u32, 2048, 4096] {
        for radius in [40.0f32, 100.0, 200.0] {
            let mut t = wetted(side, radius);
            let mid = (side / 2) as f32;
            let x0 = radius + 20.0;
            let x1 = ((side as f32) - radius - 20.0).min(x0 + 400.0);

            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
            let mut x = x0 + 40.0;
            while x <= x1 {
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
                let _ = t.take_preview_arc();
                x += 40.0;
            }
            t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));
            let stroke_ms = t0.elapsed().as_secs_f64() * 1e3;

            // A água viva: o que o app paga POR FRAME depois de soltar.
            let mut ticks = Vec::new();
            let mut areas = Vec::new();
            for _ in 0..12 {
                t.marks.clear();
                let t1 = Instant::now();
                ph2d_editor_core::tool::Tool::on_tick(&mut t, FRAME_MS);
                ticks.push(t1.elapsed().as_secs_f64() * 1e3);
                // ⚠️ A ÁREA que o tick declara suja é o que decide o UPLOAD, e o upload é do app, não
                // do tool: um tick barato que suja a tela inteira custa um documento por frame na
                // ponte. Medir só o relógio do tool responderia a pergunta errada.
                let a: u64 = t
                    .marks
                    .iter()
                    .map(|r| u64::from(r.w) * u64::from(r.h))
                    .sum();
                areas.push(a as f64);
                let _ = t.take_preview_arc();
            }
            ticks.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            areas.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let full = f64::from(side) * f64::from(side);
            println!(
                "{side:<8} {radius:>7.0} {stroke_ms:>12.2} {:>12.2} {:>12.2} {:>11.1}%",
                ticks[ticks.len() / 2],
                ticks[ticks.len() - 1],
                100.0 * areas[areas.len() / 2] / full
            );
        }
    }
    println!();
}

/// **O CUSTO DO TICK EM FUNÇÃO DO `dt`** — a pergunta que o log do produto abriu (2026-07-28).
///
/// O shell chama `on_tick(frame_ms_now)`: o `dt` **é o wall clock do frame anterior**. Se o custo do
/// tick crescer com o `dt`, um frame lento pede um tick mais caro, que deixa o frame seguinte mais
/// lento — **realimentação positiva**, e é assim que 60 FPS viram 4 sem que nenhuma medida a `dt`
/// FIXO mostre problema. A sonda irmã mede a `dt = 16,6` e vê 3,5 ms; esta varre o `dt`.
///
/// ⚠️ O log do Enio traz `frame p50=35,1` com `dispatch p50=1,9` e `periodo real 57,1 ms` — o Painter
/// é 5% do frame, então o que esta tabela procura é se a água **realimenta** o resto.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_whether_the_tick_feeds_back_on_a_slow_frame() {
    println!(
        "\n{:<8} {:>8} {:>12} {:>12}",
        "tela", "dt ms", "tick p50 ms", "tick max ms"
    );
    for side in [2048u32, 4096] {
        for dt in [16.6f32, 33.3, 57.1, 120.0, 250.0] {
            let mut t = wetted(side, 100.0);
            let mid = (side / 2) as f32;
            let x0 = 120.0;
            let x1 = ((side as f32) - 120.0).min(x0 + 400.0);
            t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
            let mut x = x0 + 40.0;
            while x <= x1 {
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
                let _ = t.take_preview_arc();
                x += 40.0;
            }
            t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));

            let mut ticks = Vec::new();
            for _ in 0..12 {
                let t1 = Instant::now();
                ph2d_editor_core::tool::Tool::on_tick(&mut t, dt);
                ticks.push(t1.elapsed().as_secs_f64() * 1e3);
                let _ = t.take_preview_arc();
            }
            ticks.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            println!(
                "{side:<8} {dt:>8.1} {:>12.2} {:>12.2}",
                ticks[ticks.len() / 2],
                ticks[ticks.len() - 1]
            );
        }
    }
    println!();
}

/// **O TICK DA ÁGUA NÃO REALIMENTA UM FRAME LENTO** — a propriedade, não o número.
///
/// O shell chama `on_tick(frame_ms_now)`, então o `dt` **é o wall clock do frame anterior**: se o custo
/// do tick crescer com o `dt`, um frame lento compra um tick caro que compra um frame mais lento. Foi
/// isso que levou 4096² a **4 FPS** no smoke de 2026-07-28, e nenhuma medida a `dt` FIXO podia vê-lo.
///
/// ⚠️ **O oráculo é uma RAZÃO** (`dt` de um frame travado ÷ `dt` de 60 Hz), e não um wall-clock: um
/// teto absoluto mede o perfil e a carga da máquina, enquanto a razão pergunta a coisa certa — *o custo
/// deste tick depende de quão lento foi o frame anterior?* Um laço fechado é ilimitado por natureza,
/// então qualquer teto o pega; a barra fica larga de propósito para não flakar sob suíte carregada.
///
/// ⚠️ **Mutação que sangra:** `WET_MAX_STEPS` de volta a 5 — a razão medida vai a **24×**.
#[test]
fn the_wet_tick_costs_the_frame_a_budget_not_a_puddle() {
    /// O `dt` de um frame de 60 Hz.
    const FAST_MS: f32 = 16.6;
    /// O `dt` de um frame travado a 4 FPS — o que o smoke reportou.
    const STALLED_MS: f32 = 250.0;
    /// Quantos frames a janela mede. Precisa ser bem maior que
    /// `custo_do_passo / orcamento` para que a AMORTIZAÇÃO apareça: com um passo
    /// de ~13 ms e 4 ms de orçamento são ~3 frames por passo, e uma janela curta
    /// pega ou só ticks vazios ou só o passo.
    const FRAMES: usize = 60;
    /// **O teto do custo MÉDIO por frame, em MILISSEGUNDOS LITERAIS.**
    ///
    /// ⚠️ **Literal, e não `WET_STEP_BUDGET_MS × k`** — a primeira versão deste
    /// gate derivava o teto da própria constante que ele existe para vigiar, e a
    /// mutação que a manda para o infinito levava o TETO junto: verde sobre o
    /// defeito, o oráculo-espelho que este repo já pagou três vezes.
    ///
    /// Medido: **5,1 ms/frame a 60 Hz e 5,3 travado** com o orçamento de 4 ms
    /// (a média de uma janela finita cerca o orçamento por cima — a dívida de um
    /// passo atômico é paga nos frames seguintes). Sem o orçamento: **12-25**.
    const CEILING_MS: f64 = 9.0;

    // ⚠️ **O redutor é a MÉDIA, não a mediana.** Sob orçamento a maioria dos ticks
    // custa ZERO e alguns custam um passo inteiro — a mediana reportaria 0,00 e o
    // gate ficaria verde por não medir nada. *O redutor é parte da fixture.*
    let mean_per_frame = |dt: f32| -> f64 {
        // ⚠️ **4096², e a tela é PARTE do fixture.** A 1024² um passo é barato o
        // bastante para caber no orçamento e o gate fica VERDE sobre o defeito
        // reportado. Uma fixture menor aqui não é "um teste mais barato": é um
        // teste de outra coisa.
        let mut t = wetted(4096, 100.0);
        t.on_canvas_pointer(cp([140.0, 2048.0], PointerPhase::Down));
        let mut x = 180.0f32;
        while x <= 1400.0 {
            t.on_canvas_pointer(cp([x, 2048.0], PointerPhase::Move));
            let _ = t.take_preview_arc();
            x += 40.0;
        }
        t.on_canvas_pointer(cp([1400.0, 2048.0], PointerPhase::Up));
        let mut total = 0.0;
        for _ in 0..FRAMES {
            let t0 = Instant::now();
            ph2d_editor_core::tool::Tool::on_tick(&mut t, dt);
            total += t0.elapsed().as_secs_f64() * 1e3;
            let _ = t.take_preview_arc();
        }
        total / FRAMES as f64
    };

    let fast = mean_per_frame(FAST_MS);
    let stalled = mean_per_frame(STALLED_MS);
    // Controle: sem trabalho medível o gate não afirma nada.
    assert!(
        fast > 0.05,
        "controle: o tick da agua nao custou nada ({fast:.4} ms/frame) — a fixture nao tem agua viva"
    );
    let ceiling = CEILING_MS;
    // A metade que o cap de CONTAGEM nunca teve: o custo por frame é do
    // ORÇAMENTO, não da poça — e vale nos dois regimes de `dt`.
    assert!(
        fast <= ceiling,
        "a 60 Hz a agua custa {fast:.2} ms/frame contra um orcamento de \
         {:.1} (teto {ceiling:.1})",
        super::wetpaint::WET_STEP_BUDGET_MS
    );
    assert!(
        stalled <= ceiling,
        "o tick REALIMENTA: {stalled:.2} ms/frame depois de um frame de {STALLED_MS} ms, \
         contra um orcamento de {:.1} (teto {ceiling:.1})",
        super::wetpaint::WET_STEP_BUDGET_MS
    );
}

/// **E O ORÇAMENTO É INERTE NUMA POÇA PEQUENA** — a outra metade do par.
///
/// Um teto que morde onde não precisa não é um teto, é uma regressão: com pouca água um passo custa
/// fração de milissegundo e a sim tem de rodar os **40 Hz cheios**. Este gate afirma a taxa, e é ele
/// que torna seguro apertar [`super::wetpaint::WET_STEP_BUDGET_MS`] — sem ele, baixar o orçamento
/// deixaria toda a suíte verde enquanto a água inteira do produto entra em câmera lenta.
///
/// ⚠️ O oráculo é a CONTAGEM de passos, não o relógio: quantos frames de 60 Hz produzem quantos passos
/// de 40 Hz é uma razão exata (0,664), e um bar de wall-clock aqui mediria a máquina.
#[test]
fn the_sim_time_budget_is_inert_on_a_small_puddle() {
    /// 60 frames de 60 Hz = 1 s ⇒ uma sim de 40 Hz deve dar ~40 passos.
    const FRAMES: usize = 60;
    /// O piso: 90% da taxa nominal. Abaixo disso o orçamento está mordendo água
    /// que ele não precisa governar.
    const MIN_STEPS: usize = 36;

    // Tela pequena + traço curto = a poça que o orçamento NÃO deve tocar.
    let mut t = wetted(512, 24.0);
    t.on_canvas_pointer(cp([100.0, 256.0], PointerPhase::Down));
    let mut x = 130.0f32;
    while x <= 400.0 {
        t.on_canvas_pointer(cp([x, 256.0], PointerPhase::Move));
        let _ = t.take_preview_arc();
        x += 30.0;
    }
    t.on_canvas_pointer(cp([400.0, 256.0], PointerPhase::Up));

    let before = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessao de agua existe apos o traco")
        .engine
        .sim
        .frame;
    for _ in 0..FRAMES {
        ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
        let _ = t.take_preview_arc();
    }
    let after = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessao sobrevive ao tick")
        .engine
        .sim
        .frame;
    let steps = (after - before) as usize;
    assert!(
        steps >= MIN_STEPS,
        "o orcamento de tempo MORDEU uma poca pequena: {steps} passos em {FRAMES} frames de 60 Hz \
         (uma sim de 40 Hz pede ~40; piso {MIN_STEPS})"
    );
}

/// **O CUSTO DO TICK CONTRA A ÁREA MOLHADA** — o que sobrou depois do cap de passos (2026-07-28).
///
/// O cap fechou o laço de realimentação (a contagem), e o re-smoke mostrou 60 FPS em duas janelas e
/// **`frame p50 = 88,2 ms` numa terceira**, com o dispatch em 3,4. O que muda entre elas é quanta água
/// existe: a queixa é *"uma pincelada GRANDE e molhada"*, e a fixture do gate molha 380 px.
///
/// Esta tabela varre o COMPRIMENTO do traço — a área molhada — com a tela e o raio fixos, para dizer se
/// o tick é limitado pela poça (e quanto), em vez de o adivinharmos.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_how_the_tick_scales_with_the_wet_area() {
    println!(
        "\n{:<10} {:>10} {:>12} {:>12} {:>12}",
        "traco px", "dabs", "tick p50 ms", "tick max ms", "sujo/tela"
    );
    let side = 4096u32;
    for span in [400.0f32, 1000.0, 2000.0, 3600.0] {
        let mut t = wetted(side, 100.0);
        let mid = (side / 2) as f32;
        let x0 = 140.0;
        let x1 = x0 + span;
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        let mut x = x0 + 40.0;
        let mut dabs = 0usize;
        while x <= x1 {
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            let _ = t.take_preview_arc();
            dabs += 1;
            x += 40.0;
        }
        t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));

        let mut ms = Vec::new();
        let mut areas = Vec::new();
        for _ in 0..10 {
            t.marks.clear();
            let t0 = Instant::now();
            ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
            let a: u64 = t
                .marks
                .iter()
                .map(|r| u64::from(r.w) * u64::from(r.h))
                .sum();
            areas.push(a as f64);
            let _ = t.take_preview_arc();
        }
        ms.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        areas.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        let full = f64::from(side) * f64::from(side);
        println!(
            "{span:<10.0} {dabs:>10} {:>12.2} {:>12.2} {:>11.1}%",
            ms[ms.len() / 2],
            ms[ms.len() - 1],
            100.0 * areas[areas.len() / 2] / full
        );
    }
    println!();
}

/// **A SIM VARRE A REGIÃO OU A BBOX DELA?** — a pergunta que decide se há ganho barato (2026-07-28).
///
/// O log do produto (`PH2D_FLUID_PROFILE=1`) diz **`tool-tick = 57,49 ms` de `total = 69,99`**: 82% do
/// frame, com o `dispatch` em 2,5. Depois do composite paralelo a sim é ~86% do tick, e ela é serial
/// por semântica — então antes de propor GPU vale saber se o custo é da ÁGUA ou da CAIXA.
///
/// Dois traços do MESMO comprimento e a MESMA água: um horizontal (bbox fina) e um diagonal (bbox
/// quadrada, ~N× maior). Se o diagonal custar muito mais, o motor paga a caixa e não a poça — e aí um
/// varrimento por tiles é ganho barato. Se custarem igual, o custo É a água e a saída é o dispositivo.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_whether_the_sim_pays_for_the_water_or_for_its_bounding_box() {
    /// O passo por eixo que mantém o COMPRIMENTO do caminho igual ao do horizontal.
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let side = 4096u32;
    println!(
        "\n{:<14} {:>8} {:>13} {:>13} {:>8} {:>10}",
        "forma", "dabs", "CAIXA p50 ms", "FAIXA p50 ms", "ganho", "bbox/tela"
    );
    // ABLAÇÃO PELA ENTRADA (`Grid::spans_enabled`), nunca por instrumentação:
    // desligada, a porta devolve a bbox inteira — exatamente o intervalo que o
    // motor varria antes da faixa viva. Uma linha por FORMA, os dois modos.
    for (name, diagonal) in [("horizontal", false), ("diagonal", true)] {
        let mut wide = 0.0f64;
        for spans in [false, true] {
            let mut t = wetted(side, 100.0);
            let x0 = 200.0f32;
            let y0 = if diagonal { 200.0 } else { 2048.0 };
            let n = 60;
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            // ⚠️ A sessão nasce no pen-DOWN, então armar o flag antes dele é um
            // `if let` que não casa — a busca negativa sem controle positivo, que
            // custou uma rodada inteira de medição mentindo "1,02x".
            {
                let sess = t
                    .paint
                    .wetpaint
                    .session
                    .as_mut()
                    .expect("a sessao de agua existe apos o pen-down");
                assert!(!sess.engine.layers.is_empty(), "sem camada, sem grid");
                for l in &mut sess.engine.layers {
                    l.grid.spans_enabled = spans;
                }
            }
            for k in 1..=n {
                let d = 40.0 * k as f32;
                // Mesmo COMPRIMENTO de caminho nos dois: o diagonal anda `d/√2` em cada eixo.
                let (x, y) = if diagonal {
                    (x0 + d * DIAG, y0 + d * DIAG)
                } else {
                    (x0 + d, y0)
                };
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                let _ = t.take_preview_arc();
            }
            let (lx, ly) = if diagonal {
                (x0 + 40.0 * n as f32 * DIAG, y0 + 40.0 * n as f32 * DIAG)
            } else {
                (x0 + 40.0 * n as f32, y0)
            };
            t.on_canvas_pointer(cp([lx, ly], PointerPhase::Up));

            let mut ms = Vec::new();
            let mut bbox = 0.0f64;
            for _ in 0..10 {
                t.marks.clear();
                let t0 = Instant::now();
                ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
                let a: u64 = t
                    .marks
                    .iter()
                    .map(|r| u64::from(r.w) * u64::from(r.h))
                    .sum();
                bbox = bbox.max(a as f64);
                let _ = t.take_preview_arc();
            }
            ms.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            let p50 = ms[ms.len() / 2];
            if spans {
                println!(
                    "{name:<14} {n:>8} {wide:>13.2} {p50:>13.2} {:>7.2}x {:>9.1}%",
                    wide / p50.max(1e-9),
                    100.0 * bbox / (f64::from(side) * f64::from(side))
                );
            } else {
                wide = p50;
            }
        }
    }
    println!();
}

/// **O EIXO QUE NENHUMA SONDA DESTE REPO VARIOU: o RAIO DO PINCEL.**
///
/// Toda medição de água até aqui fixou `radius = 100` e varreu o COMPRIMENTO do traço
/// (`measure_how_the_tick_scales_with_the_wet_area`) ou a FORMA dele
/// (`measure_whether_the_sim_pays_for_the_water_or_for_its_bounding_box`). O relato do Enio é
/// *"IMG 4096, 1 pincelada GRANDE e molhada, FPS para em 4"* — e o custo do passo é linear na
/// ÁREA MOLHADA, que cresce com o raio, não com o comprimento.
///
/// Um traço de comprimento L com raio r molha ~`2·r·L` células: dobrar o raio DOBRARIA o custo, e
/// o artista escolhe o raio com um slider. Esta tabela diz quanto, pelo tick do PRODUTO.
///
/// ⚠️ **O que ela achou (2026-07-29): o raio NÃO é o multiplicador** — de 50 a 400 px o tick fica em
/// 5,5-8,2 ms e a região suja em **2,1-2,2% da tela, constante**. O `TRAIL_HALF = 61` do engine clipa a
/// janela do traço (o item aberto que o handoff do Wet Paint já nomeia: *"clipa pincel gigante, wave
/// própria do engine"*), então um pincel gigante molha o mesmo que um de 100 px. A hipótese *"pincelada
/// GRANDE = 5× as células"* está **refutada por esta tabela**, e quem varia a área molhada é o
/// COMPRIMENTO do traço (`measure_how_the_tick_scales_with_the_wet_area`).
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_how_the_tick_scales_with_the_brush_radius() {
    println!(
        "\n{:<10} {:>10} {:>12} {:>12} {:>12} {:>10}",
        "raio px", "passos", "sim p50 ms", "comp p50 ms", "TICK p50 ms", "sujo/tela"
    );
    let side = 4096u32;
    // O TRAÇO é o mesmo em todas as linhas (mesma mão, mesmo caminho): só o raio muda.
    for radius in [50.0f32, 100.0, 200.0, 400.0] {
        let mut t = wetted(side, radius);
        let y = 2048.0f32;
        let x0 = 600.0f32;
        t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
        let mut k = 1;
        while k <= 60 {
            t.on_canvas_pointer(cp([x0 + 40.0 * k as f32, y], PointerPhase::Move));
            let _ = t.take_preview_arc();
            k += 1;
        }
        t.on_canvas_pointer(cp([x0 + 40.0 * 60.0, y], PointerPhase::Up));

        let (mut sim, mut comp, mut tick) = (Vec::new(), Vec::new(), Vec::new());
        let mut dirty = 0.0f64;
        for _ in 0..9 {
            {
                let sess = t
                    .paint
                    .wetpaint
                    .session
                    .as_mut()
                    .expect("a sessao de agua existe apos o traco");
                let t0 = Instant::now();
                sess.engine.step_simulation();
                sim.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            t.marks.clear();
            let t0 = Instant::now();
            super::wetpaint::composite_for_measure(&mut t);
            comp.push(t0.elapsed().as_secs_f64() * 1e3);
            let a: u64 = t
                .marks
                .iter()
                .map(|r| u64::from(r.w) * u64::from(r.h))
                .sum();
            dirty = dirty.max(a as f64);
            let _ = t.take_preview_arc();
            // E o tick do PRODUTO, que e quem o log `tool-tick` mede.
            let t0 = Instant::now();
            ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
            tick.push(t0.elapsed().as_secs_f64() * 1e3);
            let _ = t.take_preview_arc();
        }
        for v in [&mut sim, &mut comp, &mut tick] {
            v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        }
        println!(
            "{radius:<10.0} {:>10} {:>12.2} {:>12.2} {:>12.2} {:>9.1}%",
            60,
            sim[sim.len() / 2],
            comp[comp.len() / 2],
            tick[tick.len() / 2],
            100.0 * dirty / (f64::from(side) * f64::from(side))
        );
    }
    println!();
}

/// **O QUE O ORÇAMENTO DE TEMPO COMPRA** — a varredura que escolhe [`super::wetpaint::WET_STEP_BUDGET_MS`].
///
/// Roda uma janela de 120 frames a 60 Hz sobre uma poça já formada e reporta, POR ORÇAMENTO:
/// o custo médio do tick (o que o frame de fato paga), o PIOR tick (o hitch), e a taxa de
/// simulação alcançada (a água anda mais devagar — o trade declarado).
///
/// ⚠️ O redutor do custo por frame é a MÉDIA, não a mediana: com o orçamento a maioria dos ticks
/// custa ZERO e alguns custam um passo inteiro — a mediana reportaria 0,00 e esconderia exatamente
/// o que se quer orçar. Para o HITCH o redutor é o máximo. *O redutor é parte da fixture.*
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_what_the_sim_time_budget_buys() {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    const FRAMES: usize = 120;
    let side = 4096u32;
    println!(
        "\n{:<12} {:>9} {:>14} {:>13} {:>12} {:>12}",
        "forma", "orc ms", "tick medio ms", "pior tick ms", "passos/120f", "sim Hz"
    );
    for (name, diagonal) in [("horizontal", false), ("diagonal", true)] {
        let mut t = wetted(side, 100.0);
        let x0 = 200.0f32;
        let y0 = if diagonal { 200.0 } else { 2048.0 };
        let n = 60;
        t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
        for k in 1..=n {
            let d = 40.0 * k as f32;
            let (x, y) = if diagonal {
                (x0 + d * DIAG, y0 + d * DIAG)
            } else {
                (x0 + d, y0)
            };
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let (lx, ly) = if diagonal {
            (x0 + 40.0 * n as f32 * DIAG, y0 + 40.0 * n as f32 * DIAG)
        } else {
            (x0 + 40.0 * n as f32, y0)
        };
        t.on_canvas_pointer(cp([lx, ly], PointerPhase::Up));

        // ⚠️ Descarta os 10 primeiros frames: a sessão acabou de nascer e o
        // primeiro composite é canvas-sized. Um "pior tick" que na verdade é o
        // aquecimento nomearia a coisa errada.
        for _ in 0..10 {
            ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
            let _ = t.take_preview_arc();
        }
        let mut total = 0.0f64;
        let mut work: Vec<f64> = Vec::new();
        for _ in 0..FRAMES {
            let t0 = Instant::now();
            ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            total += ms;
            if ms > 0.5 {
                work.push(ms);
            }
            let _ = t.take_preview_arc();
        }
        work.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        let secs = FRAMES as f64 / 60.0;
        let pick = |q: f64| -> f64 {
            if work.is_empty() {
                0.0
            } else {
                work[((work.len() - 1) as f64 * q) as usize]
            }
        };
        println!(
            "{name:<12} {:>9.1} {:>14.2} {:>13.2} {:>12} {:>12.1}",
            super::wetpaint::WET_STEP_BUDGET_MS,
            total / FRAMES as f64,
            pick(1.0),
            work.len(),
            work.len() as f64 / secs,
        );
        println!(
            "             (dos {} ticks COM trabalho: p50 {:.2} · p90 {:.2} · max {:.2} ms)",
            work.len(),
            pick(0.5),
            pick(0.9),
            pick(1.0),
        );
    }
    println!();
}

/// **E de que são os milissegundos que SOBRAM** — as duas metades do tick,
/// cronometradas pelas portas que o `wetpaint_tick` chama.
///
/// O tick é literalmente `N × step_simulation()` + `wetpaint_composite()`.
/// A faixa viva estreitou a PRIMEIRA metade; esta sonda existe para dizer se a
/// segunda passou a ser a maior — e ela iterá o retângulo sujo que o ENGINE
/// declara, que é um casco pela mesma razão que a bbox era.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_two_halves_of_a_wet_tick() {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let side = 4096u32;
    println!(
        "\n{:<14} {:>12} {:>14} {:>12}",
        "forma", "sim p50 ms", "composite p50", "sujo/tela"
    );
    for (name, diagonal) in [("horizontal", false), ("diagonal", true)] {
        let mut t = wetted(side, 100.0);
        let x0 = 200.0f32;
        let y0 = if diagonal { 200.0 } else { 2048.0 };
        let n = 60;
        t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
        for k in 1..=n {
            let d = 40.0 * k as f32;
            let (x, y) = if diagonal {
                (x0 + d * DIAG, y0 + d * DIAG)
            } else {
                (x0 + d, y0)
            };
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let (lx, ly) = if diagonal {
            (x0 + 40.0 * n as f32 * DIAG, y0 + 40.0 * n as f32 * DIAG)
        } else {
            (x0 + 40.0 * n as f32, y0)
        };
        t.on_canvas_pointer(cp([lx, ly], PointerPhase::Up));

        let (mut sim, mut comp) = (Vec::new(), Vec::new());
        let mut dirty = 0.0f64;
        for _ in 0..10 {
            {
                let sess = t
                    .paint
                    .wetpaint
                    .session
                    .as_mut()
                    .expect("a sessao de agua existe apos o traco");
                let t0 = Instant::now();
                sess.engine.step_simulation();
                sim.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            t.marks.clear();
            let t0 = Instant::now();
            super::wetpaint::composite_for_measure(&mut t);
            comp.push(t0.elapsed().as_secs_f64() * 1e3);
            let a: u64 = t
                .marks
                .iter()
                .map(|r| u64::from(r.w) * u64::from(r.h))
                .sum();
            dirty = dirty.max(a as f64);
            let _ = t.take_preview_arc();
        }
        sim.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        comp.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        println!(
            "{name:<14} {:>12.2} {:>14.2} {:>11.1}%",
            sim[sim.len() / 2],
            comp[comp.len() / 2],
            100.0 * dirty / (f64::from(side) * f64::from(side))
        );
    }
    println!();
}
