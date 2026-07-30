//! **A sonda do `Flow Grid`** (plano 30 / doc 28 §5.42) — irmã do
//! [`super::measure_wetpaint_probes`], separada pelo teto de LOC.

use super::measure_wetpaint_tick::*;

/// **O que o `Flow Grid` compra, pela porta do PRODUTO** (plano 30 / doc 28 §5.42).
///
/// Irmã do [`super::measure_what_the_grid_ratio_buys`], e a diferença entre as
/// duas é a lição que esta wave pagou:
///
/// ⚠️ **A MEDIANA de passos avulsos MENTE aqui, e não mente lá.** O `Grid Size`
/// encolhe TODOS os passes na mesma proporção, então qualquer passo serve de
/// amostra. O `Flow Grid` encolhe **três** — e o maior deles,
/// `build_flow_field`, roda **1 frame em 4** (cadência do `sim_step_stage`).
/// A mediana de passos avulsos é, portanto, quase sempre um frame **SEM** o
/// passe que a wave existe para baratear: medido, ela reportava **1,04×** onde
/// o ciclo de 12 passos — que cobre as quatro cadências do motor (÷2 rebuild,
/// ÷3 project, ÷4 flow, ÷6 drying) — reporta **1,27×**.
///
/// ⚠️ E o número tem de sair da porta do ARTISTA (`on_canvas_pointer`), não do
/// `Engine` direto: a §5.40 mediu as duas fixtures "grandes" em **10,34 contra
/// 62,05 ms/passo**, seis vezes.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_flow_ratio_buys_in_the_product() {
    /// Amostras (mediana).
    const REPS: usize = 7;
    /// Passos por amostra: um CICLO de cadência completo.
    const CYCLE: usize = 12;
    println!("\n  O QUE O 'Flow Grid' COMPRA NA POCA DO PRODUTO (mediana de {REPS} ciclos)\n");
    println!("    Grid Size fica em 1 (pigmento na resolucao da tela) — so o fluxo engrossa.\n");
    let mut base = 0.0f64;
    for flow in [1u8, 2, 4, 8, 16] {
        const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
        let mut t = wetted(4096, 100.0);
        // As PORTAS do artista, antes do traço (elas encerram a sessão).
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
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        let e = &mut *sess.engine;
        let fcells = e.active_grid().flow.cells;
        // ⚠️ **O rebuild ANTES do snapshot, e ele carrega o peso todo:** o worker
        // roda `rebuild_active_region` como 1º estágio de todo passo, e sem ele a
        // máscara `active` do estado congelado está VAZIA — todo passe gateado
        // nela faz early-out em TODA célula, e a sonda mede um passo que o
        // produto nunca dá. É a fixture envenenada que o doc 28 §5.40 nomeia.
        ph2d_wet_paint::solver::rebuild_active_region(e.active_grid_mut());
        let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
            let t0 = Instant::now();
            for _ in 0..CYCLE {
                e.step_simulation();
            }
            v.push(t0.elapsed().as_secs_f64() * 1e3 / CYCLE as f64);
        }
        v.sort_by(f64::total_cmp);
        let med = v[v.len() / 2];
        if flow == 1 {
            base = med;
        }
        println!(
            "    Flow {flow:2}x  fluxo {fcells:>9} celulas  {med:8.3} ms/passo  -> {:6.1} Hz  [{:5.2}x]",
            1000.0 / med.max(1e-9),
            base / med.max(1e-9)
        );
    }
    println!(
        "\n    Leitura: o nominal da SPEC e 40 Hz (25 ms/passo). O `Grid Size` compra\n             \
         MUITO mais (ele encolhe tudo) e cobra o pigmento GROSSO; este slider\n             \
         compra menos e nao cobra a borda."
    );

    // Por PASSE, na MESMA poca -- para o ganho ser atribuido e nao suposto.
    println!("\n  POR PASSE, na poca do PRODUTO (mediana de {REPS}):\n");
    let mut cols: Vec<(u8, Vec<(String, f64)>)> = Vec::new();
    for flow in [1u8, 4] {
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
        let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
        let e = &mut *sess.engine;
        let p = e.sim.gather_params(&e.tuning);
        let grav = e.sim.gravity(&e.tuning);
        let evap = e.sim.evap_scale * p.k(ph2d_wet_paint::tuning::Knob::Evaporation);
        let rewet = e.sim.rewet_base * p.k(ph2d_wet_paint::tuning::Knob::Rewet);
        let g = e.active_grid_mut();
        // Ver acima: sem o rebuild o `active` está vazio e a decomposição mente.
        ph2d_wet_paint::solver::rebuild_active_region(g);
        let snap = ph2d_wet_paint::grid::snapshot_grid(g);
        let time = |g: &mut ph2d_wet_paint::grid::Grid,
                    f: &mut dyn FnMut(&mut ph2d_wet_paint::grid::Grid)| {
            let mut v = Vec::with_capacity(REPS);
            for _ in 0..REPS {
                ph2d_wet_paint::grid::restore_grid(g, &snap);
                let t0 = Instant::now();
                f(g);
                v.push(t0.elapsed().as_secs_f64() * 1e3);
            }
            v.sort_by(f64::total_cmp);
            v[v.len() / 2]
        };
        use ph2d_wet_paint::solver;
        let out = vec![
            (
                "build_flow_field".to_string(),
                time(g, &mut |g| {
                    solver::build_flow_field(g, &p, grav[0], grav[1], false)
                }),
            ),
            (
                "project".to_string(),
                time(g, &mut |g| solver::project(g, &p)),
            ),
            (
                "smooth_velocity".to_string(),
                time(g, &mut |g| solver::smooth_velocity(g, &p)),
            ),
            (
                "advect".to_string(),
                time(g, &mut |g| {
                    solver::advect(g, &p, grav[0], grav[1]);
                }),
            ),
            (
                "drying_pass".to_string(),
                time(g, &mut |g| {
                    ph2d_wet_paint::drying::drying_pass(g, &p, evap, rewet, false);
                }),
            ),
            (
                "rebuild_active_region".to_string(),
                time(g, &mut solver::rebuild_active_region),
            ),
        ];
        cols.push((flow, out));
    }
    println!("    passe                   Flow 1      Flow 4     razao   cadencia");
    let cad = ["/4", "/3", "x3/4", "x1", "/3", "/2"];
    let wgt = [0.25, 1.0 / 3.0, 0.75, 1.0, 1.0 / 3.0, 0.5];
    let (mut a_amort, mut b_amort) = (0.0f64, 0.0f64);
    for k in 0..cols[0].1.len() {
        let (n, a) = (&cols[0].1[k].0, cols[0].1[k].1);
        let b = cols[1].1[k].1;
        a_amort += a * wgt[k];
        b_amort += b * wgt[k];
        println!(
            "    {n:<22} {a:7.2} ms  {b:7.2} ms  {:6.2}x   {}",
            a / b.max(1e-9),
            cad[k]
        );
    }
    println!(
        "    {:<22} {a_amort:7.2} ms  {b_amort:7.2} ms  {:6.2}x   (soma amortizada)",
        "TOTAL",
        a_amort / b_amort.max(1e-9)
    );
}
