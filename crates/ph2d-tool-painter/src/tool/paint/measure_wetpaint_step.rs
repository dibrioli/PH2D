//! **DE QUE O PASSO DO WET PAINT É FEITO** — irmão de
//! [`super::measure_wetpaint_tick`] (o que o FRAME paga), separado por
//! RESPONSABILIDADE quando o pai cruzou o teto de LOC.
//!
//! O corte é o assunto: lá se pergunta *quem paga o tempo da simulação e como o
//! frame se defende dela*; aqui se pergunta *de que um PASSO é feito, e o que
//! os dois sliders de grade fazem com ele*.
//!
//! ⚠️ **As três sondas daqui compartilham a mesma disciplina, e ela custou
//! caro:** o número sai da **porta do produto** (o `on_canvas_pointer`, nunca o
//! `Engine` direto — as duas fixtures "grandes" dão números incomparáveis,
//! doc 28 §5.40), a decomposição por-passe leva a **CADÊNCIA** (o
//! `build_flow_field` roda ÷4), e o total amortizado é **reconciliado com um
//! passo MEDIDO** num ciclo de 12. Essa última linha pagou-se três vezes numa
//! única sessão: acusou 141 MB de `ensure` dentro de um relógio, uma corrida de
//! máquina ruim, e um A/B com a rota forçada à mão.

use super::*;

