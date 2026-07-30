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
//!    trabalho por célula, e o número de células é o que a mão do artista escolhe.
//! 5. **Quem PAGA o tempo** — e é aqui que a família fecha: o custo por passo é o piso da física, mas
//!    *quem* o paga era escolha nossa. A sim saiu da thread do frame
//!    (`wetpaint/offthread.rs`) e o frame passou a MOSTRAR em vez de simular.
//!
//! ⚠️ **CORREÇÃO (2026-07-29):** este cabeçalho afirmava *"não há paralelismo byte-idêntico a
//! colher (o solver é Gauss-Seidel em toda parte — ADR-0134)"*, e a MEDIÇÃO desmentiu a segunda
//! metade. O ADR-0134 nomeia **dois** mecanismos sequenciais — o freio do fluxo (que lê o `wet`
//! VIVO escrito por células anteriores) e a secagem (que lê o vizinho esquerdo pós-update) — e eles
//! somam **34%** do passo (`build_flow_field` 11,99 ms + `drying_pass` 4,70 de 48,65). Dos outros
//! 66%, dois passes **não são Gauss-Seidel nenhum**: o `project` é **JACOBI** (quatro laços de linha,
//! cada um lendo um buffer e escrevendo **outro** — `div`←`vel`, `prs`←`div`, `vel`←`prs`) e o
//! `smooth_velocity` é gather puro (lê `vel`, escreve `flow`). Os dois são row-disjoint e
//! byte-idênticos por construção sob `par_chunks_mut`.
//! ⚠️ **E FORAM FEITOS (2026-07-29, ordem do Enio: *"rayon"*) — [ADR-0145]** — mais uma terceira
//! porta que esta nota não tinha visto: **3 das 4 sub-passadas do `rebuild_active_region`** também são
//! row-disjuntas (a limpeza, o scan da extensão viva, e o passe 1, cujo trio `film[i±1]` é
//! **HORIZONTAL**); só a SAIA fica serial, porque ela escreve `active[i±s]` e a ordem é load-bearing.
//! Medido pela porta do produto, mesmo binário, poça de 5,1 M células: **um passo inteiro 16,08 →
//! 10,34 ms (1,56×)**, pior caso 26,43 → 19,07. `advect` (7,55 ms) NÃO entra: ele **subtrai** nos
//! cantos-fonte com clamp, então duas células de linhas diferentes que retro-traçam para o mesmo canto
//! são uma escrita-leitura em conflito. `build_flow_field` também não, e por DOIS mecanismos — o freio
//! do ADR-0134 e o backrun, que espalha em `susp[nb]`/`sett[nb]`.
//!
//! 6. **A CADÊNCIA** (2026-07-29, [ADR-0145 §4.1](../../../../../docs/architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md)) —
//!    e é ela que fecha a família, porque sem ela toda decomposição por-passe mente. O
//!    `sim_step_stage` **não roda todo passe em todo passo**: `advect` e `apply_boundaries` sempre ·
//!    `rebuild_active_region` a cada 2 · `project` e `drying_pass` a cada 3 · `build_flow_field` a cada
//!    **4**, e nos outros três o lugar dele é do `smooth_velocity`, ~50× mais barato. Amortizada, a
//!    decomposição prevê o passo do produto com **0,03 ms de erro** — e diz que os três passes que
//!    foram ao rayon são **6,5% do passo**, não os ~46% da soma-sem-cadência. Daí o produto ganhar
//!    **1,10×** onde a fixture da crate media 1,56×.
//!
//! ⚠️ **E DUAS fixtures "grandes" diferentes dão dois números que ninguém consegue comparar:** a
//! `measure_pass_cost::scene_big` (que dirige o `Engine` direto) custa **10,34 ms/passo** e a
//! `heavy_puddle` daqui (que dirige o `on_canvas_pointer`, o caminho do artista) custa **62,05**. Seis
//! vezes. Quando o número vira decisão de produto, ele TEM de sair da porta do produto.

use super::*;

