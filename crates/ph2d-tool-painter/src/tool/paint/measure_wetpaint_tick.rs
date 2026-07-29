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
fn sim_frame(t: &PainterTool) -> u64 {
    t.paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessao de agua existe apos o traco")
        .engine
        .sim
        .frame
}

/// **A ÁGUA GASTA A FOLGA OCIOSA DO FRAME** — a propriedade que o smoke de 2026-07-29 exigiu.
///
/// O log do Enio: `total=16.03ms (~62 fps)` com **`present/acquire-stall=11.71ms`** — a CPU parada
/// 70% do quadro esperando o vsync — e a água a **6 Hz**. Um orçamento FIXO de 4 ms num frame com
/// 12 ms de folga não protege nada: ele deixa o hardware parado (§0: *o teto é o do HARDWARE*).
///
/// A fixture reproduz exatamente esse regime: `dt` **pinado em 16,6** seja qual for o custo do tick,
/// que é o que o vsync faz. O controlador tem de descobrir a folga e subir.
#[test]
fn the_wet_sim_spends_the_frames_idle_slack() {
    const FRAMES: usize = 60;
    /// **O oráculo do MECANISMO é o estado do controlador, não o relógio.** Ele é
    /// perfeitamente determinístico: sob `dt` pinado o orçamento TEM de subir
    /// acima da semente. Duas versões anteriores deste gate mediram Hz absolutos
    /// e depois uma razão entre janelas, e as duas reprovaram sob a suíte cheia
    /// (25,5 Hz contra piso 28; 1,21× contra piso 1,3) — *um gate cujo oráculo se
    /// dissolve quando a máquina está carregada será silenciado em vez de
    /// acreditado*.
    const MIN_GROWTH: f32 = 1.5;
    /// E o CONTROLE, comportamental e DELIBERADAMENTE generoso: o orçamento maior
    /// tem de comprar tempo de sim de verdade. Medido 36-38 Hz numa máquina livre
    /// e 25 sob a suíte inteira. O piso é folgado de propósito — quem afirma o
    /// mecanismo é a asserção de ESTADO acima, e *um gate que flaka é um gate
    /// silenciado*.
    const MIN_SIM_HZ: f64 = 10.0;

    for diagonal in [false, true] {
        let forma = if diagonal { "diagonal" } else { "horizontal" };
        let mut t = big_puddle(diagonal);
        let before = sim_frame(&t);
        for _ in 0..FRAMES {
            // `dt` PINADO: é o que o vsync entrega, e é por isso que o `dt`
            // sozinho não revela a folga — o controlador a descobre gastando.
            ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
            let _ = t.take_preview_arc();
        }
        let budget = t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("a sessao de agua existe apos o traco")
            .budget
            .per_frame_ms;
        let seed = super::wetpaint::budget::SimBudget::SEED_MS;
        assert!(
            budget >= seed * MIN_GROWTH,
            "{forma}: o orcamento ficou em {budget:.2} ms depois de {FRAMES} frames com folga \
             ociosa (semente {seed}, piso {:.1}) — o controlador nao achou a folga",
            seed * MIN_GROWTH
        );
        // ⚠️ O CONTROLE é wall-clock, e um passo de sim é ~16× mais lento em
        // debug — ali ele mediria o PERFIL, não o controlador. A afirmação do
        // MECANISMO acima é de ESTADO e roda nos dois perfis, que é o ponto de
        // ela existir.
        if cfg!(debug_assertions) {
            continue;
        }
        let hz = (sim_frame(&t) - before) as f64 / (FRAMES as f64 / 60.0);
        assert!(
            hz >= MIN_SIM_HZ,
            "{forma}: controle — o orcamento subiu para {budget:.2} ms mas a sim ficou em \
             {hz:.1} Hz (piso {MIN_SIM_HZ})"
        );
    }
}

