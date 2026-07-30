//! **De que são feitos os PASSES QUENTES da sim** — a sonda que abre cada frente
//! aberta pela anterior (doc 28 §5.43).
//!
//! A ordem em que ela cresceu é o método: com o `build_flow_field` 20,49× mais
//! barato (§5.42) a **secagem** virou o maior item do passo; com a secagem
//! 1,43× mais barata (§5.43), o **`advect`** virou 70% dele. Cada frente é
//! medida aqui antes de qualquer hipótese sobre ela.
//!
//! ⚠️ Ela é a irmã de *"de que isto é FEITO"*, não a de *"o que o produto
//! PAGA"* — a distinção que o doc 28 §4.8.2 pagou: uma sonda que disseca com
//! laço próprio fica **cega à porta**, então a atribuição sai daqui e o
//! veredito de produto sai da `measure_wetpaint_flow`.
//!
//! ⚠️ E a fixture é a do PRODUTO, com o `rebuild_active_region` antes do
//! snapshot — sem ele a máscara `active` do estado congelado está vazia e a
//! decomposição mede um passo que o produto nunca dá (§5.40).

use super::measure_wetpaint_tick::*;

/// A poça do produto, congelada — a MESMA construção das sondas irmãs.
fn puddle() -> (
    crate::tool::PainterTool,
    ph2d_wet_paint::sim::Params,
    f64,
    f64,
) {
    puddle_at(1)
}

/// A mesma poça, com a razão de fluxo escolhida.
fn puddle_at(
    flow: u8,
) -> (
    crate::tool::PainterTool,
    ph2d_wet_paint::sim::Params,
    f64,
    f64,
) {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let mut t = wetted(4096, 100.0);
    t.set_wet_grid_ratio(1.0);
    t.set_wet_flow_ratio(f64::from(flow));
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
    let (p, evap, rewet) = {
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        let e = &mut *sess.engine;
        let p = e.sim.gather_params(&e.tuning);
        let evap = e.sim.evap_scale * p.k(ph2d_wet_paint::tuning::Knob::Evaporation);
        let rewet = e.sim.rewet_base * p.k(ph2d_wet_paint::tuning::Knob::Rewet);
        ph2d_wet_paint::solver::rebuild_active_region(e.active_grid_mut());
        (p, evap, rewet)
    };
    (t, p, evap, rewet)
}