/// Uma poça PESADA: três traços sobrepostos, para o passo custar muito mais que o teto do
/// orçamento. É o regime em que o teto é load-bearing — numa poça leve a sim já alcança os 40 Hz
/// e o orçamento fica inerte (foi assim que a primeira mutação do teto SOBREVIVEU).
fn heavy_puddle() -> PainterTool {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let mut t = wetted(4096, 100.0);
    for lane in 0..3 {
        let off = 260.0 * lane as f32;
        let x0 = 200.0f32 + off;
        let y0 = 200.0f32;
        t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
        for k in 1..=90 {
            let d = 40.0 * k as f32;
            t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let d = 40.0 * 90.0;
        t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Up));
    }
    t
}

/// A poça que os gates do controlador usam: 4096², traço de 2400 px, o pincel do censo.
/// ⚠️ **A tela é PARTE do fixture** — a 1024² um passo cabe no orçamento e o gate fica VERDE
/// sobre o defeito reportado.
fn big_puddle(diagonal: bool) -> PainterTool {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let mut t = wetted(4096, 100.0);
    let x0 = 200.0f32;
    let y0 = if diagonal { 200.0 } else { 2048.0 };
    t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
    for k in 1..=60 {
        let d = 40.0 * k as f32;
        let (x, y) = if diagonal {
            (x0 + d * DIAG, y0 + d * DIAG)
        } else {
            (x0 + d, y0)
        };
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        let _ = t.take_preview_arc();
    }
    let d = 40.0 * 60.0;
    let (lx, ly) = if diagonal {
        (x0 + d * DIAG, y0 + d * DIAG)
    } else {
        (x0 + d, y0)
    };
    t.on_canvas_pointer(cp([lx, ly], PointerPhase::Up));
    t
}

/// O contador de passos do engine — o oráculo da TAXA da sim (uma contagem exata,
/// não um wall-clock que mediria a máquina).
///
/// ⚠️ **`&mut` porque o motor VIAJA:** ele pode estar com o worker, e ler dali
/// daria o número de um instante que ninguém escolheu (o `Deref` do slot panica
/// dizendo isso). Trazer para casa é a porta.
fn sim_frame(t: &mut PainterTool) -> u64 {
    t.wet_bring_home();
    t.paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessao de agua existe apos o traco")
        .engine
        .sim
        .frame
}

/// **O QUE A SIM FORA DA THREAD COMPRA** — a sonda que fecha a wave.
///
/// Ela mede as DUAS coisas que a wave prometeu, e o oráculo de cada uma é
/// diferente: a **taxa da água** é uma contagem exata de passos sobre wall-clock
/// (é wall-clock que a sim agora persegue), e o **custo do frame** é o tick, que
/// deixou de conter passo nenhum.
///
/// ⚠️ A espera até o vsync é `sleep`, nunca spin: um spin queimaria o núcleo que
/// o worker acabou de ganhar, e a sonda mediria a si mesma.
#[test]
#[ignore = "sonda de medicao (wall-clock); rode com --ignored --nocapture"]
fn measure_what_the_off_thread_sim_buys() {
    const FRAMES: usize = 120;
    const PERIOD: f32 = 1.0 / 60.0;
    println!("\n  A SIM FORA DA THREAD DO FRAME (4096², 120 frames a 60 Hz)");
    for (nome, mut t) in [
        ("um traco  ", big_puddle(false)),
        ("tres tracos", heavy_puddle()),
    ] {
        let f0 = sim_frame(&mut t);
        let mut ticks: Vec<f32> = Vec::with_capacity(FRAMES);
        let wall0 = std::time::Instant::now();
        for _ in 0..FRAMES {
            let a = std::time::Instant::now();
            t.paint_tick(PERIOD);
            let spent = a.elapsed();
            ticks.push(spent.as_secs_f32() * 1e3);
            // O resto do frame é a FOLGA — o app estaria no vsync, e é dela que
            // a água agora vive.
            if let Some(rest) = std::time::Duration::from_secs_f32(PERIOD).checked_sub(spent) {
                std::thread::sleep(rest);
            }
        }
        let wall = wall0.elapsed().as_secs_f32();
        let ((step_sum, step_max, step_n), (comp_sum, _, comp_n), (wait_sum, wait_max, wait_n)) =
            crate::wet_diag::take_window();
        let (busy, away, sleep) = crate::wet_diag::take_worker();
        let steps = sim_frame(&mut t) - f0;
        ticks.sort_by(|a, b| a.partial_cmp(b).expect("sem NaN"));
        let p50 = ticks[ticks.len() / 2];
        let max = ticks[ticks.len() - 1];
        let hz = steps as f32 / wall;
        let tick_total = ticks.iter().sum::<f32>();
        println!(
            "    {nome}  sim {steps:4} passos em {wall:.2} s = {hz:5.1} Hz  |  \
             tick p50 {p50:6.3} max {max:6.3} ms"
        );
        println!(
            "                 DE QUE O TICK E FEITO: total {tick_total:6.1} ms = \
             composite {comp_sum:6.1} (x{comp_n})  +  ESPERA {wait_sum:6.1} (x{wait_n}, \
             max {wait_max:.2})  +  resto {:.1}",
            tick_total as f64 - comp_sum - wait_sum
        );
        // ⚠️ **A partição do WORKER é o que decide a frente seguinte**, e ela é
        // sobre uma janela de wall-clock que os três baldes ou explicam ou não.
        let w = f64::from(wall) * 1000.0;
        let pct = |x: f64| 100.0 * x / w;
        println!(
            "                 O WORKER, em {w:.0} ms: busy {busy:6.1} ({:4.1}%)  \
             away {away:6.1} ({:4.1}%)  sleep {sleep:6.1} ({:4.1}%)  \
             resto {:.1} ({:4.1}%)",
            pct(busy),
            pct(away),
            pct(sleep),
            w - busy - away - sleep,
            pct(w - busy - away - sleep),
        );
        println!(
            "                 UM PASSO custa {:6.3} ms de COMPUTE (pico {step_max:6.3}, \
             x{step_n}) e leva {:6.1} ms de PAREDE",
            if step_n > 0 {
                step_sum / step_n as f64
            } else {
                0.0
            },
            if steps > 0 { w / steps as f64 } else { 0.0 },
        );
    }
    println!(
        "    (a taxa NOMINAL da sim e 40 Hz — o worker se ritma nisso e nao passa dela;\n     \
         o tick e o que o FRAME paga, e ele nao contem passo nenhum)"
    );
}

