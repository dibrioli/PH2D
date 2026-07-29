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
        let (_, (comp_sum, _, comp_n), (wait_sum, wait_max, wait_n)) =
            crate::wet_diag::take_window();
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

#[path = "measure_wetpaint_probes.rs"]
mod measure_wetpaint_probes; // as MEDIÇÕES (#[ignore]) — irmãs dos gates, por LOC