/// **A FORMA antes do relógio** — quantas células o laço VISITA, quantas
/// TRABALHAM, e quantas de fato mudam alguma coisa.
///
/// A ordem é deliberada: uma pergunta de forma custa nada e pode dissolver a de
/// custo (foi ela que eliminou o composite como suspeito na §5.12).
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_drying_pass_visits() {
    let (mut t, _p, _e, _r) = puddle();
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    let g = sess.engine.active_grid();
    let s = g.s;

    let (mut span, mut worked, mut edged, mut settled, mut rewetted, mut pigment) =
        (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    for y in g.by0..=g.by1 {
        let (bx0, bx1) = g.span_x(y);
        if bx0 > bx1 {
            continue;
        }
        span += (bx1 - bx0 + 1) as u64;
        let base = y as usize * s;
        for x in bx0 as usize..=bx1 as usize {
            let i = base + x;
            let (film, susp, sett) = (g.film[i], g.susp[i], g.sett[i]);
            if film > 0.0 || susp > 0.0 {
                worked += 1;
                if film > 0.0 && sett < 1000.0 {
                    edged += 1;
                }
                if susp > 0.0 {
                    settled += 1;
                }
                if film > 0.0 && sett > 0.0 {
                    rewetted += 1;
                }
                if susp > 0.0 || sett > 0.0 {
                    pigment += 1;
                }
            }
        }
    }
    let cells = (g.w * g.h) as u64;
    let pct = |n: u64| 100.0 * n as f64 / span.max(1) as f64;
    println!("\n  O QUE O `drying_pass` VISITA (poca do produto, 4096x4096)\n");
    println!("    grade                {cells:>10} celulas");
    println!(
        "    faixa viva (o laco)  {span:>10}  ({:.1}% da grade)",
        100.0 * span as f64 / cells as f64
    );
    println!(
        "    trabalham            {worked:>10}  ({:.1}% da faixa)",
        pct(worked)
    );
    println!("    -> gather 3x3        {edged:>10}  ({:.1}%)", pct(edged));
    println!(
        "    -> settle            {settled:>10}  ({:.1}%)",
        pct(settled)
    );
    println!(
        "    -> re-wet            {rewetted:>10}  ({:.1}%)",
        pct(rewetted)
    );
    println!(
        "    tem PIGMENTO         {pigment:>10}  ({:.1}%)",
        pct(pigment)
    );
    println!(
        "\n    Leitura: as duas cores (`susp_rgb`/`sett_rgb`) sao 24 B lidos e 24 B\n             \
         escritos por celula TRABALHADA, e so servem a settle/re-wet."
    );
}

/// **O custo do passe inteiro, pela porta** — o número que a varredura de
/// ablação do produto compara.
///
/// ⚠️ **A decomposição NÃO mora aqui, e isso é uma lição paga.** A primeira
/// versão desta sonda dissecava o passe com laços PRÓPRIOS (só a varredura ·
/// varredura + gather · varredura + tocar as duas cores) e reportou **2,97 ·
/// 5,93 · 2,93 ms contra um passe de 47,0** — números que não reconciliam com
/// o total, porque o LLVM **apagou** duas delas: ler `susp_rgb[i]` e escrever
/// o mesmo valor de volta é morto, e o compilador remove. *Uma ablação que o
/// otimizador pode provar inútil mede zero e parece um achado.*
///
/// A atribuição veio de cortar o **PRODUTO** peça por peça e medir por esta
/// porta (doc 28 §5.43): `alpha_of_mass` **13,2 ms**, o gather 3×3 ~**11,6**,
/// o bloco de settle o resto.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_a_drying_pass_is_made_of() {
    const REPS: usize = 7;
    let (mut t, p, evap, rewet) = puddle();
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    let g = sess.engine.active_grid_mut();
    let snap = ph2d_wet_paint::grid::snapshot_grid(g);
    let mut v = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        ph2d_wet_paint::grid::restore_grid(g, &snap);
        let t0 = Instant::now();
        ph2d_wet_paint::drying::drying_pass(g, &p, evap, rewet, false);
        v.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    v.sort_by(f64::total_cmp);
    println!("\n  DE QUE O `drying_pass` E FEITO (mediana de {REPS}, poca do produto)\n");
    println!("    passe inteiro (a porta)      {:8.3} ms", v[v.len() / 2]);
    println!(
        "\n    Corte uma peca do PRODUTO e re-rode: a diferenca e a atribuicao.\n\
         \x20   Um laco proprio aqui seria apagado pelo otimizador (ver o doc acima)."
    );
}

/// **O suspeito, medido isolado:** `alpha_of_mass` chama `to_int32_wrapping`,
/// que faz `trunc().rem_euclid(2^32)` — e `%` em `f64` é uma chamada a `fmod`
/// da libm, não uma instrução.
///
/// ⚠️ A comparação é contra a versão que a MESMA função computaria no domínio
/// que de fato ocorre (`0 <= m < 3000`), onde ToInt32 **é** truncamento — não
/// contra uma aproximação.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_opacity_lookup_costs() {
    use ph2d_wet_paint::opacity::alpha_of_mass;
    const N: usize = 2_600_000; // ~2 chamadas por celula trabalhada da poca
    let masses: Vec<f64> = (0..N).map(|k| (k % 2900) as f64 + 0.5).collect();
    let mut best_door = f64::MAX;
    let mut best_trunc = f64::MAX;
    for _ in 0..7 {
        let t0 = Instant::now();
        let mut acc = 0.0f64;
        for &m in &masses {
            acc += alpha_of_mass(m);
        }
        std::hint::black_box(acc);
        best_door = best_door.min(t0.elapsed().as_secs_f64() * 1e3);

        let t0 = Instant::now();
        let mut acc = 0.0f64;
        for &m in &masses {
            // O que a porta computa neste dominio, sem o `fmod`.
            acc += ph2d_wet_paint::opacity::table_at(m as i32);
        }
        std::hint::black_box(acc);
        best_trunc = best_trunc.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    println!("\n  O CUSTO DA CONSULTA DE OPACIDADE ({N} chamadas)\n");
    println!(
        "    alpha_of_mass (a porta de hoje)   {best_door:8.3} ms   {:5.2} ns/chamada",
        best_door * 1e6 / N as f64
    );
    println!(
        "    a MESMA resposta sem o `fmod`     {best_trunc:8.3} ms   {:5.2} ns/chamada",
        best_trunc * 1e6 / N as f64
    );
    println!(
        "    razao                             {:8.2}x",
        best_door / best_trunc.max(1e-9)
    );
}