/// **DE QUE OS 62 ms SÃO FEITOS — na poça que o PRODUTO constrói.**
///
/// ⚠️ **Esta sonda existe porque a irmã dela na crate do motor mediu OUTRA
/// cena.** O `measure_pass_cost::scene_big` monta três traços chamando o
/// `Engine` direto e mede um passo em **10,3 ms**; a `heavy_puddle` abaixo monta
/// três traços pelo `on_canvas_pointer` — o caminho do artista — e o worker do
/// produto mede **62 ms**. Seis vezes, e o número que decide a frente é o do
/// produto. *Duas fixtures "grandes" diferentes dão dois números que ninguém
/// consegue comparar* (o próprio doc-comment do `scene_big` diz isso, e eu
/// medi na fixture da crate de qualquer jeito).
///
/// Metodologia idêntica à da irmã: `snapshot_grid`/`restore_grid` devolvem o
/// MESMO estado antes de cada amostra — sem isso o `rebuild_active_region`
/// APERTA a bbox que ele próprio varre e as amostras 2..N medem uma janela
/// menor que a que o produto vê.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_a_step_of_the_products_puddle_is_made_of() {
    use ph2d_wet_paint::grid::{Grid, restore_grid, snapshot_grid};
    use ph2d_wet_paint::solver;
    use ph2d_wet_paint::tuning::Knob;

    const REPS: usize = 7;
    let mut t = heavy_puddle();
    t.wet_bring_home();
    let sess = t
        .paint
        .wetpaint
        .session
        .as_mut()
        .expect("a sessao de agua existe apos o traco");
    let e = &mut *sess.engine;
    let p = e.sim.gather_params(&e.tuning);
    let grav = e.sim.gravity(&e.tuning);
    let evap = e.sim.evap_scale * p.k(Knob::Evaporation);
    let rewet = e.sim.rewet_base * p.k(Knob::Rewet);
    let bypass = e.sim.ext_bypass;
    // ⚠️ **A ROTA que o produto de fato roda.** O `sim_step_stage` escolhe
    // `advect_jacobi`/`drying_pass_jacobi` sob `order_invariant` (ADR-0147), e uma
    // sonda que chamasse os irmãos CONGELADOS mediria o mundo pré-wave — a lição
    // do doc 28 §5.11 (*sonda que re-implementa o laço fica cega à porta*), aqui
    // na forma mais barata de errar: chamar a função de nome parecido.
    let order_inv = e.sim.order_invariant;
    // A CADÊNCIA — sem ela a soma dos passes não é o passo (doc 28 §5.40).
    let dry_every = e.sim.dry_every;
    let g = e.active_grid_mut();
    // ⚠️ **A MÁSCARA tem de estar VIVA antes do snapshot.** A 1ª versão desta
    // sonda tirou o snapshot do estado logo após o pen-up, onde `active` está
    // VAZIO — e todo passe gated em `active[i] == 0` fazia early-out em TODA
    // célula: `project` mediu 0,88 ms (contra 3,48 na fixture da crate) e
    // `smooth_velocity` 0,24. A soma casava com os 62 ms do worker **por
    // coincidência**, porque o `drying_pass` (que não é gated) dominava.
    // O worker roda o rebuild como 1º estágio; a fixture tem de fazer o mesmo.
    solver::rebuild_active_region(g);
    let (rows, span) = (
        (g.by1 - g.by0 + 1).max(0) as usize,
        (g.bx1 - g.bx0 + 1).max(0) as usize,
    );
    let live = g.active.iter().filter(|a| **a != 0).count();
    // As células que de fato carregam água — o predicado do `drying_pass`, e o
    // divisor honesto do custo dele.
    let wet = (0..g.film.len())
        .filter(|&i| g.film[i] > 0.0 || g.susp[i] > 0.0)
        .count();
    // ⚠️ **A bbox NÃO é o que os passes percorrem** — eles andam a faixa viva
    // por LINHA (`span_x_of`), e é contra ela que "quantos % estão ativos" quer
    // dizer alguma coisa. Sem este número não dá para saber se um passe é caro
    // por célula ou por percorrer células que não fazem nada.
    let spanned: usize = (g.by0..=g.by1)
        .map(|y| {
            let (lo, hi) = ph2d_wet_paint::grid::span_x_of(
                &g.row_lo,
                &g.row_hi,
                g.spans_enabled,
                g.bx0,
                g.bx1,
                y,
            );
            (hi - lo + 1).max(0) as usize
        })
        .sum();
    let snap = snapshot_grid(g);
    let time = |g: &mut Grid, f: &mut dyn FnMut(&mut Grid)| -> f64 {
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            restore_grid(g, &snap);
            let t0 = Instant::now();
            f(g);
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let mut out: Vec<(&str, f64)> = Vec::new();
    out.push((
        "rebuild_active_region",
        time(g, &mut solver::rebuild_active_region),
    ));
    out.push((
        "drying_pass",
        time(g, &mut |g| {
            if order_inv {
                ph2d_wet_paint::drying::drying_pass_jacobi(g, &p, evap, rewet, bypass);
            } else {
                ph2d_wet_paint::drying::drying_pass(g, &p, evap, rewet, bypass);
            }
        }),
    ));
    out.push((
        "build_flow_field",
        time(g, &mut |g| {
            if order_inv {
                solver::build_flow_field_jacobi(g, &p, grav[0], grav[1], bypass);
            } else {
                solver::build_flow_field(g, &p, grav[0], grav[1], bypass);
            }
        }),
    ));
    // ⚠️ **O A/B do campo de fluxo mora DENTRO da corrida, e é por isso.** A
    // máquina desta linha compartilha os 32 núcleos com outras worktrees e com
    // o app do Enio; comparar duas CORRIDAS mediu o mesmo passe intocado
    // oscilando 36% (doc 28 §5.39), e nesta sessão o passo do produto foi de
    // 14,9 a 30,2 ms sem uma linha de código mudar. Medidos costas-com-costas
    // sobre o MESMO estado restaurado, os dois números são comparáveis mesmo
    // com a máquina ocupada — o que a carga faz é levantar os dois juntos.
    let gs_flow = time(g, &mut |g| {
        solver::build_flow_field(g, &p, grav[0], grav[1], bypass);
    });
    out.push((
        "smooth_velocity",
        time(g, &mut |g| solver::smooth_velocity(g, &p)),
    ));
    out.push((
        "advect",
        time(g, &mut |g| {
            if order_inv {
                solver::advect_jacobi(g, &p, grav[0], grav[1]);
            } else {
                solver::advect(g, &p, grav[0], grav[1]);
            }
        }),
    ));
    out.push((
        "apply_boundaries",
        time(g, &mut |g| solver::apply_boundaries(g, false)),
    ));
    out.push(("project", time(g, &mut |g| solver::project(g, &p))));
    restore_grid(g, &snap);

    println!("\n  A POCA DO PRODUTO (heavy_puddle, 4096x4096, mediana de {REPS})\n");
    let cells = (rows * span).max(1);
    println!(
        "    bbox {rows} x {span} = {cells} celulas ({:.1}% da tela)\n    \
         FAIXA VIVA {spanned} ({:.1}% da bbox) | ativas {live} ({:.1}% da FAIXA) | \
         com agua {wet}",
        100.0 * cells as f64 / (4096.0 * 4096.0),
        100.0 * spanned as f64 / cells as f64,
        100.0 * live as f64 / spanned.max(1) as f64,
    );
    // ⚠️ **A CADÊNCIA é parte da resposta, não um detalhe** (doc 28 §5.40): o
    // `sim_step_stage` não roda todo passe em todo passo, então a soma CRUA
    // sobrestima quem roda ÷3 ou ÷4 e faz otimizar o passe errado. A fração
    // abaixo é a do `sim.rs` — `smooth_velocity` é o complemento do `build`.
    let share = |name: &str| -> f64 {
        match name {
            "rebuild_active_region" => 0.5,
            "drying_pass" => 1.0 / dry_every as f64,
            "build_flow_field" => 0.25,
            "smooth_velocity" => 0.75,
            "project" => 1.0 / 3.0,
            // `advect` e `apply_boundaries` correm em TODO passo (o segundo
            // roda de novo dentro do estágio da projeção, daí 1 + 1/3).
            "apply_boundaries" => 1.0 + 1.0 / 3.0,
            _ => 1.0,
        }
    };
    let total: f64 = out.iter().map(|(_, t)| t).sum();
    let amort: f64 = out.iter().map(|(n, t)| t * share(n)).sum();
    println!(
        "    {:<24} {:>8}  {:>7}  {:>8}   % do passo",
        "passe", "cru(ms)", "cadencia", "amort(ms)"
    );
    for (name, ms) in &out {
        let a = ms * share(name);
        println!(
            "    {name:<24} {ms:8.3}  {:7.3}  {a:8.3}   {:5.1}%",
            share(name),
            100.0 * a / amort,
        );
    }
    println!(
        "    {:<24} {total:8.3}  {:7}  {amort:8.3}   (dry_every = {dry_every})",
        "SOMA", ""
    );
    // ⚠️ **E o passo MEDIDO, pela porta do produto** — sem isto a tabela acima
    // é uma conta que ninguém conferiu. A unidade é o CICLO DE CADÊNCIA (12
    // passos cobrem ÷2, ÷3, ÷4 e ÷6 de uma vez): a mediana de passos avulsos
    // esconde justamente os passes caros, que rodam 1 em 3 ou 1 em 4.
    const CYCLE: usize = 12;
    let frame0 = e.sim.frame;
    let mut v = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        restore_grid(e.active_grid_mut(), &snap);
        e.sim.frame = frame0;
        let t0 = Instant::now();
        for _ in 0..CYCLE {
            e.step_simulation();
        }
        v.push(t0.elapsed().as_secs_f64() * 1e3 / CYCLE as f64);
    }
    v.sort_by(f64::total_cmp);
    let measured = v[REPS / 2];
    println!(
        "\n    PASSO MEDIDO (ciclo de {CYCLE}, porta do produto)  {measured:7.3} ms  \
         -> {:6.1} Hz\n    a SOMA amortizada acima da {amort:7.3} ms  (erro {:+.1}%)",
        1000.0 / measured.max(1e-9),
        100.0 * (amort - measured) / measured,
    );
    let ji_flow = out
        .iter()
        .find(|(n, _)| *n == "build_flow_field")
        .map_or(0.0, |(_, t)| *t);
    println!(
        "\n    O CAMPO DE FLUXO, as duas rotas na MESMA corrida:\n    \
         gauss-seidel {gs_flow:7.3} ms   independente de ordem {ji_flow:7.3} ms   \
         {:5.2}x\n    e no PASSO isso vale {:+.3} ms (cadencia 0,25) -> \
         {:7.3} ms contra {:7.3}",
        gs_flow / ji_flow.max(1e-9),
        0.25 * (ji_flow - gs_flow),
        amort,
        amort + 0.25 * (gs_flow - ji_flow),
    );
    println!(
        "\n    Leitura: a coluna AMORT e a que decide onde gastar esforco — a CRUA\n             \
         sobrestima quem roda ÷3 ou ÷4. Se os dois totais nao reconciliam, a\n             \
         fixture esta envenenada (doc 28 §5.40)."
    );
}