/// **UM FRAME LENTO POR CULPA DE OUTRO INQUILINO NÃO ESTRANGULA A ÁGUA** — a atribuição, e o
/// segundo smoke que a exigiu.
///
/// Enio, 2026-07-29 (a segunda vez com o MESMO sintoma): 60 fps e a simulação parada, com o log
/// dizendo de quem era a conta —
///
/// ```text
/// [frame] total=19.15ms | stamps=13.96ms  | tool-tick=0.00ms
/// [frame] total=32.90ms | stamps=116.03ms | tool-tick=0.00ms
/// ```
///
/// O `stamps` é o carimbo de dabs dentro do `on_canvas_pointer` — **outro inquilino do frame**. O
/// controlador lia o `dt` inteiro, via *"não há espaço"* e estrangulava a sim. Aqui o `dt` fica
/// lento por um custo que a água **não** produz, e o orçamento tem de SEGURAR.
#[test]
fn a_frame_slowed_by_another_tenant_does_not_starve_the_water() {
    /// O custo do inquilino ESTRANGEIRO por frame — o `stamps` do log, bem acima
    /// do vsync.
    const FOREIGN_MS: f64 = 60.0;
    const FRAMES: usize = 80;
    /// O orçamento tem de continuar acima da semente: o inquilino estrangeiro é
    /// caro, mas a culpa não é da água. Sem a atribuição ele desaba no piso
    /// (`WET_BUDGET_MIN_MS`, 1 ms) e a sim vai a ~2 Hz — o sintoma reportado.
    const MIN_BUDGET_MS: f32 = 3.0;
    /// E o teto: a régua é o PISO do `dt` (o vsync, ~16,6), então o orçamento fica
    /// preso ao próprio período por mais lento que o frame fique. Com uma média no
    /// lugar do piso ele sobe para ~60.
    const MAX_BUDGET_MS: f32 = 22.0;

    let mut t = big_puddle(true);
    // Aquece com frames NORMAIS para o controlador medir o período do vsync.
    for _ in 0..40 {
        ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
        let _ = t.take_preview_arc();
    }
    // Agora todo frame chega lento — e a água não é a causa.
    //
    // ⚠️ O `dt` é REALIMENTADO (`estrangeiro + o que o tick custou`), não um número
    // fixo: a primeira versão deste gate passava `dt = 60` enquanto o próprio tick
    // custava ~40, o que descreve um frame impossível — e nele o `non_sim` caía
    // abaixo do alvo, o recuo disparava com razão e o gate reprovava sobre produto
    // correto. *Uma fixture que não fecha a própria conta acusa o código errado.*
    let mut dt = FOREIGN_MS;
    for _ in 0..FRAMES {
        let t0 = Instant::now();
        ph2d_editor_core::tool::Tool::on_tick(&mut t, dt as f32);
        dt = FOREIGN_MS + t0.elapsed().as_secs_f64() * 1e3;
        let _ = t.take_preview_arc();
    }
    let budget = t
        .paint
        .wetpaint
        .session
        .as_ref()
        .expect("a sessao de agua existe apos o traco")
        .budget
        .per_frame_ms;
    assert!(
        budget >= MIN_BUDGET_MS,
        "o orcamento desabou para {budget:.2} ms sob um frame lento por culpa de OUTRO inquilino \
         (piso {MIN_BUDGET_MS}) — o controlador punia a agua por uma conta que nao era dela"
    );
    // ⚠️ **E a outra metade, que é a do PISO do período:** a régua é o `min` do
    // `dt` (o intervalo do vsync), não uma média dele. Com uma média, um frame
    // lento por culpa alheia LEVANTA a régua — o teto vira `0,6 × 100 ms` e a
    // água passa a ter licença para comer 60 ms de quadro. As duas metades vivem
    // no mesmo gate porque é a mesma fixture que as separa das outras camadas.
    assert!(
        budget <= MAX_BUDGET_MS,
        "o orcamento CRESCEU para {budget:.2} ms porque o app ficou lento (teto {MAX_BUDGET_MS}) — \
         a regua do periodo deixou de ser o piso do `dt`"
    );
}