/// **O custo do `advect`, pela porta** — a frente que a §5.43 abriu (70% do
/// passo). Irmã exata da sonda da secagem: o número que a varredura de ablação
/// do produto compara.
///
/// Mede nas DUAS razões de fluxo porque o `advect` é o único passe que fica
/// mais CARO com o fluxo grosso (a amostragem bilinear do bloco), então uma
/// medida só esconderia metade da resposta.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_an_advect_is_made_of() {
    const REPS: usize = 7;
    println!("\n  DE QUE O `advect` E FEITO (mediana de {REPS}, poca do produto)\n");
    for flow in [1u8, 4] {
        let (mut t, p, _e, _r) = puddle_at(flow);
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        let grav = sess.engine.sim.gravity(&sess.engine.tuning);
        let g = sess.engine.active_grid_mut();
        let snap = ph2d_wet_paint::grid::snapshot_grid(g);
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            ph2d_wet_paint::grid::restore_grid(g, &snap);
            let t0 = Instant::now();
            ph2d_wet_paint::solver::advect(g, &p, grav[0], grav[1]);
            v.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        v.sort_by(f64::total_cmp);
        println!("    Flow {flow}x   advect  {:8.3} ms", v[v.len() / 2]);
    }
    println!(
        "\n    Corte uma peca do PRODUTO e re-rode: a diferenca e a atribuicao.\n\
         \x20   (a §5.43 pagou a licao de que um laco proprio aqui e apagado)"
    );
}

// ---------------------------------------------------------------------------
// O GATHER (doc 28 §5.45) — quanto ele CUSTA e quanto ele MUDA
// ---------------------------------------------------------------------------

/// Os três planos que decidem a aparência, copiados para fora.
fn planes(t: &mut crate::tool::PainterTool) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    let g = sess.engine.active_grid();
    (g.film.clone(), g.susp.clone(), g.sett.clone())
}

/// Roda `steps` passos a partir do estado congelado, na rota pedida.
fn run_from(
    t: &mut crate::tool::PainterTool,
    snap: &ph2d_wet_paint::grid::GridSnapshot,
    gather: bool,
    steps: usize,
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    {
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        let e = &mut *sess.engine;
        e.sim.frame = 0;
        e.sim.dry_every = 6;
        e.sim.evap_scale = 0.001;
        e.sim.rewet_base = 0.0001;
        e.sim.order_invariant = gather;
        ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), snap);
    }
    for _ in 0..steps {
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        sess.engine.step_simulation();
    }
    planes(t)
}