/// **O que a razão da grade compra, pela PORTA do artista** (o slider "Grid
/// Size (px)" — `wetpaint::grid_map`).
///
/// A grade do fluido era **1:1 com os pixels do canvas**, então a 4096² a
/// física pagava **16,7 M células** — e o custo é comprovadamente **linear nas
/// células** (o `ns/célula` é PLANO de 512² a 4096²,
/// `ph2d-wet-paint/tests/measure_density.rs`). A razão desacopla as duas.
///
/// ⚠️ **Esta sonda substituiu a `measure_what_a_coarser_grid_would_buy`, e o
/// doc dela continha uma AFIRMAÇÃO QUE A CONSTRUÇÃO DERRUBOU:** *"mudar a razão
/// grade:canvas re-pina o fingerprint"*. **Não re-pina.** O motor sempre foi
/// agnóstico de dimensão — a suíte de aceitação dele roda em 900×450, 300×200 e
/// 60×60 justamente porque a dimensão nunca foi parte da física —, então
/// `Engine::new(gw, gh)` com números menores é o MESMO código e
/// `tests/fingerprint.rs` fica **intacto**. O que a razão muda é de quantos
/// pixels o HOST fala com ele, e essa conversão vive toda no `grid_map`. A
/// estimativa era minha; o fato é do produto.
///
/// A outra metade da correção: a sonda antiga ESTIMAVA o ganho desenhando a
/// mesma figura física num canvas menor. Era honesta mas indireta; agora o
/// número sai do caminho que o artista percorre — a mesma correção que a §5.40
/// fez ao descobrir que duas fixtures "grandes" davam números incomparáveis
/// (10,34 contra 62,05 ms/passo).
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_grid_ratio_buys() {
    const REPS: usize = 9;
    println!("\n  A MESMA CENA EM RAZOES DE GRADE DIFERENTES (mediana de {REPS})\n");
    println!("    O slider 'Grid Size (px)' e a porta; a figura em PIXELS e sempre a mesma.\n");
    let mut base: Option<(f64, usize)> = None;
    for ratio in [1u8, 2, 3, 4, 8] {
        const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
        let mut t = wetted(4096, 100.0);
        // A PORTA do artista — e ela encerra a sessao, entao vem antes do traco.
        t.set_wet_grid_ratio(f64::from(ratio));
        for lane in 0..3 {
            let off = 260.0 * lane as f32;
            let (x0, y0) = (200.0 + off, 200.0);
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            for k in 1..=90u32 {
                let d = 40.0 * k as f32;
                t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Move));
                let _ = t.take_preview_arc();
            }
            let d = 40.0 * 90.0;
            t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Up));
        }
        t.wet_bring_home();
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        let grid = sess.grid;
        let e = &mut *sess.engine;
        let live = {
            let g = e.active_grid_mut();
            ph2d_wet_paint::solver::rebuild_active_region(g);
            g.active.iter().filter(|a| **a != 0).count()
        };
        // O passo INTEIRO pela porta do produto, estado restaurado por amostra.
        let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
            let t0 = Instant::now();
            e.step_simulation();
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        v.sort_by(f64::total_cmp);
        let med = v[v.len() / 2];
        let hz = 1000.0 / med.max(1e-9);
        print!(
            "    {ratio:2}:1  grade {:4}x{:<4}  {live:>8} vivas  {med:8.3} ms/passo  -> {hz:6.1} Hz",
            grid.0, grid.1
        );
        match base {
            None => println!("   (a referencia)"),
            Some((b_med, b_live)) => println!(
                "   [{:5.2}x mais rapido, {:5.2}x menos celulas]",
                b_med / med.max(1e-9),
                b_live as f64 / live.max(1) as f64
            ),
        }
        if base.is_none() {
            base = Some((med, live));
        }
    }
    println!(
        "\n    Leitura: o custo e LINEAR nas celulas vivas, entao a razao de tempo tem de\n             acompanhar a razao de celulas. O nominal da SPEC e 40 Hz (25 ms/passo)."
    );
}