/// **E NUM FRAME SEM FOLGA ELE RECUA** — a outra ponta do controlador, e a que impede o desastre.
///
/// Aqui o `dt` é REALIMENTADO (`overhead + custo do tick`, com o piso do vsync): é o app CPU-bound,
/// onde cada milissegundo gasto na água aparece no `dt` do frame seguinte. Sem recuo isto é um laço
/// de realimentação positiva — exatamente o que derrubou o produto a 4 FPS antes do cap.
#[test]
fn the_wet_tick_does_not_run_the_frame_away() {
    /// O custo do RESTO do frame. ⚠️ **20 ms, não os 5 do log** — e a escolha é o
    /// que torna este gate capaz de falhar: com 5 ms o piso do vsync (16,6)
    /// ABSORVE tudo que a água gasta, `dt` nunca passa do alvo, o recuo nunca
    /// dispara e o TETO sozinho segura o número. Foi assim que a mutação do
    /// recuo sobreviveu à primeira versão. Num app CPU-bound — que é o regime de
    /// 4 FPS que o Enio reportou — cada milissegundo da água aparece no `dt`.
    const OVERHEAD_MS: f64 = 20.0;
    /// O período do vsync: um frame nunca é mais rápido que isso.
    const VSYNC_MS: f64 = 16.6;
    const FRAMES: usize = 200;
    /// **O oráculo é a RAZÃO `dt / overhead`**, não um wall-clock: sob carga o
    /// overhead simulado não muda mas o custo da água sim, e uma razão contra a
    /// linha de base do próprio experimento não deriva com a máquina.
    /// Com o recuo o frame assenta no próprio overhead (**1,05×**); sem ele a
    /// água soma o orçamento inteiro em cima (**1,64×**).
    const MAX_SETTLED_RATIO: f64 = 1.35;

    let mut t = big_puddle(true);
    let mut dt = VSYNC_MS;
    let mut settled: Vec<f64> = Vec::new();
    for k in 0..FRAMES {
        let t0 = Instant::now();
        ph2d_editor_core::tool::Tool::on_tick(&mut t, dt as f32);
        let tick = t0.elapsed().as_secs_f64() * 1e3;
        let _ = t.take_preview_arc();
        dt = (OVERHEAD_MS + tick).max(VSYNC_MS);
        // Os últimos 50 frames são o REGIME; os primeiros são o transiente.
        if k >= FRAMES - 50 {
            settled.push(dt);
        }
    }
    settled.sort_by(f64::total_cmp);
    let median = settled[settled.len() / 2];
    let ratio = median / OVERHEAD_MS;
    assert!(
        ratio <= MAX_SETTLED_RATIO,
        "o frame FUGIU: mediana de {median:.2} ms em regime = {ratio:.2}x o overhead de \
         {OVERHEAD_MS} (teto {MAX_SETTLED_RATIO}x) — o controlador nao recuou"
    );
}

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
            ph2d_wet_paint::drying::drying_pass(g, &p, evap, rewet, bypass);
        }),
    ));
    out.push((
        "build_flow_field",
        time(g, &mut |g| {
            solver::build_flow_field(g, &p, grav[0], grav[1], bypass);
        }),
    ));
    out.push((
        "smooth_velocity",
        time(g, &mut |g| solver::smooth_velocity(g, &p)),
    ));
    out.push((
        "advect",
        time(g, &mut |g| {
            solver::advect(g, &p, grav[0], grav[1]);
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
        "    janela {rows} x {span} = {cells} celulas ({:.1}% da tela)\n    \
         ativas {live} ({:.1}% da janela) | COM AGUA {wet} ({:.1}% da janela)",
        100.0 * cells as f64 / (4096.0 * 4096.0),
        100.0 * live as f64 / cells as f64,
        100.0 * wet as f64 / cells as f64,
    );
    let total: f64 = out.iter().map(|(_, t)| t).sum();
    for (name, ms) in &out {
        println!(
            "    {name:<24} {ms:7.3} ms   ({:4.1}%)   {:5.1} ns/celula-da-janela",
            100.0 * ms / total,
            ms * 1e6 / cells as f64,
        );
    }
    println!("    {:<24} {total:7.3} ms", "SOMA dos passes");
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

#[path = "measure_wetpaint_probes.rs"]
mod measure_wetpaint_probes; // as MEDIÇÕES (#[ignore]) — irmãs dos gates, por LOC

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

/// **O custo do AA de ENTRADA** (o supersampling da silhueta por célula,
/// `grid_map::cell_subsamples`) — para a constante `MAX_AA` sair de uma medição
/// e não de um palpite (CLAUDE.md §0).
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_deposit_aa_costs() {
    const REPS: usize = 9;
    println!(
        "\n  O CARIMBO (move) POR RAZAO — o AA de entrada e {n}x{n} taps/celula\n",
        n = "n"
    );
    println!(
        "    {:>5}  {:>12}  {:>10}",
        "razao", "ms/move", "vs razao 1"
    );
    let mut base = 0.0f64;
    for ratio in [1u8, 2, 4, 8, 16, 30] {
        let mut t = wetted(4096, 100.0);
        t.set_wet_grid_ratio(f64::from(ratio));
        // Um traco longo; mede o MOVE (o carimbo), que e o `INPUT` do log.
        t.on_canvas_pointer(cp([300.0, 2000.0], PointerPhase::Down));
        let mut v = Vec::with_capacity(REPS);
        for k in 0..REPS {
            let x = 300.0 + 40.0 * (k + 1) as f32;
            let t0 = Instant::now();
            t.on_canvas_pointer(cp([x, 2000.0], PointerPhase::Move));
            v.push(t0.elapsed().as_secs_f64() * 1e3);
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([300.0 + 40.0 * REPS as f32, 2000.0], PointerPhase::Up));
        v.sort_by(f64::total_cmp);
        let med = v[v.len() / 2];
        if ratio == 1 {
            base = med;
        }
        println!(
            "    {ratio:>3}:1  {med:>11.3}ms  {:>9.2}x",
            med / base.max(1e-9)
        );
    }
    println!(
        "\n    Leitura: o carimbo e O(celulas do dab) = O(area/ratio^2), e o AA multiplica por\n    \
         min(ratio, MAX_AA)^2 — entao o produto dos dois e que decide."
    );
}