/// **O QUE O GATHER MUDA** — a pergunta que decide o ADR-0146.
///
/// A reformulação de Jacobi é, por construção, um SEGUNDO MODELO (o serial é
/// Gauss-Seidel). A pergunta não é *"é byte-idêntico?"* — sabemos que não é —,
/// é ***quanto*** ele difere, e a régua tem de ser a que o olho usa: o **byte
/// de opacidade**, não a massa crua.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_how_far_the_gather_advect_drifts_from_the_reference() {
    use ph2d_wet_paint::opacity::alpha_of_mass;
    let (mut t, _p, _e, _r) = puddle();
    let snap = {
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        ph2d_wet_paint::grid::snapshot_grid(sess.engine.active_grid_mut())
    };
    println!("\n  O QUE O GATHER MUDA (poca do produto, 4096x4096)\n");
    println!(
        "    {:>6}  {:>12} {:>12}  {:>9} {:>9} {:>9}",
        "passos", "massa serial", "massa gather", "max byte", ">1 byte", ">4 bytes"
    );
    // A régua honesta: comparar o VÃO ENTRE MODELOS com o vão que o modelo
    // serial abre contra SI MESMO em UM passo a mais. Se o primeiro é menor
    // que o segundo, os dois quadros diferem por menos do que a água muda
    // enquanto o artista pisca — e nenhum número de "% de células" diz isso.
    let alpha = |s: &[f32], t: &[f32], i: usize| -> f64 {
        alpha_of_mass(f64::from(s[i])) + alpha_of_mass(f64::from(t[i]))
    };
    let compare = |a: &(Vec<f32>, Vec<f32>), b: &(Vec<f32>, Vec<f32>)| -> (i32, f64, u64) {
        let (mut max_b, mut sum, mut n) = (0i32, 0.0f64, 0u64);
        for i in 0..a.0.len() {
            let (aa, bb) = (alpha(&a.0, &a.1, i), alpha(&b.0, &b.1, i));
            if aa <= 0.0 && bb <= 0.0 {
                continue;
            }
            n += 1;
            let d = (aa - bb).abs() * 255.0;
            sum += d;
            let d = d.round() as i32;
            if d > max_b {
                max_b = d;
            }
        }
        (max_b, sum / n.max(1) as f64, n)
    };
    println!(
        "    {:>6}  {:>13} {:>13}  {:>9} {:>9}  {:>7}",
        "passos", "massa serial", "massa gather", "MODELOxMODELO", "1 PASSO", "razao"
    );
    for steps in [1usize, 10, 40] {
        let (fa, sa, ta) = run_from(&mut t, &snap, false, steps);
        let a = (sa, ta);
        let (_, sb, tb) = run_from(&mut t, &snap, true, steps);
        let b = (sb, tb);
        // O controle: o MESMO modelo serial, um passo adiante.
        let (_, sn, tn) = run_from(&mut t, &snap, false, steps + 1);
        let nxt = (sn, tn);
        let mass = |p: &(Vec<f32>, Vec<f32>)| -> f64 {
            p.0.iter().map(|&v| f64::from(v)).sum::<f64>()
                + p.1.iter().map(|&v| f64::from(v)).sum::<f64>()
        };
        let (gap_max, gap_mean, _) = compare(&a, &b);
        let (step_max, step_mean, _) = compare(&a, &nxt);
        println!(
            "    {steps:>6}  {:>13.1} {:>13.1}  {:>4}/{:>6.3} {:>4}/{:>6.3}  {:>6.2}x",
            mass(&a),
            mass(&b),
            gap_max,
            gap_mean,
            step_max,
            step_mean,
            gap_mean / step_mean.max(1e-9)
        );
        let _ = &fa;
    }
    println!(
        "\n    Cada celula e `max/media` do |delta| em BYTES sobre as celulas com\n\
         \x20   tinta. `razao` < 1 significa: os dois MODELOS estao mais perto um do\n\
         \x20   outro do que o modelo serial esta de si mesmo um passo adiante."
    );
}

/// **O que o gather CUSTA**, pelas três rotas, na mesma poça e no mesmo
/// processo (a lição do ADR-0145 §4: uma soma cross-run atribui deriva de
/// máquina ao ganho).
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_gather_advect_costs() {
    use ph2d_wet_paint::par::Rows;
    const REPS: usize = 7;
    println!("\n  O CUSTO DO `advect` PELAS TRES ROTAS (mediana de {REPS})\n");
    println!(
        "    {:>6}  {:>12} {:>14} {:>14}  {:>8}",
        "flow", "serial (GS)", "gather serial", "gather par", "ganho"
    );
    for flow in [1u8, 4] {
        let (mut t, p, _e, _r) = puddle_at(flow);
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        let grav = sess.engine.sim.gravity(&sess.engine.tuning);
        let g = sess.engine.active_grid_mut();
        let snap = ph2d_wet_paint::grid::snapshot_grid(g);
        let mut med = |which: u8| -> f64 {
            let mut v = Vec::with_capacity(REPS);
            for _ in 0..REPS {
                ph2d_wet_paint::grid::restore_grid(g, &snap);
                let t0 = Instant::now();
                match which {
                    0 => {
                        ph2d_wet_paint::solver::advect(g, &p, grav[0], grav[1]);
                    }
                    1 => {
                        ph2d_wet_paint::solver::advect_jacobi_rows(
                            g,
                            &p,
                            grav[0],
                            grav[1],
                            Rows::Serial,
                        );
                    }
                    _ => {
                        ph2d_wet_paint::solver::advect_jacobi_rows(
                            g,
                            &p,
                            grav[0],
                            grav[1],
                            Rows::Parallel,
                        );
                    }
                }
                v.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        let (a, b, c) = (med(0), med(1), med(2));
        println!(
            "    {flow:>6}  {a:>12.3} {b:>14.3} {c:>14.3}  {:>7.2}x",
            a / c
        );
    }
}
