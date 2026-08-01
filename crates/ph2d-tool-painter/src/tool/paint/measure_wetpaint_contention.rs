//! **O QUE A MÁQUINA FAZ COM O PASSO** — irmão de
//! [`super::measure_wetpaint_step`] (de que o passo é FEITO), separado por
//! RESPONSABILIDADE quando o pai cruzou o teto de LOC.
//!
//! O corte é o assunto: lá se pergunta *quanto cada passe custa e o que os
//! sliders de grade fazem com ele*; aqui, *quanto do custo é do TRABALHO e
//! quanto é de a máquina estar disputada com a thread do frame*.
//!
//! ⚠️ **As três sondas nasceram de um log de smoke e derrubaram TRÊS hipóteses
//! minhas** (2026-07-31, doc 28 §5.47): o passo é limitado por NÚCLEO e por
//! BANDA (nunca só por banda) · encolher o pool do rayon **não** o protege ·
//! e a água **não** publica um retângulo maior que os outros meios. Nenhuma
//! delas sobreviveu à medição, e é por isso que elas ficam aqui em vez de
//! virarem otimização.

use super::*;

/// **POR QUE O PASSO CUSTA 2,2× ENQUANTO O ARTISTA PINTA** — a pergunta que o
/// log do smoke fez e que nenhuma sonda desta linha cobria.
///
/// Medido no produto (2026-07-31, `PH2D_FLUID_PROFILE`), três janelas de 2 s da
/// MESMA sessão: pintando, `sim media 45,52 ms x35` e a água a **17,7 Hz**; só
/// assistindo, `20,11 ms x70` a **35,0 Hz**. O tempo BUSY do worker é o mesmo
/// nas duas (1593 contra 1408 ms de 2000) — ele trabalha o mesmo e entrega
/// METADE dos passos.
///
/// ⚠️ **Duas explicações cabem nesse log e elas pedem curas OPOSTAS**, e é
/// exatamente a armadilha de atribuição que esta linha já pagou três vezes
/// (doc 28 §5.13, §5.40, §5.44):
///
/// 1. **mais TRABALHO** — pintar acrescenta células molhadas, então a poça da
///    janela 1 é maior que a das janelas 2-3, que já estão secando. Se for
///    isto, não há nada a consertar: o preço é honesto e o slider `Grid Size`
///    é a resposta que já existe.
/// 2. **CONTENÇÃO** — o solver é limitado por LARGURA DE BANDA sobre a faixa
///    viva (§5.46), e enquanto o artista pinta a thread do frame move um canvas
///    de 4096² por quadro (`painter-dispatch 6,90 ms`, contra 0,03 quando
///    ninguém pinta) pelo mesmo barramento e pelo mesmo pool do rayon. Se for
///    isto, o trabalho é o mesmo e o que dobrou foi o wall-clock.
///
/// O log **não distingue as duas** porque as duas metades mudam juntas nele. A
/// medição que distingue é esta: **congelar a poça** (mesmo estado, mesmo
/// número de células vivas, restaurado a cada amostra) e acrescentar carga por
/// fora, separando os dois recursos —
///
/// - **THREADS**: giro puro de ALU em `n` threads (quase zero tráfego de
///   memória) — se o passo dobrar aqui, quem falta é NÚCLEO;
/// - **BANDA**: um `copy_from_slice` serial sobre 64 MiB, UMA thread — se o
///   passo dobrar aqui, quem falta é BARRAMENTO;
/// - **AMBOS**: a cópia em paralelo, que é o que o dispatch de fato é.
///
/// A carga reporta a própria vazão, senão *"não mudou nada"* é indistinguível
/// de *"a carga não rodou"* ([[feedback_a_negative_search_needs_a_positive_control]]).
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_the_step_under_the_load_the_frame_puts_on_the_machine() {
    use std::sync::Arc as StdArc;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    const REPS: usize = 7;
    const CYCLE: usize = 12;
    const BUF_MIB: usize = 64;

    let cores = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);

    let mut t = heavy_puddle();
    t.wet_bring_home();
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    let e = &mut *sess.engine;
    ph2d_wet_paint::solver::rebuild_active_region(e.active_grid_mut());
    let live = e.active_grid().active.iter().filter(|a| **a != 0).count();
    let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
    let frame0 = e.sim.frame;

    // Um ciclo de CADÊNCIA por amostra (12 passos cobrem ÷2, ÷3, ÷4 e ÷6), o
    // estado restaurado antes de cada um — a metodologia das irmãs.
    let cycle = |e: &mut ph2d_wet_paint::painter::Engine| -> f64 {
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
            e.sim.frame = frame0;
            let t0 = Instant::now();
            for _ in 0..CYCLE {
                e.step_simulation();
            }
            v.push(t0.elapsed().as_secs_f64() * 1e3 / CYCLE as f64);
        }
        v.sort_by(f64::total_cmp);
        v[REPS / 2]
    };

    println!("\n  O PASSO SOB A CARGA QUE A THREAD DO FRAME POE NA MAQUINA\n");
    println!(
        "    poca CONGELADA: {live} celulas vivas, restaurada a cada amostra\n    \
         {cores} nucleos | mediana de {REPS} ciclos de {CYCLE} passos\n"
    );

    let mut base = 0.0f64;
    for (name, threads, copy) in [
        ("controle (nada rodando)", 0usize, false),
        ("THREADS  (giro de ALU)", cores, false),
        ("BANDA    (memcpy serial)", 1, true),
        // ⚠️ **A linha que separa as duas metades do confundimento.** Uma cópia
        // serial toma ~12% do barramento e 1 núcleo de 32; a paralela em TODOS
        // toma o barramento E os núcleos, e não distingue nada. Quatro threads
        // de cópia tomam MUITA banda e POUCO núcleo — se o passo mal se move
        // aqui, a frase *"o advect e limitado por LARGURA DE BANDA"* (doc 28
        // §5.46) esta ERRADA, e ela e a frase que fecha a frente de CPU.
        ("BANDA    (memcpy x4)", 4, true),
        // ⚠️ **O CONTROLE da linha acima, e sem ele ela não decide nada:** quatro
        // threads de cópia tomam banda E quatro núcleos. Se quatro threads de
        // ALU — zero banda, os MESMOS quatro núcleos — custarem o mesmo, então
        // os 16,8 GB/s custaram ZERO e o limite é de núcleo, ponto.
        ("nucleo   (ALU x4, controle)", 4, false),
        ("AMBOS    (memcpy paralelo)", cores, true),
    ] {
        let stop = StdArc::new(AtomicBool::new(false));
        let work = StdArc::new(AtomicU64::new(0));
        let mut hands = Vec::new();
        for _ in 0..threads {
            let (stop, work) = (StdArc::clone(&stop), StdArc::clone(&work));
            hands.push(std::thread::spawn(move || {
                if copy {
                    let src = vec![7u8; BUF_MIB << 20];
                    let mut dst = vec![0u8; BUF_MIB << 20];
                    while !stop.load(Ordering::Relaxed) {
                        dst.copy_from_slice(&src);
                        work.fetch_add(BUF_MIB as u64, Ordering::Relaxed);
                        std::hint::black_box(&dst);
                    }
                } else {
                    let mut x = 1u64;
                    let mut n = 0u64;
                    while !stop.load(Ordering::Relaxed) {
                        for _ in 0..1_000_000u32 {
                            x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                        }
                        n += 1;
                        std::hint::black_box(x);
                    }
                    work.fetch_add(n, Ordering::Relaxed);
                }
            }));
        }
        let t_all = Instant::now();
        let ms = cycle(e);
        let secs = t_all.elapsed().as_secs_f64();
        stop.store(true, Ordering::Release);
        for h in hands {
            let _ = h.join();
        }
        let did = work.load(Ordering::Relaxed);
        // A carga tem de se ANUNCIAR: sem isto, "nao mudou nada" e
        // indistinguivel de "a carga nao rodou".
        let load = if threads == 0 {
            String::from("--")
        } else if copy {
            format!("{:.1} GB/s", did as f64 / 1024.0 / secs.max(1e-9))
        } else {
            format!("{:.0} Mop/s", did as f64 / secs.max(1e-9))
        };
        if base == 0.0 {
            base = ms;
            println!(
                "    {name:<28} {ms:8.3} ms/passo  {:6.1} Hz   carga {load}",
                1000.0 / ms
            );
        } else {
            println!(
                "    {name:<28} {ms:8.3} ms/passo  {:6.1} Hz   carga {load}   [{:5.2}x]",
                1000.0 / ms,
                ms / base,
            );
        }
    }
    println!(
        "\n    Leitura: a poca e a MESMA nas quatro linhas, entao toda diferenca e CONTENCAO.\n    \
         Se so BANDA sobe, o solver disputa BARRAMENTO com o dispatch (a cura e mover\n    \
         menos bytes). Se so THREADS sobe, disputa NUCLEO (a cura e particionar o pool).\n    \
         Se nenhuma sobe, a janela 1 do log era mais TRABALHO e nao ha nada a consertar."
    );
}

