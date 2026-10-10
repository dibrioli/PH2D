//! **As medições da W1e** (filho de `offthread_tests` por LOC — vê os ajudantes dele): o quadro a pintar
//! com a água a correr no traço (F6) e o 1.º lote de uma sessão a 4096² (F2). Medições, nunca barras.
use super::*;

/// **O quadro A PINTAR, com a água parada × a correr no traço** (W1e, F6) — medição, nunca barra.
///
/// A mesma poça pesada (4096², três traços) em DUAS ferramentas lado a lado, uma por variante de
/// `sim_while_painting`; blocos de 20 quadros (um traço novo cada, 3 eventos de ponteiro + o tick por
/// quadro) intercalados em 7 rodadas de ordem rodada; o resumo é o MÍNIMO das rodadas, a mediana ao
/// lado (MEDIR_VELOCIDADE.md). O quadro é o que a thread da UI paga: os eventos (que trazem o motor) e o
/// tick (que o devolve).
#[test]
#[ignore = "medicao — rode com --profile smoke --ignored --nocapture --test-threads=1"]
fn measure_a_painting_frame_with_the_water_running() {
    // A 3.ª variante é o CONTROLO: a mesma poça com a água a correr e SEM pintar (o produto depois do Up).
    let mut ts = [heavy_puddle(), heavy_puddle(), heavy_puddle()];
    for (k, t) in ts.iter_mut().enumerate() {
        t.wet_bring_home();
        t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("sessao")
            .engine
            .sim
            .sim_while_painting = k == 1;
    }
    let bloco = |t: &mut PainterTool, y: f32, pinta: bool| -> (Vec<f32>, u64) {
        let s0 = sim_frame(t);
        let mut q = Vec::with_capacity(20);
        let x0 = 400.0f32;
        if pinta {
            t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
        }
        for f in 0..20u16 {
            let a = Instant::now();
            for m in 0..3u16 {
                if !pinta {
                    break;
                }
                let x = x0 + 12.0 * f32::from(f * 3 + m + 1);
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                let _ = t.take_preview_arc();
            }
            t.paint_tick(1.0 / 60.0);
            let spent = a.elapsed();
            q.push(spent.as_secs_f32() * 1e3);
            if let Some(rest) = Duration::from_micros(16_666).checked_sub(spent) {
                std::thread::sleep(rest);
            }
        }
        let fim = sim_frame(t);
        if pinta {
            t.on_canvas_pointer(cp([x0 + 720.0, y], PointerPhase::Up));
        }
        (q, fim - s0)
    };
    // Aquece as três fora da régua.
    for (k, t) in ts.iter_mut().enumerate() {
        let _ = bloco(t, 3000.0, k < 2);
    }
    let mut res: [Vec<(f32, f32, u64)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
    for r in 0..7usize {
        for i in 0..3 {
            let k = (r + i) % 3;
            let (mut q, passos) = bloco(&mut ts[k], 1200.0 + 250.0 * r as f32, k < 2);
            q.sort_by(f32::total_cmp);
            res[k].push((q[q.len() / 2], q[q.len() - 1], passos));
        }
    }
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let nomes = [
        "agua parada no traco",
        "agua a correr no traco",
        "agua a correr, sem pintar",
    ];
    for (k, nome) in nomes.iter().enumerate() {
        let mut p50: Vec<f32> = res[k].iter().map(|v| v.0).collect();
        let mut pior: Vec<f32> = res[k].iter().map(|v| v.1).collect();
        p50.sort_by(f32::total_cmp);
        pior.sort_by(f32::total_cmp);
        let passos: Vec<u64> = res[k].iter().map(|v| v.2).collect();
        println!(
            "[quadro a pintar] {nome:<24} p50 min {:.2} med {:.2} ms · pior min {:.2} med {:.2} ms · passos da sim em 20 quadros {passos:?}",
            p50[0], p50[3], pior[0], pior[3]
        );
    }
    println!("[quadro a pintar] load {}", carga.trim());
}

/// **O 1.º lote de uma sessão a 4096²** (W1e, F2) — medição, nunca barra: o nascimento + a semente do
/// papel (sem Kind = o Cold de fábrica do painel, pela lei do painel; antes era o ladrilho memorizado do
/// motor) contra um lote seguinte. O mínimo de 5 sessões.
#[test]
#[ignore = "medicao — rode com --profile smoke --ignored --nocapture --test-threads=1"]
fn measure_the_first_batch_of_a_session() {
    let (mut primeiro, mut seguinte) = (f32::INFINITY, f32::INFINITY);
    for _ in 0..5 {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 4096 * 4096 * 4], 4096, 4096);
        t.set_paint_media(PaintMedia::WetPaint);
        let a = Instant::now();
        t.on_canvas_pointer(cp([2000.0, 2000.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([2010.0, 2000.0], PointerPhase::Move));
        primeiro = primeiro.min(a.elapsed().as_secs_f32() * 1e3);
        let a = Instant::now();
        t.on_canvas_pointer(cp([2020.0, 2000.0], PointerPhase::Move));
        seguinte = seguinte.min(a.elapsed().as_secs_f32() * 1e3);
        t.on_canvas_pointer(cp([2020.0, 2000.0], PointerPhase::Up));
    }
    // As duas portas do papel, intercaladas sobre o MESMO motor: o re-assar do motor (o «antes») e a
    // semente pela lei do painel com o Cold de fábrica (o «agora»).
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 4096 * 4096 * 4], 4096, 4096);
    t.set_paint_media(PaintMedia::WetPaint);
    assert!(t.ensure_wet_session());
    let cold = crate::tool::paint::substrate_relief::papel_de_fabrica();
    let rot = ph2d_painter_brush::texture::angle_basis(cold.angle_deg);
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    let (mut assar, mut semear) = (f32::INFINITY, f32::INFINITY);
    for _ in 0..5 {
        let a = Instant::now();
        s.engine.rebake_paper();
        assar = assar.min(a.elapsed().as_secs_f32() * 1e3);
        let a = Instant::now();
        s.engine.seed_paper_with(&|cx, cy| {
            f64::from(ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                &cold,
                cx,
                cy,
                None,
                rot,
                [0.0, 0.0],
            ))
        });
        semear = semear.min(a.elapsed().as_secs_f32() * 1e3);
    }
    println!(
        "[primeiro lote] 4096²: o 1.º {primeiro:.1} ms · o seguinte {seguinte:.2} ms · o papel: re-assar o do \
         motor {assar:.1} ms · semear o do painel {semear:.1} ms · load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