/// **O que o Pigment Mixing (K–M) custa, e o que a razão da grade faz com ele**
/// (report do Enio 2026-07-29: *"em Tuning: Pigment Mixing (K-M) temos séria
/// queda de FPS"*).
///
/// O doc 24 já tabelou a transferência sRGB e levou o flood de 122,8 para 18,9
/// ms/passo, deixando NOMEADO que *"o flood com K–M ligado fica em 18,9 ms
/// contra o kill de 12"*. Esta sonda pergunta a coisa que mudou desde então: o
/// custo do K–M é **por-célula** (9 misturas de cor por célula advectada), e a
/// razão da grade corta células — então ela tem de cortar o K–M na mesma
/// proporção. Se cortar, o item se dissolve sem uma linha de otimização.
///
/// Mede as DUAS metades pelas portas do produto: `km_mixing` (a sim) e
/// `km_glaze` (o composite).
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_km_costs_at_each_grid_ratio() {
    const REPS: usize = 7;
    println!("\n  O K-M POR RAZAO DE GRADE (mediana de {REPS}, 4096x4096, pincel r=100)\n");
    println!(
        "    {:>5}  {:>10} {:>10} {:>8}   {:>10} {:>10} {:>8}",
        "razao", "passo off", "passo ON", "custo", "comp off", "comp ON", "custo"
    );
    for ratio in [1u8, 2, 4, 8] {
        let mut row = [0.0f64; 4];
        for (slot, km) in [(0usize, false), (1, true)] {
            const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
            let mut t = wetted(4096, 100.0);
            t.set_wet_grid_ratio(f64::from(ratio));
            // As duas metades do EXPERIMENTAL, pelas portas do produto.
            t.paint.wetpaint.km_mixing = km;
            t.paint.wetpaint.km_glaze = km;
            for lane in 0..3 {
                let off = 260.0 * lane as f32;
                let (x0, y0) = (200.0 + off, 200.0);
                t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
                for k in 1..=90u32 {
                    let d = 40.0 * k as f32;
                    t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Move));
                    let _ = t.take_preview_arc();
                }
                let d = 40.0 * 90.0;
                t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Up));
            }
            t.wet_bring_home();
            // (a) o PASSO da sim.
            let step = {
                let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
                let e = &mut *sess.engine;
                {
                    let g = e.active_grid_mut();
                    ph2d_wet_paint::solver::rebuild_active_region(g);
                }
                let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
                let mut v = Vec::with_capacity(REPS);
                for _ in 0..REPS {
                    ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
                    let t0 = Instant::now();
                    e.step_simulation();
                    v.push(t0.elapsed().as_secs_f64() * 1e3);
                }
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            // (b) o COMPOSITE (a outra metade: o glaze mora aqui).
            let comp = {
                let mut v = Vec::with_capacity(REPS);
                for _ in 0..REPS {
                    if let Some(sess) = t.paint.wetpaint.session.as_mut() {
                        sess.bring_home();
                        sess.engine.mark_dirty_full();
                    }
                    let t0 = Instant::now();
                    crate::tool::paint::wetpaint::composite_for_measure(&mut t);
                    v.push(t0.elapsed().as_secs_f64() * 1e3);
                }
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            row[slot * 2] = step;
            row[slot * 2 + 1] = comp;
        }
        // row = [step_off, comp_off, step_on, comp_on]
        println!(
            "    {ratio:>3}:1  {:>9.3}ms {:>9.3}ms {:>7.2}x   {:>9.3}ms {:>9.3}ms {:>7.2}x",
            row[0],
            row[2],
            row[2] / row[0].max(1e-9),
            row[1],
            row[3],
            row[3] / row[1].max(1e-9),
        );
    }
    println!(
        "\n    Leitura: o K-M e custo POR CELULA, entao a razao o corta na mesma proporcao\n             que corta o passo. O kill do ADR-0134 e 12 ms/passo; o nominal, 25 ms (40 Hz)."
    );
}

#[cfg(test)]
#[path = "measure_wetpaint_contention.rs"]
mod measure_wetpaint_contention; // o que a MAQUINA faz com o passo - irmao por LOC