/// **QUANTOS NÚCLEOS O SOLVER QUER, E QUANTO ELE PERDE QUANDO NÃO OS TEM.**
///
/// A sonda irmã acima mediu que o passo é limitado por **NÚCLEO** e não por
/// banda (1,09× sob memcpy contra **11,62×** sob giro de ALU). ⚠️ **E 11,62× é
/// patológico, não é partilha:** 32 threads a mais em 32 núcleos deveria dar
/// ~2× a cada lado. O fator que falta é a **BARREIRA** — um passe row-parallel
/// termina quando o ÚLTIMO chunk termina, então basta um worker preemptado para
/// segurar o passe inteiro, e um passo roda SETE passes.
///
/// Se a causa é essa, a cura não é "mais rápido": é **menos sensível**. Um pool
/// de `k` threads com `k < núcleos` quase nunca tem todos os workers preemptados
/// ao mesmo tempo, e deixa o resto da máquina para a thread do frame. O preço é
/// o passo isolado ficar mais lento.
///
/// A tabela abaixo é a decisão inteira: **custo isolado × degradação sob carga**,
/// por tamanho de pool. O ponto de operação certo é o que minimiza o custo SOB
/// CARGA, porque é nele que o artista pinta.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_how_many_cores_the_solver_wants() {
    use std::sync::Arc as StdArc;
    use std::sync::atomic::{AtomicBool, Ordering};

    const REPS: usize = 5;
    const CYCLE: usize = 12;

    let cores = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);

    let mut t = heavy_puddle();
    t.wet_bring_home();
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    let e = &mut *sess.engine;
    ph2d_wet_paint::solver::rebuild_active_region(e.active_grid_mut());
    let live = e.active_grid().active.iter().filter(|a| **a != 0).count();
    let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
    let frame0 = e.sim.frame;

    let cycle = |e: &mut ph2d_wet_paint::painter::Engine| -> f64 {
        let mut v = Vec::with_capacity(REPS);
        for _ in 0..REPS {
            ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
            e.sim.frame = frame0;
            let t0 = Instant::now();
            for _ in 0..CYCLE {
                e.step_simulation();
            }
            v.push(t0.elapsed().as_secs_f64() * 1e3 / CYCLE as f64);
        }
        v.sort_by(f64::total_cmp);
        v[REPS / 2]
    };

    println!("\n  QUANTOS NUCLEOS O SOLVER QUER (poca CONGELADA, {live} celulas vivas)\n");
    println!(
        "    {cores} nucleos | mediana de {REPS} ciclos de {CYCLE} passos\n    \
         a CARGA e o giro de ALU em {cores} threads: o pior caso, e o que separa\n    \
         'rapido sozinho' de 'rapido quando o artista esta pintando'\n"
    );
    println!(
        "    {:<12} {:>12} {:>10} {:>12} {:>10}  {:>8}",
        "pool", "sozinho", "Hz", "sob carga", "Hz", "perda"
    );

    // ⚠️ O pool GLOBAL é o que o produto usa hoje — ele entra como a 1ª linha,
    // e é contra ele que toda a tabela quer dizer alguma coisa.
    for k in [0usize, cores / 2, cores / 4, cores / 8, 4] {
        let pool = (k > 0).then(|| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(k)
                .build()
                .expect("pool")
        });
        let run = |e: &mut ph2d_wet_paint::painter::Engine| match &pool {
            Some(p) => p.install(|| cycle(e)),
            None => cycle(e),
        };
        let alone = run(e);

        let stop = StdArc::new(AtomicBool::new(false));
        let mut hands = Vec::new();
        for _ in 0..cores {
            let stop = StdArc::clone(&stop);
            hands.push(std::thread::spawn(move || {
                let mut x = 1u64;
                while !stop.load(Ordering::Relaxed) {
                    for _ in 0..1_000_000u32 {
                        x = x.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    }
                    std::hint::black_box(x);
                }
            }));
        }
        let loaded = run(e);
        stop.store(true, Ordering::Release);
        for h in hands {
            let _ = h.join();
        }

        let name = if k == 0 {
            format!("global ({cores})")
        } else {
            format!("{k}")
        };
        println!(
            "    {name:<12} {alone:>9.3} ms {:>9.1} {loaded:>9.3} ms {:>9.1}  {:>7.2}x",
            1000.0 / alone,
            1000.0 / loaded,
            loaded / alone,
        );
    }
    println!(
        "\n    Leitura: a coluna SOB CARGA e a que o artista sente. O nominal da SPEC e\n    \
         40 Hz (25 ms/passo) — passar disso sozinho nao compra nada (o worker dorme),\n    \
         entao o pool certo e o menor que ainda alcanca 40 Hz SOB CARGA."
    );
}