/// **E NUMA POÇA QUE A SIM NÃO ALCANÇA, O TETO SEGURA O FRAME** — a camada que o gate da folga
/// não consegue ver.
///
/// ⚠️ **Este gate existe porque a mutação do teto SOBREVIVEU ao gate da folga**, e o motivo é
/// estrutural: numa poça leve a sim já alcança os 40 Hz (o `acc` é quem a limita, não o orçamento),
/// então um teto infinito não muda um milissegundo — *verde por a fixture não conter o fenômeno*.
///
/// Aqui a poça é PESADA (três traços sobrepostos): a sim fica em ~9,5 Hz, o passo custa muito mais
/// que o teto, e é o teto que decide de quantos em quantos frames ela roda. Medido: **11,75 ms/frame
/// médios com o teto**; sem ele o tick passa a rodar o passo em TODO frame.
#[test]
fn the_budget_ceiling_holds_the_frame_when_the_sim_cannot_keep_up() {
    const FRAMES: usize = 120;
    /// **O oráculo primário é a TAXA da sim, e ele é robusto à carga na direção
    /// certa:** com o teto a sim é ESTRANGULADA (9,5 Hz medidos) e a carga só a
    /// abaixa mais; sem o teto ela roda no limite do `acc` (~40 Hz) e a carga não
    /// a levanta. Um wall-clock aqui inverteria sob suíte cheia.
    const MAX_THROTTLED_HZ: f64 = 22.0;
    /// A consequência em ms, com folga para máquina carregada: 11,75 medidos com
    /// o teto contra **37,74 sem ele**.
    const MAX_MEAN_MS: f64 = 25.0;

    // ⚠️ Gate de PERF: um passo de sim é ~16× mais lento em debug e o número
    // mediria o perfil de compilação. O mecanismo que ele vigia (o teto do
    // orçamento) é o mesmo nos dois; o que não sobrevive ao debug é o relógio.
    if cfg!(debug_assertions) {
        return;
    }
    let mut t = heavy_puddle();
    for _ in 0..10 {
        ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
        let _ = t.take_preview_arc();
    }
    let mut total = 0.0f64;
    let before = sim_frame(&t);
    for _ in 0..FRAMES {
        let t0 = Instant::now();
        // `dt` PINADO (vsync): o frame nunca reclama, então o RECUO nunca dispara
        // — o que sobra a segurar o custo é exclusivamente o TETO.
        ph2d_editor_core::tool::Tool::on_tick(&mut t, 16.6);
        total += t0.elapsed().as_secs_f64() * 1e3;
        let _ = t.take_preview_arc();
    }
    let hz = (sim_frame(&t) - before) as f64 / (FRAMES as f64 / 60.0);
    let mean = total / FRAMES as f64;
    assert!(
        hz <= MAX_THROTTLED_HZ,
        "sem teto: a sim rodou a {hz:.1} Hz numa poca que ela nao alcanca (teto {MAX_THROTTLED_HZ}) \
         — o orcamento nao estrangulou"
    );
    assert!(
        mean <= MAX_MEAN_MS,
        "sem teto: a agua custa {mean:.2} ms/frame medios numa poca que a sim nao alcanca \
         (teto {MAX_MEAN_MS})"
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

/// **E O ORÇAMENTO É INERTE NUMA POÇA PEQUENA** — a outra metade do par.
///
/// Um teto que morde onde não precisa não é um teto, é uma regressão: com pouca água um passo custa
/// fração de milissegundo e a sim tem de rodar os **40 Hz cheios**. Este gate afirma a taxa, e é ele
/// que torna seguro mexer no controlador do orçamento — sem ele, apertá-lo
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

#[path = "measure_wetpaint_probes.rs"]
mod measure_wetpaint_probes; // as MEDIÇÕES (#[ignore]) — irmãs dos gates, por LOC
