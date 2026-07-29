//! **As MEDIÇÕES do tick do Wet Paint** (filho de [`super`] — teto de LOC): as sondas `#[ignore]`
//! que produziram os números dos gates irmãos. O corte é responsabilidade: lá o que se AFIRMA,
//! aqui o que se MEDE.

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
/// **O QUE O ORÇAMENTO DE TEMPO COMPRA** — a janela que mostra o controlador em regime.
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
            0.0,
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
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_the_heavy_puddle_regime() {
    let mut t = heavy_puddle();
    for _ in 0..10 {
        ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
        let _ = t.take_preview_arc();
    }
    let before = sim_frame(&t);
    let mut total = 0.0f64;
    let mut worst = 0.0f64;
    for _ in 0..120 {
        let t0 = Instant::now();
        ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        total += ms;
        worst = worst.max(ms);
        let _ = t.take_preview_arc();
    }
    println!(
        "\n  POCA PESADA: tick medio {:.2} ms | pior {:.2} | sim {:.1} Hz\n",
        total / 120.0,
        worst,
        (sim_frame(&t) - before) as f64 / 2.0
    );
}

/// **DE QUE É FEITO O `stamps` DO LOG** — o eixo que nenhuma sonda de água mediu.
///
/// Smoke do Enio (2026-07-29), com o tick já orçado: `tool-tick=0.00ms` em TODA amostra e
/// **`stamps=13.96ms` e depois `stamps=116.03ms`**. O `stamps` é `last_paint_stamp_us`: o custo dos
/// dabs dirigidos pelo PONTEIRO, dentro do `on_canvas_pointer` — fora do frame, e portanto fora de
/// tudo que o orçamento do tick governa.
///
/// Esta sonda cronometra CADA chamada de `on_canvas_pointer` de um traço, pela porta do produto.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn measure_what_a_wet_stamp_costs() {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    println!(
        "\n{:<10} {:>8} {:>10} {:>10} {:>10} {:>10}",
        "tela", "raio", "down ms", "move p50", "move p90", "move MAX"
    );
    for side in [2048u32, 4096] {
        for radius in [100.0f32, 300.0] {
            let mut t = wetted(side, radius);
            let x0 = 300.0f32;
            let y0 = 300.0f32;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            let down = t0.elapsed().as_secs_f64() * 1e3;
            let _ = t.take_preview_arc();
            let mut ms = Vec::new();
            for k in 1..=60 {
                let d = 40.0 * k as f32;
                let t0 = Instant::now();
                t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Move));
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
                let _ = t.take_preview_arc();
                // O produto TICKA entre eventos de ponteiro; sem isso a sonda
                // mede um traço que nunca simula, que é outra coisa.
                ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
                let _ = t.take_preview_arc();
            }
            let d = 40.0 * 60.0;
            t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Up));
            ms.sort_by(f64::total_cmp);
            println!(
                "{:<10} {radius:>8.0} {down:>10.2} {:>10.2} {:>10.2} {:>10.2}",
                format!("{side}x{side}"),
                ms[ms.len() / 2],
                ms[ms.len() * 9 / 10],
                ms[ms.len() - 1],
            );
        }
    }
    println!();
}

/// **O REGIME DO INQUILINO ESTRANGEIRO, frame a frame** — a sonda que achou o orçamento preso.
///
/// Ela imprime `dt`, custo do tick e ORÇAMENTO ao longo de 80 frames em que outro inquilino domina
/// o quadro. Foi ela que mostrou o orçamento parado em **1,04 ms por 80 frames** depois de três
/// recuos: o ramo *"a culpa é de outro"* SEGURAVA o orçamento onde estava, e segurar num piso é
/// ficar preso nele para sempre. Hoje esse ramo CRESCE, e quem protege o frame é o teto.
#[test]
#[ignore = "diagnostico — rode com --release --ignored --nocapture"]
fn measure_the_tenant_regime() {
    const FOREIGN_MS: f64 = 60.0;
    let mut t = big_puddle(true);
    for _ in 0..40 {
        ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
        let _ = t.take_preview_arc();
    }
    let mut dt = FOREIGN_MS;
    println!("\n  {:>5} {:>9} {:>9} {:>9}", "k", "dt", "tick", "orcamento");
    for k in 0..80 {
        let t0 = Instant::now();
        ph2d_editor_core::tool::Tool::on_tick(&mut t, dt as f32);
        let tick = t0.elapsed().as_secs_f64() * 1e3;
        dt = FOREIGN_MS + tick;
        let _ = t.take_preview_arc();
        if k % 8 == 0 {
            let b = t.paint.wetpaint.session.as_ref().expect("sessao").budget.per_frame_ms;
            println!("  {k:>5} {dt:>9.2} {tick:>9.2} {b:>9.2}");
        }
    }
}

/// Quanto o orçamento assenta no regime da CATRACA — o número que o gate irmão vigia, com a máquina
/// livre e sob carga, para o piso dele ser escolhido em vez de chutado.
#[test]
#[ignore = "diagnostico — rode com --release --ignored --nocapture"]
fn measure_the_ratchet_regime() {
    const OVERHEAD_MS: f64 = 3.0;
    const VSYNC_MS: f64 = 16.6;
    let mut t = big_puddle(true);
    let mut dt = VSYNC_MS;
    for _ in 0..200 {
        let t0 = Instant::now();
        ph2d_editor_core::tool::Tool::on_tick(&mut t, dt as f32);
        dt = (OVERHEAD_MS + t0.elapsed().as_secs_f64() * 1e3).max(VSYNC_MS);
        let _ = t.take_preview_arc();
    }
    let b = t.paint.wetpaint.session.as_ref().expect("sessao").budget.per_frame_ms;
    println!("\n  orcamento em regime: {b:.2} ms\n");
}