/// **O QUE O FRAME RE-ENVIA POR QUADRO ENQUANTO O ARTISTA PINTA** — o outro
/// lado da conta, e hoje o maior item do lado do frame.
///
/// A tabela acima fecha o solver: sozinho ele roda a **70 Hz** contra 40 de
/// nominal, e nenhum tamanho de pool o protege da contenção. O que sobra é
/// **quanto da máquina o FRAME toma enquanto o artista pinta**, e o log do
/// produto nomeia o item: `painter-dispatch(cpu)` mede **6,90 ms** pintando
/// contra **0,03 ms** só assistindo.
///
/// O dispatch é proporcional ao RETÂNGULO que o tool declara sujo (a pista de
/// GPU re-envia exatamente esse sub-rect da camada), então a pergunta é
/// medível aqui, sem janela: **de que tamanho é o retângulo que cada meio
/// publica por evento de ponteiro?** Um dab de pincel tem a pegada do pincel;
/// a água tem a região por onde ela ESCORRE, que cresce sozinha.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_frame_re_uploads_while_the_artist_paints() {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    const SIDE: u32 = 4096;

    println!("\n  O RETANGULO QUE O TOOL DECLARA SUJO, POR EVENTO DE PONTEIRO ({SIDE}x{SIDE})\n");
    println!(
        "    {:<14} {:>10} {:>12} {:>10} {:>12} {:>6} {:>10}",
        "meio", "eventos", "mediana px", "pior px", "pior % tela", "FULLs", "telas/traco"
    );

    let screen = f64::from(SIDE) * f64::from(SIDE);
    for media in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let mut t = wetted(SIDE, 100.0);
        t.set_paint_media(media);
        let mut areas: Vec<u64> = Vec::new();
        // ⚠️ **TRÊS traços, e é por isso:** um full-canvas no PRIMEIRO composite
        // de uma sessão é legítimo (não existe cache anterior a remendar); um
        // POR TRAÇO é custo estrutural. Com um traço só as duas leituras são
        // indistinguíveis — a fixture tem de conter a diferença.
        let mut full_at: Vec<usize> = Vec::new();
        for lane in 0..3u32 {
            let (x0, y0) = (200.0f32 + 260.0 * lane as f32, 200.0f32);
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            for k in 1..=90u32 {
                let d = 40.0 * k as f32;
                t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Move));
                // O que a pista de GPU do shell de fato re-envia neste frame.
                if t.take_preview_dirty()
                    && let Some((_, _, w, h)) = t.preview_gpu_region()
                {
                    let a = u64::from(w) * u64::from(h);
                    if a as f64 >= screen {
                        full_at.push(areas.len());
                    }
                    areas.push(a);
                }
                // A água só publica região nova quando o worker entrega um passo —
                // o tick do produto é quem composita, então ele entra aqui também.
                t.wetpaint_tick(1.0 / 60.0);
            }
            let d = 40.0 * 90.0;
            t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Up));
        }
        if !full_at.is_empty() {
            println!(
                "      ^ tela-inteira nos publishes {full_at:?} (de {})",
                areas.len()
            );
        }
        // ⚠️ **A CONTAGEM de tela-inteira é a pergunta, não o pior caso.** Um
        // único full-canvas é o 1º composite de uma sessão (legítimo: não há
        // cache anterior); um POR QUADRO é o custo estrutural que o log mede.
        let fulls = areas.iter().filter(|a| **a as f64 >= screen).count();
        areas.sort_unstable();
        let (med, worst) = (
            areas.get(areas.len() / 2).copied().unwrap_or(0),
            areas.last().copied().unwrap_or(0),
        );
        let total: u64 = areas.iter().sum();
        println!(
            "    {:<14} {:>10} {med:>12} {worst:>10} {:>11.1}%  {fulls:>5}  {:>9.1}",
            format!("{media:?}"),
            areas.len(),
            100.0 * worst as f64 / screen,
            total as f64 / screen,
        );
    }
    println!(
        "\n    Leitura: o dispatch do shell e proporcional a esta area. Se a agua publica\n    \
         um retangulo muito maior que um dab, o custo do frame durante a pintura e\n    \
         estrutural — e o alvo e o retangulo, nao o solver."
    );
}

#[cfg(test)]
#[path = "measure_wetpaint_stamp.rs"]
mod measure_wetpaint_stamp; // o que o CARIMBO custa - irmao por LOC
